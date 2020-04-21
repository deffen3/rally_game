# Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual 2D game.

## How to run

To run the game, run the following command, which defaults to the `vulkan` graphics backend:

```bash
cargo run
```

Windows and Linux users may explicitly choose `"vulkan"` with the following command:

```bash
cargo run --no-default-features --features "vulkan"
```

Mac OS X users may explicitly choose `"metal"` with the following command:

```bash
cargo run --no-default-features --features "metal"
```
