//! Integration tests for trailing #tag metadata on lines and commands.

mod common;

use bubbles::{DialogueEvent, HashMapProvider, HashMapStorage, Runner, VariableStorage, compile};

#[test]
fn line_tags_emitted() {
    let src = "title: Start\n---\nHello. #greeting #important\n===\n";
    let events = common::play(src, "Start");
    let tags = events.iter().find_map(|e| {
        if let DialogueEvent::Line { tags, .. } = e {
            Some(tags.clone())
        } else {
            None
        }
    });
    assert_eq!(
        tags.as_deref(),
        Some(["greeting".to_owned(), "important".to_owned()].as_slice())
    );
}

#[test]
fn line_prefix_tag_uses_provider_for_text() {
    let prog = compile("title: A\n---\nHi. #line:abc\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    let mut provider = HashMapProvider::new();
    provider.insert("abc", "Hola");
    runner.set_provider(provider);
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hola"));
}

#[test]
fn line_id_field_set_from_line_tag() {
    let prog = compile("title: A\n---\nHi. #line:my_key #foo\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    let id = loop {
        let ev = runner.next_event().unwrap();
        if let Some(DialogueEvent::Line { line_id, .. }) = ev {
            break line_id;
        }
    };
    assert_eq!(id.as_deref(), Some("my_key"));
}

#[test]
fn option_line_id_field_set_from_tag() {
    let src = "title: A\n---\n-> Pick this #line:opt1\n  ok\n===\n";
    let mut runner = Runner::new(compile(src).unwrap(), HashMapStorage::new());
    runner.start("A").unwrap();
    let id = loop {
        let ev = runner.next_event().unwrap();
        if let Some(DialogueEvent::Options(opts)) = ev {
            break opts[0].line_id.clone();
        }
    };
    assert_eq!(id.as_deref(), Some("opt1"));
}

#[test]
fn line_prefix_not_in_provider_keeps_interpolated_source() {
    let prog = compile("title: A\n---\nHello. #line:missing\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    runner.set_provider(HashMapProvider::new());
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hello."));
}

// ── translate-then-format ─────────────────────────────────────────────────────

#[test]
fn provider_template_variables_interpolated() {
    // The translated string still contains {$name} — it must be evaluated
    // against current storage AFTER the translation lookup.
    let prog = compile("title: A\n---\nHi {$name}. #line:abc\n===\n").unwrap();
    let mut storage = HashMapStorage::new();
    storage.set("$name", bubbles::Value::Text("Alice".into()));
    let mut runner = Runner::new(prog, storage);
    let mut provider = HashMapProvider::new();
    provider.insert("abc", "Hola {$name}.");
    runner.set_provider(provider);
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hola Alice."));
}

#[test]
fn provider_template_with_plural_builtin() {
    let prog = compile("title: A\n---\nYou have {$n} apples. #line:fruit\n===\n").unwrap();
    let mut storage = HashMapStorage::new();
    storage.set("$n", bubbles::Value::Number(3.0));
    let mut runner = Runner::new(prog, storage);
    let mut provider = HashMapProvider::new();
    // Spanish translation uses plural() to pick the right noun form
    provider.insert("fruit", "Tienes {$n} {plural($n, \"manzana\", \"manzanas\")}.");
    runner.set_provider(provider);
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Tienes 3 manzanas."));
}

#[test]
fn provider_template_no_placeholders_unchanged() {
    // Plain translation (no {expr}) must still work as before.
    let prog = compile("title: A\n---\nHi. #line:abc\n===\n").unwrap();
    let mut runner = Runner::new(prog, HashMapStorage::new());
    let mut provider = HashMapProvider::new();
    provider.insert("abc", "Hola.");
    runner.set_provider(provider);
    runner.start("A").unwrap();
    let mut line = None;
    while let Some(ev) = runner.next_event().unwrap() {
        if let DialogueEvent::Line { text, .. } = ev {
            line = Some(text);
            break;
        }
    }
    assert_eq!(line.as_deref(), Some("Hola."));
}

#[test]
fn command_tags_emitted() {
    let src = "title: Start\n---\n<<shake camera>> #vfx\n===\n";
    let events = common::play(src, "Start");
    let cmd_tags = events.iter().find_map(|e| {
        if let DialogueEvent::Command { tags, .. } = e {
            Some(tags.clone())
        } else {
            None
        }
    });
    assert_eq!(cmd_tags.as_deref(), Some(["vfx".to_owned()].as_slice()));
}
