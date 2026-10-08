use crate::{features::zones::domain::ZoneError, shared::DomainError};

/// How dry a zone is, read off its dryness index. The band is always derived
/// from the index and never stored, so the two cannot disagree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DrynessBand {
    MuchGreener,
    Greener,
    Normal,
    Dry,
    VeryDry,
}

impl DrynessBand {
    pub const ALL: [DrynessBand; 5] = [
        DrynessBand::MuchGreener,
        DrynessBand::Greener,
        DrynessBand::Normal,
        DrynessBand::Dry,
        DrynessBand::VeryDry,
    ];

    /// 0 to 24 much greener, 25 to 44 greener, 45 to 59 normal, 60 to 79 dry,
    /// 80 to 100 very dry.
    pub fn of(dryness: i32) -> Self {
        match dryness {
            ..=24 => DrynessBand::MuchGreener,
            25..=44 => DrynessBand::Greener,
            45..=59 => DrynessBand::Normal,
            60..=79 => DrynessBand::Dry,
            _ => DrynessBand::VeryDry,
        }
    }
}

impl From<DrynessBand> for String {
    fn from(value: DrynessBand) -> Self {
        match value {
            DrynessBand::MuchGreener => "much_greener".to_string(),
            DrynessBand::Greener => "greener".to_string(),
            DrynessBand::Normal => "normal".to_string(),
            DrynessBand::Dry => "dry".to_string(),
            DrynessBand::VeryDry => "very_dry".to_string(),
        }
    }
}

impl TryFrom<&str> for DrynessBand {
    type Error = ZoneError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "much_greener" => Ok(DrynessBand::MuchGreener),
            "greener" => Ok(DrynessBand::Greener),
            "normal" => Ok(DrynessBand::Normal),
            "dry" => Ok(DrynessBand::Dry),
            "very_dry" => Ok(DrynessBand::VeryDry),
            _ => Err(DomainError::InvalidValue(format!("Invalid dryness band: {value}")).into()),
        }
    }
}

/// A crop the data jobs can advise for a zone. Unlike a farm cell, advice is
/// never "empty": a zone with no advice has an empty list.
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
}

impl Crop {
    pub const ALL: [Crop; 11] = [
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
        }
    }
}

impl TryFrom<&str> for Crop {
    type Error = ZoneError;

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
            _ => Err(ZoneError::UnknownCrop(value.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_band_round_trips() {
        for band in DrynessBand::ALL {
            let stored = String::from(band);
            let parsed = DrynessBand::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{band:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, band, "{stored:?} did not round trip");
        }
    }

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
    fn each_band_starts_and_ends_where_the_dashboard_legend_says() {
        for (dryness, band) in [
            (0, DrynessBand::MuchGreener),
            (24, DrynessBand::MuchGreener),
            (25, DrynessBand::Greener),
            (44, DrynessBand::Greener),
            (45, DrynessBand::Normal),
            (59, DrynessBand::Normal),
            (60, DrynessBand::Dry),
            (79, DrynessBand::Dry),
            (80, DrynessBand::VeryDry),
            (100, DrynessBand::VeryDry),
        ] {
            assert_eq!(DrynessBand::of(dryness), band, "dryness {dryness}");
        }
    }

    #[test]
    fn an_unknown_crop_code_is_rejected_and_named() {
        assert!(matches!(
            Crop::try_from("rice"),
            Err(ZoneError::UnknownCrop(code)) if code == "rice"
        ));
        assert!(Crop::try_from("Wheat").is_err());
        assert!(
            Crop::try_from("empty").is_err(),
            "empty is a farm cell state, not advice"
        );
    }

    #[test]
    fn an_unknown_band_is_rejected_rather_than_defaulted() {
        assert!(DrynessBand::try_from("wet").is_err());
        assert!(DrynessBand::try_from("").is_err());
    }
}
