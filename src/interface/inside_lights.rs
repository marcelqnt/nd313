use lotus_extra::{
    bb_system::{DomainInterface, basic::BackBone, cockpit::TwoStepState},
    math::IfElse,
};

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313InsideLightsInterface;

impl DomainInterface<Modules, Backbone> for Nd313InsideLightsInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_electricity = &mut backbone.electricity;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;

        let step_switch = bb_cockpit.inside_light_step_switch.state().get_state();
        let secondary_switch = bb_cockpit.inside_light_secondary_switch.state().get_state();

        modules.electricity.set_lamp_input(
            bb_electricity,
            idx.lamp_inside_lower_deck,
            (step_switch != TwoStepState::Off).if_else(1.0, 0.0),
        );
        modules.electricity.set_lamp_input(
            bb_electricity,
            idx.lamp_inside_lower_front_right,
            (step_switch == TwoStepState::StepB).if_else(1.0, 0.0),
        );
        modules.electricity.set_lamp_input(
            bb_electricity,
            idx.lamp_inside_upper_deck,
            secondary_switch.if_else(1.0, 0.0),
        );
    }
}
