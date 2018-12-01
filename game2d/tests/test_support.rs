//! Misc utility methods useful across tests

use std::{collections::HashSet, fmt::Debug, hash::Hash};

pub fn assert_eq_f32(actual: f32, expected: f32, epsilon: f32) {
    if (expected - actual).abs() > epsilon {
        assert!(
            false,
            format!("{} is not within {} of {}", actual, epsilon, expected)
        );
    }
}

/// Assert the following set contains only the target elements
pub fn assert_set_contains_exactly<T>(set: HashSet<&T>, actual: &[T])
where
    T: Debug + Eq + Hash,
{
    for element in actual.iter() {
        assert!(
            set.contains(element),
            format!("{:?} is not in collection", element)
        );
    }
    assert_eq!(
        set.len(),
        actual.len(),
        "Collection has size {} but there are {} expected elements",
        set.len(),
        actual.len()
    );
}
