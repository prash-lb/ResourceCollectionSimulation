use rand::prelude::IndexedRandom;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use crate::types::{Map, Pos, RobotMessage, Tile};

pub struct Scout {
    pub id: usize,
    pub pos: Pos,
    pub known_obstacles: HashSet<Pos>,
    pub known_resources: HashSet<Pos>,
    pub tx: Sender<RobotMessage>,
    pub map: Arc<Map>,
}

impl Scout {
    pub fn new(id: usize, tx: Sender<RobotMessage>, map: Arc<Map>) -> Self {
        let base_pos = map.base_pos;
        Self {
            id,
            pos: base_pos,
            known_obstacles: HashSet::new(),
            known_resources: HashSet::new(),
            tx,
            map,
        }
    }

    pub fn run(mut self) {
        let mut rng = rand::rng();

        loop {
            self.scan_surroundings();

            if let Some(next_pos) = self.choose_next_move(&mut rng) {
                self.pos = next_pos;
            }

            thread::sleep(Duration::from_millis(250));
        }
    }

    pub fn scan_surroundings(&mut self) {
        let directions = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dy) in directions {
            let nx = self.pos.x as i32 + dx;
            let ny = self.pos.y as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let target_pos = Pos {
                    x: nx as usize,
                    y: ny as usize,
                };

                if self.map.in_bounds(target_pos) {
                    if let Some(tile) = self.map.get_tile(target_pos) {
                        match tile {
                            Tile::Obstacle => {
                                if self.known_obstacles.insert(target_pos) {
                                    let _ = self
                                        .tx
                                        .send(RobotMessage::DiscoveredObstacle { pos: target_pos });
                                }
                            }
                            Tile::Resource(_) => {
                                if self.known_resources.insert(target_pos) {
                                    if let Some(res) = self.map.resources.get(&target_pos) {
                                        let _ = self.tx.send(RobotMessage::DiscoveredResource {
                                            pos: target_pos,
                                            resource: res.clone(),
                                        });
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn choose_next_move(&self, rng: &mut impl rand::Rng) -> Option<Pos> {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut valid_moves = Vec::new();

        for (dx, dy) in directions {
            let nx = self.pos.x as i32 + dx;
            let ny = self.pos.y as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let next_pos = Pos {
                    x: nx as usize,
                    y: ny as usize,
                };

                if self.map.in_bounds(next_pos) && !self.known_obstacles.contains(&next_pos) {
                    valid_moves.push(next_pos);
                }
            }
        }

        valid_moves.choose(rng).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Resource, ResourceKind};
    use std::collections::HashMap;
    use std::sync::mpsc;

    #[test]
    fn test_scout_only_discovers_resource_once() {
        let (tx, rx) = mpsc::channel();

        let mut resources = HashMap::new();
        let res_pos = Pos { x: 1, y: 0 };
        resources.insert(
            res_pos,
            Resource {
                kind: ResourceKind::Energy,
                quantity: 120,
            },
        );

        let map = Arc::new(Map {
            width: 2,
            height: 1,
            tiles: vec![vec![Tile::Base, Tile::Resource(ResourceKind::Energy)]],
            base_pos: Pos { x: 0, y: 0 },
            resources,
        });

        let mut scout = Scout::new(1, tx, map);

        scout.scan_surroundings();
        assert!(scout.known_resources.contains(&res_pos));

        scout.scan_surroundings();

        let mut msg_count = 0;
        while rx.try_recv().is_ok() {
            msg_count += 1;
        }

        assert_eq!(
            msg_count, 1,
            "Le scout a spammé le canal avec la même ressource !"
        );
    }
}
