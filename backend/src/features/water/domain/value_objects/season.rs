use crate::features::water::domain::WaterError;

/// A growing season, which runs from autumn into the next year: `2026-27`.
/// The text form sorts in time order, so the latest season is the greatest.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Season(String);

impl Season {
    pub fn new(value: String) -> Result<Self, WaterError> {
        let (Some(first), Some("-"), Some(second), 7) =
            (value.get(..4), value.get(4..5), value.get(5..), value.len())
        else {
            return Err(WaterError::InvalidSeason);
        };

        let all_digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());

        if !all_digits(first) || !all_digits(second) {
            return Err(WaterError::InvalidSeason);
        }

        let first_year: u32 = first.parse().map_err(|_| WaterError::InvalidSeason)?;
        let second_year: u32 = second.parse().map_err(|_| WaterError::InvalidSeason)?;

        // The two years must follow each other. 1999-00 is a season too.
        if (first_year + 1) % 100 != second_year {
            return Err(WaterError::InvalidSeason);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&Season> for String {
    fn from(value: &Season) -> Self {
        value.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_consecutive_years_are_a_season() {
        assert_eq!(
            Season::new("2026-27".to_string()).expect("season").as_str(),
            "2026-27"
        );
        assert!(Season::new("2009-10".to_string()).is_ok());
    }

    #[test]
    fn a_season_can_cross_a_century() {
        assert!(Season::new("1999-00".to_string()).is_ok());
        assert!(Season::new("2099-00".to_string()).is_ok());
    }

    #[test]
    fn years_that_do_not_follow_each_other_are_refused() {
        for bad in ["2026-28", "2026-26", "2026-25", "2026-00"] {
            assert!(
                matches!(Season::new(bad.to_string()), Err(WaterError::InvalidSeason)),
                "{bad:?} was accepted"
            );
        }
    }

    #[test]
    fn any_other_shape_is_refused() {
        for bad in [
            "",
            "2026",
            "2026-2027",
            "2026/27",
            "26-27",
            "2026-7",
            "20a6-27",
            "2026-2b",
            "+026-27",
            " 2026-27",
            "٢٠٢٦-٢٧",
        ] {
            assert!(
                Season::new(bad.to_string()).is_err(),
                "{bad:?} was accepted"
            );
        }
    }

    #[test]
    fn later_seasons_sort_after_earlier_ones() {
        let earlier = Season::new("2025-26".to_string()).expect("season");
        let later = Season::new("2026-27".to_string()).expect("season");

        assert!(later > earlier);
    }
}
