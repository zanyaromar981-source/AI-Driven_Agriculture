use crate::{features::crops::domain::CropError, shared::DomainError};

/// The colour a crop is drawn in on maps and charts: `#rrggbb`. Kept in
/// lower case so the same colour is always the same text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CropColor(String);

impl CropColor {
    pub fn new(value: String) -> Result<Self, CropError> {
        let well_formed = value.len() == 7
            && value.starts_with('#')
            && value.bytes().skip(1).all(|byte| byte.is_ascii_hexdigit());

        if !well_formed {
            return Err(DomainError::InvalidValue(
                "A colour is written as #rrggbb, for example #e0b13a".to_string(),
            )
            .into());
        }

        Ok(Self(value.to_ascii_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&CropColor> for String {
    fn from(value: &CropColor) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_six_digit_hex_colour_is_accepted_and_kept_in_lower_case() {
        assert_eq!(
            CropColor::new("#E0B13A".to_string())
                .expect("colour")
                .as_str(),
            "#e0b13a"
        );
    }

    #[test]
    fn anything_else_is_refused() {
        for bad in [
            "",
            "e0b13a",
            "#e0b13",
            "#e0b13a0",
            "#e0b13g",
            "red",
            "#ڕەنگ",
        ] {
            assert!(CropColor::new(bad.to_string()).is_err(), "{bad:?}");
        }
    }
}
