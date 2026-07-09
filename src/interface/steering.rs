use lotus_extra::bb_system::DomainInterface;

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313SteeringInterface;

impl DomainInterface<Modules, Backbone> for Nd313SteeringInterface {
    fn wire(&mut self, _modules: &Modules, backbone: &mut Backbone) {
        backbone.steering.speed_mps_abs = backbone.axle.v_axle_mps().abs();
    }
}
