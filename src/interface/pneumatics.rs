use lotus_extra::bb_system::{DomainInterface, basic::BackBone};
use lotus_extra::math::IfElse;

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313PneumaticsInterface;

impl DomainInterface<Modules, Backbone> for Nd313PneumaticsInterface {
    fn wire(&mut self, _modules: &Modules, backbone: &mut Backbone) {
        backbone.pneumatics.n_engine_rpm = backbone.piston_traction_transfer.rpm;

        backbone.pneumatics.target_air_brake = backbone.throttle_brake_control.brake_value();

        backbone.pneumatics.target_stop_brake = backbone.doors.stop_brake().get_state();

        backbone
            .cockpit
            .parking_brake
            .state()
            .call_on_changed(|state| {
                backbone.pneumatics.sw_parkingbrake_pos = state.if_else(1.0, 0.0);
            });
    }
}
