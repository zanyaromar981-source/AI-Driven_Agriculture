use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::features::outlooks::{
    app::{AppError, OutlookRepository},
    domain::{
        Confidence, IssueMonth, Outlook, OutlookRun, RunMethod, Season, ZoneOutlook, ZoneSlug,
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

        Ok(OutlookRun::rehydrate(
            1,
            run.season().clone(),
            *run.issued(),
            *run.seasons_tested(),
            *run.seasons_right(),
            run.method().clone(),
            *run.updated_at(),
        ))
    }
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
