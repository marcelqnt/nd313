use lotus_extra::{
    bb_system::{
        self,
        basic::{BackBoneTick, ElementTrait, ElementTraitResetInputOutput, ElementTraitResetType},
        lights::{BBOutsideLights, IndicatorLights, OutsideLights},
        piston_traction::BBPistonTractionTransfer,
        power::{BBPowerSupply, Battery, ElectricBus, PowerSupply},
        road_vehicle::{
            AxleProperties, BBRoadVehiclePneumatics, BBSteering, BBThrottleBrakeControl,
            RoadVehiclePneumatics, Steering, SteeringProperties, ThrottleBrakeControl,
        },
        vdv_dashboard::BBVdvDashboard,
    },
    vehicle::Rattling,
};
use lotus_script::{Animation, prelude::*, vehicle::RoadWheel};

use crate::{
    cockpit::CockpitNd313,
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
            outside_lights: OutsideLights::default().with_indicator(
                IndicatorLights::new(
                    vec![
                        bb_system::lights::Light::new("Light_Indicator_Left".to_string())
                            .with_exp_fade_in_out((20.0, 15.0)),
                        bb_system::lights::Light::new("Light_Indicator_Left_LED".to_string()),
                    ],
                    vec![
                        bb_system::lights::Light::new("Light_Indicator_Right".to_string())
                            .with_exp_fade_in_out((20.0, 15.0)),
                        bb_system::lights::Light::new("Light_Indicator_Right_LED".to_string()),
                    ],
                    0.40,
                    0.35,
                    0.43,
                )
                .with_sound("snd_IndicatorRelayOn", "snd_IndicatorRelayOff"),
            ),
            cockpit: CockpitNd313::default(),
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

        self.backbone.reset(ElementTraitResetType::Output);

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

        self.backbone.reset(ElementTraitResetType::Input);

        self.tick_interface();

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(10000.0);
        // self.wheels[1].set_brake_force_newton(10000.0);

        if let Some(test) = self.backbone.powersupply.bus_active_refreshed(0) {
            set_var("Lm_MasterError", test as i8 as f32);
        }
        if let Some(test) = self.backbone.powersupply.bus_active_refreshed(1) {
            set_var("Lm_MasterWarning", test as i8 as f32);
        }

        for axle in self.axles.iter_mut() {
            axle.kneeling(self.backbone.cockpit.btn_doors.first().unwrap().get_state());
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
    pub cockpit: BBVdvDashboard<BBRoadVehiclePneumatics>,
    pub powersupply: BBPowerSupply,
    pub traction: BBTraction,
    pub piston_traction_transfer: BBPistonTractionTransfer,
    pub outside_lights: BBOutsideLights,
}

impl ElementTraitResetInputOutput for Backbone {
    fn reset(&mut self, reset_type: ElementTraitResetType) {
        self.cockpit.reset(reset_type);
        self.powersupply.reset(reset_type);
        self.traction.reset(reset_type);
        self.throttle_brake_control.reset(reset_type);
        self.outside_lights.reset(reset_type);
    }
}
