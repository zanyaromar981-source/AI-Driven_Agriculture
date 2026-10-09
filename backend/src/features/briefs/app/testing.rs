use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{NaiveDate, TimeZone, Utc};

use crate::{
    app::{AuthContext, Pagination, Permission, StaffContext, User},
    features::briefs::{
        app::{AppError, BriefFarmOwnership, BriefRepository, use_cases::RecordBriefInput},
        domain::{
            Author, BriefPoint, BriefScope, BriefSource, BriefSummary, DailyBrief, DayRange,
            FarmZone, Headline, PointLevel, PointText, SourceTitle, SourceUrl, ZoneSlug,
        },
    },
    shared::Phone,
};

pub const OWNER: &str = "+9647501234567";

/// The farm the fakes say `OWNER` owns.
pub const FARM_ID: i32 = 7;

pub const STAFF_ID: i32 = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Call {
    IsOwnedBy {
        farm_id: i32,
        phone: String,
    },
    FindLatest {
        scope: String,
    },
    FindBetween {
        scope: String,
        from: NaiveDate,
        to: NaiveDate,
    },
    Upsert {
        day: NaiveDate,
        scope: String,
    },
    Delete {
        day: NaiveDate,
        scope: String,
    },
    FindFarmZone {
        farm_id: i32,
    },
    UpsertFarmZones {
        farm_ids: Vec<i32>,
    },
    FindPage {
        scope: Option<String>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        page: u64,
    },
    Update {
        day: NaiveDate,
        scope: String,
    },
}

#[derive(Debug, Default)]
struct Script {
    owned_farm: Option<i32>,
    briefs: Vec<DailyBrief>,
    farm_zones: Vec<FarmZone>,
    fail_with_database_error: bool,
}

/// One fake standing in for every port of the feature, so a test can read
/// the calls of a whole use case in the order they happened.
#[derive(Debug, Clone, Default)]
pub struct Fakes {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<Call>>>,
}

impl Fakes {
    pub fn new() -> Self {
        Self::default()
    }

    /// `OWNER` owns `FARM_ID`.
    pub fn with_owned_farm(self) -> Self {
        self.script.lock().expect("script lock").owned_farm = Some(FARM_ID);
        self
    }

    pub fn with_stored(self, brief: DailyBrief) -> Self {
        self.script.lock().expect("script lock").briefs.push(brief);
        self
    }

    pub fn with_farm_zone(self, farm_id: i32, slug: &str) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .farm_zones
            .push(a_farm_zone(farm_id, slug));
        self
    }

    pub fn failing(self) -> Self {
        self.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        self
    }

    /// The brief as the fake holds it now, after the writes of a test.
    pub fn stored(&self, day: NaiveDate, scope: &str) -> Option<DailyBrief> {
        self.script
            .lock()
            .expect("script lock")
            .briefs
            .iter()
            .find(|brief| *brief.day() == day && brief.scope().as_str() == scope)
            .cloned()
    }

    /// The district the fake holds for a farm now.
    pub fn farm_zone(&self, farm_id: i32) -> Option<String> {
        self.script
            .lock()
            .expect("script lock")
            .farm_zones
            .iter()
            .find(|zone| *zone.farm_id() == farm_id)
            .map(|zone| zone.zone_slug().into())
    }

    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn record(&self, call: Call) {
        self.calls.lock().expect("calls lock").push(call);
    }

    fn guard(&self) -> Result<(), AppError> {
        if self
            .script
            .lock()
            .expect("script lock")
            .fail_with_database_error
        {
            return Err(crate::app::AppError::DatabaseError("fake".to_string()).into());
        }

        Ok(())
    }

    /// The stored briefs that pass the filters, newest day first and then
    /// by scope, as the real repository orders them.
    fn matching(
        &self,
        scope: Option<&BriefScope>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
    ) -> Vec<DailyBrief> {
        let mut briefs: Vec<DailyBrief> = self
            .script
            .lock()
            .expect("script lock")
            .briefs
            .iter()
            .filter(|brief| scope.is_none_or(|scope| brief.scope() == scope))
            .filter(|brief| from.is_none_or(|from| *brief.day() >= from))
            .filter(|brief| to.is_none_or(|to| *brief.day() <= to))
            .cloned()
            .collect();

        briefs.sort_by(|a, b| {
            b.day()
                .cmp(a.day())
                .then_with(|| a.scope().as_str().cmp(b.scope().as_str()))
        });

        briefs
    }
}

#[async_trait]
impl BriefFarmOwnership for Fakes {
    async fn is_owned_by(&self, farm_id: i32, phone: &Phone) -> Result<bool, AppError> {
        self.record(Call::IsOwnedBy {
            farm_id,
            phone: String::from(phone),
        });
        self.guard()?;

        let owned = self.script.lock().expect("script lock").owned_farm;

        Ok(owned == Some(farm_id) && phone.as_str() == OWNER)
    }
}

#[async_trait]
impl BriefRepository for Fakes {
    async fn find_latest(&self, scope: &BriefScope) -> Result<Option<DailyBrief>, AppError> {
        self.record(Call::FindLatest {
            scope: scope.into(),
        });
        self.guard()?;

        Ok(self.matching(Some(scope), None, None).into_iter().next())
    }

    async fn find_between(
        &self,
        scope: &BriefScope,
        range: &DayRange,
    ) -> Result<Vec<DailyBrief>, AppError> {
        self.record(Call::FindBetween {
            scope: scope.into(),
            from: *range.from(),
            to: *range.to(),
        });
        self.guard()?;

        Ok(self.matching(Some(scope), Some(*range.from()), Some(*range.to())))
    }

