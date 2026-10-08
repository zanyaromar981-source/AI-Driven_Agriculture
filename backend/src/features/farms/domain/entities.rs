use std::collections::HashMap;

use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};

use crate::{
    features::farms::domain::{
        Crop, FarmError, FarmName, GridCell, IdempotencyKey, Outline, PaintedCell,
    },
    shared::Phone,
};

/// One cell of a farm and the crop painted on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Cell {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    position: GridCell,
    crop: Crop,
}

impl Cell {
    pub fn rehydrate(id: i32, position: GridCell, crop: Crop) -> Self {
        Self {
            id: Some(id),
            position,
            crop,
        }
    }
}

/// The land under one crop. `Empty` cells are never reported as a crop.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct CropArea {
    crop: Crop,
    dunam: f64,
}

#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct Farm {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    name: FarmName,
    owner: Phone,
    outline: Outline,
    cells: Vec<Cell>,
    /// Set when the app sent one with the upload; see `IdempotencyKey`.
    idempotency_key: Option<IdempotencyKey>,
    /// When the farmer drew the farm, if the app was offline at the time.
    created_offline_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Farm {
    /// Builds a farm from the walked outline. Every cell inside the outline
    /// becomes part of the farm; the ones the farmer did not paint are
    /// `Empty`. Painted cells that fall outside the outline are returned so
    /// the caller can report them as dropped.
    pub fn new(
        name: FarmName,
        owner: Phone,
        outline: Outline,
        painted: Vec<PaintedCell>,
        idempotency_key: Option<IdempotencyKey>,
        created_offline_at: Option<DateTime<Utc>>,
        max_cells: usize,
    ) -> Result<(Self, Vec<GridCell>), FarmError> {
        let cells = outline
            .cells(max_cells)?
            .into_iter()
            .map(|position| Cell {
                id: None,
                position,
                crop: Crop::Empty,
            })
            .collect();

        let now = Utc::now();

        let mut farm = Self {
            id: None,
            name,
            owner,
            outline,
            cells,
            idempotency_key,
            created_offline_at,
            created_at: now,
            updated_at: now,
        };

        let dropped = farm.paint(painted);

        Ok((farm, dropped))
    }

    /// Reconstruct from persisted state.
    #[allow(clippy::too_many_arguments)]
    pub fn rehydrate(
        id: i32,
        name: FarmName,
        owner: Phone,
        outline: Outline,
        cells: Vec<Cell>,
        idempotency_key: Option<IdempotencyKey>,
        created_offline_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Some(id),
            name,
            owner,
            outline,
            cells,
            idempotency_key,
            created_offline_at,
            created_at,
            updated_at,
        }
    }

    /// Changes the crop on the given cells and leaves every other cell as it
    /// was. Returns the cells that are not part of this farm.
    pub fn repaint(&mut self, painted: Vec<PaintedCell>) -> Vec<GridCell> {
        let dropped = self.paint(painted);

        self.updated_at = Utc::now();

        dropped
    }

    pub fn is_owned_by(&self, phone: &Phone) -> bool {
        &self.owner == phone
    }

    /// The area inside the walked outline. The cell count is close to it but
    /// not equal, because cells on the edge are in or out as a whole.
    pub fn area_dunam(&self) -> f64 {
        self.outline.area_dunam()
    }

    pub fn crop_areas(&self) -> Vec<CropArea> {
        let mut counts: HashMap<Crop, usize> = HashMap::new();

        for cell in &self.cells {
            *counts.entry(cell.crop).or_default() += 1;
        }

        crop_areas(counts.into_iter().collect())
    }

    /// When the same cell is painted twice in one request the later crop
    /// wins, which is what the farmer saw last on the screen.
    fn paint(&mut self, painted: Vec<PaintedCell>) -> Vec<GridCell> {
        let mut wanted: HashMap<GridCell, Crop> = painted
            .into_iter()
            .map(|cell| (cell.position(), cell.crop()))
            .collect();

        for cell in &mut self.cells {
            if let Some(crop) = wanted.remove(&cell.position) {
                cell.crop = crop;
            }
        }

        let mut dropped: Vec<GridCell> = wanted.into_keys().collect();
        dropped.sort();

        dropped
    }
}

/// What the farms list shows for one farm: enough to draw the card without
/// loading every cell.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct FarmSummary {
    id: i32,
    name: FarmName,
    area_dunam: f64,
    crops: Vec<CropArea>,
    /// `(lat, lon)`
    centroid: (f64, f64),
    created_at: DateTime<Utc>,
}

impl FarmSummary {
    /// Reconstruct from persisted state: the farm row and how many cells it
    /// has under each crop.
    pub fn rehydrate(
        id: i32,
        name: FarmName,
        outline: &Outline,
        cells_per_crop: Vec<(Crop, usize)>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            area_dunam: outline.area_dunam(),
            crops: crop_areas(cells_per_crop),
            centroid: outline.centroid(),
            created_at,
        }
    }
}

/// Where one farm is and how large, for the data jobs that compute readings
/// per farm. It carries no owner: a job never needs to know whose farm it is.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct FarmLocation {
    id: i32,
    /// `(lat, lon)`
    centroid: (f64, f64),
    area_dunam: f64,
}

impl FarmLocation {
    /// Reconstruct from persisted state.
    pub fn rehydrate(id: i32, outline: &Outline) -> Self {
        Self {
            id,
            centroid: outline.centroid(),
            area_dunam: outline.area_dunam(),
        }
    }
}

