use lotus_extra::bb_system::vdv_dashboard::VdvDisplayDoorState;
use lotus_extra::bb_system::{basic::BackBone, DomainInterface};

use crate::{Backbone, Modules, NOMINAL_VOLTAGE};

#[derive(Default)]
pub struct Nd313CockpitInterface;

impl DomainInterface<Modules, Backbone> for Nd313CockpitInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_doors = &mut backbone.doors;

        bb_cockpit.unified_voltage =
            backbone.electricity.unit_voltage(idx.electricity_bus_1) / NOMINAL_VOLTAGE;
        bb_cockpit.unified_voltage_available = backbone
            .electricity
            .unit_voltage_available(idx.electricity_bus_1)
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
