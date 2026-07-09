use lotus_extra::{
    bb_system::{
        VehicleInterface,
        basic::BackBone,
        cockpit_enhanced::IgnitionSwitchStep,
        lights::IndicatorState,
        vdv_dashboard::{DoorLeafLockState, VdvDisplayDoorState},
    },
    math::IfElse,
    messages::{
        self,
        std::{EngineStartStop, RetarderRequest},
    },
};
use lotus_script::{message, prelude::*};

use crate::{
    Backbone, ELECTRICITY_INDEX_BUS_1, ELECTRICITY_INDEX_BUS_2,
    ELECTRICITY_INDEX_MIN_VOLTAGE_RELAY, Modules, NOMINAL_VOLTAGE,
};

const DOORS_MAX_SPEED_MPS: f32 = 3.0 / 3.6;
const MIN_THROTTLE_RELEASE_STOP_BRAKE: f32 = 0.1;

/// Verdrahtung zwischen den BB-Modulen des nd313 — getrennt von der Modul-Konfiguration.
#[derive(Default)]
pub struct Nd313Interface;

impl VehicleInterface<Modules, Backbone> for Nd313Interface {
    fn after_init(&mut self, _modules: &Modules, _backbone: &mut Backbone) {
        log::info!("Initializing script ==========================================");
        set_var("Lm_MasterError", 0.0);
    }

    fn tick_interface(&mut self, modules: &Modules, backbone: &mut Backbone) {
        self.powersupply_input(backbone);

        self.pneumatics_input(backbone);

        self.traction_input(modules, backbone);

        self.steering_input(backbone);

        self.outsidelights_input(backbone);

        self.cockpit_input(backbone);

        self.doors_input(modules, backbone);

        self.send_messages(backbone);
    }

    fn interface_on_message(
        &mut self,
        _modules: &Modules,
        backbone: &mut Backbone,
        msg: &message::Message,
    ) {
        let _ = msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            backbone.cockpit.vdv_dashboard.current_gear.set(g);
            Ok(())
        });
    }
}

