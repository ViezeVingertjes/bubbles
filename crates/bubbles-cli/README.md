# bubbles-cli

Command-line front end for the [`bubbles-dialogue`](https://crates.io/crates/bubbles-dialogue)
runtime. Two verbs: `check` compiles `.bub` scripts and reports what is wrong with them,
`play` walks through one in a terminal.

## Check a script

```sh
cargo install bubbles-cli --no-default-features

bubbles check dialogue/*.bub
```

This crate is not on the index yet. Until the next release publishes it, install from git —
`cargo install --git https://github.com/ViezeVingertjes/bubbles bubbles-cli`, with the same
flags wherever this page says `cargo install bubbles-cli`.

`check` compiles every file you give it as one programme — node titles merge and
`<<jump>>`/`<<detour>>` targets resolve across files, the same way they do at runtime —
then prints one diagnostic per line and exits non-zero if anything failed:

```text
dialogue/tavern.bub:42: Parse error: in <<if>> ``: unexpected end of expression
Validation error: node 'Options' references unknown node 'MapSeller'
```

Diagnostics go to stdout, so you can pipe them. `--quiet` drops the `ok: 3 files, 17 nodes`
summary and leaves only diagnostics, which is usually what you want in CI:

```yaml
- run: cargo install bubbles-cli --no-default-features
- run: bubbles check --quiet dialogue/*.bub
```

`--no-default-features` is worth typing. It drops the terminal UI and leaves a binary that
adds nothing to the library's own three direct dependencies, which is the difference
between a check step people add and one they skip.

## Play a script

```sh
cargo install bubbles-cli

bubbles play dialogue/tavern.bub           # starts at "Start"
bubbles play dialogue/*.bub TownSquare     # explicit start node
bubbles dialogue/tavern.bub                # same as `play`
```

The player runs the same `Runner` a game would, so the transcript shows exactly what a
real integration receives: node markers, lines, options, commands, and runtime errors.
Press `?` for the key list. `Ctrl+R` reloads from disk and discards every variable; `r`
reruns from the start node and keeps them, which is how you reach second-visit lines.

## More

The [CLI chapter](https://viezevingertjes.github.io/bubbles/getting-started/cli.html) of
the guide covers both verbs in full. If you are embedding dialogue in a Rust or
WebAssembly program, you want [`bubbles-dialogue`](https://crates.io/crates/bubbles-dialogue);
if your host is not Rust, [`bubbles-ffi`](https://github.com/ViezeVingertjes/bubbles/tree/main/crates/bubbles-ffi)
is a C ABI wrapper with [prebuilt binaries](https://github.com/ViezeVingertjes/bubbles/releases)
for Windows, macOS, and Linux.
