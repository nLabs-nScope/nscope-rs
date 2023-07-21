//
//                           3.3V
//                             │
//                             │
//                          ┌─────┐
//                          │6.4k │
//                          └─────┘
//                ┌───────┐    │
//       ±5V  ────│  10k  │────┼───────── Vo
//                └───────┘    │
//                          ┌─────┐
//                          │19.6k│
//                          └─────┘
//                             │
//                             │
//                            GND
//
//
//            (6.4k)(19.6k)Vi + (10k)(19.6k)3.3V
//    Vo = ────────────────────────────────────────
//          (6.4k)(19.6k)+(6.4k)(10k)+(10k)(19.6k)
//
//    ┌──────────────────────────────────────────┐
//    │ Vo = (125.44/385.44)Vi + (196/385.44)3.3 │
//    └──────────────────────────────────────────┘
//
//                                 │╲
//                    Vo ────── + ─┤ ╲
//                                 │  ╲▁▁▁▁▁▁▁▁▁▁▁▁▁▁Vm
//                                 │  ╱           │
//                          ┌── - ─┤ ╱            │
//                          │      │╱             │
//                          │                     │
//                          │                     │
//                  ┌────┐  │        ┌────┐       │
//           Vl ────│ 5k │──┴────────│ Rg │───────┘
//                  └────┘           └────┘
//
//  ┌──────────────────────────────────────────┐
//  │     Vm =  ((Rg+5k)/5k)Vo - (Rg/5k)Vl     │
//  └──────────────────────────────────────────┘

//  ┌──────────────────────────────────────────────────────────────┐
//  │   The variable resistor is set with a digital register, G    │
//  │                                                              │
//  │   Rg = 100k * G / 256 + 75                                   │
//  │   G  = 256*(Rg-75)/100k                                      │
//  └──────────────────────────────────────────────────────────────┘
//

const DELTA1: f64 = 1.65;
const DELTA2: f64 = 3.3 / 10.0;

const ALPHA1: f64 = 50.0 / 5000.0;
const ALPHA2: f64 = 20.0 / 257.0;

const BETA1: f64 = 3.3 * 50.0 / 63.0 / 5000.0;
const BETA2: f64 = 3.3 * 20.0 / 63.0 / 257.0;

impl super::AnalogInput {
    #[allow(dead_code)]
    fn set_gain(&mut self, gain_resistance: f64) {
        let desired_gain_setting = 256.0 * (gain_resistance - 75.0) / 100_000.0;
        self.gain_setting = desired_gain_setting as u8;
    }

    fn gain_resistance(&self) -> f64 {
        100_000.0 * self.gain_setting as f64 / 256.0 + 75.0
    }

    fn set_level_legacy(&mut self, level: f64) {
        let ch_gain = self.gain_setting as f64;
        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        if gain < 1.1 {
            self.offset_setting = 32;
            return;
        }
        let desired_level_setting = (level * DELTA2 * gain + DELTA1 * (gain - 1.0)) / (BETA1 + BETA2 * ch_gain);
        self.offset_setting = (desired_level_setting + 0.5) as u8
    }

    fn set_gain_legacy(&mut self, gain: f64) {
        let desired_gain_setting = (gain - 1.0 - ALPHA1) / ALPHA2;
        self.gain_setting = desired_gain_setting as u8;
    }
}

//  ┌──────────────────────────────────────────────────────────────┐
//  │   The voltage level is set with a digital register, L        │
//  │                                                              │
//  │   Vl = 3.3*L/64                                              │
//  │   L  = Vl*64/3.3                                             │
//  └──────────────────────────────────────────────────────────────┘

impl super::AnalogInput {
    #[allow(dead_code)]
    fn set_offset(&mut self, offset_voltage: f64) {
        let desired_offset_setting = offset_voltage * 64.0 / 3.3;
        self.offset_setting = desired_offset_setting as u8;
        self.offset_setting = self.offset_setting.clamp(0, 64);
    }

    fn offset_voltage(&self) -> f64 {
        3.3 * self.offset_setting as f64 / 64.0
    }
}


