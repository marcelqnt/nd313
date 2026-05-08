use lotus_extra::{
    bb_system::{
        self,
        basic::{
            BBSimple, BackBoneResetInputOutput, BackBoneResetType, ModuleInit, ModuleOnMessage,
            ModuleTick,
        },
        doors::{
            BBDoors, DoorRelease, DoorUnit, DoorUnitAutomatic, Doors, PneumaticDoor,
            StopBrakeController,
        },
        lights::{BBOutsideLights, Bulb, IndicatorLights, OutsideLights},
        piston_traction::{BBPistonTractionTransfer, BBThrottleBrakeControl, ThrottleBrakeControl},
        power::{BBPowerSupply, Battery, ElectricBus, PowerSupply},
        road_vehicle::{
            Axle, BBAxle, BBRoadVehiclePneumatics, BBSteering, RoadVehiclePneumatics, Steering,
            SteeringProperties,
        },
    },
    vehicle::Rattling,
};
use lotus_script::{Animation, prelude::*};

use crate::{
    cockpit::{BBCockpitNd313, CockpitNd313},
    traction::{BBTraction, Traction},
};

mod cockpit;
mod interface;
mod traction;

const WHEEL_DIAMETER: f32 = 0.9;

const BULB_INDEX_PARKNREAR: usize = 0;
const BULB_INDEX_PARKNREAR_LED: usize = 1;
const BULB_INDEX_DIMLIGHT: usize = 2;
const BULB_INDEX_DIMLIGHT_SCALE: usize = 3;
const BULB_INDEX_DIMLIGHT_BLUE: usize = 4;
const BULB_INDEX_BRAKE: usize = 5;
const BULB_INDEX_BRAKE_LED: usize = 6;
const BULB_INDEX_REARNBRAKE: usize = 7;

pub struct MyScript {
    modules: Modules,
    backbone: Backbone,

    rattling: Rattling,
}

impl Default for MyScript {
    fn default() -> Self {
        Self {
            modules: Modules::default(),
            backbone: Backbone::default(),
            rattling: Rattling::builder()
                .animation(Animation::get("Main").unwrap())
                .sound_variable("snd_Rattling")
                .build(),
        }
    }
}

pub struct Modules {
    axles: Vec<Axle>,
    powersupply: PowerSupply,
    pneumatics: RoadVehiclePneumatics,
    traction: Traction,
    throttle_brake_control: ThrottleBrakeControl,
    outside_lights: OutsideLights,
    doors: Doors,
    cockpit: CockpitNd313,
    steering: Steering,
}

impl Default for Modules {
    fn default() -> Self {
        let axles = vec![
            Axle::new(0, WHEEL_DIAMETER),
            Axle::new(1, WHEEL_DIAMETER)
                .with_tacho("DiffGear_mps".to_string())
                .with_traction(1, 5.74),
            Axle::new(2, WHEEL_DIAMETER),
        ];

        let pneumatics = RoadVehiclePneumatics::builder()
            .axles(axles.clone())
            .build();

        Self {
            steering: Steering::new(SteeringProperties::new(40.0)),

            powersupply: PowerSupply::new(
                vec![Battery],
                vec![
                    ElectricBus::new(vec![0], 0.75),
                    ElectricBus::new(vec![0], 0.75),
                ],
            ),
            axles,
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
                .add_bulb(
                    Bulb::new("Light_ParkingNRear".to_string()).with_exp_fade_in_out((20.0, 15.0)),
                )
                .add_bulb(Bulb::new("Light_ParkingNRear_LED".to_string()))
                .add_bulb(
                    Bulb::new("Light_DimLight".to_string()).with_exp_fade_in_out((20.0, 15.0)),
                )
                .add_bulb(
                    Bulb::new("Light_DimLight_Scale".to_string())
                        .with_exp_fade_in_out((20.0, 15.0)),
                )
                .add_bulb(
                    Bulb::new("Light_DimLight_Blue".to_string()).with_exp_fade_in_out((20.0, 15.0)),
                )
                .add_bulb(Bulb::new("Light_Brake".to_string()).with_exp_fade_in_out((20.0, 15.0)))
                .add_bulb(Bulb::new("Light_Brake_LED".to_string()))
                .add_bulb(
                    Bulb::new("Light_RearNBrake".to_string()).with_exp_fade_in_out((20.0, 15.0)),
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
                    DoorUnitAutomatic::Manual,
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
                    DoorUnitAutomatic::Manual,
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
                    DoorUnitAutomatic::Timer(5.5),
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
                    DoorUnitAutomatic::Timer(5.5),
                    Some(0),
                )),
            traction: Traction::default(),
        }
    }
}

impl ModuleTick<Backbone> for Modules {
    fn tick(&self, backbone: &mut Backbone) {
        self.pneumatics.tick(&mut backbone.pneumatics);
        self.throttle_brake_control
            .tick(&mut backbone.throttle_brake_control);
        self.steering.tick(&mut backbone.steering);
        self.cockpit.tick(&mut backbone.cockpit);
        self.powersupply.tick(&mut backbone.powersupply);
        self.traction.tick(&mut backbone.traction);
        self.outside_lights.tick(&mut backbone.outside_lights);
        self.doors.tick(&mut backbone.doors);
        self.axles[1].tick(&mut backbone.axle);
    }
}

impl ModuleInit<Backbone> for Modules {
    fn init(&self, backbone: &mut Backbone) {
        log::info!("Modules init");
        self.pneumatics.init(&mut backbone.pneumatics);
        self.powersupply.init(&mut backbone.powersupply);
        self.outside_lights.init(&mut backbone.outside_lights);
        self.doors.init(&mut backbone.doors);
        self.axles[1].init(&mut backbone.axle);
        self.cockpit.init(&mut backbone.cockpit);
    }
}

script!(MyScript);

impl Script for MyScript {
    fn init(&mut self) {
        log::info!("Initializing script ==========================================");

        self.modules.init(&mut self.backbone);

        set_var("Lm_MasterError", 0.0);
    }

    fn tick(&mut self) {
        self.backbone.reset_inputs();

        self.modules.tick_interface(&mut self.backbone);

        self.backbone.reset_outputs();

        self.modules.tick(&mut self.backbone);

        self.rattling.tick();
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        self.modules
            .traction
            .on_message(&mut self.backbone.piston_traction_transfer, &msg);
        // on_message is only necessary for the axle with traction
        self.modules.axles[1].on_message(&mut self.backbone.axle, &msg);
        self.modules.interface_on_message(&mut self.backbone, &msg);
        self.modules
            .cockpit
            .on_message(&mut self.backbone.cockpit, &msg);
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
    pub axle: BBAxle,
    // own
    pub retarder_request: BBSimple<i8>,
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
