const DELTA1: f64 = 1.65;
const DELTA2: f64 = 3.3 / 10.0;

const ALPHA1: f64 = 50.0 / 5000.0;
const ALPHA2: f64 = 20.0 / 256.0;

const BETA1: f64 = 3.3 * 50.0 / 62.0 / 5000.0;
const BETA2: f64 = 3.3 * 20.0 / 62.0 / 256.0;

impl super::AnalogInput {
    fn set_level(&mut self, level: f64) {
        let ch_gain = self.gain_setting as f64;
        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        if gain < 1.1 {
            self.offset_setting = 31;
            return;
        }
        let desired_level_setting = (level * DELTA2 * gain + DELTA1 * (gain - 1.0)) / (BETA1 + BETA2 * ch_gain);
        self.offset_setting = (desired_level_setting + 0.5) as u8
    }

    fn set_gain(&mut self, gain: f64) {
        let desired_gain_setting = (gain - 1.0 - ALPHA1) / ALPHA2;
        self.gain_setting = desired_gain_setting as u8;
    }

    pub fn gain(&self) -> f64 {
        self.gain_setting as f64 * ALPHA2 + ALPHA1 + 1.0
    }
}
impl super::AnalogInput {
    pub(crate) fn measurement_from_voltage(&self, voltage: f64) -> i16 {
        let ch_gain = self.gain_setting as f64;
        let ch_level = self.offset_setting as f64;

        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        let level = (ch_level * (BETA1 + BETA2 * ch_gain) - DELTA1 * (gain - 1.0)) / DELTA2 / gain;

        ((voltage - level) * gain / 10.0 * 4095.0 + 2047.0) as i16
    }
}

impl super::AnalogInput {
    pub(crate) fn voltage_from_measurement(&self, adc_data: u16) -> f64 {
        let adc_reading = adc_data as f64;
        let ch_gain = self.gain_setting as f64;
        let ch_level = self.offset_setting as f64;

        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        let level = (ch_level * (BETA1 + BETA2 * ch_gain) - DELTA1 * (gain - 1.0)) / DELTA2 / gain;

        10.0 / gain * (adc_reading - 2047.0) / 4095.0 + level
    }
}


impl super::AnalogInput {
    pub fn set_range(&mut self, vmin: f64, vmax: f64) {
        let level = (vmax + vmin) / 2.0;
        let gain =  10.0 / (vmax - vmin);

        self.set_gain(gain);
        self.set_level(level);
    }
}