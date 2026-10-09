use std::collections::HashMap;

use chrono::{DateTime, Utc};
use getset::{CopyGetters, Getters};

use crate::{
    features::farms::domain::{
        Crop, FarmError, FarmName, FarmPlace, GridCell, IdempotencyKey, Outline, PaintedCell,
        TouchedCell,
    },
    shared::Phone,
};

/// One cell of a farm and the crop painted on it.
#[derive(Clone, Copy, Debug, PartialEq, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Cell {
    /// None = new (not yet persisted), Some = existing (persisted)
    id: Option<i32>,
    position: GridCell,
    crop: Crop,
    /// The share of the cell inside the farm's outline, above 0 and at most
    /// 100. A crop's area is the sum of these, not a count of cells.
    inside_pct: f64,
    /// True once a repaint has changed this cell's crop since it was loaded.
    /// The repository writes only these, so two farmers' devices repainting
    /// different cells at the same moment do not undo each other.
    repainted: bool,
}

impl Cell {
    pub fn rehydrate(id: i32, position: GridCell, crop: Crop, inside_pct: f64) -> Self {
        Self {
            id: Some(id),
            position,
            crop,
            inside_pct,
            repainted: false,
        }
    }

    /// A cell the outline touches, not yet painted and not yet stored.
    fn unpainted(touched: TouchedCell) -> Self {
        Self {
            id: None,
            position: touched.position(),
            crop: Crop::EMPTY,
            inside_pct: touched.inside_pct(),
            repainted: false,
        }
    }

    /// Whether both are the same square with the same crop and share,
    /// whatever their ids.
    fn shows_the_same_as(&self, other: &Cell) -> bool {
        self.position == other.position
            && self.crop == other.crop
            && self.inside_pct == other.inside_pct
    }
}

/// What redrawing a farm did.
#[derive(Clone, Debug, PartialEq)]
pub struct Redraw {
    /// The painted cells the new outline does not touch.
    pub dropped_cells: Vec<GridCell>,
    /// False when the farm already had this name, outline and these cells,
    /// which is how a repeated edit looks: there is then nothing to store.
    pub changed: bool,
}

