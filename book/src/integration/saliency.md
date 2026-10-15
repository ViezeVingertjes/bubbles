# Saliency Strategies

A **saliency strategy** is the rule Bubbles uses to pick one item from a list of eligible options. It fires whenever a [line group](../language/line-groups.md) or [node group](../language/node-groups.md) needs to choose.

Bubbles ships three. You can also write your own.

## `FirstAvailable` (default)

```rust,ignore
use bubbles::saliency::FirstAvailable;

runner.set_saliency(FirstAvailable);
```

Picks the first eligible candidate in declaration order. Fully deterministic, zero state, always safe.

Use this when:

- You want predictable behaviour for tests and replays.
- Your groups are ranked by priority (most specific first, fallback last).

## `RandomAvailable` (requires `rand` feature)

```rust,ignore
use bubbles::saliency::RandomAvailable;

runner.set_saliency(RandomAvailable);
```

Picks uniformly at random from every eligible candidate. Great for flavour lines where anything goes.

It draws from the runner's own generator, so a session built with [`with_seed`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.RunnerBuilder.html#method.with_seed) makes the same choices every run, and a snapshot restores mid-stream.

Use this when:

- Order doesn't matter and variety does.
- You don't mind hearing the same line twice in a row occasionally.

## `BestLeastRecentlyViewed` (BLRV)

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;

runner.set_saliency(BestLeastRecentlyViewed::new());
```

Prefers the candidate you've seen least recently. Over time, every candidate comes up before any repeats. This is the strategy that makes NPCs feel *alive*.

Use this when:

- You have 3–10 variants and want each one to feel fresh.
- Repetition would break immersion.
- You care about variety more than randomness.

> **Tip:** BLRV is usually the right default for ambient barks. Set it on the runner and forget it. The visit tracking lives inside the strategy; it persists for the lifetime of the runner.

## Writing your own strategy

Implement the [`SaliencyStrategy`](https://docs.rs/bubbles-dialogue/latest/bubbles/trait.SaliencyStrategy.html) trait. You get a slice of [`Candidate`](https://docs.rs/bubbles-dialogue/latest/bubbles/struct.Candidate.html)s - each with an `id` and an `available` flag - plus the runner's generator; return the index of the one you want.

```rust,ignore
use bubbles::{Candidate, DialogueRng, SaliencyStrategy};

/// A strategy that prefers candidates whose id ends in `_urgent`.
pub struct UrgentFirst;

impl SaliencyStrategy for UrgentFirst {
    fn select(&mut self, candidates: &[Candidate<'_>], _rng: &DialogueRng) -> Option<usize> {
        let urgent = candidates
            .iter()
            .position(|c| c.available && c.id.ends_with("_urgent"));
        urgent.or_else(|| candidates.iter().position(|c| c.available))
    }
}

runner.set_saliency(UrgentFirst);
```

**If your strategy chooses randomly, draw from `rng`, not from a thread-local RNG.** That is the whole reason it is handed to you: `rng.choose_index(len)` keeps your strategy inside the seeded, snapshot-able stream, so a replay stays a replay. Deterministic strategies ignore the argument.

```rust,ignore
fn select(&mut self, candidates: &[Candidate<'_>], rng: &DialogueRng) -> Option<usize> {
    let available: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, c)| c.available)
        .map(|(i, _)| i)
        .collect();
    rng.choose_index(available.len()).map(|n| available[n])
}
```

That's the whole surface. Ideas worth trying:

- **Weighted random** - each variant carries a weight tag like `#weight:5`.
- **Mood-aware** - boost candidates whose tags match the current scene's mood.
- **Player-preferring** - prefer ones tagged `#player_class_<x>` when the player is that class.
- **Exhaustive** - walk every candidate in order, then loop.

The strategy is called synchronously during `next_event`. Keep it fast - no network calls, no disk reads.

## Strategy scope

The strategy on the runner applies to *all* groups: line groups, node groups, and anything that uses the saliency machinery. If you need per-scene variation (e.g. "BLRV for barks, random for greetings"), the cleanest path is writing a dispatching strategy that reads tags off the candidate and picks a behaviour accordingly.

## A complete setup

Most games want something like this:

```rust,ignore
use bubbles::saliency::BestLeastRecentlyViewed;

let mut runner = Runner::new(program, HashMapStorage::new());
runner.set_saliency(BestLeastRecentlyViewed::new());
```

Two lines, every line and node group now picks fresh variants. Writers author `=>` lines freely; players hear the world talk without ever clocking the repetition.

---

> **Next:** [Save and Load](../advanced/save-load.md)