/// Largest area first, so the main crop leads; ties keep the order the crop
/// codes are declared in.
fn crop_areas(cells_per_crop: Vec<(Crop, usize)>) -> Vec<CropArea> {
    let mut planted: Vec<(Crop, usize)> = cells_per_crop
        .into_iter()
        .filter(|(crop, cells)| *crop != Crop::Empty && *cells > 0)
        .collect();

    let declared = |crop: &Crop| Crop::ALL.iter().position(|other| other == crop);

    planted.sort_by(|(first, first_cells), (second, second_cells)| {
        second_cells
            .cmp(first_cells)
            .then(declared(first).cmp(&declared(second)))
    });

    planted
        .into_iter()
        .map(|(crop, cells)| CropArea {
            crop,
            dunam: GridCell::dunams(cells),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::farms::domain::Point;

    const MAX_CELLS: usize = 1_000;

    fn outline() -> Outline {
        Outline::new(vec![
            Point::new(36.0300, 44.6000, None, None).expect("point"),
            Point::new(36.0300, 44.6010, None, None).expect("point"),
            Point::new(36.0310, 44.6010, None, None).expect("point"),
            Point::new(36.0310, 44.6000, None, None).expect("point"),
        ])
        .expect("outline")
    }

    fn farm(painted: Vec<PaintedCell>) -> (Farm, Vec<GridCell>) {
        Farm::new(
            FarmName::new("Upper field".to_string()).expect("name"),
            Phone::new("+9647501234567".to_string()).expect("phone"),
            outline(),
            painted,
            None,
            None,
            MAX_CELLS,
        )
        .expect("farm")
    }

    fn a_cell_inside() -> GridCell {
        outline().cells(MAX_CELLS).expect("cells")[0]
    }

    fn a_cell_outside() -> GridCell {
        GridCell::new(1, 1)
    }

    #[test]
    fn every_cell_inside_the_outline_belongs_to_the_farm_even_unpainted() {
        let (farm, dropped) = farm(vec![]);

        assert_eq!(
            farm.cells().len(),
            outline().cells(MAX_CELLS).expect("cells").len()
        );
        assert!(farm.cells().iter().all(|cell| cell.crop() == Crop::Empty));
        assert!(dropped.is_empty());
    }

    #[test]
    fn a_painted_cell_inside_the_outline_takes_its_crop() {
        let (farm, dropped) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)]);

        let painted = farm
            .cells()
            .iter()
            .find(|cell| cell.position() == a_cell_inside())
            .expect("the painted cell");

        assert_eq!(painted.crop(), Crop::Wheat);
        assert!(dropped.is_empty());
    }

    #[test]
    fn a_painted_cell_outside_the_outline_is_dropped_not_added() {
        let (farm, dropped) = farm(vec![PaintedCell::new(a_cell_outside(), Crop::Wheat)]);

        assert_eq!(dropped, vec![a_cell_outside()]);
        assert!(
            farm.cells()
                .iter()
                .all(|cell| cell.position() != a_cell_outside()),
            "painting must never grow the farm past its outline"
        );
    }

    #[test]
    fn the_later_crop_wins_when_a_cell_is_painted_twice() {
        let (farm, _) = farm(vec![
            PaintedCell::new(a_cell_inside(), Crop::Wheat),
            PaintedCell::new(a_cell_inside(), Crop::Tomato),
        ]);

        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(farm.crop_areas()[0].crop(), Crop::Tomato);
    }

    #[test]
    fn the_area_is_the_outlines_and_the_crops_skip_the_empty_cells() {
        let (farm, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)]);

        assert_eq!(farm.area_dunam(), outline().area_dunam());
        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(farm.crop_areas()[0].dunam(), GridCell::dunams(1));
    }

    #[test]
    fn repainting_changes_only_the_cells_it_names() {
        let cells = outline().cells(MAX_CELLS).expect("cells");
        let (mut farm, _) = farm(vec![
            PaintedCell::new(cells[0], Crop::Wheat),
            PaintedCell::new(cells[1], Crop::Wheat),
        ]);

        let dropped = farm.repaint(vec![
            PaintedCell::new(cells[0], Crop::Barley),
            PaintedCell::new(a_cell_outside(), Crop::Barley),
        ]);

        let crop_at = |position: GridCell| {
            farm.cells()
                .iter()
                .find(|cell| cell.position() == position)
                .expect("cell")
                .crop()
        };

        assert_eq!(crop_at(cells[0]), Crop::Barley);
        assert_eq!(
            crop_at(cells[1]),
            Crop::Wheat,
            "an unnamed cell keeps its crop"
        );
        assert_eq!(dropped, vec![a_cell_outside()]);
    }

    #[test]
    fn a_cell_can_be_cleared_by_painting_it_empty() {
        let (mut farm, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::Wheat)]);

        farm.repaint(vec![PaintedCell::new(a_cell_inside(), Crop::Empty)]);

        assert!(farm.crop_areas().is_empty());
    }

    #[test]
    fn the_summary_lists_the_largest_crop_first() {
        let summary = FarmSummary::rehydrate(
            7,
            FarmName::new("Upper field".to_string()).expect("name"),
            &outline(),
            vec![
                (Crop::Tomato, 400),
                (Crop::Empty, 200),
                (Crop::Wheat, 2_400),
            ],
            Utc::now(),
        );

        assert_eq!(*summary.area_dunam(), outline().area_dunam());
        assert_eq!(
            summary
                .crops()
                .iter()
                .map(|area| (area.crop(), area.dunam()))
                .collect::<Vec<_>>(),
            vec![(Crop::Wheat, 96.0), (Crop::Tomato, 16.0)]
        );
    }
}
