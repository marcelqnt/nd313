use std::{io::WriterPanicked, iter::Map};

use lotus_actor::{
    base_actors::{Execute, MapperActor},
    cockpit::{
        ButtonProperties, StepSwitchProperties, StepSwitchSpringLoaded, simple_button, step_switch,
    },
    inputs::InputEvent,
    prelude::*,
    variables::VarWriter,
};
use lotus_script::prelude::*;

use lotus_extra::messages;

use crate::{HashMaps, vdv_dashboard::VdvDashboard};

#[derive(Default)]
pub struct Cockpit {
    vdv_dashboard: VdvDashboard,
}

impl SystemUnit<HashMaps> for Cockpit {
    fn children(&self) -> Vec<&dyn SystemUnit<HashMaps>> {
        vec![&self.vdv_dashboard]
    }
    fn pre_init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        hash_maps.indicator_lights.insert(
            IndicatorLights::MasterError,
            hub.insert_actor(VarWriter::<f32>::new("Lm_MasterError")),
        );
    }

    fn init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        hash_maps.ignition_state = Some(hub.insert_actor(Execute::new(Box::new(|value| {
            log::info!("*** ignition_state: {:?} ***", value);
        }))));

        hash_maps.gear_box_mode_switch = Some(hub.insert_actor(Execute::new(Box::new(|value| {
            log::info!("*** gear_box_mode_switch: {:?} ***", value);
        }))));
    }

    fn post_init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        if let Some(master_error) = hash_maps
            .indicator_lights
            .get(&IndicatorLights::MasterCaution)
        {
            // let input_master_error = hub.insert_actor(InputActor::<f32>::new(
            //     InputEvent::new("Door1Toggle", 1),
            //     Some(ActorIdValue::new(*master_error, 1.0)),
            //     Some(ActorIdValue::new(*master_error, 0.0)),
            // ));
            // hub.call_actor(input_master_error, &true);

            let input_master_error = hub.insert_actor(MapperActor::<bool, f32>::new(
                *master_error,
                Box::new(|value| Some(if *value { 1.0 } else { 0.0 })),
            ));

            simple_button(
                hub,
                ButtonProperties::builder()
                    .input(InputEvent::new("Door1Toggle", 1))
                    .sound_press("snd_Btn_Door1_Press")
                    .sound_release("snd_Btn_Door1_Release")
                    .position(("Btn_Door1_Pos".to_string(), 1.0))
                    .output(input_master_error)
                    .springloaded(true)
                    .build(),
            );
        }
    }
}