/// The land under one crop. Unpainted (`empty`) cells are never reported as a crop.
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
    /// Where the centre of the outline lies. `None` when it is outside every
    /// sub-district, and for a farm stored before places were kept.
    place: Option<FarmPlace>,
    cells: Vec<Cell>,
    /// Set when the app sent one with the upload; see `IdempotencyKey`.
    idempotency_key: Option<IdempotencyKey>,
    /// When the farmer drew the farm, if the app was offline at the time.
    created_offline_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Farm {
    /// Builds a farm from the walked outline. Every cell the outline touches
    /// becomes part of the farm; the ones the farmer did not paint are
    /// `empty`. Painted cells the outline does not touch are returned so
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
            .map(Cell::unpainted)
            .collect();

        let now = Utc::now();

        let mut farm = Self {
            id: None,
            name,
            owner,
            outline,
            place: None,
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
            place: None,
            cells,
            idempotency_key,
            created_offline_at,
            created_at,
            updated_at,
        }
    }

    /// Says where the farm lies. The place follows the outline, so whoever
    /// draws or redraws a farm looks its centre up and tells the farm; a
    /// farm rebuilt from storage is told the place stored with it.
    pub fn place_at(&mut self, place: Option<FarmPlace>) {
        self.place = place;
    }

    /// Changes the crop on the given cells and leaves every other cell as it
    /// was. Returns the cells that are not part of this farm.
    pub fn repaint(&mut self, painted: Vec<PaintedCell>) -> Vec<GridCell> {
        let dropped = self.paint(painted);

        self.updated_at = Utc::now();

        dropped
    }

    /// Replaces the name, the outline and every cell, as when the farmer
    /// edits the farm in the app. The farm keeps its id, its owner and the
    /// time it was first drawn. The new cells carry only the crops in
    /// `painted`: an edit sends the whole farm, so nothing is carried over
    /// from the old cells. The rules are those of `new`, and a refused
    /// outline leaves the farm as it was.
    pub fn redraw(
        &mut self,
        name: FarmName,
        outline: Outline,
        painted: Vec<PaintedCell>,
        max_cells: usize,
    ) -> Result<Redraw, FarmError> {
        let mut cells: Vec<Cell> = outline
            .cells(max_cells)?
            .into_iter()
            .map(Cell::unpainted)
            .collect();

        let dropped_cells = paint(&mut cells, painted);

        let changed = name != self.name
            || outline != self.outline
            || cells.len() != self.cells.len()
            || !shows_the_same(&cells, &self.cells);

        if changed {
            for cell in &mut cells {
                cell.repainted = false;
            }

            self.name = name;
            self.outline = outline;
            self.cells = cells;
            self.updated_at = Utc::now();
        }

        Ok(Redraw {
            dropped_cells,
            changed,
        })
    }

    pub fn is_owned_by(&self, phone: &Phone) -> bool {
        &self.owner == phone
    }

    /// The area inside the walked outline. The crops and the empty land add
    /// up to it, because each cell counts only for its share inside.
    pub fn area_dunam(&self) -> f64 {
        self.outline.area_dunam()
    }

    pub fn area_m2(&self) -> f64 {
        self.outline.area_m2()
    }

    pub fn crop_areas(&self) -> Vec<CropArea> {
        crop_areas(self.inside_per_crop())
    }

    /// The shares of the cells under each crop, added up. The cells are
    /// added in grid order whatever order they are held in, so the same farm
    /// always gives the same sum to the last digit.
    fn inside_per_crop(&self) -> Vec<(Crop, f64)> {
        let mut cells: Vec<&Cell> = self.cells.iter().collect();
        cells.sort_by_key(|cell| cell.position);

        let mut inside: HashMap<Crop, f64> = HashMap::new();

        for cell in cells {
            *inside.entry(cell.crop).or_default() += cell.inside_pct;
        }

        inside.into_iter().collect()
    }

    /// The crops this painting would put on a cell that does not carry them
    /// yet, without `empty`. These are the new data in a repaint or an edit:
    /// a cell sent again with the crop it already has changes nothing, so a
    /// crop staff switched off since may stay where it is.
    pub fn crops_introduced_by(&self, painted: &[PaintedCell]) -> Vec<Crop> {
        let current: HashMap<GridCell, Crop> = self
            .cells
            .iter()
            .map(|cell| (cell.position, cell.crop))
            .collect();

        let mut introduced: Vec<Crop> = painted
            .iter()
            .filter(|cell| !cell.crop().is_empty())
            .filter(|cell| current.get(&cell.position()) != Some(&cell.crop()))
            .map(|cell| cell.crop())
            .collect();
        introduced.sort();
        introduced.dedup();

        introduced
    }

    fn paint(&mut self, painted: Vec<PaintedCell>) -> Vec<GridCell> {
        paint(&mut self.cells, painted)
    }
}

/// Puts the painted crops on the cells and returns the painted cells that
/// are not among them. When the same cell is painted twice in one request
/// the later crop wins, which is what the farmer saw last on the screen.
fn paint(cells: &mut [Cell], painted: Vec<PaintedCell>) -> Vec<GridCell> {
    let mut wanted: HashMap<GridCell, Crop> = painted
        .into_iter()
        .map(|cell| (cell.position(), cell.crop()))
        .collect();

    for cell in cells {
        if let Some(crop) = wanted.remove(&cell.position)
            && cell.crop != crop
        {
            cell.crop = crop;
            cell.repainted = true;
        }
    }

    let mut dropped: Vec<GridCell> = wanted.into_keys().collect();
    dropped.sort();

    dropped
}

/// Whether two sets of cells of the same size are the same squares with the
/// same crops and shares, in any order.
fn shows_the_same(first: &[Cell], second: &[Cell]) -> bool {
    let known: HashMap<GridCell, &Cell> = second.iter().map(|cell| (cell.position, cell)).collect();

    first.iter().all(|cell| {
        known
            .get(&cell.position)
            .is_some_and(|other| cell.shows_the_same_as(other))
    })
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
    place: Option<FarmPlace>,
    created_at: DateTime<Utc>,
}

impl FarmSummary {
    /// Reconstruct from persisted state: the farm row and, for each crop,
    /// the sum of the `inside_pct` of its cells.
    pub fn rehydrate(
        id: i32,
        name: FarmName,
        outline: &Outline,
        place: Option<FarmPlace>,
        inside_per_crop: Vec<(Crop, f64)>,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            area_dunam: outline.area_dunam(),
            crops: crop_areas(inside_per_crop),
            centroid: outline.centroid(),
            place,
            created_at,
        }
    }
}

/// A farm's summary together with whose farm it is. Only Ministry staff see
/// farms across owners, so only their listing carries the owner.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct OwnedFarmSummary {
    owner: Phone,
    summary: FarmSummary,
}

