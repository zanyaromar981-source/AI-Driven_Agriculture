use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

use crate::{
    features::jobs::{
        app::{AppError, use_cases::RecordJobRunInput},
        domain::{self, JobCode, JobRun, JobStatus, RunMessage},
    },
    shared::DomainError,
};

/// The start of a run travels in the path as an RFC 3339 timestamp. It is
/// parsed here rather than by the extractor so that a bad one is answered
/// like every other invalid value.
pub(super) fn parse_started_at(raw: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(raw)
        .map(|started_at| started_at.with_timezone(&Utc))
        .map_err(|_| {
            DomainError::InvalidValue(
                "started_at must be a timestamp like 2026-10-09T06:00:00Z".to_string(),
            )
            .into()
        })
}

/// How a job stands right now.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobRunState {
    Ok,
    Late,
    Failed,
    Never,
}

impl From<JobRunState> for domain::JobState {
    fn from(value: JobRunState) -> Self {
        match value {
            JobRunState::Ok => domain::JobState::Ok,
            JobRunState::Late => domain::JobState::Late,
            JobRunState::Failed => domain::JobState::Failed,
            JobRunState::Never => domain::JobState::Never,
        }
    }
}

impl From<domain::JobState> for JobRunState {
    fn from(value: domain::JobState) -> Self {
        match value {
            domain::JobState::Ok => JobRunState::Ok,
            domain::JobState::Late => JobRunState::Late,
            domain::JobState::Failed => JobRunState::Failed,
            domain::JobState::Never => JobRunState::Never,
        }
    }
}

/// How one day went for a job.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobDayMark {
    Ok,
    Late,
    Failed,
}

impl From<JobDayMark> for domain::DayMark {
    fn from(value: JobDayMark) -> Self {
        match value {
            JobDayMark::Ok => domain::DayMark::Ok,
            JobDayMark::Late => domain::DayMark::Late,
            JobDayMark::Failed => domain::DayMark::Failed,
        }
    }
}

impl From<domain::DayMark> for JobDayMark {
    fn from(value: domain::DayMark) -> Self {
        match value {
            domain::DayMark::Ok => JobDayMark::Ok,
            domain::DayMark::Late => JobDayMark::Late,
            domain::DayMark::Failed => JobDayMark::Failed,
        }
    }
}

#[derive(Serialize, Deserialize, Validate, Debug, Clone, ToSchema)]
pub struct RecordJobRunParams {
    /// Null while the run is still going.
    pub finished_at: Option<DateTime<Utc>>,
    /// Null while the run is still going; set together with `finished_at`.
    pub ok: Option<bool>,
    /// How many rows the run wrote, if the job counts them.
    pub rows: Option<i64>,
    /// Up to 500 characters: a short summary, or why the run failed.
    pub message: Option<String>,
}

impl RecordJobRunParams {
    pub fn into_input(self, job: JobCode, started_at: &str) -> Result<RecordJobRunInput, AppError> {
        Ok(RecordJobRunInput {
            job,
            started_at: parse_started_at(started_at)?,
            finished_at: self.finished_at,
            ok: self.ok,
            rows: self.rows,
            message: self.message.map(RunMessage::new).transpose()?.flatten(),
        })
    }
}

/// One run as its job reported it.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct JobRunResponse {
    pub started_at: DateTime<Utc>,
    /// Null while the run is still going.
    pub finished_at: Option<DateTime<Utc>>,
    pub ok: Option<bool>,
    pub rows: Option<i64>,
    pub message: Option<String>,
}

