use amethyst::{
    core::Transform,
    ecs::prelude::{Component, DenseVecStorage, Entity, World},
};

use rand::Rng;
use serde::Deserialize;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::components::{
    ArenaProperties, Armor, DurationDamage, Health, Player, Repair, Shield, WeaponNameInstall,
    WeaponNames,
};
use crate::entities::ui::PlayerStatusText;
use crate::load_ron_asset;
use crate::resources::GameModes;

//VehicleNames correspond to the vehicle_properties.ron
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum VehicleNames {
    MediumCombat,
    LightRacer,
    HeavyTank,
    CivilianCruiser,
    Interceptor,
    TSpeeder,
}

//VehicleTypes correspond to sprites
#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Hash, Eq)]
pub enum VehicleTypes {
    MediumCombat,
    LightRacer,
    HeavyTank,
    CivilianCruiser,
    Interceptor,
    TSpeeder,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize)]
pub enum VehicleMovementType {
    Hover, //hover craft can turn to spin in place, and have the same friction regardless of velocity/vehicle angles
    Car, //cars can only turn if moving, and have high friction when velocity angle differs greatly from vehicle angle
    Tank, //tanks can turn to spin in place, and have high friction when velocity angle differs greatly from vehicle angle
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VehicleState {
    Active,
    InActive,
    InRespawn,
}

pub struct Vehicle {
    pub movement_type: VehicleMovementType,
    pub width: f32,
    pub height: f32,
    pub dx: f32,
    pub dy: f32,
    pub dr: f32,
    pub angle_to_closest_vehicle: Option<f32>,
    pub dist_to_closest_vehicle: Option<f32>,
    pub angle_to_closest_targetable_vehicle: Option<f32>,
    pub dist_to_closest_targetable_vehicle: Option<f32>,
    pub collision_cooldown_timer: f32,
    pub health: Health,
    pub armor: Armor,
    pub shield: Shield,
    pub repair: Repair,
    pub heal_pulse_amount: f32,
    pub heal_pulse_rate: f32,
    pub heal_cooldown_timer: f32,
    pub engine_weight: f32,
    pub weapon_weight: f32,
    pub engine_force: f32,
    pub max_velocity: f32,
    pub restricted_max_velocity: f32,
    pub restricted_velocity_timer: f32,
    pub stuck_accel_effect_timer: f32,
    pub malfunction: f32,
    pub malfunction_cooldown_timer: f32,
    pub ion_malfunction_pct: f32,
    pub duration_damages: Vec<(Option<usize>, Option<WeaponNames>, DurationDamage)>,
    pub respawn_timer: f32,
    pub death_x: f32,
    pub death_y: f32,
    pub death_angle: f32,
    pub state: VehicleState,
    pub player_status_text: PlayerStatusText,
}

impl Component for Vehicle {
    type Storage = DenseVecStorage<Self>;
}

impl Vehicle {
    pub fn new(
        player_status_text: PlayerStatusText,
        health_entity: Entity,
        armor_entity: Entity,
        shield_entity: Entity,
        repair_entity: Entity,
        stats: VehicleStats,
        weapon_weight: f32,
    ) -> Vehicle {
        Vehicle {
            movement_type: stats.movement_type,
            width: stats.width,
            height: stats.height,
            dx: 0.0,
            dy: 0.0,
            dr: 0.0,
            angle_to_closest_vehicle: None,
            dist_to_closest_vehicle: None,
            angle_to_closest_targetable_vehicle: None,
            dist_to_closest_targetable_vehicle: None,
            collision_cooldown_timer: 0.0,
            health: Health {
                value: stats.max_health,
                max: stats.max_health,
                repair_rate: stats.health_repair_rate,
                entity: health_entity,
            },
            armor: Armor {
                value: stats.max_armor,
                max: stats.max_armor,
                entity: armor_entity,
            },
            shield: Shield {
                value: stats.max_shield,
                max: stats.max_shield,
                recharge_rate: stats.shield_recharge_rate,
                cooldown_timer: 0.0,
                cooldown_reset: stats.shield_cooldown,
                repair_timer: 0.0,
                repair_reboot_time: stats.shield_repair_reboot_time,
                radius: stats.shield_radius,
                entity: shield_entity,
            },
            repair: Repair {
                activated: false,
                init_timer: 0.0,
                init_time_threshold: stats.health_repair_init_time,
                entity: repair_entity,
            },
            heal_pulse_amount: stats.heal_pulse_amount,
            heal_pulse_rate: stats.heal_pulse_rate,
            heal_cooldown_timer: 0.0,
            engine_force: stats.engine_force,
            engine_weight: stats.engine_weight,
            weapon_weight,
            max_velocity: stats.max_velocity,
            restricted_max_velocity: stats.max_velocity,
            restricted_velocity_timer: 0.0,
            stuck_accel_effect_timer: 0.0,
            malfunction: 0.0,
            malfunction_cooldown_timer: 0.0,
            ion_malfunction_pct: 0.0,
            duration_damages: Vec::new(),
            respawn_timer: 5.0,
            death_x: 0.0,
            death_y: 0.0,
            death_angle: 0.0,
            state: VehicleState::Active,
            player_status_text,
        }
    }
}

pub fn determine_vehicle_weight(vehicle: &Vehicle) -> f32 {
    //typical vehicle weight = 100 at S:100/A:100/H:100 with normal engine efficiency

    //health makes up the main hull of the vehicle, and contributes 20 base weight + 20 per 100 health
    //shields make up 15 weight
    //armor another 25 weight
    //engine another 20 weight

    //typical weapon weight adds about 10.0
    //  for a total of about 110.0

    //a lighter racing vehicle with s:25/A:0/H:100 would weigh:
    //  B:20 + H:20 + S:3.75 + E:20 + W:10 = 73.75,
    //  and therefore would have about 50% quicker acceleration
    //  but could only take about 42% typical damage before blowing up

    //a heavy-weight tank combat vehicle with s:200/A:200/H:150 would weigh:
    //  B:20 + H:30 + S:30 + A:50 + E:20 + W:10 = 160,
    //  and therefore would have about 45% slower acceleration
    //  but would take almost 550 damage, an 83% increase

    //NOTE: lost armor does not contribute to weight, only the current value of armor matters
    let vehicle_weight = (20.0 + vehicle.health.max * 20. / 100.)
        + (vehicle.shield.max * 15. / 100.)
        + (vehicle.armor.value * 25. / 100.)
        + vehicle.engine_weight
        + vehicle.weapon_weight;

    vehicle_weight
}

pub fn determine_vehicle_weight_stats(vehicle: VehicleStats) -> f32 {
    let vehicle_weight = (20.0 + vehicle.max_health * 20. / 100.)
        + (vehicle.max_shield * 15. / 100.)
        + (vehicle.max_armor * 25. / 100.)
        + vehicle.engine_weight;

    vehicle_weight
}

pub fn kill_restart_vehicle(
    player: &Player,
    vehicle: &mut Vehicle,
    transform: &Transform,
    stock_lives: i32,
) {
    vehicle.death_x = transform.translation().x;
    vehicle.death_y = transform.translation().y;

    let (_, _, vehicle_angle) = transform.rotation().euler_angles();
    vehicle.death_angle = vehicle_angle;

    //transform.set_translation_x(10.0 * self.arena_properties.width);
    //transform.set_translation_y(10.0 * self.arena_properties.height);

    if stock_lives > 0 && player.deaths >= stock_lives {
        vehicle.state = VehicleState::InActive;
    } else {
        vehicle.state = VehicleState::InRespawn;
    }
}

pub fn check_respawn_vehicle(
    vehicle: &mut Vehicle,
    transform: &mut Transform,
    dt: f32,
    game_mode: GameModes,
    last_spawn_index: u32,
    arena_properties: &ArenaProperties,
) -> u32 {
    let mut rng = rand::thread_rng();

    let mut spawn_index = last_spawn_index;

    if vehicle.state == VehicleState::InRespawn {
        vehicle.respawn_timer -= dt;

        if vehicle.respawn_timer < 0.0 {
            vehicle.state = VehicleState::Active;

            vehicle.respawn_timer = 5.0;

            vehicle.dx = 0.0;
            vehicle.dy = 0.0;
            vehicle.dr = 0.0;

            vehicle.restricted_max_velocity = vehicle.max_velocity;
            vehicle.restricted_velocity_timer = 0.0;
            vehicle.stuck_accel_effect_timer = 0.0;
            vehicle.malfunction = 0.0;
            vehicle.malfunction_cooldown_timer = 0.0;
            vehicle.ion_malfunction_pct = 0.0;
            vehicle.duration_damages = Vec::new();

            vehicle.shield.value = vehicle.shield.max;
            vehicle.shield.cooldown_timer = -1.;

            vehicle.armor.value = vehicle.armor.max;
            vehicle.health.value = vehicle.health.max;

            if game_mode == GameModes::Race {
                transform.set_rotation_2d(vehicle.death_angle);
                transform.set_translation_xyz(vehicle.death_x, vehicle.death_y, 0.0);
            } else {
                //Ensure that the spawn_index != last_spawn_index
                spawn_index =
                    rng.gen_range(0, arena_properties.player_spawn_points.len() - 1) as u32;
                if spawn_index >= last_spawn_index {
                    spawn_index += 1;
                }
                let player_spawn = arena_properties.player_spawn_points[spawn_index as usize];

                transform.set_rotation_2d(player_spawn.rotation / 180.0 * PI);
                transform.set_translation_xyz(player_spawn.x, player_spawn.y, 0.0);
            }
        }
    }

    spawn_index
}

pub fn vehicle_damage_model(
    vehicle: &mut Vehicle,
    damager_id: Option<usize>,
    weapon_name: Option<WeaponNames>,
    mut damage: f32,
    piercing_damage_pct: f32,
    shield_damage_pct: f32,
    armor_damage_pct: f32,
    health_damage_pct: f32,
    duration_damage: DurationDamage,
) -> bool {
    let mut vehicle_destroyed = false;

    if vehicle.state == VehicleState::Active {
        let mut shield_damage_applied: bool = false;
        let mut armor_damage_applied: bool = false;
        let mut health_damage_applied: bool = false;

        if damage >= 0.0 {
            let mut piercing_damage: f32 = 0.0;

            if piercing_damage_pct > 0.0 {
                piercing_damage = damage * piercing_damage_pct / 100.0;
                damage -= piercing_damage;
            }

            if vehicle.shield.value > 0.0 {
                if damage * shield_damage_pct > 0.0 {
                    shield_damage_applied = true;

                    vehicle.shield.value -= damage * shield_damage_pct / 100.0;
                    damage = 0.0;

                    if vehicle.shield.value < 0.0 {
                        damage = vehicle.shield.value.abs(); //over damage on shields, needs taken from armor
                        vehicle.shield.value = 0.0;
                    } else {
                        //take damage to shields, but shields are still alive, reset shield recharge cooldown
                        vehicle.shield.cooldown_timer = vehicle.shield.cooldown_reset;
                    }
                } else {
                    damage = 0.0;
                    //This would happen if the vehicle has shields, but is being hit with a weapon that does no damage to shields
                    //Therefore no damage can be done, unless it is piercing damage
                }
            }

            if vehicle.armor.value > 0.0 {
                if damage * armor_damage_pct > 0.0 {
                    armor_damage_applied = true;

                    vehicle.armor.value -= damage * armor_damage_pct / 100.0;
                    damage = 0.0;

                    if vehicle.armor.value < 0.0 {
                        damage = vehicle.armor.value.abs(); //over damage on armor, needs taken from health
                        vehicle.armor.value = 0.0;
                    }
                } else {
                    damage = 0.0;
                    //This would happen if the vehicle has armor, but is being hit with a weapon that does no damage to armor
                    //Therefore no damage can be done, unless it is piercing damage
                }
            }

            let health_damage: f32 = (damage + piercing_damage) * health_damage_pct / 100.0;
            if health_damage > 0.0 {
                health_damage_applied = true;
            }

            if vehicle.health.value > 0.0 {
                //only destroy once
                if vehicle.health.value <= health_damage {
                    vehicle_destroyed = true;
                    vehicle.health.value = 0.0;
                } else {
                    vehicle.health.value -= health_damage;
                }
            }
        } else {
            //damage is negative -> healing
            //NOTE: Piercing has no effect on healing

            if vehicle.health.value < vehicle.health.max {
                //missing some health
                if damage * health_damage_pct < 0.0 {
                    health_damage_applied = true;

                    vehicle.health.value -= damage * health_damage_pct / 100.0;
                    damage = 0.0;

                    if vehicle.health.value > vehicle.health.max {
                        //too much healing, apply to next type
                        damage = vehicle.health.value - vehicle.health.max;
                        vehicle.health.value = vehicle.health.max;
                    }
                }
            }

            if vehicle.armor.value < vehicle.armor.max {
                //missing some armor
                if damage * armor_damage_pct < 0.0 {
                    armor_damage_applied = true;

                    vehicle.armor.value -= damage * armor_damage_pct / 100.0;
                    damage = 0.0;

                    if vehicle.armor.value > vehicle.armor.max {
                        //too much healing, apply to next type
                        damage = vehicle.armor.value - vehicle.armor.max;
                        vehicle.armor.value = vehicle.armor.max;
                    }
                }
            }

            if vehicle.shield.value < vehicle.shield.max {
                //missing some armor
                if damage * shield_damage_pct < 0.0 {
                    shield_damage_applied = true;

                    vehicle.shield.value -= damage * shield_damage_pct / 100.0;

                    if vehicle.shield.value > vehicle.shield.max {
                        //too much healing
                        vehicle.shield.value = vehicle.shield.max;
                    }
                }
            }
        }

        //Check if duration damage should be applied,
        //  which should only be in cases where the correct damage type was done
        //  by the original damage shot
        if duration_damage.timer >= 0.0
            && ((shield_damage_applied && duration_damage.shield_damage_pct > 0.0)
                || (armor_damage_applied && duration_damage.armor_damage_pct > 0.0)
                || (health_damage_applied && duration_damage.health_damage_pct > 0.0))
        {
            vehicle
                .duration_damages
                .push((damager_id, weapon_name, duration_damage));
        }
    }

    vehicle_destroyed
}

#[derive(Clone)]
pub struct VehicleStoreResource {
    pub properties: HashMap<VehicleNames, VehicleStats>,
    pub order: Vec<VehicleNames>,
    pub type_sprites: HashMap<VehicleTypes, (usize, usize, usize)>,
}

pub fn build_vehicle_store(world: &mut World) -> VehicleStoreResource {
    let vehicle_store = VehicleStoreResource {
        properties: load_ron_asset(&["game", "vehicle_properties.ron"]),
        order: load_ron_asset(&["game", "vehicle_selection_order.ron"]),
        type_sprites: load_ron_asset(&["game", "vehicle_type_sprites.ron"]),
    };
    world.insert(vehicle_store.clone());

    vehicle_store
}

pub fn get_none_vehicle() -> VehicleStats {
    VehicleStats {
        display_name: "None".to_string(),
        vehicle_type: VehicleTypes::MediumCombat,
        max_shield: 0.0,
        max_armor: 0.0,
        max_health: 0.0,
        engine_force: 0.0,
        engine_weight: 0.0,
        width: 0.0,
        height: 0.0,
        sprite_scalar: 0.0,
        max_velocity: 0.0,
        movement_type: VehicleMovementType::Hover,
        health_repair_rate: 0.0,
        health_repair_init_time: 0.0,
        shield_recharge_rate: 0.0,
        shield_cooldown: 0.0,
        shield_repair_reboot_time: 0.0,
        shield_radius: 0.0,
        heal_pulse_amount: 0.0,
        heal_pulse_rate: 0.0,
        default_weapons: vec![],
    }
}

pub fn get_vehicle_sprites(world: &World, vehicle_type: VehicleTypes) -> (usize, usize, usize) {
    let vehicle_store = world.fetch::<VehicleStoreResource>();

    let vehicle_sprites_option = vehicle_store.type_sprites.get(&vehicle_type);
    let vehicle_sprites_out: (usize, usize, usize);

    if let Some(vehicle_properties) = vehicle_sprites_option {
        vehicle_sprites_out = *vehicle_properties;
    } else {
        vehicle_sprites_out = (0, 0, 0);
    }
    vehicle_sprites_out
}

pub fn get_next_vehicle_name(world: &World, name: VehicleNames) -> VehicleNames {
    let vehicle_store = world.fetch::<VehicleStoreResource>();

    let length = vehicle_store.order.len();
    let index = vehicle_store.order.iter().position(|&r| r == name).unwrap();
    let vehicle_out: VehicleNames;
    if index == length - 1 {
        vehicle_out = vehicle_store.order[0]
    } else {
        vehicle_out = vehicle_store.order[index + 1];
    }
    vehicle_out
}

pub fn get_prev_vehicle_name(world: &World, name: VehicleNames) -> VehicleNames {
    let vehicle_store = world.fetch::<VehicleStoreResource>();

    let length = vehicle_store.order.len();
    let index = vehicle_store.order.iter().position(|&r| r == name).unwrap();

    let vehicle_out: VehicleNames;
    if index == 0 {
        vehicle_out = vehicle_store.order[length - 1]
    } else {
        vehicle_out = vehicle_store.order[index - 1];
    }
    vehicle_out
}

#[derive(Clone, Debug, Deserialize)]
pub struct VehicleStats {
    pub display_name: String,
    pub vehicle_type: VehicleTypes,
    pub max_shield: f32,
    pub max_armor: f32,
    pub max_health: f32,
    pub engine_force: f32,
    pub engine_weight: f32,
    pub width: f32,
    pub height: f32,
    pub sprite_scalar: f32,
    pub max_velocity: f32,
    pub movement_type: VehicleMovementType,
    pub health_repair_rate: f32,
    pub health_repair_init_time: f32,
    pub shield_recharge_rate: f32,
    pub shield_cooldown: f32,
    pub shield_repair_reboot_time: f32,
    pub shield_radius: f32,
    pub heal_pulse_amount: f32,
    pub heal_pulse_rate: f32,
    pub default_weapons: Vec<WeaponNameInstall>,
}
