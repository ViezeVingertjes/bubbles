//! Dialogue sources used by the benchmarks.
//!
//! `SMALL` and the generators below describe their inputs by shape (node count,
//! lines per node, branching) rather than by a checked-in file that could be
//! edited for unrelated reasons and silently move the numbers. The `harbour`
//! sources are read from `examples/`, which is the closest thing in the
//! repository to a real script a game would ship.
//!
//! Each `benches/*.rs` file is its own binary and sees this whole module while
//! using only a subset of it, so unused items here are not dead code across the
//! suite as a whole.

#![allow(dead_code)]

use std::fmt::Write as _;

/// The smallest script that still exercises start, a line, and node completion.
pub const SMALL: &str = "\
title: Greet
---
Innkeeper: Evening. You look like you walked a long road.
===
";

/// Builds a script with `nodes` nodes of `lines_per_node` plain lines each,
/// chained so that every node jumps to the next.
///
/// `prefix` is prepended to every node title so several generated scripts can be
/// compiled together by `compile_many`, which requires titles to be unique across
/// all of its inputs.
pub fn chain(prefix: &str, nodes: usize, lines_per_node: usize) -> String {
    let mut src = String::new();
    for n in 0..nodes {
        let _ = writeln!(src, "title: {prefix}N{n}\n---");
        for l in 0..lines_per_node {
            let _ = writeln!(src, "Speaker: Line {l} of node {n}, plain text.");
        }
        if n + 1 < nodes {
            let _ = writeln!(src, "<<jump {prefix}N{}>>", n + 1);
        }
        src.push_str("===\n");
    }
    src
}

/// A medium script: 40 nodes of 10 lines each, every node also running a
/// conditional and a `<<set>>`, so the measured path is not purely line emission.
pub fn medium() -> String {
    let mut src =
        String::from("title: Entry\n---\n<<declare $gold = 10>>\n<<declare $trust = 0>>\n");
    src.push_str("<<jump N0>>\n===\n");
    for n in 0..40 {
        let _ = writeln!(src, "title: N{n}\n---");
        for l in 0..10 {
            let _ = writeln!(src, "Speaker: Line {l} of node {n}.");
        }
        src.push_str(
            "<<if $gold > 5>>\n    You can afford it.\n<<else>>\n    Too dear.\n<<endif>>\n",
        );
        src.push_str("<<set $trust = $trust + 1>>\n");
        if n + 1 < 40 {
            let _ = writeln!(src, "<<jump N{}>>", n + 1);
        }
        src.push_str("===\n");
    }
    src
}

/// A script whose every line interpolates variables, for the interpolation path.
///
/// Mixes a text variable, a number variable, and an arithmetic expression,
/// because those are three different costs and a single `{$var}` would hide two
/// of them.
pub fn interpolation(lines: usize) -> String {
    let mut src = String::from("title: Interp\n---\n");
    src.push_str("<<declare $name = \"Aurelia the Navigator\">>\n");
    src.push_str("<<declare $gold = 42>>\n");
    src.push_str("<<declare $port = \"Tide's End\">>\n");
    for _ in 0..lines {
        src.push_str(
            "Clerk: {$name}, you hold {$gold} gold at {$port}, or {$gold * 2} by spring.\n",
        );
    }
    src.push_str("===\n");
    src
}

/// A script whose every line carries inline markup, for the markup path.
pub fn markup(lines: usize) -> String {
    let mut src = String::from("title: Markup\n---\n");
    for _ in 0..lines {
        src.push_str(
            "Crier: [wave]Hear ye[/wave], the [b]tide[/b] turns at [i]dawn[/i].[pause /]\n",
        );
    }
    src.push_str("===\n");
    src
}

/// A script that presents four options per round across `rounds` rounds, for the
/// option-presentation and `select_option` path.
pub fn options(rounds: usize) -> String {
    let mut src = String::new();
    for n in 0..rounds {
        let _ = writeln!(src, "title: O{n}\n---\nPick one.");
        for choice in 0..4 {
            let _ = writeln!(src, "-> Choice {choice} in round {n}");
            if n + 1 < rounds {
                let _ = writeln!(src, "    <<jump O{}>>", n + 1);
            }
        }
        src.push_str("===\n");
    }
    src
}

/// The `examples/harbour/` sources, as `(name, text)` pairs for `compile_many`.
///
/// Embedded at compile time so no benchmark does file IO inside a timed region
/// or depends on the directory it was launched from.
pub const HARBOUR: [(&str, &str); 2] = [
    (
        "harbour.bub",
        include_str!("../../../../examples/harbour/harbour.bub"),
    ),
    (
        "services.bub",
        include_str!("../../../../examples/harbour/services.bub"),
    ),
];
