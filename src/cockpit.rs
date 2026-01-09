use lotus_extra::{
    backbone::{self, BackBoneTick},
    cockpit::SimpleButton,
    input::InputEvent,
    vdv_dashboard::VdvDashboard,
};

pub struct CockpitNd313 {
    vdv_dashboard: VdvDashboard,
}

impl Default for CockpitNd313 {
    fn default() -> Self {
        Self {
            vdv_dashboard: VdvDashboard::new().with_btn_door1(
                SimpleButton::new(true)
                    .with_input(InputEvent::new("Door1Toggle", 0))
                    .with_position("Btn_Door1_Pos", 1.0)
                    .with_sound_press("snd_Btn_Door1_Press")
                    .with_sound_release("snd_Btn_Door1_Release"),
            ),
        }
    }
}

impl BackBoneTick<backbone::Cockpit> for CockpitNd313 {
    fn tick(&mut self, backbone: &mut backbone::Cockpit) {
        self.vdv_dashboard.tick(backbone);
    }
}
