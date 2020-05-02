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

pub fn ver() -> String {
    let mut build = String::from(env!("CARGO_PKG_VERSION"));
    build += "+";

    if built_info::GIT_VERSION.is_some() {
        build.push_str(built_info::GIT_VERSION.unwrap_or(""));
    }
    if built_info::CI_PLATFORM.is_some() {
        build.push_str(built_info::CI_PLATFORM.unwrap_or(""));
    }
    build
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn version_is_valid_semver() {
        let v = ver();
        assert!(Version::parse(&v).is_ok());
        println!("v{}", v);
    }
}
