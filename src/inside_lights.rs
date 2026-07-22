use lotus_extra::bb_system::{
    basic::{ModuleInit, ModuleTick, ModuleTickInput},
    lights::Bulb,
};

#[derive(Clone)]
pub struct InsideLightsNd313 {
    pub lower_deck: Bulb,
    pub upper_deck: Bulb,
    pub lower_front_right: Bulb,
}

impl Default for InsideLightsNd313 {
    fn default() -> Self {
        Self {
            lower_deck: Bulb::new("Light_Cabin_OtherLights").with_exp_fade_in_out((5.0, 20.0)),
            upper_deck: Bulb::new("Light_Cabin_Upper").with_exp_fade_in_out((5.0, 20.0)),
            lower_front_right: Bulb::new("Light_Cabin_FirstLight")
                .with_exp_fade_in_out((5.0, 20.0)),
        }
    }
}

#[derive(Default)]
pub struct BBInsideLightsNd313 {
    pub lower_deck_input: f32,
    pub upper_deck_input: f32,
    pub lower_front_right_input: f32,
    pub lower_deck: f32,
    pub upper_deck: f32,
    pub lower_front_right: f32,
}

impl ModuleTick<BBInsideLightsNd313> for InsideLightsNd313 {
    fn tick(&self, backbone: &mut BBInsideLightsNd313) {
        self.lower_deck
            .tick(&mut backbone.lower_deck, backbone.lower_deck_input);
        self.upper_deck
            .tick(&mut backbone.upper_deck, backbone.upper_deck_input);
        self.lower_front_right.tick(
            &mut backbone.lower_front_right,
            backbone.lower_front_right_input,
        );
    }
}

impl ModuleInit<BBInsideLightsNd313> for InsideLightsNd313 {
    fn init(&self, backbone: &mut BBInsideLightsNd313) {
        self.lower_deck.init(&mut backbone.lower_deck);
        self.upper_deck.init(&mut backbone.upper_deck);
        self.lower_front_right.init(&mut backbone.lower_front_right);
    }
}
