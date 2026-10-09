use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum JobError {
    #[error("A run cannot finish before it starts")]
    FinishesBeforeItStarts,

    #[error("A run cannot start in the future")]
    StartsInTheFuture,

    #[error("Runs older than {days} days are not kept")]
    TooOld { days: i64 },

    #[error(
        "finished_at and ok come together: both null while the run is going, both set once it is over"
    )]
    HalfFinished,

    #[error("rows must not be negative")]
    NegativeRows,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