impl From<&JobRun> for JobRunResponse {
    fn from(run: &JobRun) -> Self {
        Self {
            started_at: *run.started_at(),
            finished_at: *run.finished_at(),
            ok: *run.ok(),
            rows: *run.rows(),
            message: run.message().as_ref().map(String::from),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SavedJobRunResponse {
    pub job: String,
    pub run: JobRunResponse,
}

impl From<&JobRun> for SavedJobRunResponse {
    fn from(run: &JobRun) -> Self {
        Self {
            job: run.job().into(),
            run: run.into(),
        }
    }
}

/// One job, how it stands now and how its last fourteen days went.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct JobStatusResponse {
    pub job: String,
    pub name_en: String,
    /// Null until the Sorani name has been written.
    pub name_ku: Option<String>,
    /// How often the job is meant to run. Null = on demand, never late.
    pub every_hours: Option<i32>,
    /// The run that started last, also when it is still going.
    pub last_run: Option<JobRunResponse>,
    /// When a run last finished ok.
    pub last_ok: Option<DateTime<Utc>>,
    /// `every_hours` after the last run started. Null for a job on demand
    /// and for one that never ran.
    pub next_due: Option<DateTime<Utc>>,
    /// `never`: no run at all. `failed`: the run that finished last was not
    /// ok. `late`: no run finished ok within `every_hours` x 1.5 of now.
    pub state: JobRunState,
    /// Fourteen UTC days, oldest first, today last. `failed`: a run that
    /// started that day failed. `ok`: at least one finished ok. `late`: the
    /// job was due and nothing finished ok. Null: before the job's first
    /// run, or the job was not due.
    pub last_14_days: Vec<Option<JobDayMark>>,
    /// What the run that finished last said, or the run now going when none
    /// has finished.
    pub message: Option<String>,
}

impl From<&JobStatus> for JobStatusResponse {
    fn from(status: &JobStatus) -> Self {
        Self {
            job: status.job().code().into(),
            name_en: status.job().name_en().clone(),
            name_ku: status.job().name_ku().clone(),
            every_hours: *status.job().every_hours(),
            last_run: status.last_run().as_ref().map(JobRunResponse::from),
            last_ok: *status.last_ok(),
            next_due: *status.next_due(),
            state: (*status.state()).into(),
            last_14_days: status
                .last_14_days()
                .iter()
                .map(|mark| mark.map(JobDayMark::from))
                .collect(),
            message: status.message().as_ref().map(String::from),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct JobsResponse {
    pub jobs: Vec<JobStatusResponse>,
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;
    use crate::features::jobs::domain::{Job, RunHistory, history_since};

    fn code(value: &str) -> JobCode {
        JobCode::new(value.to_string()).expect("code")
    }

    fn params(json: serde_json::Value) -> RecordJobRunParams {
        serde_json::from_value(json).expect("params")
    }

    #[test]
    fn every_state_and_mark_survives_the_dto_and_matches_its_stored_name() {
        for state in domain::JobState::ALL {
            assert_eq!(domain::JobState::from(JobRunState::from(state)), state);
            assert_eq!(
                serde_json::to_value(JobRunState::from(state)).expect("json"),
                serde_json::json!(String::from(state)),
            );
        }

        for mark in domain::DayMark::ALL {
            assert_eq!(domain::DayMark::from(JobDayMark::from(mark)), mark);
            assert_eq!(
                serde_json::to_value(JobDayMark::from(mark)).expect("json"),
                serde_json::json!(String::from(mark)),
            );
        }
    }

    #[test]
    fn the_start_is_read_from_the_path_in_any_offset() {
        let utc = parse_started_at("2026-10-09T06:00:00Z").expect("timestamp");
        let baghdad = parse_started_at("2026-10-09T09:00:00+03:00").expect("timestamp");

        assert_eq!(utc, Utc.with_ymd_and_hms(2026, 10, 9, 6, 0, 0).unwrap());
        assert_eq!(utc, baghdad, "the same instant is the same run");
    }

    #[test]
    fn a_start_that_is_not_a_timestamp_is_refused() {
        for bad in [
            "",
            "yesterday",
            "2026-10-09",
            "2026-10-09T06:00:00",
            "1760000000",
        ] {
            assert!(parse_started_at(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn a_report_with_every_field_left_out_is_a_run_still_going() {
        let input = params(serde_json::json!({}))
            .into_input(code("fires"), "2026-10-09T06:00:00Z")
            .expect("input");

        assert_eq!(input.finished_at, None);
        assert_eq!(input.ok, None);
        assert_eq!(input.rows, None);
        assert_eq!(input.message, None);
    }

    #[test]
    fn nulls_are_the_same_as_fields_left_out() {
        let input = params(serde_json::json!({
            "finished_at": null, "ok": null, "rows": null, "message": null
        }))
        .into_input(code("fires"), "2026-10-09T06:00:00Z")
        .expect("input");

        assert_eq!(input.finished_at, None);
        assert_eq!(input.ok, None);
    }

    #[test]
    fn a_closing_report_carries_its_fields_and_an_empty_message_is_none() {
        let input = params(serde_json::json!({
            "finished_at": "2026-10-09T06:04:00Z", "ok": true, "rows": 312, "message": "  "
        }))
        .into_input(code("fires"), "2026-10-09T06:00:00Z")
        .expect("input");

        assert_eq!(
            input.finished_at,
            Some(Utc.with_ymd_and_hms(2026, 10, 9, 6, 4, 0).unwrap())
        );
        assert_eq!(input.ok, Some(true));
        assert_eq!(input.rows, Some(312));
        assert_eq!(input.message, None);
    }

    #[test]
    fn a_message_over_the_limit_is_refused() {
        let result = params(serde_json::json!({"message": "a".repeat(501)}))
            .into_input(code("fires"), "2026-10-09T06:00:00Z");

        assert!(result.is_err());
    }

    #[test]
    fn a_job_that_never_ran_is_sent_with_nulls_and_fourteen_empty_days() {
        let job = Job::rehydrate(
            code("farm_analysis"),
            "Farm analysis".to_string(),
            None,
            None,
        );
        let now = Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap();
        let status = JobStatus::judge(&job, &RunHistory::empty(), now);

        let json = serde_json::to_value(JobStatusResponse::from(&status)).expect("json");

        assert_eq!(json["job"], "farm_analysis");
        assert_eq!(json["every_hours"], serde_json::Value::Null);
        assert_eq!(json["last_run"], serde_json::Value::Null);
        assert_eq!(json["last_ok"], serde_json::Value::Null);
        assert_eq!(json["next_due"], serde_json::Value::Null);
        assert_eq!(json["state"], "never");
        assert_eq!(json["message"], serde_json::Value::Null);
        assert_eq!(
            json["last_14_days"],
            serde_json::Value::Array(vec![serde_json::Value::Null; 14]),
            "always fourteen entries"
        );
    }

    #[test]
    fn a_job_that_ran_is_sent_with_its_last_run_and_marks_as_text() {
        let job = Job::rehydrate(code("dams"), "Dams".to_string(), None, Some(24));
        let now = Utc.with_ymd_and_hms(2026, 10, 9, 12, 0, 0).unwrap();
        let started_at = Utc.with_ymd_and_hms(2026, 10, 9, 6, 0, 0).unwrap();
        let run = JobRun::rehydrate(
            1,
            code("dams"),
            started_at,
            Some(started_at + chrono::Duration::minutes(4)),
            Some(true),
            Some(2),
            RunMessage::new("2 dams".to_string()).expect("message"),
        );
        let history = RunHistory::of(&[run], history_since(std::slice::from_ref(&job), now));
        let status = JobStatus::judge(&job, &history, now);

        let json = serde_json::to_value(JobStatusResponse::from(&status)).expect("json");

        assert_eq!(json["state"], "ok");
        assert_eq!(json["last_run"]["started_at"], "2026-10-09T06:00:00Z");
        assert_eq!(json["last_run"]["ok"], true);
        assert_eq!(json["last_run"]["rows"], 2);
        assert_eq!(json["last_ok"], "2026-10-09T06:04:00Z");
        assert_eq!(json["next_due"], "2026-10-10T06:00:00Z");
        assert_eq!(json["message"], "2 dams");
        assert_eq!(json["last_14_days"][13], "ok");
        assert_eq!(json["last_14_days"][12], serde_json::Value::Null);
    }
}
