use hidapi::HidApi;
use std::fmt;

mod nscope;
use crate::nscope::Nscope;

pub struct LabBench {
    hid_api: HidApi,
    nscopes: Vec<Nscope>,
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Available devices:{:#X?}", self.nscopes)
    }
}

impl LabBench {
    pub fn new() -> Result<LabBench, String> {
        let hid_api = match HidApi::new() {
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(format!("{}", e));
            }
            Ok(api) => api,
        };

        let mut bench = LabBench {
            hid_api,
            nscopes: vec![],
        };
        bench.refresh();
        Ok(bench)
    }

    pub fn refresh(&mut self) {
        self.nscopes.clear();
        for d in self.hid_api.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                self.nscopes.push(Nscope::new(d));
            }
        }
    }

    pub fn open(&mut self) -> Result<(), String> {
        if self.nscopes.len() > 0 {
            return self.nscopes[0].open(&self.hid_api);
        }
        Err(String::from("Help"))
    }
}
