use lotus_extra::{
    bb_system::{DomainInterface, basic::BackBone, cockpit::TwoStepState},
    math::IfElse,
};

use crate::{Backbone, Modules, NOMINAL_VOLTAGE};

#[derive(Default)]
pub struct Nd313InsideLightsInterface;

impl DomainInterface<Modules, Backbone> for Nd313InsideLightsInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_inside_lights = &mut backbone.inside_lights;
        let bb_cockpit = &mut backbone.cockpit;
        let voltage_available = backbone
            .electricity
            .unit_voltage(idx.electricity_min_voltage_relay)
            / NOMINAL_VOLTAGE;

        bb_inside_lights.lower_deck_input = (bb_cockpit
            .vdv_dashboard
            .inside_light_step_switch
            .state()
            .get_state()
            != TwoStepState::Off)
            .if_else(voltage_available, 0.0);

        bb_inside_lights.lower_front_right_input = (bb_cockpit
            .vdv_dashboard
            .inside_light_step_switch
            .state()
            .get_state()
            == TwoStepState::StepB)
            .if_else(voltage_available, 0.0);

        bb_inside_lights.upper_deck_input = bb_cockpit
            .vdv_dashboard
            .inside_light_secondary_switch
            .state()
            .get_state()
            .if_else(voltage_available, 0.0);
    }
}
