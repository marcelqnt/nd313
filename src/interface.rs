use lotus_extra::{
    backbone::{ElementTrait, ElementTraitReset},
    cockpit_enhanced::IgnitionSwitchStep,
    messages::std::EngineStartStop,
};
use lotus_script::log;

use crate::MyScript;

impl MyScript {
    pub fn tick_interface(&mut self) {
        self.cockpit_power_supply_tick();
        self.cockpit_traction_tick();
    }

    fn cockpit_power_supply_tick(&mut self) {
        let p = &mut self.backbone.powersupply;
        if let Some((state, i)) = self.backbone.cockpit.ignition_switch.state.get_refreshed() {
            log::info!("Ignition Switch State: {}", i);

            p.get_bus(0)
                .unwrap()
                .main_relay
                .set(state >= IgnitionSwitchStep::Step1);
            p.get_bus(1)
                .unwrap()
                .main_relay
                .set(state >= IgnitionSwitchStep::Step2);

            self.backbone.cockpit.ignition_switch.reset();
        }
    }

    fn cockpit_traction_tick(&mut self) {
        let input = match self.backbone.cockpit.ignition_switch.state.get_state().0 {
            IgnitionSwitchStep::Off | IgnitionSwitchStep::Step1 => EngineStartStop::Stop,
            IgnitionSwitchStep::Step2 => EngineStartStop::None,
            IgnitionSwitchStep::Starter => EngineStartStop::Start,
        };
        self.traction.piston.starter_relay(
            &mut self.backbone.traction.piston_traction,
            input,
            self.backbone.powersupply.get_battery(0).unwrap(),
        );
        self.backbone.cockpit.ignition_key.reset();

        if let Some((state, _)) = self
            .backbone
            .cockpit
            .automatic_gear_box_mode_switch_group
            .state
            .get_refreshed()
        {
            log::info!("Automatic Gear Box Mode Switch Group State: {:?}", state);
            self.traction.piston.send_gearbox_mode(&state);

            self.backbone
                .cockpit
                .automatic_gear_box_mode_switch_group
                .reset();
        }
    }
}
