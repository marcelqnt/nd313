use lotus_extra::{
    bb_system::{DomainInterface, basic::BackBone},
    math::IfElse,
    messages::std::{EngineStartStop, RetarderRequest},
};

use crate::{Backbone, ELECTRICITY_INDEX_MIN_VOLTAGE_RELAY, Modules};

#[derive(Default)]
pub struct Nd313TractionInterface;

impl DomainInterface<Modules, Backbone> for Nd313TractionInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_electricity = &mut backbone.electricity;

        let elec_source = bb_electricity.unit_active(ELECTRICITY_INDEX_MIN_VOLTAGE_RELAY);

        if bb_cockpit.ignition_switch.state().changed() || elec_source.changed() {
            let state = if !elec_source.get_state() {
                EngineStartStop::None
            } else {
                bb_cockpit.ignition_switch.state().get_state().into()
            };

            modules
                .traction
                .piston
                .starter_relay(&mut backbone.traction.piston_traction, state);
        }

        bb_cockpit
            .automatic_gear_box_mode_switch_group
            .state()
            .call_on_changed(|state| {
                modules.traction.piston.send_gearbox_mode(&state);
            });

        let retarder_request = (backbone.throttle_brake_control.brake_value() > 0.02).if_else(1, 0);

        if backbone.retarder_request.set_if_different(retarder_request) {
            modules
                .traction
                .piston
                .send_retarder_request(&RetarderRequest(retarder_request));
        }
    }
}
