# Répartition des tâches — Resource Collection Simulation

---

## Fatat — Génération de la carte

- Génération de la map avec bruit de Perlin (obstacles `O`)
- Placement aléatoire des ressources `E` et `C` (quantités 50-200)
- Placement de la base centrale `#`
- Structures de données de base : `Map`, `Tile`, `Resource`

---

## Corentin — Système de base + Architecture concurrente

- Système de base : point de départ, stockage des ressources, compteurs énergie/cristaux
- Architecture des threads : canaux `mpsc` / `Arc<Mutex<>>` entre robots et base
- Boucle principale de simulation (tick par tick, non-bloquant)

---

## Prashath — Robot Scout

- Comportement d'exploration aléatoire
- Détection et broadcast des ressources/obstacles découverts
- Évitement des obstacles connus
- Messages vers la base via channels

---

## Brice — Robot Collector + Pathfinding 

- Navigation vers les ressources connues (A\* ou BFS)
- Collecte une unité à la fois, retour à la base, déchargement
- Évitement de collisions entre collectors
- Stratégie d'allocation (quel collector va où)

---

## Mohamed — UI Ratatui + Communication/Synchronisation

- Rendu Ratatui en temps réel avec les couleurs définies
- Compteur de ressources collectées affiché
- Système de messages entre scouts, collectors et base (knowledge sharing)
- Gestion de l'input utilisateur (quitter au clavier)

---

## `src/types.rs` — Le fichier partagé

```rust
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceKind {
    Energy,
    Crystal,
}

// ── Map (owned by Brice, read by tous) ───────────────────────
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
    pub base_pos: Pos,
    pub resources: HashMap<Pos, Resource>,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub kind: ResourceKind,
    pub quantity: u32,  // 50-200
}

// ── Messages entre robots et base (owned by Corentin) ────────
#[derive(Clone, Debug)]
pub enum RobotMessage {
    // Scout → Base
    DiscoveredResource { pos: Pos, resource: Resource },
    DiscoveredObstacle { pos: Pos },

    // Collector → Base
    CollectedUnit { pos: Pos, kind: ResourceKind },
    Unloading { robot_id: usize, kind: ResourceKind, amount: u32 },
}

// ── État partagé de la base (lu par Ratatui / Mohamed) ───────
pub struct BaseState {
    pub total_energy: u32,
    pub total_crystals: u32,
    pub known_resources: HashMap<Pos, Resource>,   // mis à jour par les scouts
    pub known_obstacles: HashSet<Pos>,
}

// ── État d'un robot (lu par Ratatui pour affichage) ──────────
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
```
