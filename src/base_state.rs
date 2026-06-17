use resource_collection_simulation::{BaseState, ResourceKind, RobotMessage};

pub fn handle_message(base: &mut BaseState, msg: &RobotMessage) {
    match msg {
        RobotMessage::DiscoveredResource { pos, resource } => {
            base.known_resources.insert(*pos, resource.clone());
        }
        RobotMessage::DiscoveredObstacle { pos } => {
            base.known_obstacles.insert(*pos);
        }
        RobotMessage::Unloading {
            robot_id: _,
            kind,
            amount,
        } => match kind {
            ResourceKind::Crystal => base.total_crystals += amount,
            ResourceKind::Energy => base.total_energy += amount,
        },
        RobotMessage::CollectedUnit { pos, kind: _ } => {
            if let Some(res) = base.known_resources.get_mut(pos) {
                res.quantity = res.quantity.saturating_sub(1);
                if res.quantity == 0 {
                    base.known_resources.remove(pos);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use resource_collection_simulation::{BaseState, Pos, Resource, ResourceKind, RobotMessage};

    use super::*;

    #[test]
    fn add_discovered_resource_crystal() {
        let mut base_state: BaseState = BaseState::default();
        let robot_message: RobotMessage = RobotMessage::DiscoveredResource {
            pos: (Pos { x: 1, y: 2 }),
            resource: Resource {
                kind: ResourceKind::Crystal,
                quantity: 5,
            },
        };

        handle_message(&mut base_state, &robot_message);

        let mut expected_resources = HashMap::new();
        expected_resources.insert(
            Pos { x: 1, y: 2 },
            Resource {
                kind: ResourceKind::Crystal,
                quantity: 5,
            },
        );

        assert_eq!(base_state.total_crystals, 0);
        assert_eq!(base_state.total_energy, 0);
        assert_eq!(base_state.known_resources, expected_resources);
        assert_eq!(base_state.known_obstacles, HashSet::new());
    }

    #[test]
    fn add_discovered_resource_energy() {
        let mut base_state: BaseState = BaseState::default();
        let robot_message: RobotMessage = RobotMessage::DiscoveredResource {
            pos: (Pos { x: 1, y: 2 }),
            resource: Resource {
                kind: ResourceKind::Energy,
                quantity: 5,
            },
        };

        handle_message(&mut base_state, &robot_message);

        let mut expected_resources = HashMap::new();
        expected_resources.insert(
            Pos { x: 1, y: 2 },
            Resource {
                kind: ResourceKind::Energy,
                quantity: 5,
            },
        );

        assert_eq!(base_state.total_crystals, 0);
        assert_eq!(base_state.total_energy, 0);
        assert_eq!(base_state.known_resources, expected_resources);
        assert_eq!(base_state.known_obstacles, HashSet::new());
    }

    #[test]
    fn add_multiple_discovered_resource() {
        let mut base_state: BaseState = BaseState::default();
        let robot_message: RobotMessage = RobotMessage::DiscoveredResource {
            pos: (Pos { x: 1, y: 2 }),
            resource: Resource {
                kind: ResourceKind::Crystal,
                quantity: 5,
            },
        };

        handle_message(&mut base_state, &robot_message);
        handle_message(&mut base_state, &robot_message);

        let mut expected_resources = HashMap::new();
        expected_resources.insert(
            Pos { x: 1, y: 2 },
            Resource {
                kind: ResourceKind::Crystal,
                quantity: 5,
            },
        );

        assert_eq!(base_state.total_crystals, 0);
        assert_eq!(base_state.total_energy, 0);
        assert_eq!(base_state.known_resources, expected_resources);
        assert_eq!(base_state.known_obstacles, HashSet::new());
    }

    #[test]
    fn add_discovered_obstacle() {
        let mut base_state: BaseState = BaseState::default();
        let robot_message: RobotMessage = RobotMessage::DiscoveredObstacle {
            pos: Pos { x: 3, y: 2 },
        };

        handle_message(&mut base_state, &robot_message);

        let mut expected_obstacles = HashSet::new();
        expected_obstacles.insert(Pos { x: 3, y: 2 });

        assert_eq!(base_state.total_crystals, 0);
        assert_eq!(base_state.total_energy, 0);
        assert_eq!(base_state.known_resources, HashMap::new());
        assert_eq!(base_state.known_obstacles, expected_obstacles);
    }

    #[test]
    fn collected_unit_decrements_quantity() {
        let mut base_state = BaseState::default();
        let discovery = RobotMessage::DiscoveredResource {
            pos: Pos { x: 1, y: 1 },
            resource: Resource {
                kind: ResourceKind::Crystal,
                quantity: 5,
            },
        };
        handle_message(&mut base_state, &discovery);

        let collect = RobotMessage::CollectedUnit {
            pos: Pos { x: 1, y: 1 },
            kind: ResourceKind::Crystal,
        };
        handle_message(&mut base_state, &collect);

        assert_eq!(base_state.known_resources[&Pos { x: 1, y: 1 }].quantity, 4);
        assert_eq!(base_state.total_crystals, 0);
    }

    #[test]
    fn collected_unit_removes_when_empty() {
        let mut base_state = BaseState::default();
        let discovery = RobotMessage::DiscoveredResource {
            pos: Pos { x: 2, y: 2 },
            resource: Resource {
                kind: ResourceKind::Energy,
                quantity: 1,
            },
        };
        handle_message(&mut base_state, &discovery);

        let collect = RobotMessage::CollectedUnit {
            pos: Pos { x: 2, y: 2 },
            kind: ResourceKind::Energy,
        };
        handle_message(&mut base_state, &collect);

        assert!(!base_state.known_resources.contains_key(&Pos { x: 2, y: 2 }));
    }

    #[test]
    fn collected_unit_on_unknown_resource_does_not_panic() {
        let mut base_state = BaseState::default();
        let collect = RobotMessage::CollectedUnit {
            pos: Pos { x: 99, y: 99 },
            kind: ResourceKind::Crystal,
        };
        handle_message(&mut base_state, &collect);
        assert!(base_state.known_resources.is_empty());
    }
}
