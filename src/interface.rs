use lotus_extra::{
    bb_system::{
        basic::ElementTrait, cockpit_enhanced::IgnitionSwitchStep, lights::IndicatorState,
    },
    messages::{self},
};
use lotus_script::{log, message};

use crate::MyScript;

impl MyScript {
    pub fn tick_interface(&mut self) {
        self.powersupply_in();

        self.pneumatics_in();

        self.traction_in();

        self.outsidelights_in();

        self.cockpit_in();
    }

    fn powersupply_in(&mut self) {
        let bb_powersupply = &mut self.backbone.powersupply;

        if let Some(state) = self.backbone.cockpit.ignition_switch.state.get_refreshed() {
            bb_powersupply.set_main_relay(0, state >= IgnitionSwitchStep::Step1);
            bb_powersupply.set_main_relay(1, state >= IgnitionSwitchStep::Step2);
        }
    }

    fn pneumatics_in(&mut self) {
        self.backbone.pneumatics.n_engine_rpm = self.backbone.piston_traction_transfer.rpm;

        self.backbone.pneumatics.target_air_brake =
            self.backbone.throttle_brake_control.brake_value.get_state();
    }

    fn traction_in(&mut self) {
        let bb_cockpit = &mut self.backbone.cockpit;
        let bb_powersupply = &mut self.backbone.powersupply;

        // Engine Start/Stop:
        self.traction.piston.starter_relay(
            &mut self.backbone.traction.piston_traction,
            bb_cockpit.ignition_switch.state.get_state().into(),
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
    }

    fn outsidelights_in(&mut self) {
        let bb_cockpit = &mut self.backbone.cockpit;
        let bb_outside_lights = &mut self.backbone.outside_lights;
        let bus_2 = self.backbone.powersupply.bus_active(1);

        bb_outside_lights.input.voltage = self
            .backbone
            .powersupply
            .get_bus(1)
            .unwrap()
            .voltage_available;

        if !bus_2 {
            bb_outside_lights.input.indicator = IndicatorState::Off;
        } else {
            bb_outside_lights.input.indicator = bb_cockpit.indicator_switch.get_state().into();
        }
    }

    fn cockpit_in(&mut self) {
        let bb_cockpit = &mut self.backbone.cockpit;

        if let Some(electricity_available) = self.backbone.powersupply.bus_active_refreshed(0) {
            bb_cockpit.set_electricity_available(electricity_available);
        }

        bb_cockpit.pneumatics = self.backbone.pneumatics;

        bb_cockpit
            .indicator_switch
            .set_steering_normalized(self.backbone.steering.angle_normalized);
    }

    pub fn interface_on_message(&mut self, msg: &message::Message) {
        msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            self.backbone.cockpit.current_gear.set(g);
            Ok(())
        })
        .unwrap();
    }
}
