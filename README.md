# Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual 2D game.

For now I am prioritizing the goal of having fun making a game higher than the goal of writing good rust code. I know some of the code is pretty crap for now, lots of code duplication!

TODO:
1. Create a Hitbox component, and create a hitbox detection function - in progress
1. Improve missile heat-seeking tracking - pretty good now, still has bugs though
1. Improve collision algorithms
1. What happens when you die from collision? Lose a gun-level? If another player recently hit you then they get the kill?
1. Vehicle mass should affect physics
1. Weapon type should affect vehicle mass
1. UI Transform works different on Mac? UI is in the middle of the screen.
1. Red laser weapon can go through opponents on computers with potato frame rate

DONE:
1. Add simple AI opponents - DONE
1. Add current weapon type icon into UI - DONE
1. AI sword mode seems to not work very well, bad target tracking when in reverse? - FIXED DONE
1. Put weapon properties into a .ron config file - DONE
1. Visible Shield and Armor icons. Shield should be a faint bubble over top vehicle. Armor can be some grey stuff surrounding your vehicles sides. - DONE
1. Weapon icon blinks if weapon on cooldown? - DONE

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
