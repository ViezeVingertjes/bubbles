use crate::compiler::compile;
use crate::error::DialogueError;

// compile() now always validates - these tests assert that broken targets
// are caught at compile time with a Validation error.

#[test]
fn valid_jump_passes() {
    assert!(compile("title: A\n---\n<<jump B>>\n===\ntitle: B\n---\n===\n").is_ok());
}

#[test]
fn unknown_jump_target_fails() {
    let err = compile("title: A\n---\n<<jump Nonexistent>>\n===\n").unwrap_err();
    assert!(
        matches!(err, DialogueError::Validation(_)),
        "expected Validation, got: {err:?}"
    );
}

#[test]
fn unknown_jump_inside_if_branch_fails() {
    let err =
        compile("title: A\n---\n<<if true>>\n    <<jump Ghost>>\n<<endif>>\n===\n").unwrap_err();
    assert!(matches!(err, DialogueError::Validation(_)));
}

#[test]
fn unknown_detour_in_else_fails() {
    let err = compile(
        "title: A\n---\n<<if false>>\n    idle\n<<else>>\n    <<detour Missing>>\n<<endif>>\n===\n",
    )
    .unwrap_err();
    assert!(matches!(err, DialogueError::Validation(_)));
}

#[test]
fn unknown_jump_inside_once_fails() {
    let err =
        compile("title: A\n---\n<<once>>\n    <<jump Nope>>\n<<endonce>>\n===\n").unwrap_err();
    assert!(matches!(err, DialogueError::Validation(_)));
}

#[test]
fn unknown_jump_inside_option_body_fails() {
    let err = compile("title: A\n---\n-> Go\n    <<jump Bad>>\n===\n").unwrap_err();
    assert!(matches!(err, DialogueError::Validation(_)));
}

#[test]
fn unknown_jump_inside_once_else_fails() {
    let err =
        compile("title: A\n---\n<<once>>\n    ok\n<<else>>\n    <<jump Nope>>\n<<endonce>>\n===\n")
            .unwrap_err();
    assert!(matches!(err, DialogueError::Validation(_)));
}