impl Nd313Interface {
    fn powersupply_input(&self, backbone: &mut Backbone) {
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

    fn pneumatics_input(&self, backbone: &mut Backbone) {
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

    fn traction_input(&self, modules: &Modules, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_electricity = &mut backbone.electricity;

        // Engine Start/Stop:
        let elec_source = bb_electricity.unit_active(ELECTRICITY_INDEX_MIN_VOLTAGE_RELAY);

        if bb_cockpit.ignition_switch.state().changed() || elec_source.changed() {
            let state = if !elec_source.get_state() {
                EngineStartStop::None
            } else {
                bb_cockpit.ignition_switch.state().get_state().into()
            };

            modules
                .traction
                .piston
                .starter_relay(&mut backbone.traction.piston_traction, state);
        }

        // Gearbox Mode
        bb_cockpit
            .automatic_gear_box_mode_switch_group
            .state()
            .call_on_changed(|state| {
                modules.traction.piston.send_gearbox_mode(&state);
            });

        let retarder_request = (backbone.throttle_brake_control.brake_value() > 0.02).if_else(1, 0);

        if backbone.retarder_request.set_if_different(retarder_request) {
            modules
                .traction
                .piston
                .send_retarder_request(&RetarderRequest(retarder_request));
        }
    }

    fn steering_input(&self, backbone: &mut Backbone) {
        backbone.steering.speed_mps_abs = backbone.axle.v_axle_mps().abs();
    }

    fn outsidelights_input(&self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_outside_lights = &mut backbone.outside_lights;
        let bus_2 = backbone.electricity.unit_active(ELECTRICITY_INDEX_BUS_2);
        let voltage_available = backbone
            .electricity
            .unit_voltage_available(ELECTRICITY_INDEX_BUS_1)
            / NOMINAL_VOLTAGE;

        bb_outside_lights.set_unified_voltage(
            backbone
                .electricity
                .unit_voltage_available(ELECTRICITY_INDEX_BUS_2)
                / NOMINAL_VOLTAGE,
        );

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

    fn cockpit_input(&self, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_doors = &mut backbone.doors;

        bb_cockpit.unified_voltage =
            backbone.electricity.unit_voltage(ELECTRICITY_INDEX_BUS_1) / NOMINAL_VOLTAGE;
        bb_cockpit.unified_voltage_available = backbone
            .electricity
            .unit_voltage_available(ELECTRICITY_INDEX_BUS_1)
            / NOMINAL_VOLTAGE;

        bb_cockpit.pneumatics = backbone.pneumatics;

        bb_cockpit
            .indicator_switch
            .set_steering_normalized(backbone.steering.angle_normalized);

        backbone
            .outside_lights
            .blink_relay()
            .forward_on_changed(&mut bb_cockpit.indicators_bulbs);

        bb_doors.door_closed(0, 0).call_on_changed_two(
            bb_doors.door_closed(0, 1),
            |closed1, closed2| {
                bb_cockpit.il_doors_target[0].set(!(closed1 && closed2));
            },
        );

        bb_doors.door_closed(1, 0).call_on_changed_two(
            bb_doors.door_closed(2, 0),
            |closed1, closed2| {
                bb_cockpit.rear_doors = !(closed1 && closed2);
            },
        );

        bb_doors
            .stop_sign(1)
            .forward_on_changed(&mut bb_cockpit.stop_request_middle);

        bb_doors
            .stop_sign(2)
            .forward_on_changed(&mut bb_cockpit.stop_request_rear);

        bb_doors
            .stop_brake()
            .forward_on_changed(&mut bb_cockpit.stop_brake);

        let door_released =
            bb_doors.release_target(0).get_state() && bb_doors.release_activatable.get_state();

        bb_cockpit
            .display_door_1_1
            .set_if_different(vdv_display_door_state(
                bb_doors.door_closed(0, 0).get_state(),
                bb_doors.wing_lock(0, 0).get_state(),
                door_released,
            ));
        bb_cockpit
            .display_door_1_2
            .set_if_different(vdv_display_door_state(
                bb_doors.door_closed(0, 1).get_state(),
                bb_doors.wing_lock(0, 1).get_state(),
                door_released,
            ));
        bb_cockpit
            .display_door_2
            .set_if_different(vdv_display_door_state(
                bb_doors.door_closed(1, 0).get_state(),
                false,
                door_released,
            ));
        bb_cockpit
            .display_door_3
            .set_if_different(vdv_display_door_state(
                bb_doors.door_closed(2, 0).get_state(),
                false,
                door_released,
            ));

        bb_cockpit
            .engine_running
            .set_if_different(backbone.piston_traction_transfer.rpm > 100.0);
    }

    fn doors_input(&self, modules: &Modules, backbone: &mut Backbone) {
        let bb_doors = &mut backbone.doors;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.electricity;

        bb_doors.set_p_available(800_000.0);

        // Stop brake and release

        bb_powersupply
            .unit_active(ELECTRICITY_INDEX_BUS_2)
            .forward_on_changed(&mut bb_doors.power_available);

        bb_doors.set_throttle_pressed(
            backbone.throttle_brake_control.throttle_value() > MIN_THROTTLE_RELEASE_STOP_BRAKE,
        );

        let vehicle_stopped = backbone.axle.v_axle_mps() < DOORS_MAX_SPEED_MPS;

        bb_doors.release_activatable.set_if_different(
            vehicle_stopped
                && backbone
                    .electricity
                    .unit_active(ELECTRICITY_INDEX_BUS_2)
                    .get_state(),
        );

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
                    modules.doors.toggle_door(bb_doors, 0);
                }
            });

        bb_cockpit.btn_doors[1].state().call_on_changed(|pos| {
            modules.doors.set_door_target(bb_doors, 1, pos);
        });

        bb_cockpit.btn_doors[2].state().call_on_changed(|pos| {
            modules.doors.set_door_target(bb_doors, 2, pos);
        });

        bb_cockpit
            .sw_door_leaf_lock
            .state()
            .call_on_changed(|state| {
                bb_doors
                    .wing_lock(0, 0)
                    .set(state == DoorLeafLockState::Left);
                bb_doors
                    .wing_lock(0, 1)
                    .set(state == DoorLeafLockState::Right);
            });
    }

    fn send_messages(&self, _backbone: &mut Backbone) {}
}

fn vdv_display_door_state(closed: bool, blocked: bool, released: bool) -> VdvDisplayDoorState {
    if blocked {
        VdvDisplayDoorState::Blocked
    } else if !closed {
        VdvDisplayDoorState::Open
    } else if released {
        VdvDisplayDoorState::Released
    } else {
        VdvDisplayDoorState::Closed
    }
}
