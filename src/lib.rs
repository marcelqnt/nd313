use lotus_extra::{
    bb_modules,
    bb_system::{
        self, BBVehicle, TickExtra,
        basic::BBSimple,
        doors::{
            BBDoors, DoorRelease, DoorUnit, DoorUnitAutomatic, DoorUnitWing, DoorUnitWingLockMode,
            DoorUnitWingLockRelease, Doors, PneumaticDoor, StopBrakeController,
        },
        electric_power::{
            BBElectricPower, ElectricBatteryProperties, ElectricLimiter, ElectricPower,
            ElectricUnit, SendPowerSignalMessage,
        },
        lights::{BBOutsideLights, Bulb, IndicatorLights, OutsideLights},
        piston_traction::{BBPistonTractionTransfer, BBThrottleBrakeControl, ThrottleBrakeControl},
        road_vehicle::{
            Axle, BBAxle, BBRoadVehiclePneumatics, BBSteering, RoadVehiclePneumatics, Steering,
            SteeringProperties,
        },
    },
    messages,
    vehicle::Rattling,
};
use lotus_script::{Animation, prelude::*};

use crate::{
    cockpit::{BBCockpitNd313, CockpitNd313},
    interface::Nd313Interface,
    traction::{BBTraction, Traction},
};

mod cockpit;
mod interface;
mod traction;

const WHEEL_DIAMETER: f32 = 0.9;

pub(crate) const DOORS_MAX_SPEED_MPS: f32 = 3.0 / 3.6;
pub(crate) const MIN_THROTTLE_RELEASE_STOP_BRAKE: f32 = 0.1;

pub const NOMINAL_VOLTAGE: f32 = 24.0;

/// Von `add_unit_get_index` / `add_bulb_get_index` beim Aufbau befüllt — keine festen Konstanten.
pub(crate) struct Nd313Indices {
    pub electricity_battery: usize,
    pub electricity_min_voltage_relay: usize,
    pub electricity_bus_1: usize,
    pub electricity_bus_2: usize,
    pub lamp_inside_lower_deck: usize,
    pub lamp_inside_upper_deck: usize,
    pub lamp_inside_lower_front_right: usize,
    pub bulb_park_n_rear: usize,
    pub bulb_park_n_rear_led: usize,
    pub bulb_dim_light: usize,
    pub bulb_dim_light_scale: usize,
    pub bulb_dim_light_blue: usize,
    pub bulb_brake: usize,
    pub bulb_brake_led: usize,
    pub bulb_rear_n_brake: usize,
}

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

type MyScript = BBVehicle<Modules, Backbone, Nd313Extras, Nd313Interface>;

pub struct Modules {
    axles: Vec<Axle>,
    electricity: ElectricPower,
    pneumatics: RoadVehiclePneumatics,
    pub(crate) traction: Traction,
    throttle_brake_control: ThrottleBrakeControl,
    outside_lights: OutsideLights,
    pub(crate) doors: Doors,
    cockpit: CockpitNd313,
    steering: Steering,
    pub(crate) indices: Nd313Indices,
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

        let mut electricity = ElectricPower::default();
        let electricity_battery = electricity.add_unit_get_index(
            ElectricUnit::new(vec![], true)
                .with_battery(ElectricBatteryProperties::new(NOMINAL_VOLTAGE)),
        );
        let electricity_min_voltage_relay = electricity.add_unit_get_index(
            ElectricUnit::new(vec![electricity_battery], true)
                .with_limiter(ElectricLimiter::default().with_voltage_limiter(10.0, Some(18.0))),
        );
        let electricity_bus_1 = electricity.add_unit_get_index(
            ElectricUnit::new(vec![electricity_min_voltage_relay], false)
                .with_send_power_signal_message(SendPowerSignalMessage::new(
                    messages::std::PowerSignalCabin::A,
                    MessageTarget::broadcast_all(),
                )),
        );
        let electricity_bus_2 = electricity.add_unit_get_index(ElectricUnit::new(
            vec![electricity_min_voltage_relay],
            false,
        ));

