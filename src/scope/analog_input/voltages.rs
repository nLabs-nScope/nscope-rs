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
impl super::AnalogInput {
    fn set_gain(&mut self, gain_resistance: f64) {
        let desired_gain_setting = 256.0 * (gain_resistance - 75.0) / 100_000.0;
        self.gain_setting = desired_gain_setting as u8;
    }

    fn gain_resistance(&self) -> f64 {
        100_000.0 * self.gain_setting as f64 / 256.0 + 75.0
    }
}

//  ┌──────────────────────────────────────────────────────────────┐
//  │   The voltage level is set with a digital register, L        │
//  │                                                              │
//  │   Vl = 3.3*L/64                                              │
//  │   L  = Vl*64/3.3                                             │
//  └──────────────────────────────────────────────────────────────┘

impl super::AnalogInput {
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
    pub(crate) fn measurement_from_voltage(&self, voltage: f64) -> i16 {
        let a = (self.gain_resistance() + 5000.0) / 5000.0;
        let b = 125.44 / 385.44;
        let c = 196.0 / 385.44;
        let d =  self.gain_resistance() / 5000.0;

        let adc_voltage = a * b * voltage + a * c * 3.3 - d * self.offset_voltage();

        (adc_voltage * 4095.0 / 3.3) as i16
    }
}

impl super::AnalogInput {
    pub(crate) fn voltage_from_measurement(&self, adc_data: u16) -> f64 {

        let adc_voltage = adc_data as f64 * 3.3 / 4095.0;

        let a = 385.44 / 125.44;
        let b = 5000.0 / (self.gain_resistance() + 5000.0);
        let c = 196.0 / 125.44;

        a * b * adc_voltage + (1.0 - b) * a * self.offset_voltage() - c * 3.3
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
        let gain_resistance = 5000.0 * (3.3 * 385.44 / 125.44 / (vmax - vmin) - 1.0);
        let offset_voltage = (vmin + 196.0 / 125.44 * 3.3) / ((vmin - vmax) / 3.3 + 385.44 / 125.44);

        self.set_gain(gain_resistance);
        self.set_offset(offset_voltage);
    }
}