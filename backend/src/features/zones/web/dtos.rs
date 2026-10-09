use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::features::zones::{
    app::{
        AppError,
        use_cases::{RecordSubZoneReadingInput, RecordZoneReadingInput, ZoneWithSubZones},
    },
    domain::{
        self, Dryness, GreennessPctVsNormal, Month, RainPctOfNormal, RankedReading, ReadingSource,
        RegionComparison, RegionOverview, RegionSummary, SubZone, SubZoneDryness, SubZoneReading,
        WaterNeed, YearAverage, YearComparison, YearDryness, ZoneComparison, ZoneDetail,
        ZoneOverview, ZoneReading, ZoneSlug,
    },
};

/// 0 to 24 `much_greener`, 25 to 44 `greener`, 45 to 59 `normal`, 60 to 79
/// `dry`, 80 to 100 `very_dry`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DrynessBand {
    MuchGreener,
    Greener,
    Normal,
    Dry,
    VeryDry,
}

impl From<DrynessBand> for domain::DrynessBand {
    fn from(value: DrynessBand) -> Self {
        match value {
            DrynessBand::MuchGreener => domain::DrynessBand::MuchGreener,
            DrynessBand::Greener => domain::DrynessBand::Greener,
            DrynessBand::Normal => domain::DrynessBand::Normal,
            DrynessBand::Dry => domain::DrynessBand::Dry,
            DrynessBand::VeryDry => domain::DrynessBand::VeryDry,
        }
    }
}

