# rs_pico

Rust embedded workspace for Raspberry Pi Pico (RP2040).

## Workspace Structure

| Crate | Purpose |
|-------|---------|
| `pico-core` | Pure domain types, no HAL deps |
| `pico-setup` | HAL init, USB, clock configuration |
| `pico-app` | Binary target — blinky entry point |

## Build

```bash
cargo build --release -p pico-app
```

## Flash

```bash
cargo run -p pico-app
```

Requires `probe-rs` and a debug probe.

## Memory

- Flash: 2048K
- RAM: 264K
