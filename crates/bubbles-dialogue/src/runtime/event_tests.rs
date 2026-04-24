use super::line_id_from_tags;

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
