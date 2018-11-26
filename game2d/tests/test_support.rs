//! Misc utility methods useful across tests

pub fn assert_eq_f32(actual: f32, expected: f32, epsilon: f32) {
    if (expected - actual).abs() > epsilon {
        assert!(
            false,
            format!("{} is not within {} of {}", actual, epsilon, expected)
        );
    }
}
