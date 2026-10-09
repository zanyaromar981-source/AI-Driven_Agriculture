use crate::{features::jobs::domain::JobError, shared::DomainError};

/// How a job stands right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum JobState {
    Ok,
    Late,
    Failed,
    Never,
}

impl JobState {
    pub const ALL: [JobState; 4] = [
        JobState::Ok,
        JobState::Late,
        JobState::Failed,
        JobState::Never,
    ];
}

impl From<JobState> for String {
    fn from(value: JobState) -> Self {
        match value {
            JobState::Ok => "ok".to_string(),
            JobState::Late => "late".to_string(),
            JobState::Failed => "failed".to_string(),
            JobState::Never => "never".to_string(),
        }
    }
}

impl TryFrom<&str> for JobState {
    type Error = JobError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ok" => Ok(JobState::Ok),
            "late" => Ok(JobState::Late),
            "failed" => Ok(JobState::Failed),
            "never" => Ok(JobState::Never),
            _ => Err(DomainError::InvalidValue(format!("Invalid job state: {value}")).into()),
        }
    }
}

/// How one day went for a job. A day with nothing to say has no mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DayMark {
    Ok,
    Late,
    Failed,
}

impl DayMark {
    pub const ALL: [DayMark; 3] = [DayMark::Ok, DayMark::Late, DayMark::Failed];
}

impl From<DayMark> for String {
    fn from(value: DayMark) -> Self {
        match value {
            DayMark::Ok => "ok".to_string(),
            DayMark::Late => "late".to_string(),
            DayMark::Failed => "failed".to_string(),
        }
    }
}

impl TryFrom<&str> for DayMark {
    type Error = JobError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ok" => Ok(DayMark::Ok),
            "late" => Ok(DayMark::Late),
            "failed" => Ok(DayMark::Failed),
            _ => Err(DomainError::InvalidValue(format!("Invalid day mark: {value}")).into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_state_round_trips() {
        for state in JobState::ALL {
            let stored = String::from(state);
            let parsed = JobState::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{state:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, state, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn every_day_mark_round_trips() {
        for mark in DayMark::ALL {
            let stored = String::from(mark);
            let parsed = DayMark::try_from(stored.as_str())
                .unwrap_or_else(|err| panic!("{mark:?} stored as {stored:?} but {err:?}"));

            assert_eq!(parsed, mark, "{stored:?} did not round trip");
        }
    }

    #[test]
    fn an_unknown_value_is_rejected_rather_than_defaulted() {
        assert!(JobState::try_from("running").is_err());
        assert!(JobState::try_from("OK").is_err());
        assert!(
            DayMark::try_from("never").is_err(),
            "a day is never 'never': it has no mark"
        );
    }
}
