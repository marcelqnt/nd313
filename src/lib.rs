use lotus_extra::{
    backbone::{self, BackBoneTick},
    input::Input,
    road_vehicle::{AxleProperties, Steering, SteeringProperties},
};
use lotus_script::{prelude::*, vehicle::RoadWheel};

use crate::{cockpit::CockpitNd313, powersupply::Powersupply, traction::Traction};

mod cockpit;
mod powersupply;
mod traction;

pub struct MyScript {
    backbone: Backbone,
    steering: Steering,
    axle: AxleProperties,
    powersupply: Powersupply,
    traction: Traction,
    cockpit: CockpitNd313,
    // test: Input,
}

impl Default for MyScript {
    fn default() -> Self {
        Self {
            backbone: Backbone::default(),
            steering: Steering::new(SteeringProperties::builder().build()),
            axle: AxleProperties::new(1, 5.74, "DiffGear_mps"),
            powersupply: Powersupply::default(),
            cockpit: CockpitNd313::default(),
            traction: Traction::default(),
            // test: Input::new("Door1Toggle"),
        }
    }
}

script!(MyScript);

impl Script for MyScript {
    fn init(&mut self) {
        log::info!("Initializing script ==========================================");

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

        self.cockpit.tick(&mut self.backbone.cockpit);

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(0.0);
        // self.wheels[1].set_brake_force_newton(0.0);

        // Test:

        // self.test.tick(&mut self.backbone.test);
        // if let Some(test) = self.backbone.test {
        //     set_var("Lm_MasterError", test as i8 as f32);
        // }
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        // log::info!("on_message: {:?}", msg);

        self.traction.on_message(msg);
    }
}

impl MyScript {}

#[derive(Default)]
pub struct Backbone {
    // test: Option<bool>,
    pub cockpit: backbone::Cockpit,
}
