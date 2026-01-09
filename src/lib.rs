use std::collections::HashMap;

use lotus_actor::{
    piston_traction::PistonTractionTransfer, prelude::*, traction::RoadVehicleAxleProperties,
};
use lotus_extra::{
    messages::std::AutomaticGearboxMode,
    road_steering::{RoadSteering, RoadSteeringProperties},
};
use lotus_script::{prelude::*, vehicle::RoadWheel};

use crate::{cockpit::Cockpit, powersupply::Powersupply, traction::Traction};

mod cockpit;
mod powersupply;
mod traction;
mod vdv_dashboard;

pub struct MyScript {
    hub: ActorHub,
    hash_maps: HashMaps,
    steering: RoadSteering,
    axle: RoadVehicleAxleProperties,
    powersupply: Powersupply,
    traction: Traction,
    cockpit: Cockpit,
}

impl Default for MyScript {
    fn default() -> Self {
        Self {
            hub: ActorHub::default(),
            hash_maps: HashMaps::default(),
            steering: RoadSteering::new(RoadSteeringProperties::builder().build()),
            axle: RoadVehicleAxleProperties::new(1, 5.74, "DiffGear_mps"),
            powersupply: Powersupply::default(),
            cockpit: Cockpit::default(),
            traction: Traction::default(),
        }
    }
}

script!(MyScript);

impl Script for MyScript {
    fn init(&mut self) {
        log::info!("Initializing script ==========================================");

        let wheels = [RoadWheel::get(1, 0).unwrap(), RoadWheel::get(1, 1).unwrap()];

        let test_actor = self.hub.insert_actor(TestActor::new(wheels[0], wheels[1]));

        self.cockpit.pre_init(&mut self.hub, &mut self.hash_maps);

        self.hash_maps
            .indicator_lights
            .insert(IndicatorLights::MasterCaution, test_actor);

        self.cockpit.init(&mut self.hub, &mut self.hash_maps);

        self.cockpit.post_init(&mut self.hub, &mut self.hash_maps);
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

        self.hub.tick();

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(0.0);
        // self.wheels[1].set_brake_force_newton(0.0);
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        // log::info!("on_message: {:?}", msg);

        self.traction.on_message(msg);
    }
}

impl MyScript {}

#[derive(Default)]
pub struct HashMaps {
    indicator_lights: HashMap<IndicatorLights, ActorId<f32>>,
    ignition_state: Option<ActorId<IgnitionSwitchStep>>,
    gear_box_mode_switch: Option<ActorId<AutomaticGearboxMode>>,
}

struct TestActor {
    wheels_l: RoadWheel,
    wheels_r: RoadWheel,
}

impl TestActor {
    pub fn new(wheels_l: RoadWheel, wheels_r: RoadWheel) -> Self {
        Self { wheels_l, wheels_r }
    }
}

impl Actor for TestActor {
    type Data = f32;
    fn handle_event(&mut self, _: &mut Context, data: &Self::Data) {
        self.wheels_l.set_traction_force_newton(*data * 10000.0);
        self.wheels_r.set_traction_force_newton(*data * 10000.0);
    }
}
