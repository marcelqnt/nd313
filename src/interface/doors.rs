use lotus_extra::bb_system::{DomainInterface, basic::BackBone};
use lotus_extra::bb_system::vdv_dashboard::DoorLeafLockState;

use crate::{
    Backbone, ELECTRICITY_INDEX_BUS_2, Modules, MIN_THROTTLE_RELEASE_STOP_BRAKE,
    DOORS_MAX_SPEED_MPS,
};

#[derive(Default)]
pub struct Nd313DoorsInterface;

impl DomainInterface<Modules, Backbone> for Nd313DoorsInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let bb_doors = &mut backbone.doors;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_powersupply = &mut backbone.electricity;

        bb_doors.set_p_available(800_000.0);

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
}
