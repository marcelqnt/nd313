use lotus_extra::{
    bb_system::{DomainMessageInterface, basic::BackBone},
    messages,
};

use crate::{Backbone, Modules};

#[derive(Default)]
pub struct Nd313MessageInterface;

impl DomainMessageInterface<Modules, Backbone> for Nd313MessageInterface {
    fn on_message(
        &mut self,
        _modules: &Modules,
        backbone: &mut Backbone,
        msg: &lotus_script::message::Message,
    ) -> bool {
        msg.handle(|g: messages::std::AutomaticGearboxCurrentGear| {
            backbone.cockpit.vdv_dashboard.current_gear.set(g);
            Ok(())
        })
        .is_ok()
    }
}