        let lamp_inside_lower_deck =
            electricity.add_lamp_get_index(electricity_bus_1, 10.0, 5.0, 20.0, NOMINAL_VOLTAGE);
        electricity.set_appliance_variable(lamp_inside_lower_deck, "Light_Cabin_OtherLights");
        let lamp_inside_upper_deck =
            electricity.add_lamp_get_index(electricity_bus_1, 10.0, 5.0, 20.0, NOMINAL_VOLTAGE);
        electricity.set_appliance_variable(lamp_inside_upper_deck, "Light_Cabin_Upper");
        let lamp_inside_lower_front_right =
            electricity.add_lamp_get_index(electricity_bus_1, 10.0, 5.0, 20.0, NOMINAL_VOLTAGE);
        electricity.set_appliance_variable(lamp_inside_lower_front_right, "Light_Cabin_FirstLight");

        let mut outside_lights = OutsideLights::default().with_indicator(
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
        );
        let bulb_park_n_rear = outside_lights.add_bulb_get_index(
            Bulb::new("Light_ParkingNRear".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );
        let bulb_park_n_rear_led =
            outside_lights.add_bulb_get_index(Bulb::new("Light_ParkingNRear_LED".to_string()));
        let bulb_dim_light = outside_lights.add_bulb_get_index(
            Bulb::new("Light_DimLight".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );
        let bulb_dim_light_scale = outside_lights.add_bulb_get_index(
            Bulb::new("Light_DimLight_Scale".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );
        let bulb_dim_light_blue = outside_lights.add_bulb_get_index(
            Bulb::new("Light_DimLight_Blue".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );
        let bulb_brake = outside_lights.add_bulb_get_index(
            Bulb::new("Light_Brake".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );
        let bulb_brake_led =
            outside_lights.add_bulb_get_index(Bulb::new("Light_Brake_LED".to_string()));
        let bulb_rear_n_brake = outside_lights.add_bulb_get_index(
            Bulb::new("Light_RearNBrake".to_string()).with_exp_fade_in_out((20.0, 15.0)),
        );

        let indices = Nd313Indices {
            electricity_battery,
            electricity_min_voltage_relay,
            electricity_bus_1,
            electricity_bus_2,
            lamp_inside_lower_deck,
            lamp_inside_upper_deck,
            lamp_inside_lower_front_right,
            bulb_park_n_rear,
            bulb_park_n_rear_led,
            bulb_dim_light,
            bulb_dim_light_scale,
            bulb_dim_light_blue,
            bulb_brake,
            bulb_brake_led,
            bulb_rear_n_brake,
        };

        Self {
            steering: Steering::new(SteeringProperties::new(40.0)),

            electricity,

            axles,
            pneumatics,
            throttle_brake_control: ThrottleBrakeControl::new(0, 1, 0.85),

            outside_lights,
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
                    vec![
                        DoorUnitWing::new(
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
                        )
                        .with_wing_lock_mode(DoorUnitWingLockMode::BlockOpen)
                        .with_wing_lock_release(DoorUnitWingLockRelease::SyncOpen),
                        DoorUnitWing::new(
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
                        )
                        .with_wing_lock_mode(DoorUnitWingLockMode::BlockOpen)
                        .with_wing_lock_release(DoorUnitWingLockRelease::SyncOpen),
                    ],
                    DoorUnitAutomatic::Manual,
                    Some(0),
                ))
                .add_door(DoorUnit::new(
                    vec![DoorUnitWing::new(
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
                    )],
                    DoorUnitAutomatic::Timer(5.5),
                    Some(0),
                ))
                .add_door(DoorUnit::new(
                    vec![DoorUnitWing::new(
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
                    )],
                    DoorUnitAutomatic::Timer(5.5),
                    Some(0),
                )),
            traction: Traction::default(),
            indices,
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
            electricity => electricity;
            traction => traction;
            outside_lights => outside_lights;
            doors => doors;
            axles[1] => axle;
        }
        init {
            pneumatics => pneumatics;
            electricity => electricity;
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
            cockpit, electricity, traction, throttle_brake_control, outside_lights, doors,
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
    pub electricity: BBElectricPower,
    pub traction: BBTraction,
    pub piston_traction_transfer: BBPistonTractionTransfer,
    pub outside_lights: BBOutsideLights,
    pub doors: BBDoors,
    pub axle: BBAxle,
    // own
    pub retarder_request: BBSimple<i8>,
}
