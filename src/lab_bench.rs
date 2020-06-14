use crate::nscope::Nscope;
use crate::NscopeError;
use crate::NscopeError::BenchError;
use hidapi::HidApi;
use std::cell::{Ref, RefCell};
use std::fmt;
use std::ops::Deref;

pub struct LabBench {
    hid_api: HidApi,
    nscopes: RefCell<Vec<Nscope>>,
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Available devices: {:#X?}", self.nscopes.borrow())
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

        let bench = LabBench {
            hid_api,
            nscopes: RefCell::new(vec![]),
        };
        bench.refresh();
        Ok(bench)
    }

    pub fn refresh(&self) {
        self.nscopes.borrow_mut().clear();
        for d in self.hid_api.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                self.nscopes.borrow_mut().push(Nscope::new(d));
            }
        }
    }

    pub fn nscopes(&self) -> NscopeList {
        NscopeList(self.nscopes.borrow())
    }

    pub fn open(&self, name: &str) -> Result<(), NscopeError> {
        let number_of_scopes = self.nscopes.borrow().len();
        match number_of_scopes {
            0 => Err(BenchError {
                message: "Cannot find an nScope to open".to_string(),
            }),
            1 => self.open_one(0, name),
            _ => Err(BenchError {
                message: "Must specify one nScope using bench.open_one()".to_string(),
            }),
        }
    }

    pub fn open_one(&self, idx: usize, name: &str) -> Result<(), NscopeError> {
        let mut list = self.nscopes.borrow_mut();
        if idx < list.len() {
            list[idx].open(&self.hid_api, name)
        } else {
            Err(BenchError {
                message: "Cannot find that nScope".to_string(),
            })
        }
    }
}

pub struct NscopeList<'a>(Ref<'a, Vec<Nscope>>);

impl<'a> Deref for NscopeList<'a> {
    type Target = Vec<Nscope>;
    fn deref(&self) -> &Vec<Nscope> {
        &self.0
    }
}

impl<'a> fmt::Debug for NscopeList<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for nscope in self.0.iter() {
            writeln!(f, "{:?}", nscope)?;
        }
        Ok(())
    }
}
