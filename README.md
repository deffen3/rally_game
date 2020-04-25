# Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual 2D game.

For now I am prioritizing the goal of having fun making a game higher than the goal of writing good rust code. I know the code is pretty crap for now, lots of code duplication!

TODO:
1. Create a Hitbox component, and create a hitbox detection function
1. Improve missile heat-seeking tracking
1. Improve collision algorithms
1. Add current weapon type icon into UI

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
