use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use resource_collection_simulation::{BaseState, Map, Pos, RobotMessage, RobotState, Scout};

use crate::base_state::handle_message;
use crate::collector::Collector;

pub const NUM_SCOUTS: usize = 2;
pub const NUM_COLLECTORS: usize = 2;

const SCOUT_TICK_MS: u64 = 250;
const COLLECTOR_TICK_MS: u64 = 500;
const MESSAGE_TICK_MS: u64 = 100;

pub fn run(
    rx: mpsc::Receiver<RobotMessage>,
    base_state: Arc<Mutex<BaseState>>,
    map: Arc<RwLock<Map>>,
    tx: mpsc::Sender<RobotMessage>,
    running: Arc<AtomicBool>,
    robot_states: Arc<Mutex<Vec<RobotState>>>,
) -> Vec<JoinHandle<()>> {
    let mut handles = vec![];

    for id in 0..NUM_SCOUTS {
        let tx = tx.clone();
        let map = map.clone();
        let running = running.clone();
        let states = robot_states.clone();

        handles.push(thread::spawn(move || {
            let mut scout = Scout::new(id, tx, map);
            let mut rng = rand::rng();

            while running.load(Ordering::Relaxed) {
                let pos = scout.step(&mut rng);
                if let Ok(mut states) = states.lock() {
                    if let Some(state) = states.get_mut(id) {
                        state.pos = pos;
                    }
                }
                thread::sleep(Duration::from_millis(SCOUT_TICK_MS));
            }
        }));
    }

    let claimed_targets = Arc::new(Mutex::new(HashSet::<Pos>::new()));

    for id in NUM_SCOUTS..(NUM_SCOUTS + NUM_COLLECTORS) {
        let tx = tx.clone();
        let map = map.clone();
        let base_state = base_state.clone();
        let running = running.clone();
        let states = robot_states.clone();
        let claimed = claimed_targets.clone();

        handles.push(thread::spawn(move || {
            let base_pos = map.read().unwrap().base_pos;
            let mut collector = Collector::new(id, base_pos);

            while running.load(Ordering::Relaxed) {
                {
                    let map_read = map.read().unwrap();
                    let base = base_state.lock().unwrap();
                    collector.step(&map_read, &base.known_resources, base_pos, &tx, &claimed);
                }
                if let Ok(mut states) = states.lock() {
                    if let Some(state) = states.get_mut(id) {
                        state.pos = collector.pos;
                        state.carrying = collector.carrying;
                    }
                }
                thread::sleep(Duration::from_millis(COLLECTOR_TICK_MS));
            }
        }));
    }

    handles.push(thread::spawn(move || {
        drop(tx);
        loop {
            match rx.recv_timeout(Duration::from_millis(MESSAGE_TICK_MS)) {
                Ok(msg) => {
                    let mut base = base_state.lock().unwrap();
                    handle_message(&mut base, &msg);
                    if let RobotMessage::CollectedUnit { pos, .. } = &msg {
                        if !base.known_resources.contains_key(pos) {
                            let mut m = map.write().unwrap();
                            m.tiles[pos.y][pos.x] = resource_collection_simulation::Tile::Empty;
                            m.resources.remove(pos);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
            if !running.load(Ordering::Relaxed) {
                break;
            }
        }
    }));

    handles
}

#[cfg(test)]
mod tests {
    use crate::base_state::handle_message;
    use resource_collection_simulation::{BaseState, Pos, Resource, ResourceKind, RobotMessage};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn knowledge_sharing_via_arc() {
        let base = Arc::new(Mutex::new(BaseState::default()));
        let base_clone = base.clone();

        thread::spawn(move || {
            let msg = RobotMessage::DiscoveredResource {
                pos: Pos { x: 3, y: 4 },
                resource: Resource {
                    kind: ResourceKind::Energy,
                    quantity: 50,
                },
            };
            handle_message(&mut base_clone.lock().unwrap(), &msg);
        })
        .join()
        .unwrap();

        let state = base.lock().unwrap();
        assert!(state.known_resources.contains_key(&Pos { x: 3, y: 4 }));
        assert_eq!(state.known_resources[&Pos { x: 3, y: 4 }].quantity, 50);
    }

    #[test]
    fn collected_unit_visible_to_all() {
        let base = Arc::new(Mutex::new(BaseState::default()));

        handle_message(
            &mut base.lock().unwrap(),
            &RobotMessage::DiscoveredResource {
                pos: Pos { x: 1, y: 1 },
                resource: Resource {
                    kind: ResourceKind::Crystal,
                    quantity: 3,
                },
            },
        );

        let base_clone = base.clone();
        thread::spawn(move || {
            handle_message(
                &mut base_clone.lock().unwrap(),
                &RobotMessage::CollectedUnit {
                    pos: Pos { x: 1, y: 1 },
                    kind: ResourceKind::Crystal,
                },
            );
        })
        .join()
        .unwrap();

        assert_eq!(
            base.lock().unwrap().known_resources[&Pos { x: 1, y: 1 }].quantity,
            2
        );
    }
}
