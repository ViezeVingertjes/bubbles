//! `bubbles check` tests.  Nothing here touches the `tui` feature, so they
//! also run in the `--no-default-features` build that CI users install.

use bubbles_cli::{SourceSet, check};

const GREETING: &str = "title: Start\n---\nNPC: Hello.\n===\n";

#[test]
fn a_valid_script_reports_no_diagnostics() {
    let report = check::check(&SourceSet::single("a.bub", GREETING));
    assert!(report.is_ok());
    assert_eq!(report.files, 1);
    assert_eq!(report.nodes, 1);
}

#[test]
fn the_success_summary_counts_files_and_nodes() {
    let report = check::check(&SourceSet::single("a.bub", GREETING));
    assert_eq!(report.summary(), "ok: 1 file, 1 node");

    let two = SourceSet::many([
        ("a.bub", GREETING),
        ("b.bub", "title: Other\n---\nNPC: Hi.\n===\n"),
    ]);
    assert_eq!(check::check(&two).summary(), "ok: 2 files, 2 nodes");
}

#[test]
fn a_parse_error_is_one_line_with_a_file_and_line_number() {
    let report = check::check(&SourceSet::single(
        "bad.bub",
        "title: Start\n---\n<<if>>\n===\n",
    ));
    assert!(!report.is_ok());
    assert_eq!(report.diagnostics.len(), 1);

    let line = report.diagnostics[0].one_line();
    assert!(!line.contains('\n'), "diagnostics must be one line: {line}");
    assert!(
        line.starts_with("bad.bub:3: "),
        "unexpected location: {line}"
    );
}

#[test]
fn a_jump_to_a_missing_node_is_reported() {
    let source = "title: Start\n---\n<<jump Nowhere>>\n===\n";
    let report = check::check(&SourceSet::single("a.bub", source));
    assert!(!report.is_ok());

    let line = report.diagnostics[0].one_line();
    assert!(line.contains("Validation error"), "unexpected text: {line}");
    assert!(line.contains("Nowhere"), "unexpected text: {line}");
}

#[test]
fn files_are_checked_as_one_programme_so_cross_file_jumps_resolve() {
    let main = "title: Start\n---\n<<jump Shop>>\n===\n";
    let shop = "title: Shop\n---\nNPC: Welcome.\n===\n";

    // Together: the jump resolves.
    let both = SourceSet::many([("main.bub", main), ("shop.bub", shop)]);
    assert!(check::check(&both).is_ok());

    // Alone: it does not. Checking file-by-file would report this.
    let alone = SourceSet::single("main.bub", main);
    assert!(!check::check(&alone).is_ok());
}

#[test]
fn an_empty_file_is_a_failure_rather_than_an_empty_pass() {
    let report = check::check(&SourceSet::single("empty.bub", "\n"));
    assert!(!report.is_ok());
    assert_eq!(report.nodes, 0);
}
