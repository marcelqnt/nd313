use lotus_extra::traction::PistonTractionTransfer;

use crate::Backbone;

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

impl Traction {
    pub fn on_message(&self, msg: lotus_script::message::Message) {
        self.traction_transfer.on_message(msg);
    }
}
