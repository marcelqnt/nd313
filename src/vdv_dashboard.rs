use lotus_actor::{
    SystemUnit,
    actors::ActorHub,
    cockpit_enhanced::{IgnitionSwitchProperties, KeyPositions, ignition_switch},
};

use crate::HashMaps;

#[derive(Default)]
pub struct VdvDashboard;

impl SystemUnit<HashMaps> for VdvDashboard {
    fn post_init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        if let Some(ignition_state) = hash_maps.ignition_state {
            ignition_switch(
                hub,
                IgnitionSwitchProperties::builder()
                    .output(ignition_state)
                    .key_position_var("Key_Rotation")
                    .key_positions(KeyPositions::new(2.0, 1.0, 0.0, -1.0))
                    .sound("snd_KeyLock_Main")
                    .build(),
            );
        }
    }
}
