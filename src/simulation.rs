use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::thread;
use std::time::Duration;

use resource_collection_simulation::{BaseState, Map, RobotKind, RobotMessage, RobotState};

use crate::base_state::handle_message;

const TICK_MS: u64 = 100;
const NUM_SCOUTS: usize = 2;
const NUM_COLLECTORS: usize = 2;

pub fn run(
    rx: mpsc::Receiver<RobotMessage>,
    base_state: Arc<Mutex<BaseState>>,
    map: Arc<RwLock<Map>>,
    tx: mpsc::Sender<RobotMessage>,
    running: Arc<AtomicBool>,
) {
    let base_pos = {
        let map_guard = map.read().unwrap();
        map_guard.base_pos
    };

    let robot_states: Arc<Mutex<Vec<RobotState>>> = Arc::new(Mutex::new(Vec::new()));
    {
        let mut states = robot_states.lock().unwrap();
        for id in 0..NUM_SCOUTS {
            states.push(RobotState {
                id,
                kind: RobotKind::Scout,
                pos: base_pos,
                carrying: None,
            });
        }
        for id in NUM_SCOUTS..(NUM_SCOUTS + NUM_COLLECTORS) {
            states.push(RobotState {
                id,
                kind: RobotKind::Collector,
                pos: base_pos,
                carrying: None,
            });
        }
    }

    let mut handles = vec![];

    for id in 0..NUM_SCOUTS {
        let tx = tx.clone();
        let running = running.clone();
        let _map = map.clone();
        let _states = robot_states.clone();
        let handle = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                // TODO: logique de l'éclaireur (Prashath)
                // - se déplacer aléatoirement
                // - envoyer DiscoveredResource / DiscoveredObstacle via tx
                thread::sleep(Duration::from_millis(TICK_MS * 5));
            }
        });
        handles.push(handle);
    }

    for id in NUM_SCOUTS..(NUM_SCOUTS + NUM_COLLECTORS) {
        let tx = tx.clone();
        let running = running.clone();
        let _map = map.clone();
        let base_state = base_state.clone();
        let _states = robot_states.clone();
        let handle = thread::spawn(move || {
            while running.load(Ordering::Relaxed) {
                let _target = {
                    let base = base_state.lock().unwrap();
                    base.known_resources.keys().next().copied()
                };

                // TODO: logique du collecteur (Brice)
                // - se déplacer vers _target
                // - collecter une unité, revenir à la base
                // - envoyer CollectedUnit / Unloading via tx
                thread::sleep(Duration::from_millis(TICK_MS * 5));
            }
        });
        handles.push(handle);
    }

    drop(tx);

    while running.load(Ordering::Relaxed) {
        while let Ok(msg) = rx.try_recv() {
            let mut base = base_state.lock().unwrap();
            handle_message(&mut base, &msg);
        }

        // Si le canal est fermé, plus personne ne peut envoyer de message
        if matches!(rx.try_recv(), Err(mpsc::TryRecvError::Disconnected)) {
            break;
        }

        // TODO: rafraîchir l'UI ici (Mohamed) — lire base_state + robot_states

        thread::sleep(Duration::from_millis(TICK_MS));
    }

    while let Ok(msg) = rx.try_recv() {
        let mut base = base_state.lock().unwrap();
        handle_message(&mut base, &msg);
    }

    for h in handles {
        let _ = h.join();
    }
}

#[cfg(test)]
mod tests {
    use crate::base_state::handle_message;
    use resource_collection_simulation::{BaseState, Pos, Resource, ResourceKind, RobotMessage};
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[test]
    fn test_knowledge_sharing_via_arc() {
        let base = Arc::new(Mutex::new(BaseState::default()));

        let base_clone = base.clone();
        let scout = thread::spawn(move || {
            let msg = RobotMessage::DiscoveredResource {
                pos: Pos { x: 3, y: 4 },
                resource: Resource {
                    kind: ResourceKind::Energy,
                    quantity: 50,
                },
            };
            let mut state = base_clone.lock().unwrap();
            handle_message(&mut state, &msg);
        });

        scout.join().unwrap();

        let state = base.lock().unwrap();
        assert!(state.known_resources.contains_key(&Pos { x: 3, y: 4 }));
        assert_eq!(state.known_resources[&Pos { x: 3, y: 4 }].quantity, 50);
    }

    #[test]
    fn test_collected_unit_visible_to_all_via_arc() {
        let base = Arc::new(Mutex::new(BaseState::default()));

        {
            let mut state = base.lock().unwrap();
            let discovery = RobotMessage::DiscoveredResource {
                pos: Pos { x: 1, y: 1 },
                resource: Resource {
                    kind: ResourceKind::Crystal,
                    quantity: 3,
                },
            };
            handle_message(&mut state, &discovery);
        }

        let base_clone = base.clone();
        let collector = thread::spawn(move || {
            let mut state = base_clone.lock().unwrap();
            let collect = RobotMessage::CollectedUnit {
                pos: Pos { x: 1, y: 1 },
                kind: ResourceKind::Crystal,
            };
            handle_message(&mut state, &collect);
        });

        collector.join().unwrap();

        let state = base.lock().unwrap();
        assert_eq!(state.known_resources[&Pos { x: 1, y: 1 }].quantity, 2);
    }
}
