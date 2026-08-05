use lotus_extra::{
    bb_system::{DomainInterface, basic::BackBone, lights::IndicatorState},
    math::IfElse,
};

use crate::{Backbone, Modules, NOMINAL_VOLTAGE};

#[derive(Default)]
pub struct Nd313OutsideLightsInterface;

impl DomainInterface<Modules, Backbone> for Nd313OutsideLightsInterface {
    fn wire(&mut self, modules: &Modules, backbone: &mut Backbone) {
        let idx = &modules.indices;
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_outside_lights = &mut backbone.outside_lights;
        let bus_2 = backbone.electricity.unit_active(idx.electricity_bus_2);
        let voltage_available = backbone
            .electricity
            .unit_voltage_available_v(idx.electricity_bus_1)
            / NOMINAL_VOLTAGE;

        bb_outside_lights.set_voltage_normalized(
            backbone
                .electricity
                .unit_voltage_available_v(idx.electricity_bus_2)
                / NOMINAL_VOLTAGE,
        );

        if voltage_available > 0.0 && bb_cockpit.flash_light_switch.state().get_state() {
            bb_outside_lights.set_indicator(IndicatorState::Warning);
        } else if !bus_2.get_state() {
            bb_outside_lights.set_indicator(IndicatorState::Off);
        } else {
            bb_outside_lights.set_indicator(bb_cockpit.indicator_switch.state().get_state().into());
        }

        let park_n_rear = bb_cockpit
            .modern_outside_light_switch
            .parking()
            .if_else(1.0, 0.0);

        let dim_light: f32 = bb_cockpit
            .modern_outside_light_switch
            .dim()
            .if_else(1.0, 0.0);

        let brake_light: f32 =
            (backbone.throttle_brake_control.brake_value() > 0.02).if_else(1.0, 0.0);

        bb_outside_lights
            .set_bulb_brightness(idx.bulb_park_n_rear, park_n_rear * voltage_available);
        bb_outside_lights.set_bulb_brightness(idx.bulb_park_n_rear_led, park_n_rear);

        bb_outside_lights.set_bulb_brightness(idx.bulb_dim_light, dim_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(
            idx.bulb_dim_light_scale,
            dim_light.max(park_n_rear * 0.4) * voltage_available,
        );

        bb_outside_lights.set_bulb_brightness(idx.bulb_dim_light_blue, 0.5 + 0.5 * dim_light);

        bb_outside_lights.set_bulb_brightness(idx.bulb_brake, brake_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(idx.bulb_brake_led, brake_light);

        bb_outside_lights
            .set_bulb_brightness(idx.bulb_rear_n_brake, brake_light.max(park_n_rear * 0.7));
    }
}
