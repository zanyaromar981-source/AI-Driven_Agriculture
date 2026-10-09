use crate::shared::DomainError;

#[derive(thiserror::Error, Debug)]
pub enum ZoneError {
    #[error("A month is written YYYY-MM, with a year between {min_year} and {max_year}")]
    BadMonth { min_year: i32, max_year: i32 },

    #[error("A calendar month is a number from 1 to 12")]
    BadCalendarMonth,

    #[error("The `from` month must not be after the `to` month")]
    FromAfterTo,

    #[error("A year must be a number between {min} and {max}")]
    BadYear { min: i32, max: i32 },

    #[error("The two years to compare must differ")]
    SameYear,

    #[error("Dryness must be a whole number between 0 and 100")]
    DrynessOutOfRange,

    #[error("Rain must be between 0 and 400 percent of normal")]
    RainOutOfRange,

    #[error("Greenness must be between -100 and 300 percent against normal")]
    GreennessOutOfRange,

    #[error("Water need must be a whole number between 0 and 100")]
    WaterNeedOutOfRange,

    #[error("Unknown crop code: {0}")]
    UnknownCrop(String),

    #[error("A crop can be listed only once in the best crops: {0}")]
    RepeatedCrop(String),

    #[error("A shape needs at least one ring, and a ring at least three corners that are numbers")]
    BadShape,

    #[error(transparent)]
    DomainError(#[from] DomainError),
}
