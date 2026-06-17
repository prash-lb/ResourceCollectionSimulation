use std::collections::HashMap;

use noise::{NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};

use crate::types::{Map, Pos, Resource, ResourceKind, Tile};

/// Paramètres de génération de carte.
pub struct MapConfig {
    pub width: usize,
    pub height: usize,
    pub seed: u32,
    pub energy_count: usize,
    pub crystal_count: usize,
    /// Seuil du bruit de Perlin au-dessus duquel une case devient un obstacle.
    pub obstacle_threshold: f64,
    /// Échelle du bruit (plus petit = obstacles plus larges).
    pub noise_scale: f64,
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            seed: 42,
            energy_count: 10,
            crystal_count: 10,
            obstacle_threshold: 0.15,
            noise_scale: 0.12,
        }
    }
}

const MIN_RESOURCE_QTY: u32 = 50;
const MAX_RESOURCE_QTY: u32 = 200;
const BASE_CLEAR_RADIUS: i32 = 2;

/// Génère une carte procédurale avec obstacles (Perlin), base centrale et ressources.
pub fn generate_map(config: &MapConfig) -> Map {
    let base_pos = Pos {
        x: config.width / 2,
        y: config.height / 2,
    };

    let mut tiles = vec![vec![Tile::Empty; config.width]; config.height];
    let perlin = Perlin::new(config.seed);

    for y in 0..config.height {
        for x in 0..config.width {
            let pos = Pos { x, y };
            if is_in_base_clear_zone(pos, base_pos) {
                continue;
            }

            let noise_value =
                perlin.get([x as f64 * config.noise_scale, y as f64 * config.noise_scale]);

            if noise_value > config.obstacle_threshold {
                tiles[y][x] = Tile::Obstacle;
            }
        }
    }

    tiles[base_pos.y][base_pos.x] = Tile::Base;

    let mut rng = StdRng::seed_from_u64(config.seed as u64 + 1);
    let mut resources = HashMap::new();

    let mut empty_positions: Vec<Pos> = Vec::new();
    for y in 0..config.height {
        for x in 0..config.width {
            if tiles[y][x] == Tile::Empty {
                empty_positions.push(Pos { x, y });
            }
        }
    }

    empty_positions.shuffle(&mut rng);

    let total_resources = config.energy_count + config.crystal_count;
    let placement_count = total_resources.min(empty_positions.len());

    for (i, pos) in empty_positions
        .into_iter()
        .take(placement_count)
        .enumerate()
    {
        let kind = if i < config.energy_count {
            ResourceKind::Energy
        } else {
            ResourceKind::Crystal
        };

        let quantity = rng.random_range(MIN_RESOURCE_QTY..=MAX_RESOURCE_QTY);
        tiles[pos.y][pos.x] = Tile::Resource(kind);
        resources.insert(pos, Resource { kind, quantity });
    }

    Map {
        width: config.width,
        height: config.height,
        tiles,
        base_pos,
        resources,
    }
}

fn is_in_base_clear_zone(pos: Pos, base_pos: Pos) -> bool {
    let dx = pos.x as i32 - base_pos.x as i32;
    let dy = pos.y as i32 - base_pos.y as i32;
    dx.abs() <= BASE_CLEAR_RADIUS && dy.abs() <= BASE_CLEAR_RADIUS
}

/// Affiche la carte en ASCII (utile pour le debug et la vérification).
pub fn print_map(map: &Map) {
    for y in 0..map.height {
        for x in 0..map.width {
            let pos = Pos { x, y };
            print!("{}", map.char_at(pos));
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_has_base_at_center() {
        let config = MapConfig {
            width: 40,
            height: 20,
            ..Default::default()
        };
        let map = generate_map(&config);
        assert_eq!(map.base_pos, Pos { x: 20, y: 10 });
        assert_eq!(map.get_tile(map.base_pos), Some(Tile::Base));
    }

    #[test]
    fn resources_have_valid_quantities() {
        let map = generate_map(&MapConfig::default());
        for resource in map.resources.values() {
            assert!(resource.quantity >= MIN_RESOURCE_QTY);
            assert!(resource.quantity <= MAX_RESOURCE_QTY);
        }
    }

    #[test]
    fn no_resources_on_obstacles_or_base() {
        let map = generate_map(&MapConfig::default());
        for y in 0..map.height {
            for x in 0..map.width {
                let tile = map.tiles[y][x];
                if matches!(tile, Tile::Resource(_)) {
                    assert_ne!(tile, Tile::Obstacle);
                    assert_ne!(tile, Tile::Base);
                }
            }
        }
    }
}
