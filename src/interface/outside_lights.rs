use lotus_extra::{
    bb_system::{DomainInterface, basic::BackBone},
    math::IfElse,
};
use lotus_extra::bb_system::lights::IndicatorState;

use crate::{
    Backbone, BULB_INDEX_BRAKE, BULB_INDEX_BRAKE_LED, BULB_INDEX_DIMLIGHT,
    BULB_INDEX_DIMLIGHT_BLUE, BULB_INDEX_DIMLIGHT_SCALE, BULB_INDEX_PARKNREAR,
    BULB_INDEX_PARKNREAR_LED, BULB_INDEX_REARNBRAKE, ELECTRICITY_INDEX_BUS_1,
    ELECTRICITY_INDEX_BUS_2, Modules, NOMINAL_VOLTAGE,
};

#[derive(Default)]
pub struct Nd313OutsideLightsInterface;

impl DomainInterface<Modules, Backbone> for Nd313OutsideLightsInterface {
    fn wire(&mut self, _modules: &Modules, backbone: &mut Backbone) {
        let bb_cockpit = &mut backbone.cockpit.vdv_dashboard;
        let bb_outside_lights = &mut backbone.outside_lights;
        let bus_2 = backbone.electricity.unit_active(ELECTRICITY_INDEX_BUS_2);
        let voltage_available = backbone
            .electricity
            .unit_voltage_available(ELECTRICITY_INDEX_BUS_1)
            / NOMINAL_VOLTAGE;

        bb_outside_lights.set_unified_voltage(
            backbone
                .electricity
                .unit_voltage_available(ELECTRICITY_INDEX_BUS_2)
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

        bb_outside_lights.set_bulb_brightness(BULB_INDEX_PARKNREAR, park_n_rear * voltage_available);
        bb_outside_lights.set_bulb_brightness(BULB_INDEX_PARKNREAR_LED, park_n_rear);

        bb_outside_lights.set_bulb_brightness(BULB_INDEX_DIMLIGHT, dim_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(
            BULB_INDEX_DIMLIGHT_SCALE,
            dim_light.max(park_n_rear * 0.4) * voltage_available,
        );

        bb_outside_lights.set_bulb_brightness(BULB_INDEX_DIMLIGHT_BLUE, 0.5 + 0.5 * dim_light);

        bb_outside_lights.set_bulb_brightness(BULB_INDEX_BRAKE, brake_light * voltage_available);
        bb_outside_lights.set_bulb_brightness(BULB_INDEX_BRAKE_LED, brake_light);

        bb_outside_lights.set_bulb_brightness(
            BULB_INDEX_REARNBRAKE,
            brake_light.max(park_n_rear * 0.7),
        );
    }
}