impl OwnedFarmSummary {
    pub fn new(owner: Phone, summary: FarmSummary) -> Self {
        Self { owner, summary }
    }
}

/// A farm whose place or area is not stored yet: what is needed to work
/// them out, and the time of its last write, which tells whether the farm
/// changed while that was being done.
#[derive(Clone, Debug, Getters)]
#[getset(get = "pub")]
pub struct UnplacedFarm {
    id: i32,
    outline: Outline,
    /// True when only the place is missing. Such a farm was looked up
    /// before and found outside every sub-district.
    area_stored: bool,
    updated_at: DateTime<Utc>,
}

impl UnplacedFarm {
    /// Reconstruct from persisted state.
    pub fn rehydrate(
        id: i32,
        outline: Outline,
        area_stored: bool,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            outline,
            area_stored,
            updated_at,
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

/// Largest area first, so the main crop leads; crops with the same area
/// follow the alphabet of their codes, so the order never depends on the
/// crop list staff keep. Takes the summed `inside_pct` of each crop's cells.
fn crop_areas(inside_per_crop: Vec<(Crop, f64)>) -> Vec<CropArea> {
    let mut planted: Vec<(Crop, f64)> = inside_per_crop
        .into_iter()
        .filter(|(crop, inside_pct)| !crop.is_empty() && *inside_pct > 0.0)
        .collect();

    planted.sort_by(|(first, first_inside), (second, second_inside)| {
        second_inside
            .total_cmp(first_inside)
            .then(first.cmp(second))
    });

    planted
        .into_iter()
        .map(|(crop, inside_pct)| CropArea {
            crop,
            dunam: GridCell::dunams(inside_pct),
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

    fn touched() -> Vec<TouchedCell> {
        outline().cells(MAX_CELLS).expect("cells")
    }

    fn positions() -> Vec<GridCell> {
        touched().iter().map(TouchedCell::position).collect()
    }

    fn a_cell_inside() -> GridCell {
        positions()[0]
    }

    /// A smaller field inside the first one.
    fn another_outline() -> Outline {
        Outline::new(vec![
            Point::new(36.0302, 44.6002, None, None).expect("point"),
            Point::new(36.0302, 44.6008, None, None).expect("point"),
            Point::new(36.0307, 44.6008, None, None).expect("point"),
            Point::new(36.0307, 44.6002, None, None).expect("point"),
        ])
        .expect("outline")
    }

    fn name(text: &str) -> FarmName {
        FarmName::new(text.to_string()).expect("name")
    }

    /// The farm as the repository hands it back: every cell with an id.
    fn stored(farm: &Farm) -> Farm {
        let cells = farm
            .cells()
            .iter()
            .zip(1..)
            .map(|(cell, id)| Cell::rehydrate(id, cell.position(), cell.crop(), cell.inside_pct()))
            .collect();

        Farm::rehydrate(
            7,
            farm.name().clone(),
            farm.owner().clone(),
            farm.outline().clone(),
            cells,
            None,
            None,
            *farm.created_at(),
            *farm.updated_at(),
        )
    }

    fn a_cell_outside() -> GridCell {
        GridCell::new(1, 1)
    }

    #[test]
    fn every_cell_the_outline_touches_belongs_to_the_farm_even_unpainted() {
        let (farm, dropped) = farm(vec![]);

        assert_eq!(farm.cells().len(), touched().len());
        assert!(farm.cells().iter().all(|cell| cell.crop() == Crop::EMPTY));
        assert!(dropped.is_empty());
    }

    #[test]
    fn every_cell_carries_the_share_of_it_inside_the_outline() {
        let (farm, _) = farm(vec![]);

        assert!(
            farm.cells()
                .iter()
                .zip(touched())
                .all(|(cell, touched)| cell.position() == touched.position()
                    && cell.inside_pct() == touched.inside_pct())
        );
        assert!(
            farm.cells().iter().any(|cell| cell.inside_pct() < 100.0),
            "a walked field never sits on the grid lines, so it has cut cells"
        );
    }

    #[test]
    fn the_crops_and_the_empty_land_add_up_to_the_area_of_the_farm() {
        let cells = positions();
        let painted = cells
            .iter()
            .enumerate()
            .map(|(index, position)| {
                let crop = match index % 3 {
                    0 => Crop::of("wheat"),
                    1 => Crop::of("tomato"),
                    _ => Crop::EMPTY,
                };

                PaintedCell::new(*position, crop)
            })
            .collect();

        let (farm, _) = farm(painted);

        let planted: f64 = farm.crop_areas().iter().map(CropArea::dunam).sum();
        let empty: f64 = farm
            .cells()
            .iter()
            .filter(|cell| cell.crop() == Crop::EMPTY)
            .map(|cell| GridCell::dunams(cell.inside_pct()))
            .sum();
        let total = planted + empty;

        assert_eq!(farm.crop_areas().len(), 2);
        assert!(empty > 0.0);
        assert!(
            (total - farm.area_dunam()).abs() < 0.05 / 2_500.0,
            "{total} dunam of crops and empty land for a farm of {}",
            farm.area_dunam()
        );
    }

    #[test]
    fn a_crop_on_a_cut_cell_counts_only_the_part_inside() {
        let cut = touched()
            .into_iter()
            .find(|cell| cell.inside_pct() < 100.0)
            .expect("a cut cell");

        let (farm, _) = farm(vec![PaintedCell::new(cut.position(), Crop::of("wheat"))]);

        assert_eq!(
            farm.crop_areas()[0].dunam(),
            GridCell::dunams(cut.inside_pct())
        );
        assert!(farm.crop_areas()[0].dunam() < 1.0 / 25.0);
    }

    #[test]
    fn a_painted_cell_inside_the_outline_takes_its_crop() {
        let (farm, dropped) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);

        let painted = farm
            .cells()
            .iter()
            .find(|cell| cell.position() == a_cell_inside())
            .expect("the painted cell");

        assert_eq!(painted.crop(), Crop::of("wheat"));
        assert!(dropped.is_empty());
    }

    #[test]
    fn a_painted_cell_outside_the_outline_is_dropped_not_added() {
        let (farm, dropped) = farm(vec![PaintedCell::new(a_cell_outside(), Crop::of("wheat"))]);

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
            PaintedCell::new(a_cell_inside(), Crop::of("wheat")),
            PaintedCell::new(a_cell_inside(), Crop::of("tomato")),
        ]);

        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("tomato"));
    }

