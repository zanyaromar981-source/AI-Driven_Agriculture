use crate::{features::farms::domain::FarmError, shared::DomainError};

/// The crop painted on a cell. `Empty` is a cell inside the outline that the
/// farmer has not painted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Crop {
    Wheat,
    Barley,
    Tomato,
    Cucumber,
    Potato,
    Onion,
    Watermelon,
    Grape,
    Olive,
    Sunflower,
    Chickpea,
    Empty,
}

impl Crop {
    pub const ALL: [Crop; 12] = [
        Crop::Wheat,
        Crop::Barley,
        Crop::Tomato,
        Crop::Cucumber,
        Crop::Potato,
        Crop::Onion,
        Crop::Watermelon,
        Crop::Grape,
        Crop::Olive,
        Crop::Sunflower,
        Crop::Chickpea,
        Crop::Empty,
    ];
}

impl From<Crop> for String {
    fn from(value: Crop) -> Self {
        match value {
            Crop::Wheat => "wheat".to_string(),
            Crop::Barley => "barley".to_string(),
            Crop::Tomato => "tomato".to_string(),
            Crop::Cucumber => "cucumber".to_string(),
            Crop::Potato => "potato".to_string(),
            Crop::Onion => "onion".to_string(),
            Crop::Watermelon => "watermelon".to_string(),
            Crop::Grape => "grape".to_string(),
            Crop::Olive => "olive".to_string(),
            Crop::Sunflower => "sunflower".to_string(),
            Crop::Chickpea => "chickpea".to_string(),
            Crop::Empty => "empty".to_string(),
        }
    }
}

impl TryFrom<&str> for Crop {
    type Error = FarmError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "wheat" => Ok(Crop::Wheat),
            "barley" => Ok(Crop::Barley),
            "tomato" => Ok(Crop::Tomato),
            "cucumber" => Ok(Crop::Cucumber),
            "potato" => Ok(Crop::Potato),
            "onion" => Ok(Crop::Onion),
            "watermelon" => Ok(Crop::Watermelon),
            "grape" => Ok(Crop::Grape),
            "olive" => Ok(Crop::Olive),
            "sunflower" => Ok(Crop::Sunflower),
            "chickpea" => Ok(Crop::Chickpea),
            "empty" => Ok(Crop::Empty),
            _ => Err(DomainError::InvalidValue(format!("Invalid crop: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_crop_round_trips() {
        for crop in Crop::ALL {
            let stored = String::from(crop);
            let parsed = Crop::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{crop:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, crop, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_form_is_the_lower_case_code_the_app_uses() {
        assert_eq!(String::from(Crop::Wheat), "wheat");
        assert_eq!(String::from(Crop::Empty), "empty");
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Crop::try_from("rice").is_err());
        assert!(Crop::try_from("").is_err());
        assert!(Crop::try_from("Wheat").is_err());
    }
}
