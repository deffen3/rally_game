# Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual 2D game.

For now I am prioritizing the goal of *having fun making a game* higher than the goal of *writing good rust code*. I know some of the code is pretty crap for now, lots of code duplication and poor ECS architecture!

TODO:
1. Create a Hitbox component, and create a hitbox detection function - in progress
1. Improve missile heat-seeking tracking - pretty good now, still has bugs though
1. Improve collision algorithms
1. What happens when you die from collision? Lose a gun-level? If another player recently hit you then they get the kill?
1. Vehicle mass should affect physics
1. Weapon type should affect vehicle mass
1. UI Transform works different on Mac? UI is in the middle of the screen.
1. Red laser weapon can go through opponents on computers with potato frame rate

RECENTLY DONE:
1. Fixed gimballed/turret/auto-aim feature
1. Add simple AI opponents
1. Add current weapon type icon into UI
1. AI sword mode seems to not work very well, bad target tracking when in reverse? - FIXED
1. Put weapon properties into a .ron config file
1. Visible Shield and Armor icons. Shield should be a faint bubble over top vehicle. Armor can be some grey stuff surrounding your vehicles sides.
1. Weapon icon goes transparent when on cooldown


Currently this is a 2D overhead-view vehicle combat game where the goal is to get a kill with all of the weapons. Once you get a kill with a weapon then it will automatically switch to your next weapon.


__Vehicles__ can accelerate, decelerate, and turn with its __Engines__.

__Vehicles__ can also shoot or deploy its __Weapons__.

Each __Vehicle__ has __Shields__, __Armor__, and __Health__. 

__Shields__ will regenerate if you still have some __Shields__ and you haven't been hit in awhile.

__Armor__ is permanently lost. 

__Health__ can be repaired by holding the __Repair__ button. 
After fully repairing all __Health__, the __Shields__ will eventually re-boot if you continue holding __Repair__.
Note that using __Repair__ will disable all use of __Engines__ and __Weapons__.


There are various weapons with various properties, including:
* Lasers
* Bullets
* Heat-Seeking Missiles
* Rockets
* Mines
* Laser Swords
* single-shot, burst-fire, rapid-fire
* fixed aim, gimballed slight auto-aim, full 360deg auto-aim
* shield/armor piercing damage 
    (explosives do some piercing damage to health, regardless of shield/armor)
* shield damage multiplier
    (lasers are more effective against shields)
* armor damage multiplier
    (bullets are more effective against armor)
* health damage multiplier



[Imgur](https://i.imgur.com/GV4P4yT.png)



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
