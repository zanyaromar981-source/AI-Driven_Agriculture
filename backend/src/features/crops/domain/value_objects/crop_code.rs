use crate::{features::crops::domain::CropError, shared::DomainError};

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 24;

/// The farms feature's word for a cell nothing is planted on. It is stored
/// where crop codes are stored, so no crop may ever take it.
const UNPLANTED: &str = "empty";

/// The fixed name of a crop, as farms, listings and prices store it: 2 to 24
/// lower-case letters and underscores, for example `wheat`. It never changes
/// once the crop exists.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CropCode(String);

impl CropCode {
    pub fn new(value: String) -> Result<Self, CropError> {
        let well_formed = (MIN_LENGTH..=MAX_LENGTH).contains(&value.len())
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_');

        if !well_formed {
            return Err(DomainError::InvalidValue(format!(
                "A crop code is {MIN_LENGTH} to {MAX_LENGTH} lower-case letters and underscores"
            ))
            .into());
        }

        if value == UNPLANTED {
            return Err(CropError::ReservedCode(value));
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&CropCode> for String {
    fn from(value: &CropCode) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_seeded_shapes_are_accepted() {
        for code in ["wheat", "watermelon", "sugar_beet", "ab"] {
            assert!(CropCode::new(code.to_string()).is_ok(), "{code:?}");
        }
    }

    #[test]
    fn anything_that_could_not_be_a_code_is_refused() {
        for bad in ["", "a", "Wheat", "wheat 2", "wheat2", "wheat-a", "گەنم"] {
            assert!(CropCode::new(bad.to_string()).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(CropCode::new("a".repeat(MAX_LENGTH)).is_ok());
        assert!(CropCode::new("a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn the_word_for_an_unplanted_cell_can_never_be_a_crop() {
        assert!(matches!(
            CropCode::new("empty".to_string()),
            Err(CropError::ReservedCode(code)) if code == "empty"
        ));
    }
}
