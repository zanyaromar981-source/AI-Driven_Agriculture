use std::sync::Arc;

use crate::features::farms::{
    app::{AppError, FarmRepository, PlaceLocator},
    domain::FarmPlace,
};

/// How many farms are read at a time. Each one is a row with an outline of
/// at most fifty corners, so a batch is small whatever the farms' sizes.
const BATCH: u64 = 200;

/// What a run did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BackfillReport {
    /// Farms that had no place or no area stored.
    pub examined: u64,
    /// Of those, the farms now stored inside a sub-district.
    pub placed: u64,
    /// The farms outside every sub-district: their area is stored, if it
    /// was not already, and their place stays empty.
    pub outside: u64,
    /// The farms someone wrote while the run was looking at them. Whoever
    /// wrote them stored the place, or the next run does.
    pub skipped: u64,
}

/// Gives every farm that has none its place and the area of its outline.
/// Farms drawn before places were kept have neither, and a migration cannot
/// work them out. Running it again is harmless: it finds only the farms
/// outside every sub-district, sees they still are, and writes nothing.
pub struct BackfillFarmPlacesUseCase {
    repository: Arc<dyn FarmRepository>,
    places: Arc<dyn PlaceLocator>,
}

impl BackfillFarmPlacesUseCase {
    pub fn new(repository: Arc<dyn FarmRepository>, places: Arc<dyn PlaceLocator>) -> Self {
        Self { repository, places }
    }

    pub async fn execute(&self) -> Result<BackfillReport, AppError> {
        let mut report = BackfillReport::default();
        let mut after_id = 0;

        loop {
            let farms = self.repository.find_unplaced(after_id, BATCH).await?;

            let Some(last) = farms.last() else {
                break;
            };

            // The walk goes up the ids and never looks back, so a farm that
            // stays without a place cannot be handed out twice in one run.
            after_id = *last.id();

            for farm in &farms {
                let (lat, lon) = farm.outline().centroid();
                let place: Option<FarmPlace> = self.places.locate(lat, lon).await?;

                report.examined += 1;

                // Nothing to store: the area is there and there is still no
                // place. Writing the same row again would tell every website
                // that the farms changed.
                if place.is_none() && *farm.area_stored() {
                    report.outside += 1;
                    continue;
                }

                if !self.repository.fill_place(farm, place.as_ref()).await? {
                    report.skipped += 1;
                } else if place.is_some() {
                    report.placed += 1;
                } else {
                    report.outside += 1;
                }
            }
        }

        tracing::info!(
            examined = report.examined,
            placed = report.placed,
            outside = report.outside,
            skipped = report.skipped,
            "farm places backfilled"
        );

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::features::farms::{
        app::testing::{
            FakeFarmRepository, FakePlaceLocator, RepositoryCall, an_outline, an_outline_outside,
            the_place,
        },
        domain::UnplacedFarm,
    };

    fn inside(id: i32) -> UnplacedFarm {
        UnplacedFarm::rehydrate(id, an_outline(), false, Utc::now())
    }

    fn outside(id: i32) -> UnplacedFarm {
        UnplacedFarm::rehydrate(id, an_outline_outside(), false, Utc::now())
    }

    fn use_case(repository: &FakeFarmRepository) -> BackfillFarmPlacesUseCase {
        BackfillFarmPlacesUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::new()),
        )
    }

    fn fills(repository: &FakeFarmRepository) -> Vec<(i32, Option<FarmPlace>)> {
        repository
            .calls()
            .into_iter()
            .filter_map(|call| match call {
                RepositoryCall::FillPlace { id, place } => Some((id, place)),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn every_farm_without_a_place_is_given_the_one_its_centre_lies_in() {
        let repository = FakeFarmRepository::holding_unplaced(vec![inside(3), outside(5)]);

        let report = use_case(&repository).execute().await.expect("report");

        assert_eq!(
            report,
            BackfillReport {
                examined: 2,
                placed: 1,
                outside: 1,
                skipped: 0,
            }
        );
        assert_eq!(
            fills(&repository),
            vec![(3, Some(the_place())), (5, None)],
            "a farm outside every place still has its area stored"
        );
    }

    #[tokio::test]
    async fn a_farm_already_known_to_be_outside_is_counted_and_not_written_again() {
        let known = UnplacedFarm::rehydrate(5, an_outline_outside(), true, Utc::now());
        let repository = FakeFarmRepository::holding_unplaced(vec![known]);

        let report = use_case(&repository).execute().await.expect("report");

        assert_eq!(
            report,
            BackfillReport {
                examined: 1,
                placed: 0,
                outside: 1,
                skipped: 0,
            }
        );
        assert!(fills(&repository).is_empty());
    }

    #[tokio::test]
    async fn a_farm_with_its_area_but_no_place_is_placed_once_a_shape_covers_it() {
        let covered_now = UnplacedFarm::rehydrate(5, an_outline(), true, Utc::now());
        let repository = FakeFarmRepository::holding_unplaced(vec![covered_now]);

        let report = use_case(&repository).execute().await.expect("report");

        assert_eq!(report.placed, 1);
        assert_eq!(fills(&repository), vec![(5, Some(the_place()))]);
    }

    #[tokio::test]
    async fn the_walk_goes_up_the_ids_until_a_read_comes_back_empty() {
        let repository = FakeFarmRepository::holding_unplaced(vec![inside(3), inside(9)]);

        use_case(&repository).execute().await.expect("report");

        let reads: Vec<i32> = repository
            .calls()
            .into_iter()
            .filter_map(|call| match call {
                RepositoryCall::FindUnplaced { after_id, .. } => Some(after_id),
                _ => None,
            })
            .collect();

        assert_eq!(reads, vec![0, 9]);
    }

    #[tokio::test]
    async fn a_farm_written_meanwhile_is_skipped_not_overwritten() {
        let repository = FakeFarmRepository::holding_unplaced(vec![inside(3), inside(4)])
            .written_while_filling(vec![3]);

        let report = use_case(&repository).execute().await.expect("report");

        assert_eq!(
            report,
            BackfillReport {
                examined: 2,
                placed: 1,
                outside: 0,
                skipped: 1,
            }
        );
    }

    #[tokio::test]
    async fn with_nothing_to_fill_nothing_is_written() {
        let repository = FakeFarmRepository::new();

        let report = use_case(&repository).execute().await.expect("report");

        assert_eq!(report, BackfillReport::default());
        assert!(fills(&repository).is_empty());
    }

    #[tokio::test]
    async fn a_place_that_cannot_be_looked_up_stops_the_run_without_writing() {
        let repository = FakeFarmRepository::holding_unplaced(vec![inside(3)]);
        let use_case = BackfillFarmPlacesUseCase::new(
            Arc::new(repository.clone()),
            Arc::new(FakePlaceLocator::failing()),
        );

        assert!(use_case.execute().await.is_err());
        assert!(fills(&repository).is_empty());
    }
}
