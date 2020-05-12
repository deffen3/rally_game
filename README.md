# Rocket Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual game.

For now I am prioritizing the goal of *having fun making a game* higher than the goal of *writing good rust code*. I know some of the code is pretty crap for now, lots of code duplication and poor ECS architecture!

This is a 2D overhead-view vehicle combat game with a few different game modes.


![Imgur](https://i.imgur.com/nNXtVhu.png)
---

## Vehicles

__Vehicles__ can accelerate, decelerate, and turn with its __Engines__.

__Vehicles__ can also shoot or deploy its __Weapons__.

Each __Vehicle__ has __Shields__, __Armor__, and __Health__. 

__Shields__ will regenerate if you still have some __Shields__ left, and you haven't been hit in awhile.

__Armor__ is permanently lost. Armor will only be removed after your __Shields__ are gone.

__Health__ is lost after you at hit with no __Shields__ and no __Armor__. __Health__ is also lost if hit by a piercing damage weapon. __Health__ can be repaired by holding the __Repair__ button. 
After fully repairing all __Health__, the __Shields__ will eventually re-boot if you continue holding __Repair__.
Note that using __Repair__ will disable all use of __Engines__ and __Weapons__.

---

## Weapons

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
---

## Game Modes
- __Classic Gun Game__: First to get a kill with each weapon wins. Weapons are hot-swapped after each kill.
- __Deathmatch: Kills__: First to a certain number of kills wins. New weapons can be picked up from arena.
- __Deathmatch: Stock__: If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
- __Deathmatch: Timed KD__: Match ends after set time. Highest score of Kills minus Deaths is the winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
- __King of the Hill__: Players gains points for being the only person in the special "hill" zone. First player to a certain number of points wins. New weapons can be picked up from arena.
- __Combat Race__: It's a race with weapons active. First player to complete the required number of laps wins. New weapons can be picked up from race track.
---


## TODO:
1. Create a Hitbox component, and create a hitbox detection function - in progress
1. Improve missile heat-seeking tracking - pretty good now, still has bugs though
1. Improve collision algorithms
1. UI Transform works different on Mac? UI is in the middle of the screen.
1. Red laser weapon can go through opponents on computers with potato frame rate
1. Load race tracks from config file
1. Better weapon dps/skill balance
1. New bot modes to comprehend Racing, King of the Hill, repairing, and picking up new weapon boxes


---


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
