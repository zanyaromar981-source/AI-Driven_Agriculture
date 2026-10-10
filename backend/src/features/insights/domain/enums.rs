use crate::{features::insights::domain::InsightError, shared::DomainError};

/// One thing the farmer app shows about a farm. The variants are declared in
/// the order the app lists them, and that order is what sorts a farm's
/// readings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Topic {
    /// The dam or river that serves the farm.
    SurfaceWater,
    Groundwater,
    Soil,
    Rain,
    Dryness,
    Greenness,
    Weather,
    /// The crops grown around the farm, from MapSPAM 2020.
    CropsGrown,
}

impl Topic {
    pub const ALL: [Topic; 8] = [
        Topic::SurfaceWater,
        Topic::Groundwater,
        Topic::Soil,
        Topic::Rain,
        Topic::Dryness,
        Topic::Greenness,
        Topic::Weather,
        Topic::CropsGrown,
    ];
}

impl From<Topic> for String {
    fn from(value: Topic) -> Self {
        match value {
            Topic::SurfaceWater => "surface_water".to_string(),
            Topic::Groundwater => "groundwater".to_string(),
            Topic::Soil => "soil".to_string(),
            Topic::Rain => "rain".to_string(),
            Topic::Dryness => "dryness".to_string(),
            Topic::Greenness => "greenness".to_string(),
            Topic::Weather => "weather".to_string(),
            Topic::CropsGrown => "crops_grown".to_string(),
        }
    }
}

impl TryFrom<&str> for Topic {
    type Error = InsightError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "surface_water" => Ok(Topic::SurfaceWater),
            "groundwater" => Ok(Topic::Groundwater),
            "soil" => Ok(Topic::Soil),
            "rain" => Ok(Topic::Rain),
            "dryness" => Ok(Topic::Dryness),
            "greenness" => Ok(Topic::Greenness),
            "weather" => Ok(Topic::Weather),
            "crops_grown" => Ok(Topic::CropsGrown),
            _ => Err(DomainError::InvalidValue(format!("Invalid topic: {value}")).into()),
        }
    }
}

/// How far the data job trusts its own reading. The app words the reading
/// differently for each, so a rough estimate is never shown as a fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Confidence {
    Sure,
    Likely,
    Unsure,
}

impl Confidence {
    pub const ALL: [Confidence; 3] = [Confidence::Sure, Confidence::Likely, Confidence::Unsure];
}

impl From<Confidence> for String {
    fn from(value: Confidence) -> Self {
        match value {
            Confidence::Sure => "sure".to_string(),
            Confidence::Likely => "likely".to_string(),
            Confidence::Unsure => "unsure".to_string(),
        }
    }
}

impl TryFrom<&str> for Confidence {
    type Error = InsightError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "sure" => Ok(Confidence::Sure),
            "likely" => Ok(Confidence::Likely),
            "unsure" => Ok(Confidence::Unsure),
            _ => Err(DomainError::InvalidValue(format!("Invalid confidence: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant must survive a trip to the database and back. A mismatch
    /// between the two directions corrupts rows silently rather than failing.
    #[test]
    fn every_topic_round_trips() {
        for topic in Topic::ALL {
            let stored = String::from(topic);
            let parsed = Topic::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{topic:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, topic, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn every_confidence_round_trips() {
        for confidence in Confidence::ALL {
            let stored = String::from(confidence);
            let parsed = Confidence::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{confidence:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, confidence, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn the_stored_form_is_lower_case_snake_case() {
        assert_eq!(String::from(Topic::SurfaceWater), "surface_water");
        assert_eq!(String::from(Confidence::Unsure), "unsure");
    }

    #[test]
    fn an_unknown_stored_value_is_rejected_rather_than_defaulted() {
        assert!(Topic::try_from("pests").is_err());
        assert!(Topic::try_from("Soil").is_err());
        assert!(Topic::try_from("").is_err());
        assert!(Confidence::try_from("certain").is_err());
    }

    #[test]
    fn topics_sort_in_the_order_the_app_lists_them() {
        let mut shuffled = vec![
            Topic::Weather,
            Topic::Soil,
            Topic::SurfaceWater,
            Topic::Greenness,
            Topic::Rain,
            Topic::Groundwater,
            Topic::Dryness,
            Topic::CropsGrown,
        ];
        shuffled.sort();

        assert_eq!(shuffled, Topic::ALL.to_vec());
    }
}
