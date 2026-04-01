use lotus_extra::{
    bb_system::{
        basic::{BackBone, BackBoneForwarding},
        cockpit_enhanced::IgnitionSwitchStep,
        lights::{IndicatorState, OutsideLightKind},
    },
    math::IfElse,
    messages::{self, std::RetarderRequest},
};
use lotus_script::{log, message};

use crate::{Backbone, Modules};

const DOORS_MAX_SPEED_MPS: f32 = 3.0 / 3.6;
const MIN_THROTTLE_RELEASE_STOP_BRAKE: f32 = 0.1;

impl Modules {
    pub fn tick_interface(&mut self, backbone: &mut Backbone) {
        self.powersupply_in(backbone);

        self.pneumatics_in(backbone);

        self.traction_in(backbone);

        self.outsidelights_in(backbone);

        self.cockpit_in(backbone);

        self.doors_in(backbone);
    }

    fn powersupply_in(&self, backbone: &mut Backbone) {
        let bb_powersupply = &mut backbone.powersupply;

        if let Some(state) = backbone
            .cockpit
            .vdv_dashboard
            .ignition_switch
            .state
            .get_if_changed()
        {
            bb_powersupply.set_main_relay(0, state >= IgnitionSwitchStep::Step1);
            bb_powersupply.set_main_relay(1, state >= IgnitionSwitchStep::Step2);
        }
    }

    fn pneumatics_in(&mut self, backbone: &mut Backbone) {
        backbone.pneumatics.n_engine_rpm = backbone.piston_traction_transfer.rpm;

        backbone.pneumatics.target_air_brake =
            backbone.throttle_brake_control.brake_value.get_state();

        backbone.pneumatics.target_stop_brake =
            backbone.doors.stop_brake_controller.state.get_state();

        if let Some(state) = backbone.cockpit.parking_brake.switch.state.get_if_changed() {
            log::info!("(A) parking_brake_switch: {:?}", state);
        }

        backbone
            .cockpit
            .parking_brake
            .switch
            .state
            .call_on_changed(|state| {
                log::info!("parking_brake_switch: {:?}", state);
                backbone.pneumatics.sw_parkingbrake_pos = state.if_else(1.0, 0.0);
            });
    }

    fn traction_in(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.powersupply;

        // Engine Start/Stop:
        self.traction.piston.starter_relay(
            &mut backbone.traction.piston_traction,
            bb_cockpit.ignition_switch.state.get_state().into(),
            bb_powersupply.get_battery(0).unwrap(),
        );

        // Gearbox Mode
        if let Some(state) = bb_cockpit.get_gearbox_mode() {
            self.traction.piston.send_gearbox_mode(&state);
        }

        if let Some(brake_value) = backbone.throttle_brake_control.brake_value.get_if_changed() {
            let retarder_request = (brake_value > 0.02).if_else(1, 0);

            if backbone.retarder_request.set_if_different(retarder_request) {
                self.traction
                    .piston
                    .send_retarder_request(&RetarderRequest(retarder_request));
            }
        }
    }

    fn outsidelights_in(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_outside_lights = &mut backbone.outside_lights;
        let bus_2 = backbone.powersupply.bus_active(1);

        bb_outside_lights.input.voltage =
            backbone.powersupply.get_bus(1).unwrap().voltage_available;

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

    fn cockpit_in(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;

        if let Some(electricity_available) = backbone.powersupply.bus_active(0).get_if_changed() {
            bb_cockpit.set_electricity_available(electricity_available);
        }

        bb_cockpit.voltage_available = backbone.powersupply.get_bus(0).unwrap().voltage;

        bb_cockpit.pneumatics = backbone.pneumatics;

        bb_cockpit
            .indicator_switch
            .set_steering_normalized(backbone.steering.angle_normalized);

        backbone
            .outside_lights
            .indicator
            .blink_relay
            .state
            .forward_on_changed(&mut bb_cockpit.indicators_bulbs);

        backbone.doors.doors[2]
            .stop_sign
            .forward_on_changed(&mut bb_cockpit.stop_request_middle);

        backbone.doors.doors[3]
            .stop_sign
            .forward_on_changed(&mut bb_cockpit.stop_request_rear);
    }

    fn doors_in(&mut self, backbone: &mut Backbone) {
        let bb_doors = &mut backbone.doors;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.powersupply;

        bb_doors.set_p_available(800_000.0);

        // Stop brake and release

        if let Some(power_available) = bb_powersupply.bus_active(1).get_if_changed() {
            bb_doors.stop_brake_controller_conditions.power_available = power_available;
            bb_doors.power_available.set_if_different(power_available);
        }

        if let Some(throttle_pedal) = backbone
            .throttle_brake_control
            .throttle_value
            .get_if_changed()
        {
            bb_doors
                .stop_brake_controller_conditions
                .throttle_pedal_pressed = throttle_pedal > MIN_THROTTLE_RELEASE_STOP_BRAKE;

            log::info!("throttle_pedal: {:?}", throttle_pedal);
        }

        let vehicle_stopped = backbone.axle.v_axle_mps() < DOORS_MAX_SPEED_MPS;

        bb_doors
            .release_activatable
            .set_if_different(vehicle_stopped && backbone.powersupply.bus_active(1).get_state());
        bb_doors.stop_brake_controller_conditions.vehicle_stopped = vehicle_stopped;

        bb_cockpit.btn_door_releases[0].forward_on_changed(&mut bb_doors.releases[0].target);
        bb_cockpit.btn_door_releases[0]
            .forward_on_changed(&mut bb_doors.stop_brake_controller.target);

        bb_cockpit
            .btn_doors
            .first()
            .unwrap()
            .call_on_changed(|pos| {
                if pos {
                    self.doors.toggle_door(bb_doors, 0);
                    self.doors.toggle_door(bb_doors, 1);
                }
            });

        bb_cockpit.btn_doors[1].call_on_changed(|pos| {
            self.doors.set_door_target(bb_doors, 2, pos);
        });

        bb_cockpit.btn_doors[2].call_on_changed(|pos| {
            self.doors.set_door_target(bb_doors, 3, pos);
        });
    }

    pub fn interface_on_message(&mut self, backbone: &mut Backbone, msg: &message::Message) {
        msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            backbone.cockpit.vdv_dashboard.current_gear.set(g);
            Ok(())
        })
        .unwrap();
    }
}
