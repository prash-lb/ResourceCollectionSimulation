use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex, mpsc};

use resource_collection_simulation::{Map, Pos, Resource, ResourceKind, RobotMessage, Tile};

#[derive(Debug)]
pub enum CollectorState {
    Idle,
    GoingToResource,
    GoingToBase,
}

pub struct Collector {
    pub id: usize,
    pub pos: Pos,
    pub state: CollectorState,
    pub target: Option<Pos>,
    pub path: Vec<Pos>,
    pub carrying: Option<(ResourceKind, u32)>,
}

impl Collector {
    pub fn new(id: usize, start: Pos) -> Self {
        Collector {
            id,
            pos: start,
            state: CollectorState::Idle,
            target: None,
            path: Vec::new(),
            carrying: None,
        }
    }

    pub fn step(
        &mut self,
        map: &Map,
        known_resources: &HashMap<Pos, Resource>,
        base_pos: Pos,
        tx: &mpsc::Sender<RobotMessage>,
        claimed: &Arc<Mutex<HashSet<Pos>>>,
    ) {
        match self.state {
            CollectorState::Idle => {
                let target = {
                    let mut lock = claimed.lock().unwrap();
                    let found = known_resources.keys().find(|p| !lock.contains(p)).copied();
                    if let Some(pos) = found {
                        lock.insert(pos);
                    }
                    found
                };
                if let Some(target_pos) = target {
                    let path = bfs(map, self.pos, target_pos);
                    if path.is_empty() {
                        claimed.lock().unwrap().remove(&target_pos);
                    } else {
                        self.target = Some(target_pos);
                        self.path = path;
                        self.state = CollectorState::GoingToResource;
                    }
                }
            }
            CollectorState::GoingToResource => {
                let arrived = self.tick();
                if arrived {
                    if let Some(target_pos) = self.target {
                        if let Some(res) = known_resources.get(&target_pos) {
                            self.carrying = Some((res.kind, 1));
                            let _ = tx.send(RobotMessage::CollectedUnit {
                                pos: target_pos,
                                kind: res.kind,
                            });
                        }
                    }
                    self.path = bfs(map, self.pos, base_pos);
                    self.state = CollectorState::GoingToBase;
                }
            }
            CollectorState::GoingToBase => {
                let arrived = self.tick();
                if arrived {
                    if let Some((kind, amount)) = self.carrying.take() {
                        let _ = tx.send(RobotMessage::Unloading {
                            robot_id: self.id,
                            kind,
                            amount,
                        });
                    }
                    if let Some(target_pos) = self.target {
                        if known_resources.contains_key(&target_pos) {
                            self.path = bfs(map, self.pos, target_pos);
                            self.state = CollectorState::GoingToResource;
                        } else {
                            claimed.lock().unwrap().remove(&target_pos);
                            self.target = None;
                            self.state = CollectorState::Idle;
                        }
                    }
                }
            }
        }
    }

    fn tick(&mut self) -> bool {
        if self.path.is_empty() {
            return true;
        }
        self.pos = self.path.remove(0);
        self.path.is_empty()
    }
}

fn get_neighbors(map: &Map, pos: Pos) -> Vec<Pos> {
    let mut result = Vec::new();
    if pos.x > 0 && map.tiles[pos.y][pos.x - 1] != Tile::Obstacle {
        result.push(Pos {
            x: pos.x - 1,
            y: pos.y,
        });
    }
    if pos.x + 1 < map.width && map.tiles[pos.y][pos.x + 1] != Tile::Obstacle {
        result.push(Pos {
            x: pos.x + 1,
            y: pos.y,
        });
    }
    if pos.y > 0 && map.tiles[pos.y - 1][pos.x] != Tile::Obstacle {
        result.push(Pos {
            x: pos.x,
            y: pos.y - 1,
        });
    }
    if pos.y + 1 < map.height && map.tiles[pos.y + 1][pos.x] != Tile::Obstacle {
        result.push(Pos {
            x: pos.x,
            y: pos.y + 1,
        });
    }
    result
}

