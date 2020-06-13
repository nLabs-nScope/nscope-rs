use git_version::git_version;
use semver::Version;

pub fn ver() -> String {
    let cargo_version = env!("CARGO_PKG_VERSION").to_owned();
    let git_description = git_version!(args = ["--always", "--tags", "--dirty"]);

    match Version::parse(git_description) {
        Ok(git_semver) => match git_semver.pre.len() {
            0 => cargo_version,
            _ => format!("{}+{}", cargo_version, git_semver.pre[0].to_string()),
        },
        Err(_) => cargo_version,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn cargo_version_matches_git_tag() {
        let cargo_version = env!("CARGO_PKG_VERSION").to_owned();
        let git_description = git_version!(args = ["--always", "--tags", "--dirty"]);

        // Use a `+` to separate the tag info from build info in the git description
        let re = Regex::new(r"(?P<ver>\S+)-(?P<build>\d+-g[0-9a-f]{7})").unwrap();
        let git_description = re.replace_all(git_description, "${ver}+${build}");

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
                && cargo_semver.patch == git_semver.patch
                && cargo_semver.pre == git_semver.pre,
            "cargo version {} does not match git description {}",
            cargo_version,
            git_description
        );
    }

    #[test]
    fn version_is_valid_semver() {
        let v = ver();
        assert!(Version::parse(&v).is_ok(), "invalid version: {}", v);
    }
}
