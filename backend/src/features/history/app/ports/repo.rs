use async_trait::async_trait;

use crate::features::history::{
    app::AppError,
    domain::{HistoryWindow, Metric, MetricCoverage, Series, SeriesUpload},
};

#[async_trait]
pub trait HistoryRepository: Send + Sync + std::fmt::Debug {
    /// Returns the stored months of the farm inside the window for each of
    /// the metrics, in no particular order. A metric with no month in the
    /// window is not returned.
    async fn find_series(
        &self,
        farm_id: i32,
        metrics: &[Metric],
        window: &HistoryWindow,
    ) -> Result<Vec<Series>, AppError>;

    /// Returns how much is stored of every metric of every farm, without
    /// the values.
    async fn find_all_coverage(&self) -> Result<Vec<MetricCoverage>, AppError>;

    /// Stores the months of a push and the facts of their series in one
    /// transaction. A month already stored gets the pushed value; a stored
    /// month the push does not name is left as it is. Returns `true` when
    /// that is what happened. A push read from its source before the one
    /// already stored replaces nothing: it only adds the months that were
    /// missing, and the answer is `false`.
    async fn store(&self, upload: &SeriesUpload) -> Result<bool, AppError>;

    /// Removes every month of one metric of the farm, and the facts of the
    /// series. Removing what is not there is not an error.
    async fn delete_series(&self, farm_id: i32, metric: Metric) -> Result<(), AppError>;
}