pub fn bfs(map: &Map, start: Pos, goal: Pos) -> Vec<Pos> {
    let mut visited: HashSet<Pos> = HashSet::new();
    let mut queue: VecDeque<Vec<Pos>> = VecDeque::new();

    visited.insert(start);
    queue.push_back(vec![start]);

    while let Some(path) = queue.pop_front() {
        let current = *path.last().unwrap();
        if current == goal {
            return path[1..].to_vec();
        }
        for neighbor in get_neighbors(map, current) {
            if visited.insert(neighbor) {
                let mut new_path = path.clone();
                new_path.push(neighbor);
                queue.push_back(new_path);
            }
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_map(width: usize, height: usize) -> Map {
        Map {
            width,
            height,
            tiles: vec![vec![Tile::Empty; width]; height],
            base_pos: Pos { x: 0, y: 0 },
            resources: HashMap::new(),
        }
    }

    #[test]
    fn test_bfs_chemin_simple() {
        let map = make_map(5, 1);
        let chemin = bfs(&map, Pos { x: 0, y: 0 }, Pos { x: 4, y: 0 });
        assert_eq!(
            chemin,
            vec![
                Pos { x: 1, y: 0 },
                Pos { x: 2, y: 0 },
                Pos { x: 3, y: 0 },
                Pos { x: 4, y: 0 },
            ]
        );
    }

    #[test]
    fn test_bfs_contourne_obstacle() {
        let mut map = make_map(3, 3);
        map.tiles[0][1] = Tile::Obstacle;
        let chemin = bfs(&map, Pos { x: 0, y: 0 }, Pos { x: 2, y: 0 });
        assert!(!chemin.is_empty());
        assert!(!chemin.contains(&Pos { x: 1, y: 0 }));
        assert_eq!(*chemin.last().unwrap(), Pos { x: 2, y: 0 });
    }

    #[test]
    fn test_bfs_chemin_le_plus_court() {
        let map = make_map(3, 3);
        let chemin = bfs(&map, Pos { x: 0, y: 0 }, Pos { x: 2, y: 2 });
        assert_eq!(chemin.len(), 4);
    }

    #[test]
    fn test_tick_avance_dune_case() {
        let mut collector = Collector::new(0, Pos { x: 0, y: 0 });
        collector.path = vec![Pos { x: 1, y: 0 }, Pos { x: 2, y: 0 }];
        let arrivé = collector.tick();
        assert_eq!(collector.pos, Pos { x: 1, y: 0 });
        assert!(!arrivé);
    }

    #[test]
    fn test_tick_derniere_case() {
        let mut collector = Collector::new(0, Pos { x: 0, y: 0 });
        collector.path = vec![Pos { x: 1, y: 0 }];
        let arrivé = collector.tick();
        assert_eq!(collector.pos, Pos { x: 1, y: 0 });
        assert!(arrivé);
    }

    #[test]
    fn test_tick_chemin_vide() {
        let mut collector = Collector::new(0, Pos { x: 2, y: 2 });
        let arrivé = collector.tick();
        assert_eq!(collector.pos, Pos { x: 2, y: 2 });
        assert!(arrivé);
    }

    #[test]
    fn test_voisins_centre() {
        let map = make_map(3, 3);
        assert_eq!(get_neighbors(&map, Pos { x: 1, y: 1 }).len(), 4);
    }

    #[test]
    fn test_voisins_coin() {
        let map = make_map(3, 3);
        assert_eq!(get_neighbors(&map, Pos { x: 0, y: 0 }).len(), 2);
    }

    #[test]
    fn test_voisins_bord() {
        let map = make_map(3, 3);
        assert_eq!(get_neighbors(&map, Pos { x: 1, y: 0 }).len(), 3);
    }

    #[test]
    fn test_obstacle_bloque() {
        let mut map = make_map(3, 3);
        map.tiles[1][2] = Tile::Obstacle;
        let neighbors = get_neighbors(&map, Pos { x: 1, y: 1 });
        assert_eq!(neighbors.len(), 3);
        assert!(!neighbors.contains(&Pos { x: 2, y: 1 }));
    }

    #[test]
    fn test_tous_obstacles_autour() {
        let mut map = make_map(3, 3);
        map.tiles[1][0] = Tile::Obstacle;
        map.tiles[1][2] = Tile::Obstacle;
        map.tiles[0][1] = Tile::Obstacle;
        map.tiles[2][1] = Tile::Obstacle;
        assert_eq!(get_neighbors(&map, Pos { x: 1, y: 1 }).len(), 0);
    }

    #[test]
    fn test_step_collecte_et_retourne() {
        use std::sync::{Arc, Mutex, mpsc};

        let mut map = make_map(3, 1);
        let resource_pos = Pos { x: 2, y: 0 };
        map.tiles[0][2] = Tile::Resource(ResourceKind::Energy);
        let mut resources = HashMap::new();
        resources.insert(
            resource_pos,
            Resource {
                kind: ResourceKind::Energy,
                quantity: 10,
            },
        );

        let base_pos = Pos { x: 0, y: 0 };
        let (tx, rx) = mpsc::channel();
        let claimed = Arc::new(Mutex::new(HashSet::new()));
        let mut collector = Collector::new(0, base_pos);

        collector.step(&map, &resources, base_pos, &tx, &claimed);
        assert!(matches!(collector.state, CollectorState::GoingToResource));

        for _ in 0..3 {
            collector.step(&map, &resources, base_pos, &tx, &claimed);
        }

        let messages: Vec<_> = rx.try_iter().collect();
        assert!(
            messages
                .iter()
                .any(|m| matches!(m, RobotMessage::CollectedUnit { .. }))
        );
    }
}
