use lotus_extra::bb_system::{
    DomainInterface, basic::BackBone, cockpit_enhanced::IgnitionSwitchStep,
};

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313PowerInterface;

impl DomainInterface<Modules, Backbone> for Nd313PowerInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_electricity = &mut backbone.electricity;
        let bb_cockpit = &mut backbone.cockpit;
        let ignition = bb_cockpit.vdv_dashboard.ignition_switch.state().get_state();

        bb_electricity.set_switch(idx.electricity_battery, true);
        bb_electricity.set_switch(idx.electricity_min_voltage_relay, true);
        bb_electricity.set_switch(idx.electricity_bus_1, ignition >= IgnitionSwitchStep::Step1);
        bb_electricity.set_switch(idx.electricity_bus_2, ignition >= IgnitionSwitchStep::Step2);
    }
}
