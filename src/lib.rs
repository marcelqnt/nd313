use lotus_extra::{
    backbone::{BackBoneTick, ElementTraitReset},
    cockpit_enhanced::BBVdvDashboard,
    power::{BBPowerSupply, Battery, ElectricBus, PowerSupply},
    road_vehicle::{
        AxleProperties, BBRoadVehiclePneumatics, RoadVehiclePneumatics, Steering,
        SteeringProperties,
    },
};
use lotus_script::prelude::*;

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
    steering: Steering,
    axle: AxleProperties,
    powersupply: PowerSupply,
    pneumatics: RoadVehiclePneumatics,
    traction: Traction,
    cockpit: CockpitNd313,
    // test: Input,
}

impl Default for MyScript {
    fn default() -> Self {
        Self {
            backbone: Backbone::default(),
            steering: Steering::new(SteeringProperties::builder().build()),
            axle: AxleProperties::new(1, 1, 5.74, WHEEL_DIAMETER, "DiffGear_mps"),
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

            pneumatics: RoadVehiclePneumatics::builder()
                .wheel_diameter(WHEEL_DIAMETER)
                .build(),
            cockpit: CockpitNd313::default(),
            traction: Traction::default(),
        }
    }
}

script!(MyScript);

impl Script for MyScript {
    fn init(&mut self) {
        log::info!("Initializing script ==========================================");

        set_var("Lm_MasterError", 0.0);

        // let wheels = [RoadWheel::get(1, 0).unwrap(), RoadWheel::get(1, 1).unwrap()];
    }

    fn tick(&mut self) {
        // log::info!(
        //     "self.wheels: {}/{} / {}/{}",
        //     self.wheels[0].axle_index(),
        //     self.wheels[0].wheel_index(),
        //     self.wheels[1].axle_index(),
        //     self.wheels[1].wheel_index()
        // );

        self.axle.tick();

        self.steering.tick();

        self.pneumatics.tick(&mut self.backbone.pneumatics);

        self.cockpit.tick(&mut self.backbone.cockpit);
        self.powersupply.tick(&mut self.backbone.powersupply);

        self.traction.tick(&mut self.backbone.traction);

        self.tick_interface();

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(0.0);
        // self.wheels[1].set_brake_force_newton(0.0);

        if let Some(test) = self.backbone.powersupply.bus_active_refreshed(0) {
            set_var("Lm_MasterError", test as i8 as f32);
        }
        if let Some(test) = self.backbone.powersupply.bus_active_refreshed(1) {
            set_var("Lm_MasterWarning", test as i8 as f32);
        }
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        self.traction.on_message(&msg);
        self.axle.on_message(&msg);
    }
}

impl MyScript {}

#[derive(Default)]
pub struct Backbone {
    pub cockpit: BBVdvDashboard<BBRoadVehiclePneumatics>,
    pub powersupply: BBPowerSupply,
    pub traction: BBTraction,
    pub pneumatics: BBRoadVehiclePneumatics,
}

impl ElementTraitReset for Backbone {
    fn reset(&mut self) {
        self.cockpit.reset();
        self.powersupply.reset();
        self.traction.reset();
    }
}
