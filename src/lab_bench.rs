use crate::nscope::Nscope;
use crate::NscopeError;
use crate::NscopeError::BenchError;
use hidapi::HidApi;
use std::fmt;

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
    pub fn new() -> Result<LabBench, NscopeError> {
        let hid_api = match HidApi::new() {
            Err(e) => {
                return Err(BenchError {
                    message: e.to_string(),
                });
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

    pub fn refresh(&mut self) -> &Vec<Nscope> {
        self.nscopes.clear();
        for d in self.hid_api.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                self.nscopes.push(Nscope::new(d));
            }
        }
        &self.nscopes
    }

    pub fn nscopes(&self) -> &Vec<Nscope> {
        &self.nscopes
    }

    pub fn open(&mut self, name: &str) -> Result<&mut Nscope, NscopeError> {
        match self.nscopes.len() {
            0 => Err(BenchError {
                message: "Cannot find an nScope to open".to_string(),
            }),
            1 => {
                self.nscopes[0].name = String::from(name);
                match self.nscopes[0].open(&self.hid_api) {
                    Ok(()) => Ok(&mut self.nscopes[0]),
                    Err(e) => Err(e),
                }
            }
            _ => Err(BenchError {
                message: "Must specify one nScope using bench.open_one()".to_string(),
            }),
        }
    }

    pub fn open_one(&mut self, idx: usize, name: &str) -> Result<&mut Nscope, NscopeError> {
        if idx < self.nscopes.len() {
            self.nscopes[idx].name = String::from(name);
            match self.nscopes[idx].open(&self.hid_api) {
                Ok(()) => Ok(&mut self.nscopes[idx]),
                Err(e) => Err(e),
            }
        } else {
            Err(BenchError {
                message: "Cannot find that nScope".to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bench_can_refresh() {
        let bench = LabBench::new();
        assert!(bench.is_ok());
        let mut bench = bench.unwrap();
        let nscopes = bench.refresh();
        assert_eq!(nscopes.len(), 0);
    }
}
