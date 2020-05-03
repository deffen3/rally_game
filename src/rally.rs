use amethyst::{
    core::math::Vector3,
    core::transform::Transform,
    core::Time,
    ecs::prelude::{Entities, Entity, LazyUpdate, ReadExpect},
    prelude::*,
    ui::{UiText, UiFinder},
    input::{is_close_requested, is_key_down},
    audio::output::init_output,
    utils::fps_counter::FpsCounter,
    winit::VirtualKeyCode,
};



use crate::pause::PauseMenuState;


use crate::components::{
    Vehicle, Weapon, WeaponFire, get_weapon_icon
};

use crate::resources::{WeaponFireResource};



pub const ARENA_HEIGHT: f32 = 400.0;
pub const UI_HEIGHT: f32 = 35.0;
pub const ARENA_WIDTH: f32 = 400.0;

pub const BASE_COLLISION_DAMAGE: f32 = 20.0;
pub const COLLISION_PIERCING_DAMAGE_PCT: f32 = 0.0;
pub const COLLISION_SHIELD_DAMAGE_PCT: f32 = 25.0;
pub const COLLISION_ARMOR_DAMAGE_PCT: f32 = 80.0;
pub const COLLISION_HEALTH_DAMAGE_PCT: f32 = 100.0;

pub const MAX_PLAYERS: usize = 4;
pub const BOT_PLAYERS: usize = MAX_PLAYERS - 1;

pub const KILLS_TO_WIN: i32 = 15;

pub const GUN_GAME_MODE: bool = true;

// #[derive(Clone, Debug, PartialEq, Eq)]
// pub enum GameModes {
//     ClassicGunGame, //First to get a kill with each weapon, and weapons are hot-swapped after kills.
//     Deathmatch, //First to a certain number of kills. New weapons can be picked up from arena.
//     StockBattle, //Last player alive wins, with a set number of starting lives. New weapons can be picked up from arena.
//     KingOfTheHill, //Player gains points for being the only person in the special "hill" zone. First player to a certain number of points wins.
// }

// pub struct GameRules {
//     pub game_mode: Option<GameModes>,
//     pub kills_to_win: i32,
//     pub starting_lives: i32,
//     pub points_to_win: i32,
// }






#[derive(Default)]
pub struct GameplayState {
    // // If the Game is paused or not
    paused: bool,
    // The UI root entity. Deleting this should remove the complete UI
    ui_root: Option<Entity>,
    // A reference to the FPS display, which we want to interact with
    fps_display: Option<Entity>,
    // A reference to the random text, which we want to modify during updates
    random_text: Option<Entity>,
}


impl SimpleState for GameplayState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let StateData { mut world, .. } = data;

        // needed for registering audio output.
        init_output(&mut world);

        // self.ui_root =
        //     Some(world.exec(|mut creator: UiCreator<'_>| creator.create("ui/example.ron", ())));
    }

    fn on_pause(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = true;
    }

    fn on_resume(&mut self, _data: StateData<'_, GameData<'_, '_>>) {
        self.paused = false;
    }

    fn on_stop(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        if let Some(root_entity) = self.ui_root {
            data.world
                .delete_entity(root_entity)
                .expect("Failed to remove Game Screen");
        }

        self.ui_root = None;
        self.fps_display = None;
        self.random_text = None;
    }

    fn handle_event(
        &mut self,
        _: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        match &event {
            StateEvent::Window(event) => {
                if is_close_requested(&event) {
                    log::info!("[Trans::Quit] Quitting Application!");
                    Trans::Quit
                } else if is_key_down(&event, VirtualKeyCode::Escape) {
                    log::info!("[Trans::Push] Pausing Game!");
                    Trans::Push(Box::new(PauseMenuState::default()))
                } else {
                    Trans::None
                }
            }
            StateEvent::Ui(ui_event) => {
                log::info!(
                    "[HANDLE_EVENT] You just interacted with a ui element: {:?}",
                    ui_event
                );
                Trans::None
            }
            StateEvent::Input(_input) => {
                //log::info!("Input Event detected: {:?}.", input);
                Trans::None
            }
        }
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        data.world.maintain();

        let StateData { world, .. } = data;

        // this cannot happen in 'on_start', as the entity might not be fully
        // initialized/registered/created yet.
        if self.fps_display.is_none() {
            world.exec(|finder: UiFinder<'_>| {
                if let Some(entity) = finder.find("fps") {
                    self.fps_display = Some(entity);
                }
            });
        }

        if self.random_text.is_none() {
            world.exec(|finder: UiFinder| {
                if let Some(entity) = finder.find("random_text") {
                    self.random_text = Some(entity);
                }
            });
        }

        // it is important that the 'paused' field is actually pausing your game.
        // Make sure to also pause your running systems.
        if !self.paused {
            let mut ui_text = world.write_storage::<UiText>();

            if let Some(fps_display) = self.fps_display.and_then(|entity| ui_text.get_mut(entity)) {
                if world.read_resource::<Time>().frame_number() % 20 == 0 && !self.paused {
                    let fps = world.read_resource::<FpsCounter>().sampled_fps();
                    fps_display.text = format!("FPS: {:.*}", 2, fps);
                }
            }

            if let Some(random_text) = self.random_text.and_then(|entity| ui_text.get_mut(entity)) {
                if let Ok(value) = random_text.text.parse::<i32>() {
                    let mut new_value = value * 10;
                    if new_value > 100_000 {
                        new_value = 1;
                    }
                    random_text.text = new_value.to_string();
                } else {
                    random_text.text = String::from("1");
                }
            }
        }

        Trans::None
    }
}




