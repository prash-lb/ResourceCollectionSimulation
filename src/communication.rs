use resource_collection_simulation::{RobotMessage, UIState, ResourceKind};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

// ── Système de communication ────────────────────────────────
pub struct CommunicationBus {
    pub message_receiver: Arc<Mutex<Receiver<RobotMessage>>>,
    pub message_sender: Sender<RobotMessage>,
}

impl CommunicationBus {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        CommunicationBus {
            message_sender: tx,
            message_receiver: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn drain_messages(&self, state: &mut UIState) {
        let guard = match self.message_receiver.lock() {
            Ok(g) => g,
            Err(_) => return,
        };

        loop {
            match guard.try_recv() {
                Ok(msg) => handle_robot_message(&msg, state),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
    }
}

// ── Traitement des messages ────────────────────────────────
pub fn handle_robot_message(msg: &RobotMessage, state: &mut UIState) {
    match msg {
        RobotMessage::DiscoveredResource { pos, resource } => {
            state.discovered_resources.insert(*pos, resource.clone());
        }
        RobotMessage::DiscoveredObstacle { pos } => {
            if !state.discovered_obstacles.contains(pos) {
                state.discovered_obstacles.push(*pos);
            }
        }
        RobotMessage::CollectedUnit { pos: _, kind } => {
            match kind {
                ResourceKind::Energy => state.energy_collected += 1,
                ResourceKind::Crystal => state.crystals_collected += 1,
            }
        }
        RobotMessage::Unloading { robot_id: _, kind, amount } => {
            match kind {
                ResourceKind::Energy => state.energy_collected += amount,
                ResourceKind::Crystal => state.crystals_collected += amount,
            }
        }
    }
}
