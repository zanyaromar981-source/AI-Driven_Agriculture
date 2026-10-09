use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::{
    app::{Pagination, StaffContext},
    features::outlooks::{
        app::{AppError, OutlookRepository},
        domain::{
            Confidence, IssueMonth, Outlook, OutlookRun, RunMethod, Season, ZoneOutlook, ZoneSlug,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepositoryCall {
    FindLatestSeason,
    FindIssueMonths {
        season: String,
    },
    FindZoneOutlooks {
        season: String,
        issued: String,
    },
    FindZoneHistory {
        zone_slug: String,
        season: String,
    },
    FindRun {
        season: String,
        issued: String,
    },
    UpsertZoneOutlook {
        zone_slug: String,
        season: String,
        issued: String,
    },
    UpsertRun {
        season: String,
        issued: String,
    },
    FindZoneOutlooksPage {
        season: Option<String>,
        issued: Option<String>,
        page: u64,
        rows_per_page: u64,
    },
    CreateZoneOutlook {
        zone_slug: String,
        season: String,
        issued: String,
    },
    UpdateZoneOutlook {
        zone_slug: String,
        season: String,
        issued: String,
    },
    DeleteZoneOutlook {
        zone_slug: String,
        season: String,
        issued: String,
    },
    FindRuns,
    CreateRun {
        season: String,
        issued: String,
    },
    UpdateRun {
        season: String,
        issued: String,
    },
    DeleteRun {
        season: String,
        issued: String,
    },
}

#[derive(Debug, Default)]
struct Script {
    outlooks: Vec<ZoneOutlook>,
    runs: Vec<OutlookRun>,
    fail_with_database_error: bool,
}

#[derive(Debug, Clone, Default)]
pub struct FakeOutlookRepository {
    script: Arc<Mutex<Script>>,
    calls: Arc<Mutex<Vec<RepositoryCall>>>,
}

impl FakeOutlookRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn holding(outlooks: Vec<ZoneOutlook>, runs: Vec<OutlookRun>) -> Self {
        let fake = Self::new();
        {
            let mut script = fake.script.lock().expect("script lock");
            script.outlooks = outlooks;
            script.runs = runs;
        }
        fake
    }

    pub fn failing() -> Self {
        let fake = Self::new();
        fake.script
            .lock()
            .expect("script lock")
            .fail_with_database_error = true;
        fake
    }

    pub fn calls(&self) -> Vec<RepositoryCall> {
        self.calls.lock().expect("calls lock").clone()
    }

    fn record(&self, call: RepositoryCall) {
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
}

#[async_trait]
impl OutlookRepository for FakeOutlookRepository {
    async fn find_latest_season(&self) -> Result<Option<Season>, AppError> {
        self.record(RepositoryCall::FindLatestSeason);
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .outlooks
            .iter()
            .map(|outlook| outlook.season().clone())
            .max())
    }

    async fn find_issue_months(&self, season: &Season) -> Result<Vec<IssueMonth>, AppError> {
        self.record(RepositoryCall::FindIssueMonths {
            season: String::from(season),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut months: Vec<IssueMonth> = script
            .outlooks
            .iter()
            .filter(|outlook| outlook.season() == season)
            .map(|outlook| *outlook.issued())
            .collect();
        months.sort();
        months.dedup();

        Ok(months)
    }

    async fn find_zone_outlooks(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Vec<ZoneOutlook>, AppError> {
        self.record(RepositoryCall::FindZoneOutlooks {
            season: String::from(season),
            issued: String::from(issued),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .outlooks
            .iter()
            .filter(|outlook| outlook.season() == season && outlook.issued() == issued)
            .cloned()
            .collect())
    }

    async fn find_zone_history(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
    ) -> Result<Vec<ZoneOutlook>, AppError> {
        self.record(RepositoryCall::FindZoneHistory {
            zone_slug: String::from(zone_slug),
            season: String::from(season),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut history: Vec<ZoneOutlook> = script
            .outlooks
            .iter()
            .filter(|outlook| outlook.zone_slug() == zone_slug && outlook.season() == season)
            .cloned()
            .collect();
        history.sort_by_key(|outlook| *outlook.issued());

        Ok(history)
    }

    async fn find_run(
        &self,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<Option<OutlookRun>, AppError> {
        self.record(RepositoryCall::FindRun {
            season: String::from(season),
            issued: String::from(issued),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        Ok(script
            .runs
            .iter()
            .find(|run| run.season() == season && run.issued() == issued)
            .cloned())
    }

    async fn upsert_zone_outlook(&self, outlook: &ZoneOutlook) -> Result<ZoneOutlook, AppError> {
        self.record(RepositoryCall::UpsertZoneOutlook {
            zone_slug: String::from(outlook.zone_slug()),
            season: String::from(outlook.season()),
            issued: String::from(outlook.issued()),
        });
        self.guard()?;

        Ok(ZoneOutlook::rehydrate(
            1,
            outlook.zone_slug().clone(),
            outlook.season().clone(),
            *outlook.issued(),
            *outlook.outlook(),
            *outlook.confidence(),
            outlook.reason_en().clone(),
            outlook.reason_ku().clone(),
            *outlook.updated_at(),
        ))
    }

    async fn upsert_run(&self, run: &OutlookRun) -> Result<OutlookRun, AppError> {
        self.record(RepositoryCall::UpsertRun {
            season: String::from(run.season()),
            issued: String::from(run.issued()),
        });
        self.guard()?;

        Ok(persisted_run(run))
    }

    async fn find_zone_outlooks_page(
        &self,
        season: Option<&Season>,
        issued: Option<&IssueMonth>,
        pagination: &Pagination,
    ) -> Result<(Vec<ZoneOutlook>, u64), AppError> {
        self.record(RepositoryCall::FindZoneOutlooksPage {
            season: season.map(String::from),
            issued: issued.map(String::from),
            page: *pagination.page(),
            rows_per_page: *pagination.rows_per_page(),
        });
        self.guard()?;

        let script = self.script.lock().expect("script lock");

        let mut outlooks: Vec<ZoneOutlook> = script
            .outlooks
            .iter()
            .filter(|outlook| {
                season.is_none_or(|season| outlook.season() == season)
                    && issued.is_none_or(|issued| outlook.issued() == issued)
            })
            .cloned()
            .collect();
        outlooks.sort_by(|first, second| {
            second
                .issued()
                .cmp(first.issued())
                .then(second.season().cmp(first.season()))
                .then(first.zone_slug().cmp(second.zone_slug()))
        });

        let count = outlooks.len() as u64;

        Ok((
            outlooks
                .into_iter()
                .skip(pagination.skip() as usize)
                .take(*pagination.rows_per_page() as usize)
                .collect(),
            count,
        ))
    }

    async fn create_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError> {
        self.record(RepositoryCall::CreateZoneOutlook {
            zone_slug: String::from(outlook.zone_slug()),
            season: String::from(outlook.season()),
            issued: String::from(outlook.issued()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .outlooks
            .iter()
            .any(|stored| same_outlook(stored, outlook))
        {
            return Ok(None);
        }

        let stored = persisted_outlook(outlook);
        script.outlooks.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update_zone_outlook(
        &self,
        outlook: &ZoneOutlook,
    ) -> Result<Option<ZoneOutlook>, AppError> {
        self.record(RepositoryCall::UpdateZoneOutlook {
            zone_slug: String::from(outlook.zone_slug()),
            season: String::from(outlook.season()),
            issued: String::from(outlook.issued()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .outlooks
            .iter_mut()
            .find(|stored| same_outlook(stored, outlook))
        else {
            return Ok(None);
        };

        *stored = persisted_outlook(outlook);

        Ok(Some(stored.clone()))
    }

    async fn delete_zone_outlook(
        &self,
        zone_slug: &ZoneSlug,
        season: &Season,
        issued: &IssueMonth,
    ) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteZoneOutlook {
            zone_slug: String::from(zone_slug),
            season: String::from(season),
            issued: String::from(issued),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.outlooks.len();

        script.outlooks.retain(|outlook| {
            !(outlook.zone_slug() == zone_slug
                && outlook.season() == season
                && outlook.issued() == issued)
        });

        Ok(script.outlooks.len() < before)
    }

    async fn find_runs(&self) -> Result<Vec<OutlookRun>, AppError> {
        self.record(RepositoryCall::FindRuns);
        self.guard()?;

        let mut runs = self.script.lock().expect("script lock").runs.clone();
        runs.sort_by(|first, second| {
            second
                .issued()
                .cmp(first.issued())
                .then(second.season().cmp(first.season()))
        });

        Ok(runs)
    }

    async fn create_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError> {
        self.record(RepositoryCall::CreateRun {
            season: String::from(run.season()),
            issued: String::from(run.issued()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        if script
            .runs
            .iter()
            .any(|stored| stored.season() == run.season() && stored.issued() == run.issued())
        {
            return Ok(None);
        }

        let stored = persisted_run(run);
        script.runs.push(stored.clone());

        Ok(Some(stored))
    }

    async fn update_run(&self, run: &OutlookRun) -> Result<Option<OutlookRun>, AppError> {
        self.record(RepositoryCall::UpdateRun {
            season: String::from(run.season()),
            issued: String::from(run.issued()),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");

        let Some(stored) = script
            .runs
            .iter_mut()
            .find(|stored| stored.season() == run.season() && stored.issued() == run.issued())
        else {
            return Ok(None);
        };

        *stored = persisted_run(run);

        Ok(Some(stored.clone()))
    }

    async fn delete_run(&self, season: &Season, issued: &IssueMonth) -> Result<bool, AppError> {
        self.record(RepositoryCall::DeleteRun {
            season: String::from(season),
            issued: String::from(issued),
        });
        self.guard()?;

        let mut script = self.script.lock().expect("script lock");
        let before = script.runs.len();

        script
            .runs
            .retain(|run| !(run.season() == season && run.issued() == issued));

        Ok(script.runs.len() < before)
    }
}

fn same_outlook(first: &ZoneOutlook, second: &ZoneOutlook) -> bool {
    first.zone_slug() == second.zone_slug()
        && first.season() == second.season()
        && first.issued() == second.issued()
}

fn persisted_outlook(outlook: &ZoneOutlook) -> ZoneOutlook {
    ZoneOutlook::rehydrate(
        1,
        outlook.zone_slug().clone(),
        outlook.season().clone(),
        *outlook.issued(),
        *outlook.outlook(),
        *outlook.confidence(),
        outlook.reason_en().clone(),
        outlook.reason_ku().clone(),
        *outlook.updated_at(),
    )
}

fn persisted_run(run: &OutlookRun) -> OutlookRun {
    OutlookRun::rehydrate(
        1,
        run.season().clone(),
        *run.issued(),
        *run.seasons_tested(),
        *run.seasons_right(),
        run.method().clone(),
        *run.updated_at(),
    )
}

/// Staff member 7, holding no permission: the routes check those, the use
/// cases only record who acted.
pub fn staff() -> StaffContext {
    StaffContext::new(
        7,
        "officer@example.org".to_string(),
        std::collections::HashSet::new(),
    )
}

pub fn season(value: &str) -> Season {
    Season::new(value.to_string()).expect("season")
}

pub fn month(value: &str) -> IssueMonth {
    IssueMonth::new(value).expect("month")
}

pub fn zone_slug(value: &str) -> ZoneSlug {
    ZoneSlug::new(value.to_string()).expect("slug")
}

/// A persisted outlook without reasons.
pub fn an_outlook(
    zone: &str,
    for_season: &str,
    issued: &str,
    outlook: Outlook,
    confidence: f64,
) -> ZoneOutlook {
    ZoneOutlook::rehydrate(
        1,
        zone_slug(zone),
        season(for_season),
        month(issued),
        outlook,
        Confidence::new(confidence).expect("confidence"),
        None,
        None,
        chrono::Utc::now(),
    )
}

/// A persisted track record: right in 19 of 25 seasons.
pub fn a_run(for_season: &str, issued: &str) -> OutlookRun {
    OutlookRun::rehydrate(
        1,
        season(for_season),
        month(issued),
        25,
        19,
        RunMethod::new("analog years".to_string()).expect("method"),
        chrono::Utc::now(),
    )
}
