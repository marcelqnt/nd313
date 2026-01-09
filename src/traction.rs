use lotus_actor::{SystemUnit, piston_traction::PistonTractionTransfer};

use crate::HashMaps;

pub struct Traction {
    traction_transfer: PistonTractionTransfer,
}

impl Default for Traction {
    fn default() -> Self {
        Self {
            traction_transfer: PistonTractionTransfer::new(0, 1),
        }
    }
}

impl SystemUnit<HashMaps> for Traction {
    fn children(&self) -> Vec<&dyn SystemUnit<HashMaps>> {
        vec![]
    }
}

impl Traction {
    pub fn on_message(&self, msg: lotus_script::message::Message) {
        self.traction_transfer.on_message(msg);
    }
}