    #[test]
    fn the_area_is_the_outlines_and_the_crops_skip_the_empty_cells() {
        let (farm, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);

        assert_eq!(farm.area_dunam(), outline().area_dunam());
        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(
            farm.crop_areas()[0].dunam(),
            GridCell::dunams(touched()[0].inside_pct())
        );
    }

    #[test]
    fn repainting_changes_only_the_cells_it_names() {
        let cells = positions();
        let (mut farm, _) = farm(vec![
            PaintedCell::new(cells[0], Crop::of("wheat")),
            PaintedCell::new(cells[1], Crop::of("wheat")),
        ]);

        let dropped = farm.repaint(vec![
            PaintedCell::new(cells[0], Crop::of("barley")),
            PaintedCell::new(a_cell_outside(), Crop::of("barley")),
        ]);

        let crop_at = |position: GridCell| {
            farm.cells()
                .iter()
                .find(|cell| cell.position() == position)
                .expect("cell")
                .crop()
        };

        assert_eq!(crop_at(cells[0]), Crop::of("barley"));
        assert_eq!(
            crop_at(cells[1]),
            Crop::of("wheat"),
            "an unnamed cell keeps its crop"
        );
        assert_eq!(dropped, vec![a_cell_outside()]);
    }

    #[test]
    fn only_cells_whose_crop_really_changed_are_marked_repainted() {
        let cells = positions();
        let (mut farm, _) = farm(vec![PaintedCell::new(cells[0], Crop::of("wheat"))]);
        for cell in &mut farm.cells {
            cell.repainted = false;
        }

        farm.repaint(vec![
            PaintedCell::new(cells[0], Crop::of("wheat")),
            PaintedCell::new(cells[1], Crop::of("barley")),
        ]);

        let repainted: Vec<GridCell> = farm
            .cells()
            .iter()
            .filter(|cell| cell.repainted())
            .map(|cell| cell.position())
            .collect();

        assert_eq!(
            repainted,
            vec![cells[1]],
            "painting a cell the crop it already has is not a change"
        );
    }

    #[test]
    fn a_cell_can_be_cleared_by_painting_it_empty() {
        let (mut farm, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);

        farm.repaint(vec![PaintedCell::new(a_cell_inside(), Crop::EMPTY)]);

        assert!(farm.crop_areas().is_empty());
    }

