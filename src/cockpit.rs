use lotus_extra::{
    bb_system::{
        basic::ModuleTick,
        cockpit::{Button, ButtonBehaviour},
        cockpit_enhanced::{
            AutomaticGearBoxModeSwitchGroupSwitch, AutomaticGearBoxModeSwitchProperties,
            IndicatorSwitch, IndicatorSwitchProperties, automatic_gear_box_mode_switch,
        },
        road_vehicle::BBRoadVehiclePneumatics,
        vdv_dashboard::{BBVdvDashboard, VdvDashboard},
        vdv_display::{VdvBusType, VdvDisplay, VdvDisplayProperties, VdvRampType},
    },
    input::InputEvent,
    messages::std::AutomaticGearboxMode,
};

pub struct CockpitNd313 {
    pub vdv_dashboard: VdvDashboard<BBRoadVehiclePneumatics>,
}

impl Default for CockpitNd313 {
    fn default() -> Self {
        Self {
            vdv_dashboard: VdvDashboard::default()
                .add_std_ignition_key()
                .add_std_ignition_switch()
                .add_automatic_gear_box_mode_switch_group(automatic_gear_box_mode_switch(
                    AutomaticGearBoxModeSwitchProperties::new(
                        "snd_Sw_GearBox",
                        "snd_Sw_GearBox",
                        vec![
                            AutomaticGearBoxModeSwitchGroupSwitch::new(
                                AutomaticGearboxMode::Reverse,
                                InputEvent::new("Bus.GearBoxMode_R", 0),
                                ("Sw_GearBoxMode_R_Pos".to_string(), 1.0),
                            ),
                            AutomaticGearBoxModeSwitchGroupSwitch::new(
                                AutomaticGearboxMode::Neutral,
                                InputEvent::new("Bus.GearBoxMode_N", 0),
                                ("Sw_GearBoxMode_N_Pos".to_string(), 1.0),
                            ),
                            AutomaticGearBoxModeSwitchGroupSwitch::new(
                                AutomaticGearboxMode::Drive,
                                InputEvent::new("Bus.GearBoxMode_D", 0),
                                ("Sw_GearBoxMode_D_Pos".to_string(), 1.0),
                            ),
                        ],
                    ),
                ))
                .add_indicator_switch(IndicatorSwitch::new(
                    IndicatorSwitchProperties::new(
                        "Sw_Indicator_Pos",
                        "snd_Sw_Indicator_On",
                        "snd_Sw_Indicator_Off",
                    )
                    .with_automatic_off(
                        "snd_Sw_Indicator_Autooff",
                        "snd_Sw_Indicator_Autooff_Notch",
                        810.0,
                        30.0,
                    ),
                ))
                .add_std_outside_light_switch()
                .add_btn_door(
                    Button::new(ButtonBehaviour::SpringLoaded)
                        .with_input(InputEvent::new("Door1Toggle", 0))
                        .with_position_var(("Btn_Door1_Pos".to_string(), 1.0))
                        .with_sound_press("snd_Btn_Door1_Press")
                        .with_sound_release("snd_Btn_Door1_Release"),
                )
                .add_std_btn_door_release()
                .add_std_sw_door_leaf_lock()
                .add_btn_display_change_mode(
                    Button::new(ButtonBehaviour::SpringLoaded)
                        .with_input(InputEvent::new("DisplayChange", 0))
                        .with_position_var(("Sw_Display_Pos".to_string(), 1.0))
                        .with_sound_press("snd_StdTa_On")
                        .with_sound_release("snd_StdTa_Off"),
                )
                .add_btn_display_error(
                    Button::new(ButtonBehaviour::SpringLoaded)
                        .with_input(InputEvent::new("DisplayDiagnose", 0))
                        .with_position_var(("Sw_Display_Pos".to_string(), -1.0))
                        .with_sound_press("snd_StdTa_On")
                        .with_sound_release("snd_StdTa_Off"),
                )
                .add_display(VdvDisplay::new(VdvDisplayProperties::new(
                    VdvBusType::ThreeAxlesThreeDoors,
                    VdvRampType::High,
                    "DisplayIllumination".to_string(),
                    "TexID_CockpitDisplay".to_string(),
                ))),
        }
    }
}

impl ModuleTick<BBVdvDashboard<BBRoadVehiclePneumatics>> for CockpitNd313 {
    fn tick(&self, backbone: &mut BBVdvDashboard<BBRoadVehiclePneumatics>) {
        self.vdv_dashboard.tick(backbone);
    }
}
