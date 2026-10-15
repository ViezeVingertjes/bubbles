# Examples

These scripts demonstrate Bubbles dialogue features and host integrations.

| File | Demonstrates | Run from the repository root |
|---|---|---|
| [`c-host/tavern.bub`](c-host/tavern.bub) | Dialogue events consumed by the C ABI host, including commands and guarded options. | `cd examples/c-host && make run` |
| [`harbour/harbour.bub`](harbour/harbour.bub) | A branching harbour conversation with guarded choices, one-off content, interpolation, commands, detours, and jumps. | `cargo run -p bubbles-cli -- play examples/harbour/harbour.bub examples/harbour/services.bub` |
| [`harbour/services.bub`](harbour/services.bub) | A reusable `MapSeller` detour node with conditional branches, compiled with the harbour scene. | `cargo run -p bubbles-cli -- play examples/harbour/harbour.bub examples/harbour/services.bub` |
| [`snippets/commands.bub`](snippets/commands.bub) | Host-side command events with arguments, line tags, and tagged lines. | `cargo run -p bubbles-cli -- play examples/snippets/commands.bub` |
| [`snippets/markup.bub`](snippets/markup.bub) | Inline bold, italic, underline, dim, colour, and pause markup. | `cargo run -p bubbles-cli -- play examples/snippets/markup.bub` |
| [`snippets/once.bub`](snippets/once.bub) | `<<once>>` blocks that show alternate content on subsequent visits. | `cargo run -p bubbles-cli -- play examples/snippets/once.bub` |
| [`snippets/options.bub`](snippets/options.bub) | Dialogue options, once-options, guarded choices, conditionals, and jumps. | `cargo run -p bubbles-cli -- play examples/snippets/options.bub` |
| [`snippets/saliency.bub`](snippets/saliency.bub) | Salient line groups and conditionally selected storyteller nodes. | `cargo run -p bubbles-cli -- play examples/snippets/saliency.bub` |
| [`snippets/variables.bub`](snippets/variables.bub) | Declared variables, interpolation, and variable updates. | `cargo run -p bubbles-cli -- play examples/snippets/variables.bub` |

The harbour example uses two dialogue files. The C host example has additional
build instructions in [`c-host/README.md`](c-host/README.md). Any script here can
also be compiled without playing it, which is what `check` is for:

```sh
cargo run -p bubbles-cli -- check examples/snippets/options.bub
```

For installing the `bubbles` binary and the other project commands, see the main
[`README.md`](../README.md).