    #[test]
    fn redrawing_replaces_the_name_the_outline_and_every_cell() {
        let (drawn, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);
        let mut farm = stored(&drawn);
        let inside_the_new = another_outline().cells(MAX_CELLS).expect("cells")[0].position();

        let redraw = farm
            .redraw(
                name("Lower field"),
                another_outline(),
                vec![PaintedCell::new(inside_the_new, Crop::of("barley"))],
                MAX_CELLS,
            )
            .expect("redraw");

        assert!(redraw.changed);
        assert!(redraw.dropped_cells.is_empty());
        assert_eq!(*farm.id(), Some(7), "the farm keeps its id");
        assert_eq!(farm.name().as_str(), "Lower field");
        assert_eq!(farm.outline(), &another_outline());
        assert_eq!(
            farm.cells().len(),
            another_outline().cells(MAX_CELLS).expect("cells").len()
        );
        assert!(
            farm.cells().iter().all(|cell| cell.id().is_none()),
            "every cell is new: none of the old set is kept"
        );
        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(
            farm.crop_areas()[0].crop(),
            Crop::of("barley"),
            "the old wheat is not carried over"
        );
        assert_eq!(farm.created_at(), drawn.created_at());
        assert!(farm.updated_at() >= drawn.updated_at());
    }

    #[test]
    fn redrawing_reports_the_painted_cells_the_new_outline_does_not_touch() {
        let (drawn, _) = farm(vec![]);
        let mut farm = stored(&drawn);
        let only_in_the_old = positions()[0];

        let redraw = farm
            .redraw(
                name("Upper field"),
                another_outline(),
                vec![PaintedCell::new(only_in_the_old, Crop::of("wheat"))],
                MAX_CELLS,
            )
            .expect("redraw");

        assert_eq!(redraw.dropped_cells, vec![only_in_the_old]);
        assert!(farm.crop_areas().is_empty());
    }

