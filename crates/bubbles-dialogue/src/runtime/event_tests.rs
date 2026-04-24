use super::{LineMode, line_id_from_tags, line_mode_from_tags, option_group_from_tags};

#[test]
fn line_id_from_tags_first_line_prefix() {
    assert_eq!(
        line_id_from_tags(&["foo".into(), "line:abc".into(), "line:ignored".into()]),
        Some("abc".into())
    );
}

#[test]
fn line_id_from_tags_none_without_prefix() {
    assert_eq!(line_id_from_tags(&["foo".into(), "bar".into()]), None);
}

#[test]
fn line_id_from_tags_empty_after_prefix_is_none() {
    assert_eq!(line_id_from_tags(&["line:".into()]), None);
}

#[test]
fn line_mode_from_tags_defaults_to_normal() {
    assert_eq!(line_mode_from_tags(&[]), LineMode::Normal);
    assert_eq!(line_mode_from_tags(&["foo".into()]), LineMode::Normal);
}

#[test]
fn line_mode_from_tags_narration_and_debug() {
    assert_eq!(
        line_mode_from_tags(&["narration".into()]),
        LineMode::Narration
    );
    assert_eq!(line_mode_from_tags(&["debug".into()]), LineMode::Debug);
    assert_eq!(
        line_mode_from_tags(&["narration".into(), "debug".into()]),
        LineMode::Debug
    );
}

#[test]
fn option_group_from_tags_extracts_group() {
    assert_eq!(
        option_group_from_tags(&["group:allies".into()]),
        Some("allies".into())
    );
    assert_eq!(
        option_group_from_tags(&["style".into(), "group:romance".into()]),
        Some("romance".into())
    );
}

#[test]
fn option_group_from_tags_first_match_wins() {
    assert_eq!(
        option_group_from_tags(&["group:first".into(), "group:second".into()]),
        Some("first".into())
    );
}

#[test]
fn option_group_from_tags_missing_returns_none() {
    assert_eq!(
        option_group_from_tags(&["style".into(), "important".into()]),
        None
    );
    assert_eq!(option_group_from_tags(&["group:".into()]), None);
}
