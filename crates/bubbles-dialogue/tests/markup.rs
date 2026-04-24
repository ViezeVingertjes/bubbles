//! Integration tests for inline markup: `[name]…[/name]` and `[name /]`.

mod common;

use bubbles::{
    DialogueEvent, HashMapProvider, HashMapStorage, RunnerBuilder, Value, VariableStorage, compile,
};

use common::{drain, play};

// ── compile-time acceptance ───────────────────────────────────────────────────

#[test]
fn markup_source_compiles_without_error() {
    assert!(compile("title: A\n---\n[wave]Hello[/wave]\n===\n").is_ok());
}

#[test]
fn self_close_tag_compiles() {
    assert!(compile("title: A\n---\n[pause /]\n===\n").is_ok());
}

#[test]
fn markup_with_properties_compiles() {
    assert!(compile("title: A\n---\n[color value=red]Hi[/color]\n===\n").is_ok());
}

#[test]
fn markup_mixed_with_expr_compiles() {
    assert!(compile("title: A\n---\n[b]{$name}[/b]\n===\n").is_ok());
}

#[test]
fn unclosed_markup_bracket_is_a_compile_error() {
    let err = compile("title: A\n---\n[wave\n===\n")
        .unwrap_err()
        .to_string();
    assert!(err.contains("unclosed") || err.contains('['), "got: {err}");
}

// ── runtime spans on DialogueEvent::Line ─────────────────────────────────────

#[test]
fn open_close_tag_produces_span() {
    let src = "title: A\n---\n[wave]Hello[/wave]\n===\n";
    let events = play(src, "A");
    let line = events.iter().find_map(|e| {
        if let DialogueEvent::Line { text, spans, .. } = e {
            Some((text.clone(), spans.clone()))
        } else {
            None
        }
    });
    let (text, spans) = line.expect("no Line event");
    assert_eq!(text, "Hello");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "wave");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].length, 5);
    assert!(spans[0].properties.is_empty());
}

#[test]
fn span_with_surrounding_text() {
    let src = "title: A\n---\n[b]Hello[/b] world\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hello world");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "b");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].length, 5);
}

#[test]
fn self_close_tag_produces_zero_length_span() {
    let src = "title: A\n---\nWait[pause /]here\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Waithere");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "pause");
    assert_eq!(spans[0].start, 4);
    assert_eq!(spans[0].length, 0);
}

#[test]
fn span_with_property() {
    let src = "title: A\n---\n[color value=red]Hi[/color]\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hi");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "color");
    assert_eq!(
        spans[0].properties,
        vec![("value".to_owned(), "red".to_owned())]
    );
}

#[test]
fn span_byte_offsets_track_expr_evaluation() {
    // `[b]{$name}[/b]` – span length equals the rendered length of `$name`
    let src = "title: A\n---\n<<declare $name = \"Alice\">>\n[b]{$name}[/b]\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Alice");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "b");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].length, 5); // "Alice" is 5 bytes
}

#[test]
fn nested_tags_produce_two_spans() {
    let src = "title: A\n---\n[b][i]text[/i][/b]\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "text");
    assert_eq!(spans.len(), 2);
    let b_span = spans.iter().find(|s| s.name == "b").expect("no b span");
    let i_span = spans.iter().find(|s| s.name == "i").expect("no i span");
    assert_eq!(b_span.start, 0);
    assert_eq!(b_span.length, 4);
    assert_eq!(i_span.start, 0);
    assert_eq!(i_span.length, 4);
}

#[test]
fn no_markup_produces_empty_spans() {
    let src = "title: A\n---\nHello world.\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hello world.");
    assert!(spans.is_empty());
}

#[test]
fn non_markup_brackets_stay_in_text() {
    // `[has spaces]` is not markup; it stays verbatim in the text.
    let src = "title: A\n---\n[has spaces]\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "[has spaces]");
    assert!(spans.is_empty());
}

#[test]
fn span_mid_sentence() {
    let src = "title: A\n---\nHello [b]world[/b]!\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hello world!");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "b");
    assert_eq!(spans[0].start, 6); // "Hello " is 6 bytes
    assert_eq!(spans[0].length, 5); // "world" is 5 bytes
}

// ── markup on option text ─────────────────────────────────────────────────────

#[test]
fn markup_on_option_text_produces_span() {
    let src = "title: A\n---\n-> [b]Fight[/b]\n-> Run\n===\n";
    let prog = compile(src).unwrap();
    let mut runner = bubbles::Runner::new(prog, HashMapStorage::new());
    runner.start("A").unwrap();
    // Drive the runner until we get an Options event, then capture and select.
    let opts = loop {
        match runner.next_event().unwrap() {
            Some(DialogueEvent::Options(opts)) => break opts,
            Some(_) => {}
            None => panic!("dialogue ended without Options event"),
        }
    };
    runner.select_option(0).unwrap();
    // Drain the remainder so the runner finishes cleanly.
    while runner.next_event().unwrap().is_some() {}

    let opt = &opts[0];
    assert_eq!(opt.text, "Fight");
    assert_eq!(opt.spans.len(), 1);
    assert_eq!(opt.spans[0].name, "b");
}

// ── markup in localised templates ────────────────────────────────────────────

#[test]
fn markup_in_provider_template_produces_span() {
    let src = "title: A\n---\nHello. #line:greeting\n===\n";
    let prog = compile(src).unwrap();
    let mut provider = HashMapProvider::new();
    // Translator wraps a word in markup
    provider.insert("greeting", "[wave]Hola[/wave]!");
    let mut runner = RunnerBuilder::new(prog, HashMapStorage::new())
        .with_provider(provider)
        .build();
    runner.start("A").unwrap();
    let events = drain(&mut runner);
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hola!");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "wave");
    assert_eq!(spans[0].start, 0);
    assert_eq!(spans[0].length, 4);
}

#[test]
fn markup_in_provider_template_combined_with_expr() {
    let src = "title: A\n---\nHi. #line:greet\n===\n";
    let prog = compile(src).unwrap();
    let mut provider = HashMapProvider::new();
    provider.insert("greet", "[b]{$name}[/b]");
    let mut storage = HashMapStorage::new();
    storage.set("$name", Value::Text("Alice".into()));
    let mut runner = RunnerBuilder::new(prog, storage)
        .with_provider(provider)
        .build();
    runner.start("A").unwrap();
    let events = drain(&mut runner);
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Alice");
    assert_eq!(spans[0].name, "b");
    assert_eq!(spans[0].length, 5);
}

// ── line group variants carry spans ──────────────────────────────────────────

#[test]
fn markup_on_line_group_variant_produces_span() {
    let src = "title: A\n---\n=> [wave]Hi[/wave]\n===\n";
    let events = play(src, "A");
    let (text, spans) = first_line(&events);
    assert_eq!(text, "Hi");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "wave");
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn first_line(events: &[DialogueEvent]) -> (String, Vec<bubbles::MarkupSpan>) {
    events
        .iter()
        .find_map(|e| {
            if let DialogueEvent::Line { text, spans, .. } = e {
                Some((text.clone(), spans.clone()))
            } else {
                None
            }
        })
        .expect("no Line event")
}
