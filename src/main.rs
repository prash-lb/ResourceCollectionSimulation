use resource_collection_simulation::{generate_map, print_map, MapConfig};

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
}
