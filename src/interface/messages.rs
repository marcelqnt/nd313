use lotus_extra::{
    bb_system::{DomainMessageInterface, basic::BackBone},
    messages::{self, handle_message},
};
use lotus_script::var::set_var;

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
        handle_message(msg, |g: messages::std::AutomaticGearboxCurrentGear| {
            backbone.cockpit.vdv_dashboard.current_gear.set(g);
            true
        }) || handle_message(msg, |available: messages::almex::Available| {
            if available.0 {
                set_var("HasAlmex", true);
            }
            true
        })
    }
}
