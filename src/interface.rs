use lotus_extra::{
    backbone::ElementTrait,
    cockpit_enhanced::IgnitionSwitchStep,
    messages::{self, std::EngineStartStop},
};
use lotus_script::{log, message};

use crate::MyScript;

impl MyScript {
    pub fn tick_interface(&mut self) {
        self.cockpit_output();

        self.power_supply_output();

        self.traction_pneumatics_tick();
        self.throttle_brake_control_output();

        self.pneumatics_output();
    }

    fn power_supply_output(&mut self) {
        if let Some(electricity_available) = self.backbone.powersupply.bus_active_refreshed(0) {
            self.backbone
                .cockpit
                .display
                .electricity_available
                .set(electricity_available);
        }
    }

    fn cockpit_output(&mut self) {
        // Engine Start/Stop:
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

        // Gearbox Mode
        if let Some((state, _)) = self
            .backbone
            .cockpit
            .automatic_gear_box_mode_switch_group
            .state
            .get_refreshed()
        {
            log::info!("Automatic Gear Box Mode Switch Group State: {:?}", state);
            self.traction.piston.send_gearbox_mode(&state);

            // self.backbone
            //     .cockpit
            //     .automatic_gear_box_mode_switch_group
            //     .reset();
        }

        // Ignition Switch
        let p = &mut self.backbone.powersupply;
        if let Some((state, _)) = self.backbone.cockpit.ignition_switch.state.get_refreshed() {
            p.get_bus(0)
                .unwrap()
                .main_relay
                .set(state >= IgnitionSwitchStep::Step1);
            p.get_bus(1)
                .unwrap()
                .main_relay
                .set(state >= IgnitionSwitchStep::Step2);
        }
    }

    fn pneumatics_output(&mut self) {
        self.backbone.cockpit.pneumatics = self.backbone.pneumatics;
    }

    fn throttle_brake_control_output(&mut self) {
        self.backbone.pneumatics.target_air_brake =
            self.backbone.throttle_brake_control.brake_value.get_state();
    }

    fn traction_pneumatics_tick(&mut self) {
        self.backbone.pneumatics.n_engine_rpm = self.backbone.piston_traction_transfer.rpm;
    }

    pub fn interface_on_message(&mut self, msg: &message::Message) {
        msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            self.backbone.cockpit.current_gear.set(g);
            Ok(())
        })
        .unwrap();
    }
}
