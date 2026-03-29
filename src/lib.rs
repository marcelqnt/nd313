use lotus_extra::{
    bb_system::{
        self,
        basic::{BackBone, BackBoneResetInputOutput, BackBoneResetType, ModuleTick},
        doors::{BBDoors, DoorRelease, DoorUnit, Doors, PneumaticDoor, StopBrakeController},
        lights::{
            BBOutsideLights, Bulb, IndicatorLights, OutsideLightKind, OutsideLights, StandardLight,
        },
        piston_traction::BBPistonTractionTransfer,
        power::{BBPowerSupply, Battery, ElectricBus, PowerSupply},
        road_vehicle::{
            AxleProperties, BBRoadVehiclePneumatics, BBSteering, BBThrottleBrakeControl,
            RoadVehiclePneumatics, Steering, SteeringProperties, ThrottleBrakeControl,
        },
    },
    vehicle::Rattling,
};
use lotus_script::{Animation, prelude::*, vehicle::RoadWheel};

use crate::{
    cockpit::{BBCockpitNd313, CockpitNd313},
    traction::{BBTraction, Traction},
};

mod cockpit;
mod interface;
mod traction;

const WHEEL_DIAMETER: f32 = 0.9;

pub struct MyScript {
    backbone: Backbone,
    powersupply: PowerSupply,
    pneumatics: RoadVehiclePneumatics,
    traction: Traction,
    throttle_brake_control: ThrottleBrakeControl,
    outside_lights: OutsideLights,
    doors: Doors,
    cockpit: CockpitNd313,

    axles: Vec<AxleProperties>,
    steering: Steering,
    rattling: Rattling,

    _wheels: [RoadWheel; 2],
    // test: Input,
}

impl Default for MyScript {
    fn default() -> Self {
        let axles = vec![
            AxleProperties::new(0, WHEEL_DIAMETER),
            AxleProperties::new(1, WHEEL_DIAMETER)
                .with_tacho("DiffGear_mps".to_string())
                .with_traction(1, 5.74),
            AxleProperties::new(2, WHEEL_DIAMETER),
        ];

        let pneumatics = RoadVehiclePneumatics::builder()
            .axles(axles.clone())
            .build();

        Self {
            _wheels: [RoadWheel::get(1, 0).unwrap(), RoadWheel::get(1, 1).unwrap()],
            backbone: Backbone::default(),
            steering: Steering::new(SteeringProperties::new(40.0)),
            axles,
            powersupply: PowerSupply::new(
                vec![Battery],
                vec![
                    ElectricBus::new(vec![0], 0.75),
                    ElectricBus::new(vec![0], 0.75),
                ],
            ),

            pneumatics,
            throttle_brake_control: ThrottleBrakeControl::new(0, 1, 0.85),
            outside_lights: OutsideLights::default()
                .with_indicator(
                    IndicatorLights::new(
                        vec![
                            Bulb::new("Light_Indicator_Left".to_string())
                                .with_exp_fade_in_out((20.0, 15.0)),
                            Bulb::new("Light_Indicator_Left_LED".to_string()),
                        ],
                        vec![
                            Bulb::new("Light_Indicator_Right".to_string())
                                .with_exp_fade_in_out((20.0, 15.0)),
                            Bulb::new("Light_Indicator_Right_LED".to_string()),
                        ],
                        0.40,
                        0.35,
                        0.43,
                    )
                    .with_sound("snd_IndicatorRelayOn", "snd_IndicatorRelayOff"),
                )
                .with_light(
                    OutsideLightKind::Parking,
                    StandardLight::new(vec![
                        Bulb::new("Light_ParkingNRear".to_string())
                            .with_exp_fade_in_out((20.0, 15.0)),
                        Bulb::new("Light_ParkingNRear_LED".to_string()),
                    ]),
                )
                .with_light(
                    OutsideLightKind::Dim,
                    StandardLight::new(vec![
                        Bulb::new("Light_DimLight".to_string()).with_exp_fade_in_out((20.0, 15.0)),
                        Bulb::new("Light_DimLight_Scale".to_string())
                            .with_exp_fade_in_out((20.0, 15.0)),
                        Bulb::new("Light_DimLight_Blue".to_string())
                            .with_exp_fade_in_out((20.0, 15.0)),
                    ]),
                ),
            cockpit: CockpitNd313::default(),
            doors: Doors::default()
                .add_release(DoorRelease)
                .add_stop_brake_controller(
                    StopBrakeController::default()
                        .add_check_all_doors_closed()
                        .add_check_throttle_pedal()
                        .add_check_vehicle_stopped()
                        .add_sound_deactivate("snd_StopBrake_Release"),
                )
                .add_door(DoorUnit::new(
                    PneumaticDoor::new(
                        0.000_003_125,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.000_000_625,
                            p_end_normalized: 0.2,
                        },
                        (0.1, 0.1),
                    )
                    .with_friction(0.1)
                    .with_position_var("Door_1_1_Pos".to_string())
                    .with_sound_open("snd_Door11_Open")
                    .with_sound_close("snd_Door11_Close"),
                    Some(0),
                ))
                .add_door(DoorUnit::new(
                    PneumaticDoor::new(
                        0.000_003,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.000_000_5,
                            p_end_normalized: 0.21,
                        },
                        (0.105, 0.09),
                    )
                    .with_friction(0.11)
                    .with_position_var("Door_1_2_Pos".to_string())
                    .with_sound_open("snd_Door12_Open")
                    .with_sound_close("snd_Door12_Close"),
                    Some(0),
                ))
                .add_door(DoorUnit::new(
                    PneumaticDoor::new(
                        0.000_007_5,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.000_000_45,
                            p_end_normalized: 0.375,
                        },
                        (0.01, 0.01),
                    )
                    .with_friction(0.2)
                    .with_position_var("Door_2_1_Pos".to_string())
                    .with_sound_open("snd_Door2_Open")
                    .with_sound_close("snd_Door2_Close")
                    .with_sound_close_bump("snd_Door2_Close_End"),
                    Some(0),
                ))
                .add_door(DoorUnit::new(
                    PneumaticDoor::new(
                        0.000_007_5,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.000_000_45,
                            p_end_normalized: 0.375,
                        },
                        (0.01, 0.01),
                    )
                    .with_friction(0.2)
                    .with_position_var("Door_3_1_Pos".to_string())
                    .with_sound_open("snd_Door3_Open")
                    .with_sound_close("snd_Door3_Close")
                    .with_sound_close_bump("snd_Door3_Close_End"),
                    Some(0),
                )),
            traction: Traction::default(),
            rattling: Rattling::builder()
                .animation(Animation::get("Main").unwrap())
                .sound_variable("snd_Rattling")
                .build(),
        }
    }
}

