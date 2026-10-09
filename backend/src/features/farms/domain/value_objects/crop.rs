use std::fmt;

use crate::features::farms::domain::FarmError;

const MIN_LENGTH: usize = 2;
const MAX_LENGTH: usize = 24;

/// What a cell is painted with: the code of a crop, or [`Crop::EMPTY`] for
/// a cell inside the outline that the farmer has not painted.
///
/// Which crops exist is no longer fixed here: staff keep the list, and
/// [`ActiveCrops`](crate::features::farms::domain::ActiveCrops) says which
/// of them new data may use. This type only knows what a code looks like: 2
/// to 24 lower-case letters and underscores. A code is that short, so it is
/// held in place and a cell stays as cheap to copy as when the crops were
/// an enum.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Crop {
    // Unused bytes are zero, so equal codes are equal values and the derived
    // order is the alphabetical order of the codes.
    bytes: [u8; MAX_LENGTH],
    length: u8,
}

impl Crop {
    /// The farms feature's own word for an unplanted cell. It is not a crop
    /// and is never in the crop list.
    pub const EMPTY: Crop = Crop::held(b"empty");

    pub fn new(code: &str) -> Result<Self, FarmError> {
        let well_formed = (MIN_LENGTH..=MAX_LENGTH).contains(&code.len())
            && code
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_');

        if !well_formed {
            return Err(FarmError::UnknownCrop(code.to_string()));
        }

        Ok(Self::held(code.as_bytes()))
    }

    /// The caller has checked the length and the letters.
    const fn held(code: &[u8]) -> Self {
        let mut bytes = [0; MAX_LENGTH];
        let mut index = 0;

        while index < code.len() {
            bytes[index] = code[index];
            index += 1;
        }

        Self {
            bytes,
            length: code.len() as u8,
        }
    }

    pub fn as_str(&self) -> &str {
        // Only ASCII letters and underscores are ever held.
        std::str::from_utf8(&self.bytes[..usize::from(self.length)]).unwrap_or_default()
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    /// A crop for a test, where the code is known to be well formed.
    #[cfg(test)]
    pub fn of(code: &str) -> Self {
        Self::new(code).expect("crop code")
    }
}

impl From<Crop> for String {
    fn from(value: Crop) -> Self {
        value.as_str().to_string()
    }
}

impl fmt::Debug for Crop {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Crop({})", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The codes the farms feature accepted when the list was fixed in code.
    const SEEDED: [&str; 11] = [
        "wheat",
        "barley",
        "tomato",
        "cucumber",
        "potato",
        "onion",
        "watermelon",
        "grape",
        "olive",
        "sunflower",
        "chickpea",
    ];

    /// Every code must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_seeded_code_and_the_empty_mark_round_trip() {
        for code in SEEDED.into_iter().chain(["empty"]) {
            let crop = Crop::new(code).unwrap_or_else(|err| panic!("{code:?}: {err:?}"));
            let stored = String::from(crop);

            assert_eq!(stored, code, "{code:?} did not round trip");
            assert_eq!(Crop::new(&stored).expect("parsed"), crop);
        }
    }

    #[test]
    fn the_stored_form_is_the_lower_case_code_the_app_uses() {
        assert_eq!(String::from(Crop::of("wheat")), "wheat");
        assert_eq!(String::from(Crop::EMPTY), "empty");
    }

    #[test]
    fn a_code_staff_added_later_is_a_crop_like_any_other() {
        assert_eq!(Crop::of("rice").as_str(), "rice");
        assert_eq!(Crop::of("sugar_beet").as_str(), "sugar_beet");
    }

    #[test]
    fn anything_that_could_not_be_a_code_is_rejected_and_named() {
        for bad in ["", "a", "Wheat", "wheat 2", "wheat2", "گەنم"] {
            assert!(
                matches!(Crop::new(bad), Err(FarmError::UnknownCrop(code)) if code == bad),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn the_length_limit_is_inclusive() {
        assert!(Crop::new(&"a".repeat(MAX_LENGTH)).is_ok());
        assert!(Crop::new(&"a".repeat(MAX_LENGTH + 1)).is_err());
    }

    #[test]
    fn only_the_empty_mark_is_empty() {
        assert!(Crop::EMPTY.is_empty());
        assert!(Crop::of("empty").is_empty());
        assert!(!Crop::of("wheat").is_empty());
    }

    #[test]
    fn crops_order_by_their_codes() {
        let mut crops = vec![Crop::of("wheat"), Crop::of("barley"), Crop::of("bar")];
        crops.sort();

        assert_eq!(
            crops,
            vec![Crop::of("bar"), Crop::of("barley"), Crop::of("wheat")]
        );
    }

    #[test]
    fn a_crop_prints_as_its_code() {
        assert_eq!(format!("{:?}", Crop::of("wheat")), "Crop(wheat)");
    }
}