//  ┌────────────────────────────────────────────────────────────────────────────────────────────┐
//  │   Vm = ((Rg+5k)/5k)(125.44/385.44)Vi + ((Rg+5k)/5k)(196/385.44)3.3 - (Rg/5k)Vl             │
//  │                                                                                            │
//  │   Vi = (385.44/125.44)(5k/(Rg+5k))Vm + (Rg/(Rg+5k))(385.44/125.44)Vl - (196/125.44)3.3     │
//  └────────────────────────────────────────────────────────────────────────────────────────────┘

impl super::AnalogInput {
    #[allow(dead_code)]
    pub(crate) fn measurement_from_voltage(&self, voltage: f64) -> i16 {
        let a = (self.gain_resistance() + 5000.0) / 5000.0;
        let b = 125.44 / 385.44;
        let c = 196.0 / 385.44;
        let d = self.gain_resistance() / 5000.0;

        let adc_voltage = a * b * voltage + a * c * 3.3 - d * self.offset_voltage();

        (adc_voltage * 4095.0 / 3.3) as i16
    }

    pub(crate) fn measurement_from_voltage_legacy(&self, voltage: f64) -> i16 {
        let ch_gain = self.gain_setting as f64;
        let ch_level = self.offset_setting as f64;

        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        let level = (ch_level * (BETA1 + BETA2 * ch_gain) - DELTA1 * (gain - 1.0)) / DELTA2 / gain;

        ((voltage - level) * gain / 10.0 * 4095.0 + 2047.0) as i16
    }
}

impl super::AnalogInput {
    #[allow(dead_code)]
    pub(crate) fn voltage_from_measurement(&self, adc_data: u16) -> f64 {
        let adc_voltage = adc_data as f64 * 3.3 / 4095.0;

        let a = 385.44 / 125.44;
        let b = 5000.0 / (self.gain_resistance() + 5000.0);
        let c = 196.0 / 125.44;

        a * b * adc_voltage + (1.0 - b) * a * self.offset_voltage() - c * 3.3
    }

    pub(crate) fn voltage_from_measurement_legacy(&self, adc_data: u16) -> f64 {
        let adc_reading = adc_data as f64;
        let ch_gain = self.gain_setting as f64;
        let ch_level = self.offset_setting as f64;

        let gain = 1.0 + ALPHA1 + ALPHA2 * ch_gain;
        let level = (ch_level * (BETA1 + BETA2 * ch_gain) - DELTA1 * (gain - 1.0)) / DELTA2 / gain;

        10.0 / gain * (adc_reading - 2047.0) / 4095.0 + level
    }
}

//  ┌────────────────────────────────────────────────────────────────────────────────────────────┐
//  │                                                                                            │
//  │   Setting the range Vm = [0, 3.3] yields                                                   │
//  │                                                                                            │
//  │   Vmin = (Rg/(Rg+5k))(385.44/125.44)Vl - (196/125.44)3.3                                   │
//  │   Vmax = (Rg/(Rg+5k))(385.44/125.44)Vl - (196/125.44)3.3 + (385.44/125.44)(5k/(Rg+5k))3.3  │
//  │                                                                                            │
//  │                                                                                            │
//  │                                                                                            │
//  │   Solving the system of equations to get Rg and Vl                                         │
//  │                                                                                            │
//  │   Rg = 5k(3.3*385.44/125.44/(Vmax - Vmin) - 1)                                             │
//  │   Vl = (Vmin + 196/125.44*3.3) / ((Vmin - Vmax)/3.3 + 385.44/125.44)                       │
//  │                                                                                            │
//  └────────────────────────────────────────────────────────────────────────────────────────────┘

impl super::AnalogInput {
    pub fn set_range(&mut self, vmin: f64, vmax: f64) {
        // let gain_resistance = 5000.0 * (3.3 * 385.44 / 125.44 / (vmax - vmin) - 1.0);
        // let offset_voltage = (vmin + 196.0 / 125.44 * 3.3) / ((vmin - vmax) / 3.3 + 385.44 / 125.44);

        // self.set_gain(gain_resistance);
        // self.set_offset(offset_voltage);

        let level = (vmax + vmin) / 2.0;
        let gain = (vmax - vmin) / 10.0;

        self.set_gain_legacy(gain);
        self.set_level_legacy(level);
    }
}