use lotus_extra::bb_system::{
    traction::{
        BBPistonTraction, BBPistonTractionTransfer, PistonTraction, PistonTractionTransfer,
    },
    {BackBoneTick, ElementTraitResetInputOutput, ElementTraitResetType},
};

pub struct Traction {
    pub piston: PistonTraction,
    pub transfer: PistonTractionTransfer,
}

impl Default for Traction {
    fn default() -> Self {
        Self {
            transfer: PistonTractionTransfer::new(0, 1),
            piston: PistonTraction::new(0, 1, 0.5),
        }
    }
}

impl Traction {
    pub fn on_message(
        &self,
        backbone: &mut BBPistonTractionTransfer,
        msg: &lotus_script::message::Message,
    ) {
        self.transfer.on_message(backbone, msg);
    }
}

impl BackBoneTick<BBTraction> for Traction {
    fn tick(&self, _: &mut BBTraction) {}
}

#[derive(Default)]
pub struct BBTraction {
    pub piston_traction: BBPistonTraction,
}

impl ElementTraitResetInputOutput for BBTraction {
    fn reset(&mut self, reset_type: ElementTraitResetType) {
        self.piston_traction.reset(reset_type);
    }
}
