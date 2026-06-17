use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::thread;
use rand::prelude::IndexedRandom;
use crate::types::{Pos, Map, Tile, RobotMessage};

pub struct Scout {
    pub id: usize,
    pub pos: Pos,
    pub known_obstacles: HashSet<Pos>, 
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

    fn scan_surroundings(&mut self) {
        let directions = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];

        for (dx, dy) in directions {
            let nx = self.pos.x as i32 + dx;
            let ny = self.pos.y as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let target_pos = Pos { x: nx as usize, y: ny as usize };

                if self.map.in_bounds(target_pos) {
                    if let Some(tile) = self.map.get_tile(target_pos) {
                        match tile {
                            Tile::Obstacle => {
                                if self.known_obstacles.insert(target_pos) {
                                    let _ = self.tx.send(RobotMessage::DiscoveredObstacle { pos: target_pos });
                                }
                            }
                            Tile::Resource(_) => {
                               
                                if let Some(res) = self.map.resources.get(&target_pos) {
                                    let _ = self.tx.send(RobotMessage::DiscoveredResource {
                                        pos: target_pos,
                                        resource: res.clone(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    /// Sélectionne une coordonnée adjacente valide au hasard
    fn choose_next_move(&self, rng: &mut impl rand::Rng) -> Option<Pos> {
        let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
        let mut valid_moves = Vec::new();

        for (dx, dy) in directions {
            let nx = self.pos.x as i32 + dx;
            let ny = self.pos.y as i32 + dy;

            if nx >= 0 && ny >= 0 {
                let next_pos = Pos { x: nx as usize, y: ny as usize };

                if self.map.in_bounds(next_pos) && !self.known_obstacles.contains(&next_pos) {
                    valid_moves.push(next_pos);
                }
            }
        }

        valid_moves.choose(rng).copied()
    }
}