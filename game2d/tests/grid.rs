use game2d::grid::Grid;
use game2d::grid::GridRegion;

mod test_support;
use crate::test_support::*;

#[test]
fn can_insert_items_into_same_region() {
    let mut id_grid: Grid<i32> = Grid::new();
    let a_region = GridRegion::square(2, 1);

    id_grid.insert(1, a_region);
    id_grid.insert(2, a_region);

    let results = id_grid.query(a_region);
    assert_set_contains_exactly(results, &[1, 2]);
}

#[test]
fn can_remove_items() {
    let mut id_grid: Grid<i32> = Grid::new();
    let a_region = GridRegion::square(5, 10);

    id_grid.insert(1, a_region);
    {
        let results = id_grid.query(a_region);
        assert_eq!(results.contains(&1), true);
    }

    id_grid.remove(1);
    {
        let results = id_grid.query(a_region);
        assert_eq!(results.contains(&1), false);
    }
}

#[test]
fn can_insert_items_that_span_large_regions() {
    let mut id_grid: Grid<i32> = Grid::new();
    let large_region = GridRegion::new([1, 2], [10, 10]);

    id_grid.insert(1, large_region);

    assert_eq!(id_grid.query(GridRegion::square(1, 2)).contains(&1), true);
    assert_eq!(id_grid.query(GridRegion::square(5, 7)).contains(&1), true);
    assert_eq!(id_grid.query(GridRegion::square(11, 12)).contains(&1), true);

    assert_eq!(id_grid.query(GridRegion::square(0, 2)).contains(&1), false);
    assert_eq!(id_grid.query(GridRegion::square(1, 1)).contains(&1), false);
    assert_eq!(id_grid.query(GridRegion::square(12, 3)).contains(&1), false);
    assert_eq!(
        id_grid.query(GridRegion::square(11, 13)).contains(&1),
        false
    );
}

#[test]
fn item_regions_can_overlap() {
    let mut id_grid: Grid<i32> = Grid::new();
    let region_a = GridRegion::new([1, 2], [3, 0]);
    let region_b = GridRegion::new([4, 2], [0, 2]);

    id_grid.insert(1, region_a);
    id_grid.insert(2, region_b);

    assert_set_contains_exactly(id_grid.query(GridRegion::square(1, 2)), &[1]);
    assert_set_contains_exactly(id_grid.query(GridRegion::square(2, 2)), &[1]);
    assert_set_contains_exactly(id_grid.query(GridRegion::square(3, 2)), &[1]);
    assert_set_contains_exactly(id_grid.query(GridRegion::square(4, 2)), &[1, 2]);
    assert_set_contains_exactly(id_grid.query(GridRegion::square(4, 3)), &[2]);
    assert_set_contains_exactly(id_grid.query(GridRegion::square(4, 4)), &[2]);
}

#[test]
fn subsequent_calls_to_insert_overwrite_previous() {
    let mut id_grid: Grid<i32> = Grid::new();

    id_grid.insert(1, GridRegion::square(1, 2));
    id_grid.insert(1, GridRegion::square(3, 4));

    assert_eq!(id_grid.query(GridRegion::square(3, 4)).contains(&1), true);
    assert_eq!(id_grid.query(GridRegion::square(1, 2)).contains(&1), false);

    id_grid.insert(1, GridRegion::square(5, 6));

    assert_eq!(id_grid.query(GridRegion::square(3, 4)).contains(&1), false);
    assert_eq!(id_grid.query(GridRegion::square(5, 6)).contains(&1), true);
}

#[test]
fn multiple_calls_to_remove_are_harmless() {
    let mut id_grid: Grid<i32> = Grid::new();

    let a_region = GridRegion::square(1, 2);
    id_grid.insert(1, a_region);

    assert_eq!(id_grid.query(a_region).contains(&1), true);

    id_grid.remove(1);
    assert_eq!(id_grid.query(a_region).contains(&1), false);

    id_grid.remove(1);
    assert_eq!(id_grid.query(a_region).contains(&1), false);
}
