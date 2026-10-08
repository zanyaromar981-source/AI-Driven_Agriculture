use chrono::Utc;

use crate::features::zones::{
    app::{AppError, ZoneRepository},
    domain::Month,
};

/// The month the dashboard asked for or, when it asked for none, the latest
/// month that has any zone reading. Before the first reading arrives there
/// is no such month, and the current one is shown, empty.
pub(super) async fn month_or_latest(
    repository: &dyn ZoneRepository,
    asked: Option<Month>,
) -> Result<Month, AppError> {
    if let Some(month) = asked {
        return Ok(month);
    }

    let latest = repository.find_reading_months().await?.into_iter().max();

    Ok(latest.unwrap_or_else(|| Month::containing(Utc::now().date_naive())))
}
