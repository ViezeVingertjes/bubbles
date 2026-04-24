# Markup

Markup lets you annotate a range of text inside a line: a word, a phrase, an entire sentence, without changing what the text says. The tags are stripped before the event reaches your game. What you get instead is a `spans` list that tells you exactly where each annotation starts and how long it is.

```text
Alice: [wave]Hello[/wave] there!
```

The `text` field arrives as `"Hello there!"`. The `spans` field carries one entry: `{ name: "wave", start: 0, length: 5 }`. Your rendering code decides what "wave" means.

## Syntax

**Open and close:**
```text
[wave]Hello[/wave]
[b]bold phrase[/b]
```

**Self-closing** (zero-length span, good for events and triggers):
```text
Wait[pause /]here.
```

**With properties:**
```text
[color value=red]Danger![/color]
[sfx name=coin_drop /]
```

Properties are `key=value` pairs, space-separated inside the tag. Keys must be identifiers. Values are unquoted words.

## What the event looks like

```rust,ignore
DialogueEvent::Line { text, spans, .. } => {
    // text  = "Hello there!"
    // spans = [MarkupSpan { name: "wave", start: 0, length: 5, properties: [] }]
    for span in &spans {
        match span.name.as_str() {
            "wave"  => renderer.start_wave(span.start, span.length),
            "shake" => renderer.start_shake(span.start, span.length),
            _       => {}
        }
    }
}
```

`MarkupSpan` fields:

| Field | Type | Meaning |
|---|---|---|
| `name` | `String` | Tag name, e.g. `wave` |
| `start` | `usize` | Byte offset into `text` where the span begins |
| `length` | `usize` | Byte length of the spanned text. Zero for self-closing tags. |
| `properties` | `Vec<(String, String)>` | Key-value pairs from the tag, in order. |

Spans are in source order. Nested tags produce multiple spans at overlapping ranges; your renderer handles layering.

## Combined with interpolation

Markup and `{expr}` work together. Byte offsets are computed after expression evaluation, so the span always points at the right characters:

```text
<<declare $name = "Alice">>
[b]{$name}[/b] found the key.
```

Text: `"Alice found the key."`, span: `{ name: "b", start: 0, length: 5 }`. Five bytes because `"Alice"` is five characters.

## On option text

Markup works on option text the same way. The `spans` field on each `DialogueOption` follows the same rules:

```text
-> [b]Fight[/b]
-> Run
```

```rust,ignore
DialogueEvent::Options(opts) => {
    for opt in &opts {
        render_option(&opt.text, &opt.spans);
    }
}
```

## Localisation

Translators can place markup in their translated strings. Write the translation with tags wherever the visual treatment belongs in that language, and the spans come back with correct byte offsets for the translated text:

```text
# English source
[b]Warning![/b] The bridge is out.

# French translation (in your LineProvider)
[b]Attention![/b] Le pont est coupé.
```

`{expr}` interpolations inside translated strings work the same way: translate first, then evaluate, then compute spans.

## Brackets that are not markup

If the text inside `[...]` doesn't match the markup pattern (starts with a digit, contains spaces, no valid identifier name), the brackets are left in the text verbatim:

```text
The answer is [42].
[sic] as written in the original.
```

Both arrive with the brackets intact and an empty `spans` list.
