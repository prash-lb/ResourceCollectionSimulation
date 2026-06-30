use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock, mpsc};

use resource_collection_simulation::{BaseState, MapConfig, RobotMessage, generate_map, print_map};
mod base_state;
mod simulation;
mod communication;
mod ui;

fn main() {
    let config = MapConfig {
        width: 60,
        height: 20,
        seed: 99,
        energy_count: 15,
        crystal_count: 15,
        obstacle_threshold: 0.2,
        ..Default::default()
    };
    let map = generate_map(&config);

    println!(
        "Carte {}x{} — Base à ({}, {}) — {} ressources",
        map.width,
        map.height,
        map.base_pos.x,
        map.base_pos.y,
        map.resources.len()
    );
    println!();
    print_map(&map);

    let (tx, rx) = mpsc::channel::<RobotMessage>();
    let base = Arc::new(Mutex::new(BaseState::default()));
    let map = Arc::new(RwLock::new(map));
    let running = Arc::new(AtomicBool::new(true));

    simulation::run(rx, base.clone(), map.clone(), tx, running);
}