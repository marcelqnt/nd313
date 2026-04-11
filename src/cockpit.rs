use lotus_extra::{
    bb_system::{
        basic::{BackBoneResetInputOutput, BackBoneResetType, ModuleInit, ModuleTick},
        cockpit::{Button, ButtonBehaviour},
        cockpit_enhanced::{
            AutomaticGearBoxModeSwitchGroupSwitch, AutomaticGearBoxModeSwitchProperties,
            BBPneumaticHandbrakeLever, IndicatorSwitch, IndicatorSwitchProperties,
            PneumaticHandbrakeLever, PneumaticHandbrakeLeverProperties,
            automatic_gear_box_mode_switch,
        },
        road_vehicle::BBRoadVehiclePneumatics,
        vdv_dashboard::{BBVdvDashboard, VdvDashboard, standard_switch},
        vdv_display::{VdvBusType, VdvDisplay, VdvDisplayProperties, VdvRampType},
    },
    input::InputEvent,
    messages::std::AutomaticGearboxMode,
};

pub struct CockpitNd313 {
    pub vdv_dashboard: VdvDashboard<BBRoadVehiclePneumatics>,
    pub parking_brake: PneumaticHandbrakeLever,
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
                .add_std_retarder_switch()
                .add_std_asr_switch()
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
                .add_std_flash_light_switch()
                .add_std_outside_light_switch()
                .add_std_inside_light_step_switch()
                .add_std_inside_light_driver_switch()
                .add_std_inside_light_secondary_switch()
                .add_btn_door(
                    Button::new(ButtonBehaviour::SpringLoaded)
                        .with_input(InputEvent::new("Door1Toggle", 0))
                        .with_position_var(("Btn_Door1_Pos".to_string(), 1.0))
                        .with_sound_press("snd_Btn_Door1_Press")
                        .with_sound_release("snd_Btn_Door1_Release"),
                )
                .add_btn_door(standard_switch(
                    "Sw_Door2_Pos",
                    InputEvent::new("Door2Toggle", 0),
                ))
                .add_btn_door(standard_switch(
                    "Sw_Door3_Pos",
                    InputEvent::new("Door3Toggle", 0),
                ))
                .add_std_btn_door_release()
                .add_std_sw_door_leaf_lock()
                .add_std_btn_display_change_mode(1.0)
                .add_std_btn_display_brightness_plus(1.0)
                .add_std_btn_display_brightness_minus(-1.0)
                .add_std_btn_display_error(-1.0)
                .add_std_il_master_error()
                .add_std_il_master_warning()
                .add_std_il_high_beam()
                .add_std_il_indicators()
                .add_std_il_parking_brake()
                .add_std_il_abs()
                .add_std_il_stop_request_middle()
                .add_std_il_stop_request_rear()
                .add_std_il_wheelchair_request()
                .add_std_il_doors_rear()
                .add_display(VdvDisplay::new(VdvDisplayProperties::new(
                    VdvBusType::ThreeAxlesThreeDoors,
                    VdvRampType::High,
                    "DisplayIllumination".to_string(),
                    "TexID_CockpitDisplay".to_string(),
                )))
                .add_il_test(2.0),
            parking_brake: PneumaticHandbrakeLever::new(
                PneumaticHandbrakeLeverProperties::new_std(),
            ),
        }
    }
}

#[derive(Default)]
pub struct BBCockpitNd313 {
    pub vdv_dashboard: BBVdvDashboard<BBRoadVehiclePneumatics>,
    pub parking_brake: BBPneumaticHandbrakeLever,
}

impl ModuleTick<BBCockpitNd313> for CockpitNd313 {
    fn tick(&self, backbone: &mut BBCockpitNd313) {
        self.vdv_dashboard.tick(&mut backbone.vdv_dashboard);
        self.parking_brake.tick(&mut backbone.parking_brake);
    }
}

impl ModuleInit<BBCockpitNd313> for CockpitNd313 {
    fn init(&self, backbone: &mut BBCockpitNd313) {
        self.vdv_dashboard.init(&mut backbone.vdv_dashboard);
        self.parking_brake.init(&mut backbone.parking_brake);
    }
}

impl BackBoneResetInputOutput for BBCockpitNd313 {
    fn reset(&mut self, reset_type: BackBoneResetType) {
        self.vdv_dashboard.reset(reset_type);
        self.parking_brake.reset(reset_type);
    }
}
