mod lab_bench;
mod nscope;
use git_version::git_version;

pub use lab_bench::LabBench;

#[derive(Debug)]
pub enum NscopeError {
    BenchError { message: String },
    UnknownError { message: String },
}

pub fn ver() -> String {
    let mut build = String::from(env!("CARGO_PKG_VERSION"));
    build += "+";

    const GIT_VERSION: &str = git_version!();
    build.push_str(GIT_VERSION);
    build
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn version_is_valid_semver() {
        let v = ver();
        println!("v{}", v);
        assert!(Version::parse(&v).is_ok());
    }
}
