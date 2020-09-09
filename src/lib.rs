mod lab_bench;
mod nscope;
mod ver;

pub use lab_bench::LabBench;
pub use ver::ver;
use hidapi::HidApi;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref HIDAPI: HidApi = HidApi::new().unwrap();
}

#[derive(Debug)]
pub enum NscopeError {
    BenchError { message: String },
    UnknownError { message: String },
}
