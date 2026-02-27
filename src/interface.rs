use lotus_extra::{
    bb_system::{ElementTrait, cockpit_enhanced::IgnitionSwitchStep},
    messages::{self},
};
use lotus_script::{log, message};

use crate::MyScript;

impl MyScript {
    pub fn tick_interface(&mut self) {
        self.cockpit_output();

        self.power_supply_output();

        self.traction_output();
        self.throttle_brake_control_output();

        self.pneumatics_output();
    }

    fn power_supply_output(&mut self) {
        if let Some(electricity_available) = self.backbone.powersupply.bus_active_refreshed(0) {
            self.backbone
                .set_electricity_available(electricity_available);
        }
    }

    fn cockpit_output(&mut self) {
        let bb_traction = &mut self.backbone.traction;
        let bb_powersupply = &mut self.backbone.powersupply;
        let bb_cockpit = &mut self.backbone.cockpit;

        // Engine Start/Stop:
        self.traction.piston.starter_relay(
            &mut bb_traction.piston_traction,
            bb_cockpit.ignition_switch.state.get_state().0.into(),
            bb_powersupply.get_battery(0).unwrap(),
        );

        // Gearbox Mode
        if let Some(state) = bb_cockpit.get_gearbox_mode() {
            log::info!("Automatic Gear Box Mode Switch Group State: {:?}", state);
            self.traction.piston.send_gearbox_mode(&state);

            // self.backbone
            //     .cockpit
            //     .automatic_gear_box_mode_switch_group
            //     .reset();
        }

        // Ignition Switch
        if let Some((state, _)) = bb_cockpit.ignition_switch.state.get_refreshed() {
            bb_powersupply.set_main_relay(0, state >= IgnitionSwitchStep::Step1);
            bb_powersupply.set_main_relay(1, state >= IgnitionSwitchStep::Step2);
        }
    }

    fn pneumatics_output(&mut self) {
        self.backbone.cockpit.pneumatics = self.backbone.pneumatics;
    }

    fn throttle_brake_control_output(&mut self) {
        self.backbone.pneumatics.target_air_brake =
            self.backbone.throttle_brake_control.brake_value.get_state();
    }

    fn traction_output(&mut self) {
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
