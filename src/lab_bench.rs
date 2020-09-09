use crate::nscope::Nscope;
use std::fmt;
use crate::{HIDAPI, NscopeError};
use std::cell::{RefCell, RefMut};
use std::ops::{Deref, DerefMut};
use std::ffi::CString;

pub struct LabBench {
    pub nscopes: Vec<DetectedNscope>,
}

pub struct BorrowedNscope<'a> {
    path: CString,
    scope: RefMut<'a, Nscope>,
}

pub struct DetectedNscope {
    path: CString,
    scope: RefCell<Nscope>,
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
        // self.nscopes.clear();

        for d in HIDAPI.device_list() {
            if d.product_id() == 0xf3f6 && d.vendor_id() == 0x04d8 {
                self.nscopes.push(DetectedNscope {
                    path: d.path().to_owned(),
                    scope: RefCell::new(Nscope::new(d)),
                });
            }
        }
    }
}

impl DetectedNscope {
    pub fn checkout(&self) -> Result<BorrowedNscope, NscopeError> {
        let mut scope = self.scope.try_borrow_mut().unwrap();
        scope.hid_device = Some(
            HIDAPI.open_path(&self.path).unwrap()
        );
        Ok(BorrowedNscope { path: self.path.clone(), scope })
    }
}

impl BorrowedNscope<'_> {
    pub fn checkin(mut self) {
        self.scope.hid_device = None;
    }
}

impl Drop for BorrowedNscope<'_> {
    fn drop(&mut self) {
        self.scope.hid_device = None
    }
}

impl Deref for BorrowedNscope<'_> {
    type Target = Nscope;
    fn deref(&self) -> &Nscope {
        &*self.scope
    }
}

impl DerefMut for BorrowedNscope<'_> {
    fn deref_mut(&mut self) -> &mut Nscope {
        &mut *self.scope
    }
}

impl fmt::Debug for LabBench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LabBench: {:#X?}", self.nscopes)
    }
}

impl fmt::Debug for BorrowedNscope<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.scope)
    }
}

impl fmt::Debug for DetectedNscope {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.scope.try_borrow() {
            Ok(scope) => write!(f, "nScope: {:?}", scope),
            Err(_) => write!(f, "checked out nScope"),
        }
    }
}