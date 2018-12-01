use game2d::grid::Grid;
use game2d::grid::GridRegion;

mod test_support;
use crate::test_support::*;

#[test]
fn can_insert_items_into_same_region() {
    let mut id_grid: Grid<i32> = Grid::new();
    let a_region: GridRegion = (2, 1).into();

    id_grid.insert(1, a_region);
    id_grid.insert(2, a_region);

    let results = id_grid.query((2, 1).into());
    assert_set_contains_exactly(results, &[1, 2]);
}

#[test]
fn can_remove_items() {
    let mut id_grid: Grid<i32> = Grid::new();
    let a_region: GridRegion = (5, 10).into();

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
    let large_region: GridRegion = ((1, 2), (10, 10)).into();

    id_grid.insert(1, large_region);

    assert_eq!(id_grid.query((1, 2).into()).contains(&1), true);
    assert_eq!(id_grid.query((5, 7).into()).contains(&1), true);
    assert_eq!(id_grid.query((11, 12).into()).contains(&1), true);

    assert_eq!(id_grid.query((0, 2).into()).contains(&1), false);
    assert_eq!(id_grid.query((1, 1).into()).contains(&1), false);
    assert_eq!(id_grid.query((12, 3).into()).contains(&1), false);
    assert_eq!(id_grid.query((11, 13).into()).contains(&1), false);
}

#[test]
fn item_regions_can_overlap() {
    let mut id_grid: Grid<i32> = Grid::new();
    let region_a: GridRegion = ((1, 2), (3, 0)).into();
    let region_b: GridRegion = ((4, 2), (0, 2)).into();

    id_grid.insert(1, region_a);
    id_grid.insert(2, region_b);

    assert_set_contains_exactly(id_grid.query((1, 2).into()), &[1]);
    assert_set_contains_exactly(id_grid.query((2, 2).into()), &[1]);
    assert_set_contains_exactly(id_grid.query((3, 2).into()), &[1]);
    assert_set_contains_exactly(id_grid.query((4, 2).into()), &[1, 2]);
    assert_set_contains_exactly(id_grid.query((4, 3).into()), &[2]);
    assert_set_contains_exactly(id_grid.query((4, 4).into()), &[2]);
}

#[test]
fn subsequent_calls_to_insert_overwrite_previous() {
    let mut id_grid: Grid<i32> = Grid::new();

    id_grid.insert(1, (1, 2).into());
    id_grid.insert(1, (3, 4).into());

    assert_eq!(id_grid.query((3, 4).into()).contains(&1), true);
    assert_eq!(id_grid.query((1, 2).into()).contains(&1), false);

    id_grid.insert(1, (5, 6).into());

    assert_eq!(id_grid.query((3, 4).into()).contains(&1), false);
    assert_eq!(id_grid.query((5, 6).into()).contains(&1), true);
}

#[test]
fn multiple_calls_to_remove_are_harmless() {
    let mut id_grid: Grid<i32> = Grid::new();

    id_grid.insert(1, (1, 2).into());

    assert_eq!(id_grid.query((1, 2).into()).contains(&1), true);

    id_grid.remove(1);
    assert_eq!(id_grid.query((1, 2).into()).contains(&1), false);

    id_grid.remove(1);
    assert_eq!(id_grid.query((1, 2).into()).contains(&1), false);
}
