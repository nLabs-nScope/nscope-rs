/***************************************************************************************************
 *
 *  nLabs, LLC
 *  https://getnlab.com
 *  Copyright(c) 2020. All Rights Reserved
 *
 *  This file is part of the nLab API
 *
 **************************************************************************************************/

use git_version::git_version;
use regex::Regex;

/// Returns the current version of the nLab API
pub fn version() -> String {
    let git_description = git_version!(
        args = ["--always", "--tags", "--dirty"],
        suffix = "",
        cargo_suffix = ""
    );

    // Use a `+` to separate the tag info from build info in the git description
    let re = Regex::new(r"(?P<ver>\S+)-(?P<build>\d+-g[0-9a-f]{7})").unwrap();
    re.replace_all(git_description, "${ver}+${build}")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use semver::Version;

    #[test]
    fn cargo_version_matches_git_version() {
        let cargo_version = env!("CARGO_PKG_VERSION").to_owned();
        let git_version = version();

        let cargo_semver = Version::parse(&cargo_version);
        let git_semver = Version::parse(&git_version);

        assert!(
            cargo_semver.is_ok(),
            "invalid semver cargo version: {}",
            cargo_version
        );
        assert!(
            git_semver.is_ok(),
            "invalid semver in git version: {}",
            git_version
        );
        let cargo_semver = cargo_semver.unwrap();
        let git_semver = git_semver.unwrap();

        assert!(
            cargo_semver.major == git_semver.major
                && cargo_semver.minor == git_semver.minor
                && cargo_semver.patch == git_semver.patch
                && cargo_semver.pre == git_semver.pre,
            "cargo version {} does not match git version {}",
            cargo_version,
            git_version
        );
    }

    #[test]
    fn version_is_valid_semver() {
        let v = version();
        assert!(Version::parse(&v).is_ok(), "invalid version: {}", v);
    }
}
