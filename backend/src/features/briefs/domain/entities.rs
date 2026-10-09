use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};
use getset::Getters;

use crate::{
    features::briefs::domain::{
        Author, BriefError, BriefPoint, BriefScope, BriefSource, BriefSummary, Headline, ZoneSlug,
    },
    shared::DomainError,
};

pub const MAX_POINTS: usize = 8;
pub const MAX_SOURCES: usize = 12;
pub const MIN_FARM_ZONES: usize = 1;
pub const MAX_FARM_ZONES: usize = 2_000;

/// The brief of one day for one scope: the whole region or one district.
/// The nightly job writes the text; the backend only stores and serves it.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct DailyBrief {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    day: NaiveDate,
    scope: BriefScope,
    headline_en: Headline,
    headline_ku: Headline,
    summary_en: BriefSummary,
    summary_ku: BriefSummary,
    /// In the order the author gave them. May be empty.
    points: Vec<BriefPoint>,
    sources: Vec<BriefSource>,
    author: Author,
    /// When the author finished writing, which is before the push.
    generated_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl DailyBrief {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        day: NaiveDate,
        scope: BriefScope,
        headline_en: Headline,
        headline_ku: Headline,
        summary_en: BriefSummary,
        summary_ku: BriefSummary,
        points: Vec<BriefPoint>,
        sources: Vec<BriefSource>,
        author: Author,
        generated_at: DateTime<Utc>,
    ) -> Result<Self, BriefError> {
        if points.len() > MAX_POINTS {
            return Err(BriefError::TooManyPoints { max: MAX_POINTS });
        }

        if sources.len() > MAX_SOURCES {
            return Err(BriefError::TooManySources { max: MAX_SOURCES });
        }

        Ok(Self {
            id: None,
            day,
            scope,
            headline_en,
            headline_ku,
            summary_en,
            summary_ku,
            points,
            sources,
            author,
            generated_at,
            updated_at: Utc::now(),
        })
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        day: NaiveDate,
        scope: BriefScope,
        headline_en: Headline,
        headline_ku: Headline,
        summary_en: BriefSummary,
        summary_ku: BriefSummary,
        points: Vec<BriefPoint>,
        sources: Vec<BriefSource>,
        author: Author,
        generated_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            day,
            scope,
            headline_en,
            headline_ku,
            summary_en,
            summary_ku,
            points,
            sources,
            author,
            generated_at,
            updated_at,
        }
    }
}

/// Which district a farm lies in, as the nightly job worked it out from the
/// farm's outline. It lets the backend find a farm's brief without geometry.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct FarmZone {
    farm_id: i32,
    zone_slug: ZoneSlug,
}

impl FarmZone {
    pub fn new(farm_id: i32, zone_slug: ZoneSlug) -> Result<Self, BriefError> {
        if farm_id < 1 {
            return Err(
                DomainError::InvalidValue("Farm id must be a positive number".to_string()).into(),
            );
        }

        Ok(Self { farm_id, zone_slug })
    }

    /// Whether one push may name this many farms. Asked on its own before
    /// the farms are parsed, so an over-long push costs nothing.
    pub fn check_count(count: usize) -> Result<(), BriefError> {
        if !(MIN_FARM_ZONES..=MAX_FARM_ZONES).contains(&count) {
            return Err(BriefError::FarmCount {
                min: MIN_FARM_ZONES,
                max: MAX_FARM_ZONES,
            });
        }

        Ok(())
    }

    /// The farms of one push, ready to be stored together. A farm named
    /// twice would have two districts, so it is refused. They come back in
    /// farm order: two pushes that overlap then lock their rows in the same
    /// order and cannot block each other.
    pub fn batch(mut zones: Vec<FarmZone>) -> Result<Vec<FarmZone>, BriefError> {
        Self::check_count(zones.len())?;

        let mut seen = HashSet::new();

        for zone in &zones {
            if !seen.insert(zone.farm_id) {
                return Err(BriefError::DuplicateFarm(zone.farm_id));
            }
        }

        zones.sort_by_key(|zone| zone.farm_id);

        Ok(zones)
    }
}

/// What the farmer app shows for one farm: the district the farm is recorded
/// in, if any, and the brief chosen for it, if any is stored at all.
#[derive(Clone, Debug, PartialEq, Eq, Getters)]
#[getset(get = "pub")]
pub struct FarmBrief {
    farm_id: i32,
    zone_slug: Option<ZoneSlug>,
    brief: Option<DailyBrief>,
}

