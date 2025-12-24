use std::collections::HashMap;

use lotus_actor::prelude::*;
use lotus_extra::{
    messages,
    road_steering::{RoadSteering, RoadSteeringProperties},
};
use lotus_script::{prelude::*, time::delta, vehicle::RoadWheel};

use crate::cockpit::Cockpit;

mod cockpit;
mod vdv_dashboard;

pub struct MyScript {
    hub: ActorHub,
    cockpit: Cockpit,
    hash_maps: HashMaps,
    steering: RoadSteering,
}

impl Default for MyScript {
    fn default() -> Self {
        Self {
            hub: ActorHub::default(),
            cockpit: Cockpit::default(),
            hash_maps: HashMaps::default(),
            steering: RoadSteering::new(
                RoadSteeringProperties::builder()
                    // .wheel_deg_per_s(900.0)
                    .build(),
            ),
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
    }

    fn tick(&mut self) {
        // log::info!(
        //     "self.wheels: {}/{} / {}/{}",
        //     self.wheels[0].axle_index(),
        //     self.wheels[0].wheel_index(),
        //     self.wheels[1].axle_index(),
        //     self.wheels[1].wheel_index()
        // );

        // log::info!(
        //     "wheel speed (km/h): {}",
        //     get_var::<f64>("v_wheel_mps_1_0") * 3.6
        // );

        self.steering.tick();

        self.hub.tick();

        // self.wheels[0].set_traction_force_newton(0000.0);
        // self.wheels[1].set_traction_force_newton(0000.0);

        // self.wheels[0].set_brake_force_newton(0.0);
        // self.wheels[1].set_brake_force_newton(0.0);

        let current_tick = delta();
    }

    fn on_message(&mut self, msg: lotus_script::message::Message) {
        // log::info!("on_message: {:?}", msg);

        self.on_message_traction(msg);
    }
}

impl MyScript {
    fn on_message_traction(&mut self, msg: lotus_script::message::Message) {
        // Inertia Engine => Gearbox
        msg.handle(|inv_j: messages::technical::InvMomentOfInertia| {
            if msg.source().module_slot_index == Some(0) {
                send_message(&inv_j, [MessageTarget::ChildByIndex(2)]);
            }
            Ok(())
        })
        .unwrap();

        msg.handle(|t: messages::technical::Torque| {
            // Torque Engine => Gearbox
            if msg.source().module_slot_index == Some(0) {
                send_message(&t, [MessageTarget::ChildByIndex(2)]);
            }
            Ok(())
        })
        .unwrap();

        msg.handle(|s: messages::technical::RotationSpeed| {
            // Rotation Speed Gearbox => Engine
            if msg.source().module_slot_index == Some(1) {
                send_message(&s, [MessageTarget::ChildByIndex(1)]);
            }
            Ok(())
        })
        .unwrap();
    }
}

#[derive(Default)]
pub struct HashMaps {
    indicator_lights: HashMap<IndicatorLights, ActorId<f32>>,
    ignition_state: Option<ActorId<IgnitionSwitchStep>>,
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
