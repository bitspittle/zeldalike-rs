use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    iter::FromIterator,
    ops::Add,
};

/// Data that targets a square in the `Grid`
///
/// Note: You can convert a `(x, y)` tuple into a grid using `into()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridCoord {
    pub x: i16,
    pub y: i16,
}

/// Data that represents the size of a region, by offsetting from a `GridCoord`
///
/// See `GridRegion` for more details.
///
/// Note: You can convert a `(x, y)` tuple into a range using `into()`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridRange {
    pub w: u16,
    pub h: u16,
}

/// Data that represents a rectangular section of the grid, with the top-left square specified by
/// its `coord`, and the extra horizontal/vertical squares specified by its `range`.
///
/// An range of (0, 0) paired with a coordinate, `(x, y)` indicates the square at `(x, y)`, while
/// a range of (2, 4) indicates the region that stretches from `(x, y)` to `(x+2, y+4)`. A region
/// can never be smaller than a single square.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridRegion {
    pub coord: GridCoord,
    pub range: GridRange,
}

impl From<(i16, i16)> for GridCoord {
    fn from(coord: (i16, i16)) -> Self {
        GridCoord {
            x: coord.0,
            y: coord.1,
        }
    }
}

impl From<(u16, u16)> for GridRange {
    fn from(range: (u16, u16)) -> Self {
        GridRange {
            w: range.0,
            h: range.1,
        }
    }
}

impl From<((i16, i16), (u16, u16))> for GridRegion {
    fn from(coord_span: ((i16, i16), (u16, u16))) -> Self {
        GridRegion {
            coord: coord_span.0.into(),
            range: coord_span.1.into(),
        }
    }
}

impl From<(GridCoord, GridCoord)> for GridRegion {
    fn from(coords: (GridCoord, GridCoord)) -> Self {
        let coord0 = coords.0;
        let coord1 = coords.1;

        // User *probably* specified top-left / bottom-right coords, but let's convert just in
        // case so we can handle any order of any two corners.
        let coord_tl: (i16, i16) = (i16::min(coord0.x, coord1.x), i16::min(coord0.y, coord1.y));
        let coord_br: (i16, i16) = (i16::max(coord0.x, coord1.x), i16::max(coord0.y, coord1.y));
        let range: (u16, u16) = (
            (coord_br.0 - coord_tl.0) as u16,
            (coord_br.1 - coord_tl.1) as u16,
        );

        (coord_tl, range).into()
    }
}

impl From<(i16, i16)> for GridRegion {
    fn from(coord: (i16, i16)) -> Self {
        (coord, (0 as u16, 0 as u16)).into()
    }
}

impl Add<(u16, u16)> for GridCoord {
    type Output = GridCoord;

    fn add(self, rhs: (u16, u16)) -> Self::Output {
        let range: GridRange = rhs.into();
        self.add(range)
    }
}
impl Add<GridRange> for GridCoord {
    type Output = GridCoord;

    fn add(self, rhs: GridRange) -> Self::Output {
        GridCoord {
            x: self.x + rhs.w as i16,
            y: self.y + rhs.h as i16,
        }
    }
}

impl GridRegion {
    /// Return a new region that bounds both `r1` and `r2`
    pub fn bounding(r1: GridRegion, r2: GridRegion) -> GridRegion {
        if r1 == r2 {
            return r1;
        }
        let c1_tl = r1.coord;
        let c1_br = r1.coord + r1.range;
        let c2_tl = r2.coord;
        let c2_br = r2.coord + r2.range;

        let tl: GridCoord = (c1_tl.x.min(c2_tl.x), c1_tl.y.min(c2_tl.y)).into();
        let br: GridCoord = (c1_br.x.max(c2_br.x), c1_br.y.max(c2_br.y)).into();

        (tl, br).into()
    }

    /// Iterate all `GridCoord` elements covered by this region
    fn iter(self) -> impl Iterator<Item = GridCoord> {
        let w1 = self.range.w + 1;
        let h1 = self.range.h + 1;
        let num_squares = w1 * h1;
        (0..num_squares)
            .map(move |i| (i % w1, i / w1))
            .map(move |(x_delta, y_delta)| self.coord + (x_delta, y_delta))
    }
}

/// A `Grid` allows the caller to associate items with physical space, which can then be queried for
/// later.
///
/// This can be useful, for example, to a system responsible for managing collisions - it can
/// partition the world up into subsections, registering bodies with much smaller areas, so that
/// when it runs a pass to test collisions, it can vastly reduce the number of bodies to consider.
pub struct Grid<T: Copy + Eq + Hash> {
    coord_items: HashMap<GridCoord, HashSet<T>>,
    item_regions: HashMap<T, GridRegion>,
}

#[allow(clippy::new_without_default_derive)] // Explicit API is intentional
impl<T: Copy + Eq + Hash> Grid<T> {
    pub fn new() -> Self {
        Grid {
            coord_items: Default::default(),
            item_regions: Default::default(),
        }
    }

    pub fn insert(&mut self, item: T, region: GridRegion) {
        self.remove(item);
        region.iter().for_each(|coord| {
            let items = self.coord_items.entry(coord).or_default();
            items.insert(item);
        });

        self.item_regions.insert(item, region);
    }

    pub fn remove(&mut self, item: T) {
        if let Some(region) = self.item_regions.remove(&item) {
            region.iter().for_each(|coord| {
                let items = self.coord_items.entry(coord).or_default();
                items.remove(&item);
                if items.is_empty() {
                    self.coord_items.remove(&coord);
                }
            });
        }
    }

    pub fn query(&self, region: GridRegion) -> HashSet<&T> {
        HashSet::from_iter(
            region
                .iter()
                .filter_map(|coord| self.coord_items.get(&coord))
                .flatten(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removing_last_items_also_clears_inner_structs() {
        let mut id_grid: Grid<i32> = Grid::new();
        let a_region: GridRegion = (10, 10).into();

        id_grid.insert(1, a_region);
        id_grid.insert(2, a_region);
        id_grid.insert(3, a_region);

        assert_eq!(id_grid.coord_items.contains_key(&a_region.coord), true);
        assert_eq!(id_grid.item_regions.len(), 3);

        id_grid.remove(2);
        assert_eq!(id_grid.coord_items.contains_key(&a_region.coord), true);
        assert_eq!(id_grid.item_regions.len(), 2);

        id_grid.remove(1);
        assert_eq!(id_grid.coord_items.contains_key(&a_region.coord), true);
        assert_eq!(id_grid.item_regions.len(), 1);

        id_grid.remove(3);
        assert_eq!(id_grid.coord_items.contains_key(&a_region.coord), false);
        assert_eq!(id_grid.item_regions.len(), 0);
    }
}
