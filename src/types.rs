use std::collections::{HashMap, HashSet};

// ── Coordonnées ──────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

// ── Carte ────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tile {
    Empty,
    Obstacle,
    Base,
    Resource(ResourceKind),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ResourceKind {
    Energy,
    Crystal,
}

// ── Map ──────────────────────────────────────────────────────
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub base_pos: Pos,
    pub resources: HashMap<Pos, Resource>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    pub kind: ResourceKind,
    pub quantity: u32,
}

// ── Messages entre robots et base ────────────────────────────
#[derive(Clone, Debug)]
pub enum RobotMessage {
    DiscoveredResource {
        pos: Pos,
        resource: Resource,
    },
    DiscoveredObstacle {
        pos: Pos,
    },
    CollectedUnit {
        pos: Pos,
        kind: ResourceKind,
    },
    Unloading {
        robot_id: usize,
        kind: ResourceKind,
        amount: u32,
    },
}

// ── État partagé de la base ───────────────────────────────────
#[derive(Default)]
pub struct BaseState {
    pub total_energy: u32,
    pub total_crystals: u32,
    pub known_resources: HashMap<Pos, Resource>,
    pub known_obstacles: HashSet<Pos>,
}

// ── État d'un robot ──────────────────────────────────────────
#[derive(Clone, Debug)]
pub struct RobotState {
    pub id: usize,
    pub kind: RobotKind,
    pub pos: Pos,
    pub carrying: Option<(ResourceKind, u32)>,
}

#[derive(Clone, Copy, Debug)]
pub enum RobotKind {
    Scout,
    Collector,
}

impl Map {
    pub fn in_bounds(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }

    pub fn get_tile(&self, pos: Pos) -> Option<Tile> {
        if self.in_bounds(pos) {
            Some(self.tiles[pos.y][pos.x])
        } else {
            None
        }
    }

    pub fn is_walkable(&self, pos: Pos) -> bool {
        matches!(
            self.get_tile(pos),
            Some(Tile::Empty | Tile::Base | Tile::Resource(_))
        )
    }

    pub fn char_at(&self, pos: Pos) -> char {
        match self.get_tile(pos) {
            Some(Tile::Obstacle) => 'O',
            Some(Tile::Base) => '#',
            Some(Tile::Resource(ResourceKind::Energy)) => 'E',
            Some(Tile::Resource(ResourceKind::Crystal)) => 'C',
            Some(Tile::Empty) => '.',
            None => ' ',
        }
    }
}
