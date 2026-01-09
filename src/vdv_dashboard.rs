use lotus_actor::{
    SystemUnit,
    actors::ActorHub,
    cockpit::{ButtonProperties, simple_button},
    cockpit_enhanced::{
        GearBoxModeSwitchGroupProperties, GearBoxModeSwitchGroupSwitch, IgnitionSwitchProperties,
        KeyPositions, gear_box_mode_switch_group, ignition_switch,
    },
    inputs::InputEvent,
    variables::VarWriter,
};
use lotus_extra::messages::std::AutomaticGearboxMode;

use crate::HashMaps;

#[derive(Default)]
pub struct VdvDashboard;

impl SystemUnit<HashMaps> for VdvDashboard {
    fn post_init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        let key_inserted = hub.insert_actor(VarWriter::<bool>::new("Key_Inserted"));

        simple_button(
            hub,
            ButtonProperties::builder()
                .input(InputEvent::new("InsertKey_Main", 1))
                .sound_press("snd_KeyMain_Insert")
                .sound_release("snd_KeyMain_Pull")
                .output(key_inserted)
                .springloaded(false)
                .build(),
        );

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

        if let Some(gear_box_mode_switch) = hash_maps.gear_box_mode_switch {
            gear_box_mode_switch_group(
                hub,
                GearBoxModeSwitchGroupProperties::builder()
                    .output(gear_box_mode_switch)
                    .sound_press_gear("snd_Sw_GearBox".to_string())
                    .sound_press_neutral("snd_Sw_GearBox".to_string())
                    .switches(vec![
                        GearBoxModeSwitchGroupSwitch::builder()
                            .mode(AutomaticGearboxMode::Reverse)
                            .input(InputEvent::new("Bus.GearBoxMode_R", 1))
                            .position(("Sw_GearBoxMode_R_Pos".to_string(), 1.0))
                            .build(),
                        GearBoxModeSwitchGroupSwitch::builder()
                            .mode(AutomaticGearboxMode::Neutral)
                            .input(InputEvent::new("Bus.GearBoxMode_N", 1))
                            .position(("Sw_GearBoxMode_N_Pos".to_string(), 1.0))
                            .build(),
                        GearBoxModeSwitchGroupSwitch::builder()
                            .mode(AutomaticGearboxMode::Drive)
                            .input(InputEvent::new("Bus.GearBoxMode_D", 1))
                            .position(("Sw_GearBoxMode_D_Pos".to_string(), 1.0))
                            .build(),
                    ])
                    .build(),
            );
        }
    }
}
