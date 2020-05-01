mod lab_bench;
mod nscope;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

pub use lab_bench::LabBench;

#[derive(Debug)]
pub enum NscopeError {
    BenchError { message: String },
    UnknownError { message: String },
}

pub fn version() {
    println!("{:?}", built_info::GIT_VERSION);
    println!("{:?}", built_info::CI_PLATFORM);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version_works() {
        assert!(built_info::GIT_VERSION.is_some());
    }
}
