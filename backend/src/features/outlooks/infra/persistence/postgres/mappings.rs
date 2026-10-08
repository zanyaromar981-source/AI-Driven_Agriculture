use sea_orm::ActiveValue::{NotSet, Set};

use crate::features::outlooks::{
    app::AppError,
    domain::{
        Confidence, IssueMonth, Outlook, OutlookRun, Reason, RunMethod, Season, ZoneOutlook,
        ZoneSlug,
    },
    infra::persistence::postgres::entities::{outlook_runs, season_outlooks},
};

impl TryFrom<season_outlooks::Model> for ZoneOutlook {
    type Error = AppError;

    fn try_from(model: season_outlooks::Model) -> Result<Self, Self::Error> {
        Ok(ZoneOutlook::rehydrate(
            model.id,
            ZoneSlug::new(model.zone_slug)?,
            Season::new(model.season)?,
            IssueMonth::from_date(model.issued)?,
            Outlook::try_from(model.outlook.as_str())?,
            Confidence::new(model.confidence_pct)?,
            model.reason_en.map(Reason::new).transpose()?,
            model.reason_ku.map(Reason::new).transpose()?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&ZoneOutlook> for season_outlooks::ActiveModel {
    fn from(outlook: &ZoneOutlook) -> Self {
        season_outlooks::ActiveModel {
            id: match *outlook.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            zone_slug: Set(outlook.zone_slug().into()),
            season: Set(outlook.season().into()),
            issued: Set(outlook.issued().first_day()),
            outlook: Set((*outlook.outlook()).into()),
            confidence_pct: Set(outlook.confidence().value()),
            reason_en: Set(outlook.reason_en().as_ref().map(Into::into)),
            reason_ku: Set(outlook.reason_ku().as_ref().map(Into::into)),
            updated_at: Set(outlook.updated_at().naive_utc()),
        }
    }
}

impl TryFrom<outlook_runs::Model> for OutlookRun {
    type Error = AppError;

    fn try_from(model: outlook_runs::Model) -> Result<Self, Self::Error> {
        Ok(OutlookRun::rehydrate(
            model.id,
            Season::new(model.season)?,
            IssueMonth::from_date(model.issued)?,
            model.seasons_tested,
            model.seasons_right,
            RunMethod::new(model.method)?,
            model.updated_at.and_utc(),
        ))
    }
}

impl From<&OutlookRun> for outlook_runs::ActiveModel {
    fn from(run: &OutlookRun) -> Self {
        outlook_runs::ActiveModel {
            id: match *run.id() {
                Some(id) => Set(id),
                None => NotSet,
            },
            season: Set(run.season().into()),
            issued: Set(run.issued().first_day()),
            seasons_tested: Set(*run.seasons_tested()),
            seasons_right: Set(*run.seasons_right()),
            method: Set(run.method().into()),
            updated_at: Set(run.updated_at().naive_utc()),
        }
    }
}
