use std::collections::HashSet;
use std::sync::{Arc, RwLock, mpsc};

use crate::types::{Map, Pos, Resource, ResourceKind, RobotMessage, Tile};
use rand::Rng;

pub struct Scout {
    pub id: usize,
    pub pos: Pos,
    map: Arc<RwLock<Map>>,
    tx: mpsc::Sender<RobotMessage>,
    known_resources: HashSet<Pos>,
    known_obstacles: HashSet<Pos>,
}

impl Scout {
    pub fn new(id: usize, tx: mpsc::Sender<RobotMessage>, map: Arc<RwLock<Map>>) -> Self {
        let pos = map.read().unwrap().base_pos;
        Scout {
            id,
            pos,
            map,
            tx,
            known_resources: HashSet::new(),
            known_obstacles: HashSet::new(),
        }
    }

    pub fn step(&mut self, rng: &mut impl Rng) -> Pos {
        let (messages, next_pos) = {
            let map = self.map.read().unwrap();
            let messages = self.scan(&map);
            let candidates = self.walkable_neighbors(&map);
            let next = self.pick_move(rng, &candidates);
            (messages, next)
        };

        for msg in &messages {
            match msg {
                RobotMessage::DiscoveredResource { pos, .. } => {
                    self.known_resources.insert(*pos);
                }
                RobotMessage::DiscoveredObstacle { pos } => {
                    self.known_obstacles.insert(*pos);
                }
                _ => {}
            }
        }
        for msg in messages {
            let _ = self.tx.send(msg);
        }

        if let Some(pos) = next_pos {
            self.pos = pos;
        }
        self.pos
    }

    fn scan(&self, map: &Map) -> Vec<RobotMessage> {
        let mut messages = vec![];
        let neighbors = [
            (-1i32, -1i32),
            (0, -1),
            (1, -1),
            (-1, 0),
            (0, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];

        for (dx, dy) in neighbors {
            let nx = self.pos.x as i32 + dx;
            let ny = self.pos.y as i32 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let pos = Pos {
                x: nx as usize,
                y: ny as usize,
            };

            match map.get_tile(pos) {
                Some(Tile::Resource(kind)) if !self.known_resources.contains(&pos) => {
                    let quantity = map.resources.get(&pos).map(|r| r.quantity).unwrap_or(1);
                    messages.push(RobotMessage::DiscoveredResource {
                        pos,
                        resource: Resource { kind, quantity },
                    });
                }
                Some(Tile::Obstacle) if !self.known_obstacles.contains(&pos) => {
                    messages.push(RobotMessage::DiscoveredObstacle { pos });
                }
                _ => {}
            }
        }
        messages
    }

    fn walkable_neighbors(&self, map: &Map) -> Vec<Pos> {
        [(-1i32, 0i32), (1, 0), (0, -1), (0, 1)]
            .iter()
            .filter_map(|(dx, dy)| {
                let nx = self.pos.x as i32 + dx;
                let ny = self.pos.y as i32 + dy;
                if nx < 0 || ny < 0 {
                    return None;
                }
                let pos = Pos {
                    x: nx as usize,
                    y: ny as usize,
                };
                if map.is_walkable(pos) && !self.known_obstacles.contains(&pos) {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect()
    }

    fn pick_move(&self, rng: &mut impl Rng, candidates: &[Pos]) -> Option<Pos> {
        if candidates.is_empty() {
            return None;
        }
        Some(candidates[rng.random_range(0..candidates.len())])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Map, Tile};
    use std::collections::HashMap;
    use std::sync::mpsc;

    fn make_map(width: usize, height: usize) -> Arc<RwLock<Map>> {
        Arc::new(RwLock::new(Map {
            width,
            height,
            tiles: vec![vec![Tile::Empty; width]; height],
            base_pos: Pos { x: 0, y: 0 },
            resources: HashMap::new(),
        }))
    }

    #[test]
    fn scout_stays_in_bounds() {
        let map = make_map(5, 5);
        let (tx, _rx) = mpsc::channel();
        let mut scout = Scout::new(0, tx, map);
        let mut rng = rand::rng();
        for _ in 0..20 {
            let pos = scout.step(&mut rng);
            assert!(pos.x < 5 && pos.y < 5);
        }
    }

    #[test]
    fn scout_reports_resource_only_once() {
        let mut tiles = vec![vec![Tile::Empty; 3]; 3];
        tiles[0][1] = Tile::Resource(ResourceKind::Energy);
        let mut resources = HashMap::new();
        resources.insert(
            Pos { x: 1, y: 0 },
            Resource {
                kind: ResourceKind::Energy,
                quantity: 100,
            },
        );
        let map = Arc::new(RwLock::new(Map {
            width: 3,
            height: 3,
            tiles,
            base_pos: Pos { x: 0, y: 0 },
            resources,
        }));
        let (tx, rx) = mpsc::channel();
        let mut scout = Scout::new(0, tx, map);
        let mut rng = rand::rng();
        for _ in 0..5 {
            scout.step(&mut rng);
        }
        let count = rx
            .try_iter()
            .filter(|m| matches!(m, RobotMessage::DiscoveredResource { .. }))
            .count();
        assert_eq!(count, 1);
    }
}
