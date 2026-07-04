use lotus_extra::{
    bb_system::{
        VehicleInterface, basic::BackBone, cockpit_enhanced::IgnitionSwitchStep,
        lights::IndicatorState,
    },
    math::IfElse,
    messages::{self, std::RetarderRequest},
};
use lotus_script::{message, prelude::*};

use crate::{Backbone, Modules};

const DOORS_MAX_SPEED_MPS: f32 = 3.0 / 3.6;
const MIN_THROTTLE_RELEASE_STOP_BRAKE: f32 = 0.1;

impl VehicleInterface<Backbone> for Modules {
    fn after_init(&mut self, _backbone: &mut Backbone) {
        log::info!("Initializing script ==========================================");
        set_var("Lm_MasterError", 0.0);
    }

    fn tick_interface(&mut self, backbone: &mut Backbone) {
        self.powersupply_input(backbone);

        self.pneumatics_input(backbone);

        self.traction_input(backbone);

        self.steering_input(backbone);

        self.outsidelights_input(backbone);

        self.cockpit_input(backbone);

        self.doors_input(backbone);

        self.send_messages(backbone);
    }

    fn interface_on_message(&mut self, backbone: &mut Backbone, msg: &message::Message) {
        let _ = msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            backbone.cockpit.vdv_dashboard.current_gear.set(g);
            Ok(())
        });
    }
}

impl Modules {
    fn powersupply_input(&self, backbone: &mut Backbone) {
        let bb_powersupply = &mut backbone.powersupply;

        backbone
            .cockpit
            .vdv_dashboard
            .ignition_switch
            .state()
            .call_on_changed(|state| {
                bb_powersupply.set_main_relay(0, state >= IgnitionSwitchStep::Step1);
                bb_powersupply.set_main_relay(1, state >= IgnitionSwitchStep::Step2);
            });
    }

    fn pneumatics_input(&mut self, backbone: &mut Backbone) {
        backbone.pneumatics.n_engine_rpm = backbone.piston_traction_transfer.rpm;

        backbone.pneumatics.target_air_brake = backbone.throttle_brake_control.brake_value();

        backbone.pneumatics.target_stop_brake = backbone.doors.stop_brake().get_state();

        backbone
            .cockpit
            .parking_brake
            .state()
            .call_on_changed(|state| {
                backbone.pneumatics.sw_parkingbrake_pos = state.if_else(1.0, 0.0);
            });
    }

    fn traction_input(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.powersupply;

        // Engine Start/Stop:
        self.traction.piston.starter_relay(
            &mut backbone.traction.piston_traction,
            bb_cockpit.ignition_switch.state().get_state().into(),
            bb_powersupply.get_battery(0).unwrap(),
        );

        // Gearbox Mode
        bb_cockpit
            .automatic_gear_box_mode_switch_group
            .state()
            .call_on_changed(|state| {
                self.traction.piston.send_gearbox_mode(&state);
            });

        let retarder_request = (backbone.throttle_brake_control.brake_value() > 0.02).if_else(1, 0);

        if backbone.retarder_request.set_if_different(retarder_request) {
            self.traction
                .piston
                .send_retarder_request(&RetarderRequest(retarder_request));
        }
    }

    fn steering_input(&mut self, backbone: &mut Backbone) {
        backbone.steering.speed_mps_abs = backbone.axle.v_axle_mps().abs();
    }

