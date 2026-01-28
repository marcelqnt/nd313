use lotus_extra::{
    backbone::{BackBoneTick, ElementTraitReset},
    traction::{
        BBPistonTraction, BBThrottleBrakeControl, PistonTraction, PistonTractionTransfer,
        ThrottleBrakeControl,
    },
};

pub struct Traction {
    pub piston: PistonTraction,
    pub transfer: PistonTractionTransfer,
    pub throttle_control: ThrottleBrakeControl,
}

impl Default for Traction {
    fn default() -> Self {
        Self {
            transfer: PistonTractionTransfer::new(0, 1),
            piston: PistonTraction::new(0, 1, 0.5),
            throttle_control: ThrottleBrakeControl::new(0, 0.8),
        }
    }
}

impl Traction {
    pub fn on_message(&self, msg: &lotus_script::message::Message) {
        self.transfer.on_message(msg);
    }
}

impl BackBoneTick<BBTraction> for Traction {
    fn tick(&self, backbone: &mut BBTraction) {
        self.throttle_control.tick(&mut backbone.throttle_control);
    }
}

#[derive(Default)]
pub struct BBTraction {
    pub piston_traction: BBPistonTraction,
    pub throttle_control: BBThrottleBrakeControl,
}

impl ElementTraitReset for BBTraction {
    fn reset(&mut self) {
        self.piston_traction.reset();
    }
}
