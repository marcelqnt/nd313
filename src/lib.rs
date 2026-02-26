use lotus_extra::{
    backbone::{BackBoneTick, ElementTrait, ElementTraitResetInputOutput, ElementTraitResetType},
    cockpit_enhanced::BBVdvDashboard,
    power::{BBPowerSupply, Battery, ElectricBus, PowerSupply},
    road_vehicle::{
        AxleProperties, BBRoadVehiclePneumatics, BBThrottleBrakeControl, RoadVehiclePneumatics,
        Steering, SteeringProperties, ThrottleBrakeControl,
    },
    traction::BBPistonTractionTransfer,
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
            steering: Steering::new(
                SteeringProperties::builder()
                    .max_wheel_angle_deg(40.0)
                    .build(),
            ),
            axles,
            powersupply: PowerSupply::builder()
                .batteries(vec![Battery])
                .buses(vec![
                    ElectricBus::builder()
                        .batteries(vec![0])
                        .min_voltage(0.75)
                        .build(),
                    ElectricBus::builder()
                        .batteries(vec![0])
                        .min_voltage(0.75)
                        .build(),
                ])
                .build(),

            pneumatics,
            throttle_brake_control: ThrottleBrakeControl::new(0, 1, 0.85),
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
        self.steering.tick();
        self.rattling.tick();

        self.pneumatics.tick(&mut self.backbone.pneumatics);
        self.throttle_brake_control
            .tick(&mut self.backbone.throttle_brake_control);
        self.cockpit.tick(&mut self.backbone.cockpit);
        self.powersupply.tick(&mut self.backbone.powersupply);
        self.traction.tick(&mut self.backbone.traction);

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
    pub cockpit: BBVdvDashboard<BBRoadVehiclePneumatics>,
    pub powersupply: BBPowerSupply,
    pub traction: BBTraction,
    pub pneumatics: BBRoadVehiclePneumatics,
    pub throttle_brake_control: BBThrottleBrakeControl,
    pub piston_traction_transfer: BBPistonTractionTransfer,
}

impl Backbone {
    pub fn set_electricity_available(&mut self, electricity_available: bool) {
        self.cockpit
            .set_electricity_available(electricity_available);
    }
}

impl ElementTraitResetInputOutput for Backbone {
    fn reset(&mut self, reset_type: ElementTraitResetType) {
        self.cockpit.reset(reset_type);
        self.powersupply.reset(reset_type);
        self.traction.reset(reset_type);
        self.throttle_brake_control.reset(reset_type);
    }
}
