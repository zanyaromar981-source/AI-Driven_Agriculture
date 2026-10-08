use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::features::water::{
    app::{
        AppError,
        use_cases::{SetWaterPlanEntryInput, ViewWaterPlanInput},
    },
    domain::{
        DamAllocation, DamSlug, Need, Note, PlanTotals, RankedEntry, Season, WaterPlan,
        WaterPlanEntry, ZoneSlug,
    },
};

#[derive(Deserialize, Debug, Clone, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct WaterPlanQuery {
    /// Season like `2026-27`. Defaults to the latest season with a plan.
    pub season: Option<String>,
}

impl WaterPlanQuery {
    pub fn into_input(self) -> Result<ViewWaterPlanInput, AppError> {
        Ok(ViewWaterPlanInput {
            season: self.season.map(Season::new).transpose()?,
        })
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct SetWaterPlanEntryParams {
    /// 0 to 100. The plan is ranked by it, highest first.
    pub need: f64,
    /// The dam the water is to come from, once decided.
    pub dam_slug: Option<String>,
    /// Million m3 to send, once decided.
    pub send_million_m3: Option<f64>,
    #[serde(default)]
    pub urgent: bool,
    /// One short remark in English, 200 characters max.
    pub note_en: Option<String>,
    /// The same remark in Sorani, 200 characters max.
    pub note_ku: Option<String>,
}

impl SetWaterPlanEntryParams {
    pub fn into_input(
        self,
        season: String,
        zone_slug: String,
    ) -> Result<SetWaterPlanEntryInput, AppError> {
        Ok(SetWaterPlanEntryInput {
            season: Season::new(season)?,
            zone_slug: ZoneSlug::new(zone_slug)?,
            need: Need::new(self.need)?,
            dam_slug: self.dam_slug.map(DamSlug::new).transpose()?,
            send_million_m3: self.send_million_m3,
            urgent: self.urgent,
            note_en: Note::optional(self.note_en)?,
            note_ku: Note::optional(self.note_ku)?,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RankedEntryResponse {
    /// 1 = the zone that needs water most.
    pub rank: usize,
    pub zone_slug: String,
    pub need: f64,
    pub dam_slug: Option<String>,
    pub send_million_m3: Option<f64>,
    pub urgent: bool,
    pub note_en: Option<String>,
    pub note_ku: Option<String>,
}

impl From<&RankedEntry> for RankedEntryResponse {
    fn from(ranked: &RankedEntry) -> Self {
        let entry = ranked.entry();

        Self {
            rank: *ranked.rank(),
            zone_slug: entry.zone_slug().into(),
            need: entry.need().value(),
            dam_slug: entry.dam_slug().as_ref().map(Into::into),
            send_million_m3: *entry.send_million_m3(),
            urgent: *entry.urgent(),
            note_en: entry.note_en().as_ref().map(Into::into),
            note_ku: entry.note_ku().as_ref().map(Into::into),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DamAllocationResponse {
    pub dam_slug: String,
    pub planned_million_m3: f64,
    pub zones: usize,
}

impl From<&DamAllocation> for DamAllocationResponse {
    fn from(allocation: &DamAllocation) -> Self {
        Self {
            dam_slug: allocation.dam_slug().into(),
            planned_million_m3: *allocation.planned_million_m3(),
            zones: *allocation.zones(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct PlanTotalsResponse {
    pub planned_million_m3: f64,
    pub urgent_zones: usize,
    /// Largest volume first. Entries with no dam yet are in none of these.
    pub by_dam: Vec<DamAllocationResponse>,
}

impl From<&PlanTotals> for PlanTotalsResponse {
    fn from(totals: &PlanTotals) -> Self {
        Self {
            planned_million_m3: *totals.planned_million_m3(),
            urgent_zones: *totals.urgent_zones(),
            by_dam: totals.by_dam().iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct WaterPlanResponse {
    pub season: String,
    /// Highest need first.
    pub entries: Vec<RankedEntryResponse>,
    pub totals: PlanTotalsResponse,
}

impl From<&WaterPlan> for WaterPlanResponse {
    fn from(plan: &WaterPlan) -> Self {
        Self {
            season: plan.season().into(),
            entries: plan.entries().iter().map(Into::into).collect(),
            totals: plan.totals().into(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct StoredWaterPlanEntryResponse {
    pub season: String,
    pub zone_slug: String,
    pub need: f64,
    pub dam_slug: Option<String>,
    pub send_million_m3: Option<f64>,
    pub urgent: bool,
    pub note_en: Option<String>,
    pub note_ku: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// The entry as it is stored after an ingest. It carries no rank: the rank
/// depends on the other entries and is worked out when the plan is read.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedWaterPlanEntryResponse {
    pub entry: StoredWaterPlanEntryResponse,
}

impl From<&WaterPlanEntry> for SavedWaterPlanEntryResponse {
    fn from(entry: &WaterPlanEntry) -> Self {
        Self {
            entry: StoredWaterPlanEntryResponse {
                season: entry.season().into(),
                zone_slug: entry.zone_slug().into(),
                need: entry.need().value(),
                dam_slug: entry.dam_slug().as_ref().map(Into::into),
                send_million_m3: *entry.send_million_m3(),
                urgent: *entry.urgent(),
                note_en: entry.note_en().as_ref().map(Into::into),
                note_ku: entry.note_ku().as_ref().map(Into::into),
                updated_at: *entry.updated_at(),
            },
        }
    }
}
