use lotus_extra::bb_system::{DomainInterface, basic::BackBone, cockpit_enhanced::IgnitionSwitchStep};

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313PowerInterface;

impl DomainInterface<Modules, Backbone> for Nd313PowerInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_electricity = &mut backbone.electricity;

        backbone
            .cockpit
            .vdv_dashboard
            .ignition_switch
            .state()
            .call_on_changed(|state| {
                bb_electricity.set_switch(idx.electricity_bus_1, state >= IgnitionSwitchStep::Step1);
                bb_electricity.set_switch(idx.electricity_bus_2, state >= IgnitionSwitchStep::Step2);
            });
    }
}
