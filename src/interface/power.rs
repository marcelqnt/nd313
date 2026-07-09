use lotus_extra::bb_system::{DomainInterface, basic::BackBone, cockpit_enhanced::IgnitionSwitchStep};

use crate::{Backbone, ELECTRICITY_INDEX_BUS_1, ELECTRICITY_INDEX_BUS_2, Modules};

#[derive(Default)]
pub struct Nd313PowerInterface;

impl DomainInterface<Modules, Backbone> for Nd313PowerInterface {
    fn wire(&mut self, _modules: &Modules, backbone: &mut Backbone) {
        let bb_electricity = &mut backbone.electricity;

        backbone
            .cockpit
            .vdv_dashboard
            .ignition_switch
            .state()
            .call_on_changed(|state| {
                bb_electricity
                    .set_switch(ELECTRICITY_INDEX_BUS_1, state >= IgnitionSwitchStep::Step1);
                bb_electricity
                    .set_switch(ELECTRICITY_INDEX_BUS_2, state >= IgnitionSwitchStep::Step2);
            });
    }
}
