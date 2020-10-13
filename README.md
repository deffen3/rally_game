# Rocket Rally!

This is my first attempt at using rust, my first attempt to use an Entity-Component-System, and my first attempt to make an actual game.

For now I am prioritizing the goal of *having fun making a game* higher than the goal of *writing good rust code*. I know some of the code is pretty crap for now, lots of code duplication and poor ECS architecture!

This is a 2D overhead-view vehicle combat game with a few different game modes.


![Imgur](https://i.imgur.com/nNXtVhu.png)

---

## Game Modes
- __Classic Gun Game__: First to get a kill with each weapon wins. Weapons are hot-swapped after each kill.
- __Deathmatch: Kills__: First to a certain number of kills wins. New weapons can be picked up from arena.
- __Deathmatch: Stock__: If you run out of lives you are out. Last player alive wins. New weapons can be picked up from arena.
- __Deathmatch: Timed KD__: Match ends after set time. Highest score of Kills minus Deaths is the winner. Self-destructs are minus 2 deaths. New weapons can be picked up from arena.
- __King of the Hill__: Players gains points for being the only person in the special "hill" zone. First player to a certain number of points wins. New weapons can be picked up from arena.
- __Combat Race__: It's a race with weapons active. First player to complete the required number of laps wins. New weapons can be picked up from race track.

![Imgur](https://i.imgur.com/bwNjzz2.png)

---


## Vehicles

__Vehicles__ can accelerate, decelerate, and turn with their __Engines__.

__Vehicles__ can also shoot or deploy their __Weapons__.

Each __Vehicle__ has __Shields__, __Armor__, and __Health__. 

__Shields__ will regenerate if you still have some __Shields__ left and you haven't been hit within the last few seconds.

__Armor__ is permanently lost. Armor will start to be removed after you are hit with your __Shields__ gone.

__Health__ is lost once you are hit with no __Shields__ and no __Armor__. __Health__ will also be lost if hit by a piercing damage weapon. __Health__ can be repaired by holding the __Repair__ button. 
After fully repairing all __Health__, the __Shields__ will eventually re-boot if you continue holding __Repair__.
Note that using __Repair__ will disable all use of __Engines__ and __Weapons__. If __Health__ goes below 50%, expect the
__Engines__ to start malfunctioning as the vehicle is sparking.

---

## Controls

Player 1:
- Accel/Decel Vehicle: W/S
- Turn Vehicle Left/Right: A/D
- Strafe Vehicle Left/Right: Q/E
- Use Primary Weapon: Spacebar
- Use Secondary Weapon: Left Alt
- Repair Vehicle: R

Player 2:
- Accel/Decel Vehicle: NumPad-Up (8) / NumPad (5)
- Turn Vehicle Left/Right: NumPad-Left (4) / NumPad-Right (6)
- Strafe Vehicle Left/Right: NumPad (7) / NumPad (9)
- Use Primary Weapon: NumPad (0)
- Use Secondary Weapon: NumpPad (1)
- Repair Vehicle: NumPad Enter

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


## TODO:
1. Game setup customization for Vehicles, Weapons, Arenas
1. Better/unique weapon icons
1. Re-write bot AI to use Behavior Tree
1. Implement feature to play through a series of games, with best of 3, 5, or 7. Between each game enter the vehicle/weapon customization screen - call this the pit stop. 1st place has about 15s to change up their layout. 2nd place has 15s on top of that, etc...
1. Implement controller support.
1. Survival mode? Team of Humans vs. waves of bots? Support for teams in other modes?
1. Capture the flag mode?
1. Create a generic Hitbox component, and create hitbox detection functions - used for some entities, but not all
1. Make use of ncollide coarse and fine phase detection algorithm
1. Improve missile heat-seeking tracking - pretty good now, still not great though

---

How to make a game fun?
1. Overcoming a challenge (not too easy, not too hard)
1. Sense of progression
1. Sense of wonder

Maybe I need some type of mode where you go through procedurally generated rooms where you fight 1 or a few enemies.
When you die you can re-customize your vehicle. Of course you want it to specialize somewhat against the enemies in your current room, but you'd also like to make it generally robust enough to defeat enemies in the next rooms.
Maybe make the goal time based, life based, and room based?

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


For final distributable .exe release target on Windows (with controller support, slow safety checks disabled, and cmd window disabled):

```bash
cargo rustc --release --features "sdl_controller, no-slow-safety-checks" -- -Clink-args="/SUBSYSTEM:WINDOWS /ENTRY:mainCRTStartup"
```
... and make sure the following are included in release folder: configs, assets, and SDL2.dll