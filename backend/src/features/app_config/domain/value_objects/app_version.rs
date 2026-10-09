use crate::features::app_config::domain::AppConfigError;

/// More than this cannot be a version: three numbers of ten digits and two
/// dots. A header longer than that is not read at all.
const MAX_LENGTH: usize = 32;

/// A version of the farmer app: `major.minor.patch`. The order is the order
/// of the three numbers, so `1.10.0` is newer than `1.9.0` although it
/// sorts before it as text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl AppVersion {
    pub const fn from_parts(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Exactly three parts, each of digits only: no `v`, no sign, no
    /// `-beta`. What cannot be compared for certain is not a version.
    pub fn new(value: &str) -> Result<Self, AppConfigError> {
        let bad = || AppConfigError::BadVersion(value.chars().take(MAX_LENGTH).collect());

        let value = value.trim();

        if value.len() > MAX_LENGTH {
            return Err(bad());
        }

        let mut parts = value.split('.').map(|part| {
            if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }

            part.parse::<u32>().ok()
        });

        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(Some(major)), Some(Some(minor)), Some(Some(patch)), None) => {
                Ok(Self::from_parts(major, minor, patch))
            }
            _ => Err(bad()),
        }
    }
}

impl std::fmt::Display for AppVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl From<&AppVersion> for String {
    fn from(value: &AppVersion) -> Self {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(value: &str) -> AppVersion {
        AppVersion::new(value).expect("version")
    }

    #[test]
    fn reads_three_numbers() {
        assert_eq!(version("1.0.3"), AppVersion::from_parts(1, 0, 3));
        assert_eq!(version(" 12.34.56 "), AppVersion::from_parts(12, 34, 56));
    }

    #[test]
    fn the_numbers_are_compared_as_numbers_not_as_text() {
        assert!(
            version("1.10.0") > version("1.9.0"),
            "as text 1.10.0 sorts before 1.9.0"
        );
        assert!(version("2.0.0") > version("1.99.99"));
        assert!(version("1.0.10") > version("1.0.9"));
        assert!(version("1.2.3") < version("1.2.4"));
    }

    #[test]
    fn the_major_number_outweighs_the_minor_and_the_minor_the_patch() {
        assert!(version("2.0.0") > version("1.9.9"));
        assert!(version("1.1.0") > version("1.0.99"));
    }

    #[test]
    fn equal_versions_are_equal_however_they_were_written() {
        assert_eq!(version("1.02.0"), version("1.2.0"));
        assert_eq!(version("1.02.0").to_string(), "1.2.0");
    }

    #[test]
    fn it_prints_as_it_is_read() {
        assert_eq!(version("1.10.0").to_string(), "1.10.0");
        assert_eq!(String::from(&version("0.0.1")), "0.0.1");
    }

    #[test]
    fn anything_that_is_not_three_plain_numbers_is_refused() {
        for bad in [
            "",
            "1",
            "1.0",
            "1.0.0.0",
            "v1.0.0",
            "1.0.0-beta",
            "1.0.x",
            "1..0",
            "1.0.",
            "+1.0.0",
            "-1.0.0",
            "1.0.0 beta",
            "1,0,0",
            "99999999999.0.0",
        ] {
            assert!(
                matches!(AppVersion::new(bad), Err(AppConfigError::BadVersion(_))),
                "{bad:?} must not be read as a version"
            );
        }
    }

    #[test]
    fn an_absurdly_long_value_is_refused_without_being_echoed_whole() {
        let Err(AppConfigError::BadVersion(echoed)) = AppVersion::new(&"1".repeat(5_000)) else {
            panic!("a 5000 character version must be refused");
        };

        assert!(echoed.len() <= MAX_LENGTH);
    }
}
