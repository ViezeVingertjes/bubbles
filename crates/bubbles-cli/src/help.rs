//! The key list behind `?`.
//!
//! The footer only has room for a handful of hints, so this is the complete
//! reference and the single place the bindings are written down for the user.
//! Keep it in step with `terminal::key_to_intent`.

/// Every binding the player exposes, in the order the key list shows them.
///
/// Each entry is `(keys, what it does)`.
pub const KEYS: &[(&str, &str)] = &[
    ("Enter / Space", "Advance, or confirm the focused option"),
    (
        "\u{2191} / k",
        "Previous option, or scroll the transcript back",
    ),
    (
        "\u{2193} / j",
        "Next option, or scroll the transcript forward",
    ),
    ("1 - 9", "Pick an option by number"),
    ("Tab", "Swap focus between the options and the transcript"),
    ("PageUp / PageDown", "Scroll the transcript"),
    ("b / Backspace", "Step back one event"),
    (
        "r",
        "Rerun from the start node, keeping variables and <<once>> history",
    ),
    (
        "Ctrl+R / F5",
        "Reload from disk - DESTRUCTIVE, discards every variable",
    ),
    ("x", "Dismiss the error overlay"),
    ("?", "Show or hide this list"),
    ("q / Esc", "Quit, or close this list when it is open"),
];