    #[test]
    fn redrawing_a_farm_as_it_already_is_changes_nothing() {
        let painted = vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))];
        let (drawn, _) = farm(painted.clone());
        let mut farm = stored(&drawn);
        let before = farm.clone();

        let redraw = farm
            .redraw(name("Upper field"), outline(), painted, MAX_CELLS)
            .expect("redraw");

        assert!(!redraw.changed);
        assert_eq!(
            farm.cells(),
            before.cells(),
            "the stored cells and ids stay"
        );
        assert_eq!(farm.updated_at(), before.updated_at());
    }

    #[test]
    fn redrawing_with_only_another_crop_is_a_change() {
        let (drawn, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);
        let mut farm = stored(&drawn);

        let redraw = farm
            .redraw(
                name("Upper field"),
                outline(),
                vec![PaintedCell::new(a_cell_inside(), Crop::of("olive"))],
                MAX_CELLS,
            )
            .expect("redraw");

        assert!(redraw.changed);
        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("olive"));
    }

    #[test]
    fn a_farm_stored_with_whole_centre_cells_is_redrawn_into_touched_cells() {
        // What a farm saved before cells carried a share reads as: fewer
        // cells, every one whole.
        let (drawn, _) = farm(vec![]);
        let old_cells: Vec<Cell> = drawn
            .cells()
            .iter()
            .filter(|cell| cell.inside_pct() > 50.0)
            .zip(1..)
            .map(|(cell, id)| Cell::rehydrate(id, cell.position(), Crop::EMPTY, 100.0))
            .collect();
        let mut farm = stored(&drawn);
        farm.cells = old_cells;

        let redraw = farm
            .redraw(name("Upper field"), outline(), vec![], MAX_CELLS)
            .expect("redraw");

        assert!(redraw.changed, "the same outline now gives other cells");
        assert_eq!(farm.cells().len(), touched().len());
    }

    #[test]
    fn a_redraw_that_is_refused_leaves_the_farm_as_it_was() {
        let (drawn, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);
        let mut farm = stored(&drawn);
        let before = farm.clone();

        let result = farm.redraw(name("Lower field"), another_outline(), vec![], 3);

        assert!(matches!(result, Err(FarmError::TooManyCells(3))));
        assert_eq!(farm.name(), before.name());
        assert_eq!(farm.outline(), before.outline());
        assert_eq!(farm.cells(), before.cells());
    }

    #[test]
    fn the_summary_lists_the_largest_crop_first() {
        let summary = FarmSummary::rehydrate(
            7,
            FarmName::new("Upper field".to_string()).expect("name"),
            &outline(),
            None,
            vec![
                (Crop::of("tomato"), 40_000.0),
                (Crop::EMPTY, 20_000.0),
                (Crop::of("wheat"), 240_000.0),
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
            vec![(Crop::of("wheat"), 96.0), (Crop::of("tomato"), 16.0)]
        );
    }

    fn a_place() -> FarmPlace {
        FarmPlace::new(
            "Sulaymaniyah".to_string(),
            "chamchamal".to_string(),
            "sangaw".to_string(),
        )
        .expect("place")
    }

    #[test]
    fn a_new_farm_has_no_place_until_it_is_told_one() {
        let (mut farm, _) = farm(vec![]);

        assert_eq!(farm.place(), &None);

        farm.place_at(Some(a_place()));

        assert_eq!(farm.place(), &Some(a_place()));
    }

    #[test]
    fn a_farm_moved_outside_every_place_loses_the_place_it_had() {
        let (mut farm, _) = farm(vec![]);
        farm.place_at(Some(a_place()));

        farm.place_at(None);

        assert_eq!(farm.place(), &None);
    }

    #[test]
    fn the_area_in_square_metres_is_2500_times_the_area_in_dunams() {
        let (farm, _) = farm(vec![]);

        assert!((farm.area_m2() - farm.area_dunam() * 2_500.0).abs() < 1e-6);
    }

    #[test]
    fn the_summary_carries_the_farms_place() {
        let summary = FarmSummary::rehydrate(
            7,
            FarmName::new("Upper field".to_string()).expect("name"),
            &outline(),
            Some(a_place()),
            vec![],
            Utc::now(),
        );

        assert_eq!(summary.place(), &Some(a_place()));
    }

    #[test]
    fn only_crops_put_on_a_cell_that_did_not_carry_them_are_introduced() {
        let cells = positions();
        let (farm, _) = farm(vec![PaintedCell::new(cells[0], Crop::of("wheat"))]);

        let introduced = farm.crops_introduced_by(&[
            // The crop the cell already has: nothing new.
            PaintedCell::new(cells[0], Crop::of("wheat")),
            // A new crop on another cell, named twice.
            PaintedCell::new(cells[1], Crop::of("rice")),
            PaintedCell::new(cells[1], Crop::of("rice")),
            // A cell that is not part of the farm.
            PaintedCell::new(a_cell_outside(), Crop::of("barley")),
        ]);

        assert_eq!(introduced, vec![Crop::of("barley"), Crop::of("rice")]);
    }

    #[test]
    fn unpainting_introduces_no_crop() {
        let (farm, _) = farm(vec![PaintedCell::new(a_cell_inside(), Crop::of("wheat"))]);

        assert!(
            farm.crops_introduced_by(&[PaintedCell::new(a_cell_inside(), Crop::EMPTY)])
                .is_empty()
        );
    }

    #[test]
    fn a_stored_cell_with_a_crop_no_list_knows_still_reads_back_and_counts() {
        // Slices share no key, so a crop can be gone from the crop list
        // while a farm still carries it. Reading never asks the list.
        let (drawn, _) = farm(vec![]);
        let cells = drawn
            .cells()
            .iter()
            .enumerate()
            .map(|(index, cell)| {
                Cell::rehydrate(index as i32 + 1, cell.position(), Crop::of("teff"), 100.0)
            })
            .collect::<Vec<_>>();
        let count = cells.len() as f64;

        let farm = Farm::rehydrate(
            7,
            drawn.name().clone(),
            drawn.owner().clone(),
            drawn.outline().clone(),
            cells,
            None,
            None,
            *drawn.created_at(),
            *drawn.updated_at(),
        );

        assert_eq!(farm.crop_areas().len(), 1);
        assert_eq!(farm.crop_areas()[0].crop(), Crop::of("teff"));
        assert_eq!(
            farm.crop_areas()[0].dunam(),
            GridCell::dunams(count * 100.0)
        );
    }

    #[test]
    fn crops_with_the_same_area_follow_the_alphabet_of_their_codes() {
        let areas = crop_areas(vec![
            (Crop::of("wheat"), 100.0),
            (Crop::of("rice"), 100.0),
            (Crop::of("barley"), 100.0),
            (Crop::of("tomato"), 200.0),
        ]);

        assert_eq!(
            areas.iter().map(|area| area.crop()).collect::<Vec<_>>(),
            vec![
                Crop::of("tomato"),
                Crop::of("barley"),
                Crop::of("rice"),
                Crop::of("wheat")
            ]
        );
    }
}
