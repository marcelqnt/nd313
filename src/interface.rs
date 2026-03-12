use lotus_extra::{
    bb_system::{
        basic::{BackBone, BackBoneForwarding},
        cockpit_enhanced::IgnitionSwitchStep,
        lights::{IndicatorState, OutsideLightKind},
    },
    math::IfElse,
    messages::{self},
};
use lotus_script::{log, message};

use crate::MyScript;

const DOORS_MAX_SPEED_MPS: f32 = 3.0 / 3.6;
const MIN_THROTTLE_RELEASE_STOP_BRAKE: f32 = 0.1;

impl MyScript {
    pub fn tick_interface(&mut self) {
        self.powersupply_in();

        self.pneumatics_in();

        self.traction_in();

        self.outsidelights_in();

        self.cockpit_in();

        self.doors_in();
    }

    fn powersupply_in(&mut self) {
        let bb_powersupply = &mut self.backbone.powersupply;

        if let Some(state) = self.backbone.cockpit.ignition_switch.state.get_if_changed() {
            bb_powersupply.set_main_relay(0, state >= IgnitionSwitchStep::Step1);
            bb_powersupply.set_main_relay(1, state >= IgnitionSwitchStep::Step2);
        }
    }

    fn pneumatics_in(&mut self) {
        self.backbone.pneumatics.n_engine_rpm = self.backbone.piston_traction_transfer.rpm;

        self.backbone.pneumatics.target_air_brake =
            self.backbone.throttle_brake_control.brake_value.get_state();

        self.backbone.pneumatics.target_stop_brake =
            self.backbone.doors.stop_brake_controller.state.get_state();
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

        if !bus_2.get_state() {
            bb_outside_lights.input.indicator = IndicatorState::Off;
        } else {
            bb_outside_lights.input.indicator = bb_cockpit.indicator_switch.get_state().into();
        }

        bb_outside_lights.input.lights[OutsideLightKind::Parking.as_index()] = bb_cockpit
            .modern_outside_light_switch
            .parking()
            .if_else(1.0, 0.0);
        bb_outside_lights.input.lights[OutsideLightKind::Dim.as_index()] = bb_cockpit
            .modern_outside_light_switch
            .dim()
            .if_else(1.0, 0.0);
    }

    fn cockpit_in(&mut self) {
        let bb_cockpit = &mut self.backbone.cockpit;

        if let Some(electricity_available) =
            self.backbone.powersupply.bus_active(0).get_if_changed()
        {
            bb_cockpit.set_electricity_available(electricity_available);
        }

        bb_cockpit.pneumatics = self.backbone.pneumatics;

        bb_cockpit
            .indicator_switch
            .set_steering_normalized(self.backbone.steering.angle_normalized);
    }

    fn doors_in(&mut self) {
        let bb_doors = &mut self.backbone.doors;
        let bb_cockpit = &mut self.backbone.cockpit;
        let bb_powersupply = &mut self.backbone.powersupply;

        bb_doors.set_p_available(800_000.0);

        // Stop brake and release

        bb_powersupply
            .bus_active(1)
            .copy_on_changed(&mut bb_doors.stop_brake_controller_conditions.power_available);

        if let Some(throttle_pedal) = self
            .backbone
            .throttle_brake_control
            .throttle_value
            .get_if_changed()
        {
            bb_doors
                .stop_brake_controller_conditions
                .throttle_pedal_pressed = throttle_pedal > MIN_THROTTLE_RELEASE_STOP_BRAKE;

            log::info!("throttle_pedal: {:?}", throttle_pedal);
        }

        let vehicle_stopped = self.axles[1].v_axle_mps() < DOORS_MAX_SPEED_MPS;

        bb_doors.release_activatable.set_if_different(
            vehicle_stopped && self.backbone.powersupply.bus_active(1).get_state(),
        );
        bb_doors.stop_brake_controller_conditions.vehicle_stopped = vehicle_stopped;

        bb_cockpit.btn_door_releases[0].forward_on_changed(&mut bb_doors.releases[0].target);
        bb_cockpit.btn_door_releases[0]
            .forward_on_changed(&mut bb_doors.stop_brake_controller.target);

        // door 1

        if let Some(pos) = bb_cockpit.btn_doors.first().unwrap().get_if_changed()
            && pos
        {
            self.doors.toggle_door(bb_doors, 0);
            self.doors.toggle_door(bb_doors, 1);
        }
    }

    pub fn interface_on_message(&mut self, msg: &message::Message) {
        msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            self.backbone.cockpit.current_gear.set(g);
            Ok(())
        })
        .unwrap();
    }
}