    async fn upsert(&self, entity: &DailyBrief) -> Result<DailyBrief, AppError> {
        self.record(Call::Upsert {
            day: *entity.day(),
            scope: entity.scope().into(),
        });
        self.guard()?;

        let stored = persisted(entity, 1);

        let mut script = self.script.lock().expect("script lock");
        script
            .briefs
            .retain(|other| other.day() != entity.day() || other.scope() != entity.scope());
        script.briefs.push(stored.clone());

        Ok(stored)
    }

    async fn delete(&self, day: NaiveDate, scope: &BriefScope) -> Result<(), AppError> {
        self.record(Call::Delete {
            day,
            scope: scope.into(),
        });
        self.guard()?;

        self.script
            .lock()
            .expect("script lock")
            .briefs
            .retain(|brief| *brief.day() != day || brief.scope() != scope);

        Ok(())
    }

    async fn find_farm_zone(&self, farm_id: i32) -> Result<Option<ZoneSlug>, AppError> {
        self.record(Call::FindFarmZone { farm_id });
        self.guard()?;

        Ok(self
            .script
            .lock()
            .expect("script lock")
            .farm_zones
            .iter()
            .find(|zone| *zone.farm_id() == farm_id)
            .map(|zone| zone.zone_slug().clone()))
    }

    async fn upsert_farm_zones(&self, zones: &[FarmZone]) -> Result<u64, AppError> {
        self.record(Call::UpsertFarmZones {
            farm_ids: zones.iter().map(|zone| *zone.farm_id()).collect(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        for zone in zones {
            script
                .farm_zones
                .retain(|other| other.farm_id() != zone.farm_id());
            script.farm_zones.push(zone.clone());
        }

        Ok(zones.len() as u64)
    }

    async fn find_page(
        &self,
        scope: Option<&BriefScope>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        pagination: &Pagination,
    ) -> Result<(Vec<DailyBrief>, u64), AppError> {
        self.record(Call::FindPage {
            scope: scope.map(Into::into),
            from,
            to,
            page: *pagination.page(),
        });
        self.guard()?;

        let matching = self.matching(scope, from, to);
        let count = matching.len() as u64;

        Ok((
            matching
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn update(&self, entity: &DailyBrief) -> Result<Option<DailyBrief>, AppError> {
        self.record(Call::Update {
            day: *entity.day(),
            scope: entity.scope().into(),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(slot) = script
            .briefs
            .iter_mut()
            .find(|other| other.day() == entity.day() && other.scope() == entity.scope())
        else {
            return Ok(None);
        };

        *slot = persisted(entity, slot.id().unwrap_or_default());

        Ok(Some(slot.clone()))
    }
}

/// The signed-in staff member a dashboard use case is acting for.
pub fn actor() -> StaffContext {
    StaffContext::new(
        STAFF_ID,
        "officer@example.org".to_string(),
        Permission::all().into_iter().collect(),
    )
}

fn persisted(entity: &DailyBrief, id: i32) -> DailyBrief {
    DailyBrief::rehydrate(
        id,
        *entity.day(),
        entity.scope().clone(),
        entity.headline_en().clone(),
        entity.headline_ku().clone(),
        entity.summary_en().clone(),
        entity.summary_ku().clone(),
        entity.points().clone(),
        entity.sources().clone(),
        entity.author().clone(),
        *entity.generated_at(),
        *entity.updated_at(),
    )
}

pub fn phone() -> Phone {
    Phone::new(OWNER.to_string()).expect("phone")
}

pub fn auth_context() -> AuthContext {
    AuthContext::new(User::new(phone()), "token".to_string())
}

pub fn a_day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, day).expect("date")
}

pub fn a_scope(scope: &str) -> BriefScope {
    BriefScope::new(scope.to_string()).expect("scope")
}

pub fn a_farm_zone(farm_id: i32, slug: &str) -> FarmZone {
    FarmZone::new(farm_id, ZoneSlug::new(slug.to_string()).expect("slug")).expect("farm zone")
}

pub fn a_point(text_en: &str) -> BriefPoint {
    BriefPoint::new(
        PointLevel::Watch,
        PointText::new(text_en.to_string()).expect("text"),
        PointText::new("باران دواکەوتووە".to_string()).expect("text"),
    )
}

pub fn a_source() -> BriefSource {
    BriefSource::new(
        SourceTitle::new("FAO crop calendar".to_string()).expect("title"),
        SourceUrl::new("https://example.org/calendar".to_string()).expect("url"),
    )
}

/// What the nightly job pushes for one day and scope.
pub fn an_input(day: u32, scope: &str, headline_en: &str) -> RecordBriefInput {
    RecordBriefInput {
        day: a_day(day),
        scope: a_scope(scope),
        headline_en: Headline::new(headline_en.to_string()).expect("headline"),
        headline_ku: Headline::new("هەفتەیەکی وشک".to_string()).expect("headline"),
        summary_en: BriefSummary::new("No rain fell.".to_string()).expect("summary"),
        summary_ku: BriefSummary::new("باران نەباری.".to_string()).expect("summary"),
        points: vec![a_point("Rain is late")],
        sources: vec![a_source()],
        author: Author::new("codex-cli gpt-5".to_string()).expect("author"),
        generated_at: Utc
            .with_ymd_and_hms(2026, 10, day, 21, 4, 0)
            .single()
            .expect("time"),
    }
}

/// A persisted brief with one point and one source.
pub fn a_brief(day: u32, scope: &str) -> DailyBrief {
    let input = an_input(day, scope, "A dry week");

    let brief = DailyBrief::new(
        input.day,
        input.scope,
        input.headline_en,
        input.headline_ku,
        input.summary_en,
        input.summary_ku,
        input.points,
        input.sources,
        input.author,
        input.generated_at,
    )
    .expect("brief");

    persisted(&brief, 1)
}
