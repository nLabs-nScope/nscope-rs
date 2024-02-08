#[derive(Debug, Copy, Clone)]
pub(super) struct AnalogInterfaceModern {}

impl AnalogInterfaceModern {
    pub(super) fn set_level(&mut self, _level: f64) {}

    pub(super) fn set_gain(&mut self, _gain: f64) {}

    pub(super) fn gain(&self) -> f64 {
        1.0
    }

    pub(super) fn measurement_from_voltage(&self, voltage: f64) -> i16 {
        let gain = 1.0f64;
        let v_offset = 0.0f64;

        let adc_voltage = (voltage * 2.5 / 10.0 + 1.25) * gain - v_offset * (gain - 1.0);
        (adc_voltage / 2.5 * 4095.0) as i16
    }

    pub(super) fn voltage_from_measurement(&self, adc_data: u16) -> f64 {
        let gain = 1.0f64;
        let v_offset = 0.0f64;

        let adc_voltage = adc_data as f64 * 2.5 / 4095.0;
        ((adc_voltage / gain + v_offset * (gain - 1.0) / (gain)) - 1.25) * 10.0 / 2.5
    }

    pub(super) fn set_range(&mut self, vmin: f64, vmax: f64) {
        let level = (vmax + vmin) / 2.0;
        let gain = 10.0 / (vmax - vmin);

        self.set_gain(gain);
        self.set_level(level);
    }
}