    fn outsidelights_input(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_outside_lights = &mut backbone.outside_lights;
        let bus_2 = backbone.powersupply.bus_active(1);
        let voltage_available = backbone.powersupply.get_bus(0).unwrap().voltage_available();

        bb_outside_lights.set_voltage(backbone.powersupply.get_bus(1).unwrap().voltage_available());

        if voltage_available > 0.0 && bb_cockpit.flash_light_switch.state().get_state() {
            bb_outside_lights.set_indicator(IndicatorState::Warning);
        } else if !bus_2.get_state() {
            bb_outside_lights.set_indicator(IndicatorState::Off);
        } else {
            bb_outside_lights.set_indicator(bb_cockpit.indicator_switch.state().get_state().into());
        }

        let park_n_rear = bb_cockpit
            .modern_outside_light_switch
            .parking()
            .if_else(1.0, 0.0);

        let dim_light: f32 = bb_cockpit
            .modern_outside_light_switch
            .dim()
            .if_else(1.0, 0.0);

        let brake_light: f32 =
            (backbone.throttle_brake_control.brake_value() > 0.02).if_else(1.0, 0.0);

        bb_outside_lights
            .set_bulb_brightness(crate::BULB_INDEX_PARKNREAR, park_n_rear * voltage_available);
        bb_outside_lights.set_bulb_brightness(crate::BULB_INDEX_PARKNREAR_LED, park_n_rear);

        bb_outside_lights
            .set_bulb_brightness(crate::BULB_INDEX_DIMLIGHT, dim_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(
            crate::BULB_INDEX_DIMLIGHT_SCALE,
            dim_light.max(park_n_rear * 0.4) * voltage_available,
        );

        bb_outside_lights
            .set_bulb_brightness(crate::BULB_INDEX_DIMLIGHT_BLUE, 0.5 + 0.5 * dim_light);

        bb_outside_lights
            .set_bulb_brightness(crate::BULB_INDEX_BRAKE, brake_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(crate::BULB_INDEX_BRAKE_LED, brake_light);

        bb_outside_lights.set_bulb_brightness(
            crate::BULB_INDEX_REARNBRAKE,
            brake_light.max(park_n_rear * 0.7),
        );
    }

    fn cockpit_input(&mut self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_doors = &mut backbone.doors;

        bb_cockpit.voltage = backbone.powersupply.get_bus(0).unwrap().voltage();
        bb_cockpit.voltage_available = backbone.powersupply.get_bus(0).unwrap().voltage_available();

        bb_cockpit.pneumatics = backbone.pneumatics;

        bb_cockpit
            .indicator_switch
            .set_steering_normalized(backbone.steering.angle_normalized);

        backbone
            .outside_lights
            .blink_relay()
            .forward_on_changed(&mut bb_cockpit.indicators_bulbs);

        bb_doors.door_closed(0).call_on_changed(|closed| {
            bb_cockpit.il_doors_target[0].set(!closed);
        });

        bb_doors
            .door_closed(2)
            .call_on_changed_two(bb_doors.door_closed(3), |closed1, closed2| {
                bb_cockpit.rear_doors = !(closed1 && closed2);
            });

        bb_doors
            .stop_sign(2)
            .forward_on_changed(&mut bb_cockpit.stop_request_middle);

        bb_doors
            .stop_sign(3)
            .forward_on_changed(&mut bb_cockpit.stop_request_rear);

        bb_cockpit
            .engine_running
            .set_if_different(backbone.piston_traction_transfer.rpm > 100.0);
    }

    fn doors_input(&mut self, backbone: &mut Backbone) {
        let bb_doors = &mut backbone.doors;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.powersupply;

        bb_doors.set_p_available(800_000.0);

        // Stop brake and release

        bb_powersupply
            .bus_active(1)
            .forward_on_changed(&mut bb_doors.power_available);

        bb_doors.set_throttle_pressed(
            backbone.throttle_brake_control.throttle_value() > MIN_THROTTLE_RELEASE_STOP_BRAKE,
        );

        let vehicle_stopped = backbone.axle.v_axle_mps() < DOORS_MAX_SPEED_MPS;

        bb_doors
            .release_activatable
            .set_if_different(vehicle_stopped && backbone.powersupply.bus_active(1).get_state());

        bb_cockpit.btn_door_releases[0]
            .state()
            .forward_on_changed(bb_doors.release_target(0));
        bb_cockpit.btn_door_releases[0]
            .state()
            .forward_on_changed(bb_doors.stop_brake_target());

        bb_cockpit
            .btn_doors
            .first()
            .unwrap()
            .state()
            .call_on_changed(|pos| {
                if pos {
                    self.doors.toggle_door(bb_doors, 0);
                    self.doors.toggle_door(bb_doors, 1);
                }
            });

        bb_cockpit.btn_doors[1].state().call_on_changed(|pos| {
            self.doors.set_door_target(bb_doors, 2, pos);
        });

        bb_cockpit.btn_doors[2].state().call_on_changed(|pos| {
            self.doors.set_door_target(bb_doors, 3, pos);
        });
    }

    fn send_messages(&mut self, _backbone: &mut Backbone) {}
}