pub fn fire_weapon(
    entities: &Entities,
    weapon_fire_resource: &ReadExpect<WeaponFireResource>,
    weapon: Weapon,
    fire_position: Vector3<f32>,
    fire_angle: f32,
    player_id: usize,
    lazy_update: &ReadExpect<LazyUpdate>,
) {
    let fire_entity: Entity = entities.create();

    let mut weapon_fire = WeaponFire::new(
        weapon.name.clone(),
        weapon.stats.weapon_type,
        player_id,
        weapon.stats.heat_seeking,
        weapon.stats.heat_seeking_agility,
        weapon.stats.attached,
        weapon.stats.deployed,
        weapon.stats.mounted_angle,
        weapon.stats.shot_speed,
        weapon.stats.shot_life_limit,
        weapon.stats.damage,
        weapon.stats.shield_damage_pct,
        weapon.stats.armor_damage_pct,
        weapon.stats.piercing_damage_pct,
        weapon.stats.health_damage_pct,
    );

    let local_transform = {
        let mut local_transform = Transform::default();
        local_transform.set_translation(fire_position);

        let angle_x_comp: f32 = -fire_angle.sin();
        let angle_y_comp: f32 = fire_angle.cos();

        local_transform.set_rotation_2d(fire_angle);

        weapon_fire.dx = weapon_fire.shot_speed * angle_x_comp;
        weapon_fire.dy = weapon_fire.shot_speed * angle_y_comp;

        //adjust the first postion
        let x = local_transform.translation().x;
        let y = local_transform.translation().y;

        //let yaw_width = weapon_fire.height*0.5 * angle_x_comp + weapon_fire.width*0.5 * (1.0-angle_x_comp);
        //let yaw_height = weapon_fire.height*0.5 * angle_y_comp + weapon_fire.width*0.5 * (1.0-angle_y_comp);
        let yaw_width = 0.0;
        let yaw_height = 0.0;

        local_transform.set_translation_x(x - yaw_width);
        local_transform.set_translation_y(y + yaw_height);

        local_transform
    };
    lazy_update.insert(fire_entity, weapon_fire);


    let (_icon_scale, weapon_sprite) = get_weapon_icon(player_id, weapon.stats, weapon_fire_resource);

    lazy_update.insert(fire_entity, weapon_sprite);
    lazy_update.insert(fire_entity, local_transform);
}

pub fn vehicle_damage_model(
    vehicle: &mut Vehicle,
    mut damage: f32,
    piercing_damage_pct: f32,
    shield_damage_pct: f32,
    armor_damage_pct: f32,
    health_damage_pct: f32,
) -> bool {
    let mut piercing_damage: f32 = 0.0;

    if piercing_damage_pct > 0.0 {
        piercing_damage = damage * piercing_damage_pct / 100.0;
        damage -= piercing_damage;
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3} P:{:>6.3}, D:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield, piercing_damage, damage);

    if vehicle.shield.value > 0.0 {
        vehicle.shield.value -= damage * shield_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.shield.value < 0.0 {
            damage -= vehicle.shield.value; //over damage on shields, needs taken from armor
            vehicle.shield.value = 0.0;
        } else {
            //take damage to shields, but shields are still alive, reset shield recharge cooldown
            vehicle.shield.cooldown_timer = vehicle.shield.cooldown_reset;
        }
    }

    if vehicle.armor.value > 0.0 {
        vehicle.armor.value -= damage * armor_damage_pct / 100.0;
        damage = 0.0;

        if vehicle.armor.value < 0.0 {
            damage -= vehicle.armor.value; //over damage on armor, needs taken from health
            vehicle.armor.value = 0.0;
        }
    }

    let health_damage: f32 = (damage + piercing_damage) * health_damage_pct / 100.0;

    let mut vehicle_destroyed = false;

    if vehicle.health.value > 0.0 { //only destroy once
        if vehicle.health.value <= health_damage {
            vehicle_destroyed = true;
            vehicle.health.value = 0.0;
        } else {
            vehicle.health.value -= health_damage;
        }
    }

    //println!("H:{:>6.3} A:{:>6.3} S:{:>6.3}",vehicle.health, vehicle.armor, vehicle.shield);

    vehicle_destroyed
}

/*
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisBinding {
    VehicleAccel(usize),
    VehicleTurn(usize),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBinding {
    VehicleShoot(usize),
}

impl Display for AxisBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct MovementBindingTypes;

impl BindingTypes for MovementBindingTypes {
    type Axis = AxisBinding;
    type Action = ActionBinding;
}
*/
