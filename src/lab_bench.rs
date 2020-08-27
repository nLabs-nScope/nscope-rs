use crate::nscope::Nscope;
use std::fmt;
use crate::HIDAPI;
use std::cell::{RefCell, RefMut};

pub struct LabBench {
    nscopes: Vec<RefCell<Nscope>>,
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Available devices: {:#X?}", self.nscopes)
    }
}

impl LabBench {
    pub fn new() -> LabBench {

        let mut bench = LabBench {
            nscopes: vec![],
        };
        bench.refresh();
        bench
    }

    pub fn refresh(&mut self) {
        self.nscopes.clear();
        for d in HIDAPI.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                self.nscopes.push(RefCell::new(Nscope::new(d)));
            }
        }
    }

    pub fn checkout(&self, i: usize) -> RefMut<Nscope> {
        let ns = self.nscopes.get(i).unwrap();
        ns.try_borrow_mut().expect("Trying to borrow an already borrowed nScope")
    }

}