use lotus_actor::{
    SystemUnit,
    actors::ActorHub,
    powersupply::{PowerSupplyProperties, power_supply},
};

use crate::HashMaps;

#[derive(Default)]
pub struct Powersupply;

impl SystemUnit<HashMaps> for Powersupply {
    fn pre_init_self(&self, hub: &mut ActorHub, hash_maps: &mut HashMaps) {
        let powersupply = power_supply(
            hub,
            PowerSupplyProperties::builder().min_voltage(0.8).build(),
        );
    }
}
