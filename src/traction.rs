use lotus_extra::bb_system::{
    basic::{BackBoneResetInputOutput, BackBoneResetType, ModuleOnMessage, ModuleTick},
    piston_traction::{
        BBPistonTraction, BBPistonTractionTransfer, PistonTraction, PistonTractionTransfer,
    },
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

impl ModuleOnMessage<BBPistonTractionTransfer> for Traction {
    fn on_message(
        &self,
        backbone: &mut BBPistonTractionTransfer,
        msg: &lotus_script::message::Message,
    ) -> bool {
        self.transfer.on_message(backbone, msg)
    }
}

impl ModuleTick<BBTraction> for Traction {
    fn tick(&self, _: &mut BBTraction) {}
}

#[derive(Default)]
pub struct BBTraction {
    pub piston_traction: BBPistonTraction,
}

impl BackBoneResetInputOutput for BBTraction {
    fn reset(&mut self, reset_type: BackBoneResetType) {
        self.piston_traction.reset(reset_type);
    }
}