impl FarmBrief {
    pub fn new(farm_id: i32, zone_slug: Option<ZoneSlug>, brief: Option<DailyBrief>) -> Self {
        Self {
            farm_id,
            zone_slug,
            brief,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::briefs::domain::{PointLevel, PointText, SourceTitle, SourceUrl};

    fn point() -> BriefPoint {
        BriefPoint::new(
            PointLevel::Watch,
            PointText::new("Rain is late".to_string()).expect("text"),
            PointText::new("باران دواکەوتووە".to_string()).expect("text"),
        )
    }

    fn source() -> BriefSource {
        BriefSource::new(
            SourceTitle::new("FAO crop calendar".to_string()).expect("title"),
            SourceUrl::new("https://example.org/calendar".to_string()).expect("url"),
        )
    }

    fn brief(points: usize, sources: usize) -> Result<DailyBrief, BriefError> {
        DailyBrief::new(
            NaiveDate::from_ymd_opt(2026, 10, 9).expect("date"),
            BriefScope::region(),
            Headline::new("A dry week".to_string()).expect("headline"),
            Headline::new("هەفتەیەکی وشک".to_string()).expect("headline"),
            BriefSummary::new("No rain fell.".to_string()).expect("summary"),
            BriefSummary::new("باران نەباری.".to_string()).expect("summary"),
            vec![point(); points],
            vec![source(); sources],
            Author::new("codex-cli gpt-5".to_string()).expect("author"),
            Utc::now(),
        )
    }

    fn zone(farm_id: i32, slug: &str) -> FarmZone {
        FarmZone::new(farm_id, ZoneSlug::new(slug.to_string()).expect("slug")).expect("farm zone")
    }

    #[test]
    fn a_new_brief_is_not_yet_stored() {
        assert!(brief(2, 1).expect("brief").id().is_none());
    }

    #[test]
    fn a_brief_may_have_no_points_and_no_sources() {
        assert!(
            brief(0, 0).is_ok(),
            "a quiet day has nothing to point at, and that is still a brief"
        );
    }

    #[test]
    fn eight_points_is_the_most_a_brief_carries() {
        assert!(brief(MAX_POINTS, 0).is_ok());
        assert!(matches!(
            brief(MAX_POINTS + 1, 0),
            Err(BriefError::TooManyPoints { max: 8 })
        ));
    }

    #[test]
    fn twelve_sources_is_the_most_a_brief_names() {
        assert!(brief(0, MAX_SOURCES).is_ok());
        assert!(matches!(
            brief(0, MAX_SOURCES + 1),
            Err(BriefError::TooManySources { max: 12 })
        ));
    }

    #[test]
    fn a_farm_id_below_one_cannot_name_a_farm() {
        let slug = || ZoneSlug::new("kalar".to_string()).expect("slug");

        assert!(FarmZone::new(0, slug()).is_err());
        assert!(FarmZone::new(-3, slug()).is_err());
        assert!(FarmZone::new(1, slug()).is_ok());
    }

    #[test]
    fn a_push_names_between_one_and_two_thousand_farms() {
        let farms = |count: i32| (1..=count).map(|id| zone(id, "kalar")).collect::<Vec<_>>();

        assert!(matches!(
            FarmZone::batch(vec![]),
            Err(BriefError::FarmCount { min: 1, max: 2_000 })
        ));
        assert!(FarmZone::batch(farms(1)).is_ok());
        assert!(FarmZone::batch(farms(2_000)).is_ok());
        assert!(matches!(
            FarmZone::batch(farms(2_001)),
            Err(BriefError::FarmCount { .. })
        ));
    }

    #[test]
    fn the_same_farm_twice_in_one_push_is_refused_even_with_the_same_zone() {
        let result = FarmZone::batch(vec![zone(12, "kalar"), zone(3, "kifri"), zone(12, "kalar")]);

        assert!(
            matches!(result, Err(BriefError::DuplicateFarm(12))),
            "one statement cannot write the same row twice"
        );
    }

    #[test]
    fn a_batch_comes_back_in_farm_order() {
        let batch = FarmZone::batch(vec![zone(12, "kalar"), zone(3, "kifri"), zone(7, "kalar")])
            .expect("batch");

        let ids: Vec<i32> = batch.iter().map(|zone| *zone.farm_id()).collect();

        assert_eq!(ids, vec![3, 7, 12]);
    }
}
