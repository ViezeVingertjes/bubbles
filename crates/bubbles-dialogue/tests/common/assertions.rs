//! Assertion helpers for readable event comparisons.

/// Asserts that the actual event slice matches a pattern slice.
///
/// # Panics
/// Panics with a diff if lengths or contents differ.
#[macro_export]
macro_rules! assert_events {
    ($actual:expr, [$($pat:expr),* $(,)?]) => {{
        let expected: &[bubbles::DialogueEvent] = &[$($pat),*];
        let actual: &[bubbles::DialogueEvent] = &$actual;
        if actual != expected {
            panic!(
                "event mismatch\n  expected: {expected:#?}\n  actual:   {actual:#?}"
            );
        }
    }};
}
