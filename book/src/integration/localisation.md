# Localisation

Bubbles handles localisation with one trait — [`LineProvider`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.LineProvider.html) — and one convention: tag your lines with `#line:<id>`.

## The flow

1. You author in a source language (let's say English).
2. Every line that needs translating gets a `#line:some_id` tag.
3. At runtime, you install a `LineProvider` that maps ids to translated templates.
4. When Bubbles processes a tagged line, it asks the provider first. If the provider returns a string, that string is used — and its own `{…}` expressions are evaluated against the current variables.

This is the "translate then format" pattern, and it means translators can reorder interpolations however their language demands.

## Tagging your source

```text
Aria: Evening, friend. #line:aria_greet_01
Aria: You have {$gold} gold. #line:aria_gold_report
```

`#line:aria_greet_01` is the stable id. Don't change it after translation starts — those are the keys translators rely on.

## The simplest provider

Bubbles ships [`HashMapProvider`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.HashMapProvider.html) for quick experiments:

```rust,ignore
use bubbles::HashMapProvider;

let mut provider = HashMapProvider::new();
provider.insert("aria_greet_01", "¡Buenas tardes, amigo!");
provider.insert("aria_gold_report", "Tienes {$gold} monedas de oro.");

runner.set_provider(provider);
```

That's it. When Bubbles emits `#line:aria_greet_01`, the Spanish template wins. `{$gold}` in the template is evaluated after the lookup, so the count still comes from the runner's storage.

## Loading from files

Most games store translations externally — JSON, YAML, CSV, a `.po` file, whatever. Wrap your loader in a `LineProvider`:

```rust,ignore
use bubbles::LineProvider;
use std::collections::HashMap;

pub struct JsonProvider {
    translations: HashMap<String, String>,
}

impl JsonProvider {
    pub fn load(path: &str) -> std::io::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        Ok(Self {
            translations: serde_json::from_str(&data).unwrap(),
        })
    }
}

impl LineProvider for JsonProvider {
    fn get(&self, line_id: &str) -> Option<String> {
        self.translations.get(line_id).cloned()
    }
}

runner.set_provider(JsonProvider::load("locales/es.json")?);
```

Swap the provider whenever the player changes language. The dialogue continues; the next tagged line comes back in the new tongue.

## Plural and gendered forms

Translation isn't just replacing words. Bubbles has two built-in functions that help:

- **`plural(n, singular, plural)`** — picks the singular form when `|n| == 1`, the plural form otherwise.
- **`select(key, "k1:text|k2:text|other:fallback")`** — picks by key, with an `other:` fallback.

```text
# English source
You found {$n} {plural($n, "gem", "gems")}.

# Spanish template (in the provider)
Encontraste {$n} {plural($n, "gema", "gemas")}.

# German template
Du hast {$n} {plural($n, "Edelstein", "Edelsteine")} gefunden.
```

Gendered pronouns:

```text
{select($gender, "m:He|f:She|n:They|other:They")} arrived at the tavern.
```

Translators can reshape these expressions for their language — some languages have more than two plural forms, or different gender structures. Because the expression is evaluated *inside* the template, they have full control.

## Falling back gracefully

If your provider returns `None` for an id, Bubbles uses the source text from the script. You won't get a crash — just untranslated text — which makes shipping partial translations painless.

> **Tip:** Start every translation pass with a "missing keys" report. Run the dialogue in your target language and log every `line_id` the provider returns `None` for. Simple, and catches every new line added to the source.

## Lines without `#line:` ids

Only lines with a `#line:` tag are routed through the provider. Lines without an id always use their source text — so you can leave narrator description, debug lines, or developer-only content untranslated without any fuss.

## Keeping ids stable

Two rules to save your future self:

1. **Never reuse an id.** Once a translator has worked on it, the id belongs to that line forever. If the line changes meaning substantially, give it a new id.
2. **Never change an id casually.** Renaming `greeting_01` to `greet_01` invalidates every translation in flight.

Keep the ids short but descriptive — `scene_speaker_variant` is a pattern that scales (`tavern_barkeep_greet_first`, `tavern_barkeep_greet_repeat`).

---

> **Next:** [Custom Functions](./functions.md)
