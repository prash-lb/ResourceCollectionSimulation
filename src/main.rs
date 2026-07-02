use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::time::{SystemTime, UNIX_EPOCH};

use resource_collection_simulation::{
    BaseState, MapConfig, RobotKind, RobotMessage, RobotState, generate_map,
};

mod base_state;
mod collector;
mod simulation;
mod ui;

use simulation::{NUM_COLLECTORS, NUM_SCOUTS};

fn main() -> std::io::Result<()> {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(42);

    let map = generate_map(&MapConfig {
        width: 60,
        height: 20,
        seed,
        energy_count: 15,
        crystal_count: 15,
        obstacle_threshold: 0.2,
        ..Default::default()
    });

    let base_pos = map.base_pos;

    let robot_states = Arc::new(Mutex::new(
        (0..NUM_SCOUTS)
            .map(|id| RobotState {
                id,
                kind: RobotKind::Scout,
                pos: base_pos,
                carrying: None,
            })
            .chain(
                (NUM_SCOUTS..NUM_SCOUTS + NUM_COLLECTORS).map(|id| RobotState {
                    id,
                    kind: RobotKind::Collector,
                    pos: base_pos,
                    carrying: None,
                }),
            )
            .collect::<Vec<_>>(),
    ));

    let (tx, rx) = mpsc::channel::<RobotMessage>();
    let base_state = Arc::new(Mutex::new(BaseState::default()));
    let map = Arc::new(RwLock::new(map));
    let running = Arc::new(AtomicBool::new(true));

    let handles = simulation::run(
        rx,
        base_state.clone(),
        map.clone(),
        tx,
        running.clone(),
        robot_states.clone(),
    );

    ui::UIRenderer::new().run(map, base_state, robot_states, running.clone())?;

    running.store(false, Ordering::Relaxed);
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}
