use hidapi::HidApi;
use std::fmt;

pub struct LabBench {
    _hid_api: HidApi,
    nscopes: Vec<Nscope>,
}

#[derive(Debug)]
pub struct Nscope {
    is_open: bool,
    path: String,
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Available devices:{:#?}", self.nscopes)
    }
}

impl LabBench {
    pub fn new() -> Result<LabBench, String> {
        let _hid_api = match HidApi::new() {
            Err(e) => {
                eprintln!("Error: {}", e);
                return Err(format!("{}", e));
            }
            Ok(api) => api,
        };

        let mut bench = LabBench {
            _hid_api,
            nscopes: vec![],
        };
        bench.refresh();
        Ok(bench)
    }

    pub fn refresh(&mut self) {
        self.nscopes.clear();
        for d in self._hid_api.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                let path = String::from(d.path().to_str().expect("Failed to get device path"));
                self.nscopes.push(Nscope {
                    is_open: false,
                    path,
                });
            }
        }
    }
}