impl From<domain::DrynessBand> for DrynessBand {
    fn from(value: domain::DrynessBand) -> Self {
        match value {
            domain::DrynessBand::MuchGreener => DrynessBand::MuchGreener,
            domain::DrynessBand::Greener => DrynessBand::Greener,
            domain::DrynessBand::Normal => DrynessBand::Normal,
            domain::DrynessBand::Dry => DrynessBand::Dry,
            domain::DrynessBand::VeryDry => DrynessBand::VeryDry,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BestCrop {
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

impl From<BestCrop> for domain::Crop {
    fn from(value: BestCrop) -> Self {
        match value {
            BestCrop::Wheat => domain::Crop::Wheat,
            BestCrop::Barley => domain::Crop::Barley,
            BestCrop::Tomato => domain::Crop::Tomato,
            BestCrop::Cucumber => domain::Crop::Cucumber,
            BestCrop::Potato => domain::Crop::Potato,
            BestCrop::Onion => domain::Crop::Onion,
            BestCrop::Watermelon => domain::Crop::Watermelon,
            BestCrop::Grape => domain::Crop::Grape,
            BestCrop::Olive => domain::Crop::Olive,
            BestCrop::Sunflower => domain::Crop::Sunflower,
            BestCrop::Chickpea => domain::Crop::Chickpea,
        }
    }
}

impl From<domain::Crop> for BestCrop {
    fn from(value: domain::Crop) -> Self {
        match value {
            domain::Crop::Wheat => BestCrop::Wheat,
            domain::Crop::Barley => BestCrop::Barley,
            domain::Crop::Tomato => BestCrop::Tomato,
            domain::Crop::Cucumber => BestCrop::Cucumber,
            domain::Crop::Potato => BestCrop::Potato,
            domain::Crop::Onion => BestCrop::Onion,
            domain::Crop::Watermelon => BestCrop::Watermelon,
            domain::Crop::Grape => BestCrop::Grape,
            domain::Crop::Olive => BestCrop::Olive,
            domain::Crop::Sunflower => BestCrop::Sunflower,
            domain::Crop::Chickpea => BestCrop::Chickpea,
        }
    }
}

fn band_of(dryness: &Dryness) -> DrynessBand {
    dryness.band().into()
}

/// `?month=YYYY-MM`. It is read as text so that a month written wrongly is
/// answered with `bad_month` rather than a bare bad request.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct MonthQueryDto {
    pub month: Option<String>,
}

impl MonthQueryDto {
    pub fn into_month(self) -> Result<Option<Month>, AppError> {
        Ok(self.month.as_deref().map(Month::parse).transpose()?)
    }
}

/// `?year=YYYY&with=YYYY&month=MM`, read as text for the same reason.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct CompareQueryDto {
    pub year: Option<String>,
    pub with: Option<String>,
    pub month: Option<String>,
}

impl CompareQueryDto {
    pub fn into_input(self) -> Result<YearComparison, AppError> {
        Ok(YearComparison::parse(
            self.year.as_deref().unwrap_or_default(),
            self.with.as_deref().unwrap_or_default(),
            self.month.as_deref().unwrap_or_default(),
        )?)
    }
}

/// What a data job measured for one zone and month. The zone and the month
/// are in the URL.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct ZoneReadingParams {
    /// Whole number, 0 to 100. Higher is drier.
    pub dryness: f64,
    /// 0 to 400. Leave out or send `null` when not measured.
    pub rain_pct_of_normal: Option<f64>,
    /// -100 to 300. Leave out or send `null` when not measured.
    pub greenness_pct_vs_normal: Option<f64>,
    /// Whole number, 0 to 100. Leave out or send `null` when not measured.
    pub water_need: Option<f64>,
    pub nitrogen_hold: bool,
    /// Crop codes, best first. May be empty.
    #[serde(default)]
    #[schema(value_type = Vec<BestCrop>)]
    pub best_crops: Vec<String>,
    /// The job or product the numbers came from.
    pub source: String,
}

impl ZoneReadingParams {
    pub fn into_input(
        self,
        zone_slug: ZoneSlug,
        month: Month,
    ) -> Result<RecordZoneReadingInput, AppError> {
        Ok(RecordZoneReadingInput {
            zone_slug,
            month,
            dryness: Dryness::from_number(self.dryness)?,
            rain_pct_of_normal: self
                .rain_pct_of_normal
                .map(RainPctOfNormal::new)
                .transpose()?,
            greenness_pct_vs_normal: self
                .greenness_pct_vs_normal
                .map(GreennessPctVsNormal::new)
                .transpose()?,
            water_need: self.water_need.map(WaterNeed::from_number).transpose()?,
            nitrogen_hold: self.nitrogen_hold,
            // The codes are read as text so that an unknown one is answered
            // with `bad_crop` and its name.
            best_crops: self
                .best_crops
                .iter()
                .map(|code| domain::Crop::try_from(code.as_str()))
                .collect::<Result<Vec<_>, _>>()?,
            source: ReadingSource::new(self.source)?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, Copy, ToSchema)]
pub struct SubZoneReadingParams {
    /// Whole number, 0 to 100. Higher is drier.
    pub dryness: f64,
}

impl SubZoneReadingParams {
    pub fn into_input(
        self,
        zone_slug: ZoneSlug,
        sub_zone_slug: ZoneSlug,
        month: Month,
    ) -> Result<RecordSubZoneReadingInput, AppError> {
        Ok(RecordSubZoneReadingInput {
            zone_slug,
            sub_zone_slug,
            month,
            dryness: Dryness::from_number(self.dryness)?,
        })
    }
}

/// A zone reading as it is stored.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneReadingResponse {
    pub zone_slug: String,
    /// `YYYY-MM`
    pub month: String,
    pub dryness: i32,
    pub band: DrynessBand,
    pub rain_pct_of_normal: Option<f64>,
    pub greenness_pct_vs_normal: Option<f64>,
    pub water_need: Option<i32>,
    pub nitrogen_hold: bool,
    pub best_crops: Vec<BestCrop>,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl From<(&ZoneSlug, &ZoneReading)> for ZoneReadingResponse {
    fn from((zone_slug, reading): (&ZoneSlug, &ZoneReading)) -> Self {
        Self {
            zone_slug: zone_slug.into(),
            month: reading.month().into(),
            dryness: reading.dryness().value(),
            band: band_of(reading.dryness()),
            rain_pct_of_normal: reading.rain_pct_of_normal().map(|rain| rain.value()),
            greenness_pct_vs_normal: reading
                .greenness_pct_vs_normal()
                .map(|greenness| greenness.value()),
            water_need: reading.water_need().map(|need| need.value()),
            nitrogen_hold: *reading.nitrogen_hold(),
            best_crops: reading
                .best_crops()
                .iter()
                .map(|crop| (*crop).into())
                .collect(),
            source: reading.source().into(),
            updated_at: *reading.updated_at(),
        }
    }
}

/// A sub-zone reading as it is stored.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SubZoneReadingResponse {
    pub zone_slug: String,
    pub sub_zone_slug: String,
    /// `YYYY-MM`
    pub month: String,
    pub dryness: i32,
    pub band: DrynessBand,
    pub updated_at: DateTime<Utc>,
}

impl From<(&ZoneSlug, &ZoneSlug, &SubZoneReading)> for SubZoneReadingResponse {
    fn from((zone_slug, sub_zone_slug, reading): (&ZoneSlug, &ZoneSlug, &SubZoneReading)) -> Self {
        Self {
            zone_slug: zone_slug.into(),
            sub_zone_slug: sub_zone_slug.into(),
            month: reading.month().into(),
            dryness: reading.dryness().value(),
            band: band_of(reading.dryness()),
            updated_at: *reading.updated_at(),
        }
    }
}

/// One zone on the region map. Every measured field is `null` when the zone
/// has no reading for the month.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneOverviewResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub governorate: String,
    pub dryness: Option<i32>,
    pub band: Option<DrynessBand>,
    /// Dryness minus the same month one year earlier.
    pub change_vs_last_year: Option<i32>,
    /// 1 = driest. Zones equally dry share a rank.
    pub rank: Option<u32>,
    pub water_need: Option<i32>,
    pub nitrogen_hold: Option<bool>,
}

impl From<&ZoneOverview> for ZoneOverviewResponse {
    fn from(row: &ZoneOverview) -> Self {
        let reading = row.reading().as_ref();

        Self {
            slug: row.zone().slug().into(),
            name_en: row.zone().name_en().clone(),
            name_ku: row.zone().name_ku().clone(),
            governorate: row.zone().governorate().clone(),
            dryness: reading.map(|reading| reading.dryness().value()),
            band: reading.map(|reading| band_of(reading.dryness())),
            change_vs_last_year: *row.change_vs_last_year(),
            rank: *row.rank(),
            water_need: reading.and_then(|reading| reading.water_need().map(|need| need.value())),
            nitrogen_hold: reading.map(|reading| *reading.nitrogen_hold()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RegionSummaryResponse {
    pub zones_with_data: usize,
    /// Over the zones that have data, to one decimal.
    pub average_dryness: Option<f64>,
    /// Over the zones that have data in both years, to one decimal.
    pub change_vs_last_year: Option<f64>,
    /// Slugs of up to 5 zones, driest first.
    pub driest: Vec<String>,
    /// Slugs of the zones told to hold back nitrogen.
    pub nitrogen_hold: Vec<String>,
}

impl From<&RegionSummary> for RegionSummaryResponse {
    fn from(summary: &RegionSummary) -> Self {
        Self {
            zones_with_data: *summary.zones_with_data(),
            average_dryness: *summary.average_dryness(),
            change_vs_last_year: *summary.change_vs_last_year(),
            driest: summary.driest().iter().map(Into::into).collect(),
            nitrogen_hold: summary.nitrogen_hold().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RegionOverviewResponse {
    /// `YYYY-MM`
    pub month: String,
    pub zones: Vec<ZoneOverviewResponse>,
    pub summary: RegionSummaryResponse,
}

impl From<&RegionOverview> for RegionOverviewResponse {
    fn from(overview: &RegionOverview) -> Self {
        Self {
            month: overview.month().into(),
            zones: overview.zones().iter().map(Into::into).collect(),
            summary: overview.summary().into(),
        }
    }
}

/// The zone's reading for the month with its place among the zones measured
/// that month.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDetailReadingResponse {
    pub dryness: i32,
    pub band: DrynessBand,
    /// 1 = driest. Zones equally dry share a rank.
    pub rank: u32,
    /// How many zones were ranked that month.
    pub rank_of: usize,
    pub rain_pct_of_normal: Option<f64>,
    pub greenness_pct_vs_normal: Option<f64>,
    pub water_need: Option<i32>,
    pub nitrogen_hold: bool,
    pub best_crops: Vec<BestCrop>,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

impl From<&RankedReading> for ZoneDetailReadingResponse {
    fn from(ranked: &RankedReading) -> Self {
        let reading = ranked.reading();

        Self {
            dryness: reading.dryness().value(),
            band: band_of(reading.dryness()),
            rank: *ranked.rank(),
            rank_of: *ranked.rank_of(),
            rain_pct_of_normal: reading.rain_pct_of_normal().map(|rain| rain.value()),
            greenness_pct_vs_normal: reading
                .greenness_pct_vs_normal()
                .map(|greenness| greenness.value()),
            water_need: reading.water_need().map(|need| need.value()),
            nitrogen_hold: *reading.nitrogen_hold(),
            best_crops: reading
                .best_crops()
                .iter()
                .map(|crop| (*crop).into())
                .collect(),
            source: reading.source().into(),
            updated_at: *reading.updated_at(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SubZoneDrynessResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub dryness: Option<i32>,
    pub band: Option<DrynessBand>,
}

impl From<&SubZoneDryness> for SubZoneDrynessResponse {
    fn from(row: &SubZoneDryness) -> Self {
        Self {
            slug: row.sub_zone().slug().into(),
            name_en: row.sub_zone().name_en().clone(),
            name_ku: row.sub_zone().name_ku().clone(),
            dryness: row.dryness().map(|dryness| dryness.value()),
            band: row.dryness().as_ref().map(band_of),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct YearDrynessResponse {
    pub year: i32,
    pub dryness: i32,
}

impl From<&YearDryness> for YearDrynessResponse {
    fn from(point: &YearDryness) -> Self {
        Self {
            year: *point.year(),
            dryness: point.dryness().value(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDetailResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub governorate: String,
    /// `YYYY-MM`
    pub month: String,
    /// `null` when the zone has no reading for the month.
    pub reading: Option<ZoneDetailReadingResponse>,
    /// Driest first, the ones not measured last.
    pub sub_zones: Vec<SubZoneDrynessResponse>,
    /// The same calendar month in up to 5 earlier years that have data,
    /// oldest first.
    pub history: Vec<YearDrynessResponse>,
}

impl From<&ZoneDetail> for ZoneDetailResponse {
    fn from(detail: &ZoneDetail) -> Self {
        Self {
            slug: detail.zone().slug().into(),
            name_en: detail.zone().name_en().clone(),
            name_ku: detail.zone().name_ku().clone(),
            governorate: detail.zone().governorate().clone(),
            month: detail.month().into(),
            reading: detail.reading().as_ref().map(Into::into),
            sub_zones: detail.sub_zones().iter().map(Into::into).collect(),
            history: detail.history().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneComparisonResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    /// In `year`.
    pub dryness: Option<i32>,
    /// In the `with` year.
    pub dryness_with: Option<i32>,
    /// `dryness` minus `dryness_with`; `null` when either is missing.
    pub change: Option<i32>,
}

impl From<&ZoneComparison> for ZoneComparisonResponse {
    fn from(row: &ZoneComparison) -> Self {
        Self {
            slug: row.zone().slug().into(),
            name_en: row.zone().name_en().clone(),
            name_ku: row.zone().name_ku().clone(),
            dryness: row.dryness().map(|dryness| dryness.value()),
            dryness_with: row.dryness_with().map(|dryness| dryness.value()),
            change: *row.change(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema)]
pub struct YearAverageResponse {
    pub year: i32,
    /// To one decimal.
    pub average_dryness: f64,
}

impl From<&YearAverage> for YearAverageResponse {
    fn from(point: &YearAverage) -> Self {
        Self {
            year: *point.year(),
            average_dryness: *point.average_dryness(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RegionComparisonResponse {
    /// The calendar month, 1 to 12.
    pub month: u32,
    pub year: i32,
    pub with: i32,
    /// Largest change first, in either direction; zones missing a year last.
    pub zones: Vec<ZoneComparisonResponse>,
    /// The region's average for that calendar month in every year that has
    /// data, oldest first.
    pub region: Vec<YearAverageResponse>,
}

impl From<&RegionComparison> for RegionComparisonResponse {
    fn from(compared: &RegionComparison) -> Self {
        Self {
            month: compared.comparison().calendar_month(),
            year: compared.comparison().year(),
            with: compared.comparison().with(),
            zones: compared.zones().iter().map(Into::into).collect(),
            region: compared.region().iter().map(Into::into).collect(),
        }
    }
}

/// `?from=YYYY-MM&to=YYYY-MM` on the dashboard's lists of readings, read as
/// text so that a month written wrongly is answered with `bad_month`.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct ZoneDashboardReadingsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
}

impl ZoneDashboardReadingsQuery {
    pub fn into_bounds(self) -> Result<(Option<Month>, Option<Month>), AppError> {
        Ok((
            self.from.as_deref().map(Month::parse).transpose()?,
            self.to.as_deref().map(Month::parse).transpose()?,
        ))
    }
}

/// A sub-zone a reading can be filed under.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDashboardSubZoneResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
}

impl From<&SubZone> for ZoneDashboardSubZoneResponse {
    fn from(sub_zone: &SubZone) -> Self {
        Self {
            slug: sub_zone.slug().into(),
            name_en: sub_zone.name_en().clone(),
            name_ku: sub_zone.name_ku().clone(),
        }
    }
}

/// A zone as the dashboard's pickers show it: who it is, with no readings.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDashboardZoneResponse {
    pub slug: String,
    pub name_en: String,
    pub name_ku: String,
    pub governorate: String,
    pub sub_zones: Vec<ZoneDashboardSubZoneResponse>,
}

impl From<&ZoneWithSubZones> for ZoneDashboardZoneResponse {
    fn from(row: &ZoneWithSubZones) -> Self {
        Self {
            slug: row.zone.slug().into(),
            name_en: row.zone.name_en().clone(),
            name_ku: row.zone.name_ku().clone(),
            governorate: row.zone.governorate().clone(),
            sub_zones: row
                .sub_zones
                .iter()
                .map(ZoneDashboardSubZoneResponse::from)
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDashboardZonesResponse {
    pub zones: Vec<ZoneDashboardZoneResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDashboardReadingsResponse {
    /// Newest month first.
    pub readings: Vec<ZoneReadingResponse>,
    /// How many readings the range holds in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ZoneDashboardSubZoneReadingsResponse {
    /// Newest month first.
    pub readings: Vec<SubZoneReadingResponse>,
    /// How many readings the range holds in all, on every page.
    pub count: u64,
    pub page: u64,
    pub rows_per_page: u64,
}

/// A reading a staff member types in: what the data jobs send, with the
/// month in the body because the row does not exist yet.
#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateZoneDashboardReadingParams {
    /// `YYYY-MM`
    pub month: String,
    #[serde(flatten)]
    pub reading: ZoneReadingParams,
}

impl CreateZoneDashboardReadingParams {
    pub fn into_input(self, zone_slug: ZoneSlug) -> Result<RecordZoneReadingInput, AppError> {
        let month = Month::parse(&self.month)?;

        self.reading.into_input(zone_slug, month)
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct CreateZoneDashboardSubZoneReadingParams {
    /// `YYYY-MM`
    pub month: String,
    /// Whole number, 0 to 100. Higher is drier.
    pub dryness: f64,
}

impl CreateZoneDashboardSubZoneReadingParams {
    pub fn into_input(
        self,
        zone_slug: ZoneSlug,
        sub_zone_slug: ZoneSlug,
    ) -> Result<RecordSubZoneReadingInput, AppError> {
        let month = Month::parse(&self.month)?;

        SubZoneReadingParams {
            dryness: self.dryness,
        }
        .into_input(zone_slug, sub_zone_slug, month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(json: serde_json::Value) -> ZoneReadingParams {
        serde_json::from_value(json).expect("params deserialize")
    }

    fn slug() -> ZoneSlug {
        ZoneSlug::new("kalar".to_string()).expect("slug")
    }

    fn month() -> Month {
        Month::parse("2026-03").expect("month")
    }

    fn full() -> serde_json::Value {
        serde_json::json!({
            "dryness": 72,
            "rain_pct_of_normal": 61.5,
            "greenness_pct_vs_normal": -18,
            "water_need": 80,
            "nitrogen_hold": true,
            "best_crops": ["barley", "wheat"],
            "source": "chirps+modis"
        })
    }

    fn code_of(error: AppError) -> &'static str {
        use crate::app::ToErrorInfo;

        error.to_error_info().code
    }

    #[test]
    fn a_full_body_becomes_value_objects() {
        let input = params(full()).into_input(slug(), month()).expect("input");

        assert_eq!(input.dryness.value(), 72);
        assert_eq!(input.water_need.map(|need| need.value()), Some(80));
        assert_eq!(
            input.best_crops,
            vec![domain::Crop::Barley, domain::Crop::Wheat]
        );
        assert!(input.nitrogen_hold);
    }

    #[test]
    fn the_unmeasured_fields_and_the_crops_may_be_left_out() {
        let input = params(serde_json::json!({
            "dryness": 40,
            "nitrogen_hold": false,
            "source": "chirps"
        }))
        .into_input(slug(), month())
        .expect("input");

        assert!(input.rain_pct_of_normal.is_none());
        assert!(input.greenness_pct_vs_normal.is_none());
        assert!(input.water_need.is_none());
        assert!(input.best_crops.is_empty());
    }

    #[test]
    fn each_bad_field_is_answered_with_its_own_code() {
        for (field, value, code) in [
            ("dryness", serde_json::json!(101), "bad_dryness"),
            ("dryness", serde_json::json!(59.5), "bad_dryness"),
            (
                "rain_pct_of_normal",
                serde_json::json!(401),
                "bad_rain_pct_of_normal",
            ),
            (
                "greenness_pct_vs_normal",
                serde_json::json!(-101),
                "bad_greenness_pct_vs_normal",
            ),
            ("water_need", serde_json::json!(-1), "bad_water_need"),
            ("best_crops", serde_json::json!(["rice"]), "bad_crop"),
            ("source", serde_json::json!("  "), "invalid"),
        ] {
            let mut body = full();
            body[field] = value;

            let error = params(body)
                .into_input(slug(), month())
                .err()
                .unwrap_or_else(|| panic!("{field} was accepted"));

            assert_eq!(code_of(error), code, "{field}");
        }
    }

    #[test]
    fn a_month_query_is_optional_but_must_be_well_written_when_given() {
        assert_eq!(
            MonthQueryDto { month: None }.into_month().expect("none"),
            None
        );
        assert_eq!(
            MonthQueryDto {
                month: Some("2026-03".to_string())
            }
            .into_month()
            .expect("month"),
            Some(month())
        );

        let error = MonthQueryDto {
            month: Some("2026-3".to_string()),
        }
        .into_month()
        .expect_err("a loose month");

        assert_eq!(code_of(error), "bad_month");
    }

    #[test]
    fn a_compare_query_names_the_value_that_is_wrong() {
        let query = |year: Option<&str>, with: Option<&str>, month: Option<&str>| CompareQueryDto {
            year: year.map(str::to_string),
            with: with.map(str::to_string),
            month: month.map(str::to_string),
        };

        assert!(
            query(Some("2026"), Some("2025"), Some("3"))
                .into_input()
                .is_ok()
        );

        for (dto, code) in [
            (query(None, Some("2025"), Some("3")), "bad_year"),
            (query(Some("2026"), Some("x"), Some("3")), "bad_year"),
            (query(Some("2026"), Some("2025"), None), "bad_month"),
            (query(Some("2026"), Some("2025"), Some("13")), "bad_month"),
            (query(Some("2026"), Some("2026"), Some("3")), "same_year"),
        ] {
            let error = dto.into_input().expect_err("a bad query");

            assert_eq!(code_of(error), code);
        }
    }

    #[test]
    fn the_band_is_spelled_in_snake_case_on_the_wire() {
        assert_eq!(
            serde_json::to_value(DrynessBand::from(domain::DrynessBand::VeryDry)).expect("json"),
            serde_json::json!("very_dry")
        );
        assert_eq!(
            serde_json::to_value(DrynessBand::MuchGreener).expect("json"),
            serde_json::json!("much_greener")
        );
    }

    #[test]
    fn every_band_and_crop_survives_the_trip_through_the_dto_enum() {
        for band in domain::DrynessBand::ALL {
            assert_eq!(domain::DrynessBand::from(DrynessBand::from(band)), band);
        }

        for crop in domain::Crop::ALL {
            assert_eq!(domain::Crop::from(BestCrop::from(crop)), crop);
            assert_eq!(
                serde_json::to_value(BestCrop::from(crop)).expect("json"),
                serde_json::json!(String::from(crop)),
                "the wire code and the stored code must be the same word"
            );
        }
    }

    #[test]
    fn a_zone_without_a_reading_serializes_its_measured_fields_as_null() {
        let zone = domain::Zone::rehydrate(
            2,
            slug(),
            "Kalar".to_string(),
            "کەلار".to_string(),
            "Sulaymaniyah".to_string(),
        );
        let overview = RegionOverview::build(month(), vec![zone], &[], &[]);

        let json = serde_json::to_value(RegionOverviewResponse::from(&overview)).expect("json");

        assert_eq!(json["month"], "2026-03");
        assert_eq!(json["zones"][0]["slug"], "kalar");

        for field in [
            "dryness",
            "band",
            "change_vs_last_year",
            "rank",
            "water_need",
            "nitrogen_hold",
        ] {
            assert!(
                json["zones"][0][field].is_null(),
                "{field} must be present and null"
            );
            assert!(json["zones"][0].get(field).is_some());
        }

        assert!(json["summary"]["average_dryness"].is_null());
    }

    #[test]
    fn a_dashboard_create_is_the_ingest_body_with_the_month_beside_it() {
        let mut body = full();
        body["month"] = serde_json::json!("2026-03");

        let params: CreateZoneDashboardReadingParams =
            serde_json::from_value(body).expect("params deserialize");
        let input = params.into_input(slug()).expect("input");

        assert_eq!(String::from(input.month), "2026-03");
        assert_eq!(input.dryness.value(), 72);
        assert_eq!(input.source.as_str(), "chirps+modis");
    }

    #[test]
    fn a_dashboard_create_validates_exactly_as_the_ingest_route_does() {
        for (field, value, code) in [
            ("month", serde_json::json!("2026-3"), "bad_month"),
            ("dryness", serde_json::json!(101), "bad_dryness"),
            (
                "rain_pct_of_normal",
                serde_json::json!(401),
                "bad_rain_pct_of_normal",
            ),
            ("water_need", serde_json::json!(-1), "bad_water_need"),
            ("best_crops", serde_json::json!(["rice"]), "bad_crop"),
            ("source", serde_json::json!("  "), "invalid"),
        ] {
            let mut body = full();
            body["month"] = serde_json::json!("2026-03");
            body[field] = value;

            let params: CreateZoneDashboardReadingParams =
                serde_json::from_value(body).expect("params deserialize");

            assert_eq!(
                code_of(
                    params
                        .into_input(slug())
                        .err()
                        .unwrap_or_else(|| panic!("accepted"))
                ),
                code,
                "{field}"
            );
        }
    }

    #[test]
    fn a_dashboard_create_without_a_month_does_not_deserialize() {
        assert!(serde_json::from_value::<CreateZoneDashboardReadingParams>(full()).is_err());
    }

    #[test]
    fn a_sub_zone_create_carries_its_month_and_dryness() {
        let params: CreateZoneDashboardSubZoneReadingParams =
            serde_json::from_value(serde_json::json!({"month": "2026-03", "dryness": 64}))
                .expect("params deserialize");
        let input = params.into_input(slug(), slug()).expect("input");

        assert_eq!(String::from(input.month), "2026-03");
        assert_eq!(input.dryness.value(), 64);

        let bad = CreateZoneDashboardSubZoneReadingParams {
            month: "2026-03".to_string(),
            dryness: 64.5,
        };

        assert_eq!(
            code_of(
                bad.into_input(slug(), slug())
                    .err()
                    .unwrap_or_else(|| panic!("accepted"))
            ),
            "bad_dryness"
        );
    }

    #[test]
    fn the_range_bounds_are_both_optional_and_each_is_checked() {
        let query = |from: Option<&str>, to: Option<&str>| ZoneDashboardReadingsQuery {
            from: from.map(str::to_string),
            to: to.map(str::to_string),
        };

        assert_eq!(
            query(None, None).into_bounds().expect("bounds"),
            (None, None)
        );
        assert_eq!(
            query(Some("2026-01"), None)
                .into_bounds()
                .expect("bounds")
                .0
                .map(String::from),
            Some("2026-01".to_string())
        );
        assert_eq!(
            code_of(
                query(None, Some("march"))
                    .into_bounds()
                    .err()
                    .unwrap_or_else(|| panic!("accepted"))
            ),
            "bad_month"
        );
    }
}
