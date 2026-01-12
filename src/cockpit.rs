use lotus_extra::{
    backbone::{BackBoneTick, ElementTraitReset},
    cockpit::{Button, ButtonBehaviour},
    cockpit_enhanced::{
        AutomaticGearBoxModeSwitchGroupSwitch, AutomaticGearBoxModeSwitchProperties, Cockpit,
        IgnitionSwitchProperties, KeyPositions, VdvDashboard, automatic_gear_box_mode_switch,
        ignition_switch,
    },
    input::InputEvent,
    messages::std::AutomaticGearboxMode,
};

pub struct CockpitNd313 {
    pub vdv_dashboard: VdvDashboard,
}

impl Default for CockpitNd313 {
    fn default() -> Self {
        Self {
            vdv_dashboard: VdvDashboard::builder()
                .ignition_key(
                    Button::builder()
                        .visibility_change("Key_Inserted")
                        .input(InputEvent::new("InsertKey_Main", 0))
                        .sound_press("snd_KeyMain_Insert")
                        .sound_release("snd_KeyMain_Pull")
                        .behaviour(ButtonBehaviour::OnOff)
                        .build(),
                )
                .ignition_switch(ignition_switch(
                    IgnitionSwitchProperties::builder()
                        .key_position_var("Key_Rotation")
                        .key_positions(KeyPositions {
                            off: 2.0,
                            step1: 1.0,
                            step2: 0.0,
                            starter: -1.0,
                        })
                        .sound("snd_KeyLock_Main")
                        .build(),
                ))
                .automatic_gear_box_mode_switch_group(automatic_gear_box_mode_switch(
                    AutomaticGearBoxModeSwitchProperties::builder()
                        .sound_press_gear("snd_Sw_GearBox")
                        .sound_press_neutral("snd_Sw_GearBox")
                        .switches(vec![
                            AutomaticGearBoxModeSwitchGroupSwitch::builder()
                                .mode(AutomaticGearboxMode::Reverse)
                                .input(InputEvent::new("Bus.GearBoxMode_R", 0))
                                .position(("Sw_GearBoxMode_R_Pos".to_string(), 1.0))
                                .build(),
                            AutomaticGearBoxModeSwitchGroupSwitch::builder()
                                .mode(AutomaticGearboxMode::Neutral)
                                .input(InputEvent::new("Bus.GearBoxMode_N", 0))
                                .position(("Sw_GearBoxMode_N_Pos".to_string(), 1.0))
                                .build(),
                            AutomaticGearBoxModeSwitchGroupSwitch::builder()
                                .mode(AutomaticGearboxMode::Drive)
                                .input(InputEvent::new("Bus.GearBoxMode_D", 0))
                                .position(("Sw_GearBoxMode_D_Pos".to_string(), 1.0))
                                .build(),
                        ])
                        .build(),
                ))
                .btn_doors(vec![
                    Button::builder()
                        .input(InputEvent::new("Door1Toggle", 0))
                        .position(("Btn_Door1_Pos".to_string(), 1.0))
                        .sound_press("snd_Btn_Door1_Press")
                        .sound_release("snd_Btn_Door1_Release")
                        .behaviour(ButtonBehaviour::SpringLoaded)
                        .build(),
                ])
                .build(),
        }
    }
}

impl BackBoneTick<Cockpit> for CockpitNd313 {
    fn tick(&self, backbone: &mut Cockpit) {
        self.vdv_dashboard.tick(backbone);

        if let Some(door) = backbone.doors.get_mut(0) {
            door.reset();
        }
    }
}
