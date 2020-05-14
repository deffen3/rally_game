use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{
        Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
};

use std::collections::HashMap;

extern crate nalgebra as na;
use na::{Isometry2, Vector2};
use ncollide2d::query::{self, Proximity};
use ncollide2d::shape::{Ball, Cuboid};

use crate::components::{
    get_next_weapon_name, kill_restart_vehicle, update_weapon_icon, update_weapon_properties,
    vehicle_damage_model, Hitbox, HitboxShape, Player, PlayerWeaponIcon, Vehicle, VehicleState, Weapon,
    WeaponFire, WeaponStoreResource,
};

use crate::entities::spawn_weapon_boxes;

use crate::resources::{GameModeSetup, GameModes, WeaponFireResource};

use crate::audio::{play_bounce_sound, play_score_sound, Sounds};

pub const HIT_SOUND_COOLDOWN_RESET: f32 = 0.25;

#[derive(SystemDesc, Default)]
pub struct CollisionVehicleWeaponFireSystem {
    pub hit_sound_cooldown_timer: f32,
    pub weapon_spawner_cooldown_timer: f32,
}

impl<'s> System<'s> for CollisionVehicleWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Hitbox>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, PlayerWeaponIcon>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, Weapon>,
        ReadStorage<'s, WeaponFire>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        ReadExpect<'s, GameModeSetup>,
        ReadExpect<'s, WeaponStoreResource>,
    );

    fn setup(&mut self, _world: &mut World) {
        self.hit_sound_cooldown_timer = -1.0;
        self.weapon_spawner_cooldown_timer = 20.0;
    }

    fn run(
        &mut self,
        (
            entities,
            hitboxes,
            mut transforms,
            mut players,
            player_icons,
            mut vehicles,
            mut weapons,
            weapon_fires,
            time,
            storage,
            sounds,
            audio_output,
            weapon_fire_resource,
            lazy_update,
            game_mode_setup,
            weapon_store_resource,
        ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        if game_mode_setup.random_weapon_spawns
            && game_mode_setup.game_mode != GameModes::ClassicGunGame
        {
            self.weapon_spawner_cooldown_timer -= dt;
        }

        for (entity, hitbox, transform) in (&*entities, &hitboxes, &transforms).join() {
            if hitbox.is_wall {
                let hitbox_x = transform.translation().x;
                let hitbox_y = transform.translation().y;

                for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                    (&*entities, &weapon_fires, &transforms).join()
                {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(weapon_fire.width/2.0, weapon_fire.height/2.0));
                    let fire_collider_pos = Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);

                    let collision;
                    if hitbox.shape == HitboxShape::Circle {
                        let hitbox_collider_shape = Ball::new(hitbox.width / 2.0);
                        let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                        collision = query::proximity(
                            &fire_collider_pos, &fire_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0,
                        );
                    } else if hitbox.shape == HitboxShape::Rectangle {
                        let hitbox_collider_shape = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0));
                        let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                        collision = query::proximity(
                            &fire_collider_pos, &fire_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0,
                        );
                    }
                    else {
                        let hitbox_collider_shape = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0));
                        let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                        collision = query::proximity(
                            &fire_collider_pos, &fire_collider_shape,
                            &hitbox_collider_pos, &hitbox_collider_shape,
                            0.0,
                        );
                    }

                    //weapon fire hits
                    if collision == Proximity::Intersecting {
                        if !weapon_fire.attached {
                            let _ = entities.delete(weapon_fire_entity);
                        }
                    }
                }
            } else if hitbox.is_weapon_box {
                //delete old weapon_boxes
                if self.weapon_spawner_cooldown_timer <= 0.0 {
                    let _ = entities.delete(entity);
                }
            }
        }

        if game_mode_setup.random_weapon_spawns
            && game_mode_setup.game_mode != GameModes::ClassicGunGame
        {
            if self.weapon_spawner_cooldown_timer <= 0.0 {
                self.weapon_spawner_cooldown_timer = game_mode_setup.weapon_spawn_timer;

                spawn_weapon_boxes(
                    &entities,
                    &weapon_fire_resource,
                    &lazy_update,
                    game_mode_setup.weapon_spawn_count.clone(),
                );
            }
        }

        let mut player_makes_kill_map = HashMap::new();
        let mut player_got_killed_map = HashMap::new();

        for (player, vehicle, _weapon, vehicle_transform) in
            (&mut players, &mut vehicles, &mut weapons, &transforms).join()
        {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            let vehicle_rotation = vehicle_transform.rotation();
            let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

            let vehicle_collider_shape = Cuboid::new(Vector2::new(vehicle.width/2.0, vehicle.height/2.0));
            let vehicle_collider_pos = Isometry2::new(Vector2::new(vehicle_x, vehicle_y), vehicle_angle);


            player.last_hit_timer += dt;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                (&*entities, &weapon_fires, &transforms).join()
            {
                if weapon_fire.owner_player_id != player.id {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(weapon_fire.width/2.0, weapon_fire.height/2.0));
                    let fire_collider_pos = Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);

                    let collision = query::proximity(
                        &fire_collider_pos, &fire_collider_shape,
                        &vehicle_collider_pos, &vehicle_collider_shape,
                        0.0,
                    );

                    if collision == Proximity::Intersecting {
                        if !weapon_fire.attached {
                            let _ = entities.delete(weapon_fire_entity);
                        }

                        player.last_hit_by_id = Some(weapon_fire.owner_player_id.clone());
                        player.last_hit_timer = 0.0;

                        let damage: f32 = weapon_fire.damage;

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            weapon_fire.piercing_damage_pct,
                            weapon_fire.shield_damage_pct,
                            weapon_fire.armor_damage_pct,
                            weapon_fire.health_damage_pct,
                        );

                        if vehicle_destroyed && vehicle.state == VehicleState::Active {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_deref());

                            player_makes_kill_map.insert(
                                weapon_fire.owner_player_id.clone(),
                                weapon_fire.weapon_name.clone(),
                            );

                            player_got_killed_map
                                .insert(player.id.clone(), weapon_fire.owner_player_id.clone());
                        }

                        if self.hit_sound_cooldown_timer < 0.0
                            && vehicle.state == VehicleState::Active
                        {
                            play_score_sound(&*sounds, &storage, audio_output.as_deref());
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        let mut weapon_icons_old_map = HashMap::new();

        for (player, mut weapon, vehicle, transform) in
            (&mut players, &mut weapons, &mut vehicles, &mut transforms).join()
        {
            let killer_data = player_makes_kill_map.get(&player.id);

            if let Some(killer_data) = killer_data {
                let weapon_name = killer_data;

                //classic gun-game rules: hot-swap upgrade weapon type for player who got the kill
                if game_mode_setup.game_mode == GameModes::ClassicGunGame
                    && *weapon_name == weapon.name
                {
                    //if kill was using player's current weapon
                    player.kills += 1;
                    let new_weapon_name = get_next_weapon_name(weapon.name.clone());

                    if let Some(new_weapon_name) = new_weapon_name.clone() {
                        weapon_icons_old_map.insert(player.id, weapon.stats.weapon_type);

                        update_weapon_properties(weapon, new_weapon_name, &weapon_store_resource);
                        update_weapon_icon(
                            &entities,
                            &mut weapon,
                            &weapon_fire_resource,
                            player.id,
                            &lazy_update,
                        );

                        vehicle.weapon_weight = weapon.stats.weight;
                    } //else, keep current weapon installed, no kill in this mode
                } else {
                    player.kills += 1; //in all other modes the kill always counts
                }
            }

            let killed_data = player_got_killed_map.get(&player.id);

            if let Some(_killed_data) = killed_data {
                player.deaths += 1;

                kill_restart_vehicle(player, vehicle, transform, game_mode_setup.stock_lives);
            }
        }

        for (entity, player_icon) in (&*entities, &player_icons).join() {
            let weapon_icons_old = weapon_icons_old_map.get(&player_icon.id);

            if let Some(weapon_icons_old) = weapon_icons_old {
                let weapon_type = weapon_icons_old;
                if *weapon_type == player_icon.weapon_type {
                    let _ = entities.delete(entity);
                }
            }
        }

        self.hit_sound_cooldown_timer -= dt;
    }
}