script!(MyScript);

impl Script for MyScript {
    fn init(&mut self) {
        log::info!("Initializing script ==========================================");

        set_var("Lm_MasterError", 0.0);
    }

    fn tick(&mut self) {
        // log::info!(
        //     "self.wheels: {}/{} / {}/{}",
        //     self.wheels[0].axle_index(),
        //     self.wheels[0].wheel_index(),
        //     self.wheels[1].axle_index(),
        //     self.wheels[1].wheel_index()
        // );

        // tick is only necessary for the axle with traction

        self.backbone.reset(BackBoneResetType::Output);

        self.axles[1].tick();
        self.rattling.tick();

        self.pneumatics.tick(&mut self.backbone.pneumatics);
        self.throttle_brake_control
            .tick(&mut self.backbone.throttle_brake_control);
        self.steering.tick(&mut self.backbone.steering);
        self.cockpit.tick(&mut self.backbone.cockpit);
        self.powersupply.tick(&mut self.backbone.powersupply);
        self.traction.tick(&mut self.backbone.traction);
        self.outside_lights.tick(&mut self.backbone.outside_lights);
        self.doors.tick(&mut self.backbone.doors);

        self.backbone.reset(BackBoneResetType::Input);

        self.tick_interface();

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(10000.0);
        // self.wheels[1].set_brake_force_newton(10000.0);

        if let Some(test) = self.backbone.powersupply.bus_active(0).get_if_changed() {
            set_var("Lm_MasterError", test as i8 as f32);
        }
        if let Some(test) = self.backbone.powersupply.bus_active(1).get_if_changed() {
            set_var("Lm_MasterWarning", test as i8 as f32);
        }
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        self.traction
            .on_message(&mut self.backbone.piston_traction_transfer, &msg);
        // on_message is only necessary for the axle with traction
        self.axles[1].on_message(&msg);
        self.interface_on_message(&msg);
    }
}

#[derive(Default)]
pub struct Backbone {
    pub pneumatics: BBRoadVehiclePneumatics,
    pub throttle_brake_control: BBThrottleBrakeControl,
    pub steering: BBSteering,
    pub cockpit: BBCockpitNd313,
    pub powersupply: BBPowerSupply,
    pub traction: BBTraction,
    pub piston_traction_transfer: BBPistonTractionTransfer,
    pub outside_lights: BBOutsideLights,
    pub doors: BBDoors,
}

impl BackBoneResetInputOutput for Backbone {
    fn reset(&mut self, reset_type: BackBoneResetType) {
        self.cockpit.reset(reset_type);
        self.powersupply.reset(reset_type);
        self.traction.reset(reset_type);
        self.throttle_brake_control.reset(reset_type);
        self.outside_lights.reset(reset_type);
        self.doors.reset(reset_type);
    }
}
