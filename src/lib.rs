use lotus_extra::{
    bb_modules,
    bb_system::{
        self, BBVehicle, TickExtra,
        basic::BBSimple,
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

struct Nd313Extras {
    rattling: Rattling,
}

impl Default for Nd313Extras {
    fn default() -> Self {
        Self {
            rattling: Rattling::builder()
                .animation(Animation::get("Main").unwrap())
                .sound_variable("snd_Rattling")
                .build(),
        }
    }
}

impl TickExtra for Nd313Extras {
    fn tick_extra(&mut self) {
        self.rattling.tick();
    }
}

type MyScript = BBVehicle<Modules, Backbone, Nd313Extras>;

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
                vec![Battery::new(true)],
                vec![
                    ElectricBus::new(vec![0], 0.75).with_send_power_signal_message(),
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
                        2.5 / 698_700.0,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.5 / 698_700.0,
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
                        2.4 / 698_700.0,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.4 / 698_700.0,
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
                        6.0 / 698_700.0,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.36 / 698_700.0,
                            p_end_normalized: 0.3,
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
                        6.0 / 698_700.0,
                        bb_system::doors::PneumaticDoorPressureRate::Linear {
                            rate: 0.36 / 698_700.0,
                            p_end_normalized: 0.3,
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

bb_modules! {
    Modules => Backbone {
        tick {
            pneumatics => pneumatics;
            throttle_brake_control => throttle_brake_control;
            steering => steering;
            cockpit => cockpit;
            powersupply => powersupply;
            traction => traction;
            outside_lights => outside_lights;
            doors => doors;
            axles[1] => axle;
        }
        init {
            pneumatics => pneumatics;
            powersupply => powersupply;
            outside_lights => outside_lights;
            doors => doors;
            axles[1] => axle;
            cockpit => cockpit;
        }
        on_action {
            cockpit => cockpit;
        }
        on_message {
            traction => piston_traction_transfer;
            axles[1] => axle;
            cockpit => cockpit;
        }
        reset {
            cockpit, powersupply, traction, throttle_brake_control, outside_lights, doors,
        }
    }
}

script!(MyScript);

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
