mod lab_bench;
mod nscope;

use git_version::git_version;
use semver::Version;

pub use lab_bench::LabBench;

#[derive(Debug)]
pub enum NscopeError {
    BenchError { message: String },
    UnknownError { message: String },
}

pub fn ver() -> Result<String, String> {
    let cargo_version = env!("CARGO_PKG_VERSION").to_owned();
    let git_description = git_version!(args = ["--always", "--tags", "--dirty"]);

    let build_str = match Version::parse(git_description) {
        Ok(git_semver) => format!("{}", git_semver.pre[0].to_string()),
        Err(_) => git_description.to_string(),
    };
    Ok(format!("{}+{}", cargo_version, build_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_version_matches_git_tag() {
        let cargo_version = env!("CARGO_PKG_VERSION").to_owned();
        let git_description = git_version!(args = ["--always", "--tags", "--dirty"]);

        let cargo_semver = Version::parse(&cargo_version);
        let git_semver = Version::parse(&git_description);

        assert!(
            cargo_semver.is_ok(),
            "invalid semver cargo version: {}",
            cargo_version
        );
        assert!(
            git_semver.is_ok(),
            "invalid semver in git description: {}",
            git_description
        );
        let cargo_semver = cargo_semver.unwrap();
        let git_semver = git_semver.unwrap();

        assert!(
            cargo_semver.major == git_semver.major
                && cargo_semver.minor == git_semver.minor
                && cargo_semver.patch == git_semver.patch,
            "cargo version {} does not match git description {}",
            cargo_version,
            git_description
        );
    }

    #[test]
    fn version_is_valid_semver() {
        let v = ver();
        assert!(v.is_ok(), "invalid version: {}", v.unwrap());
        let v = v.unwrap();
        assert!(Version::parse(&v).is_ok(), "invalid version: {}", v);
        println!("v{}", ver().unwrap())
    }
}
