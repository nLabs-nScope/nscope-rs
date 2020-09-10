mod lab_bench;
mod nscope;
mod ver;

pub use lab_bench::LabBench;
pub use lab_bench::NscopeInfo;
pub use nscope::Nscope;
pub use ver::ver;


#[derive(Debug)]
pub enum NscopeError {
    BenchError { message: String },
    UnknownError { message: String },
}
