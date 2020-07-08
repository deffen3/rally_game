use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::math::Vector3,
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{
        Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
};

use rand::Rng;
use std::collections::HashMap;
use std::f32::consts::PI;

extern crate nalgebra as na;
use na::{Isometry2, Point2, Vector2};
use ncollide2d::query::{self, Proximity, Ray, RayCast};
use ncollide2d::shape::{Ball, Cuboid};

use crate::components::{
    get_next_gg_weapon_name, kill_restart_vehicle, update_weapon_properties, vehicle_damage_model,
    ArenaElement, ArenaNames, ArenaProperties, ArenaStoreResource, DurationDamage, HitboxShape,
    ObstacleType, Player, PlayerWeaponIcon, Vehicle, VehicleState, WeaponArray, WeaponFire,
    WeaponNames, WeaponStoreResource,
};

use crate::entities::{
    chain_fire_weapon, explosion_shockwave, hit_spray, spawn_weapon_box_from_spawner,
};

use crate::resources::{GameModeSetup, GameModes, GameWeaponSetup, WeaponFireResource};

use crate::systems::{calc_bounce_angle, clean_angle};

use crate::audio::{play_bounce_sound, play_score_sound, Sounds};

pub const HIT_SOUND_COOLDOWN_RESET: f32 = 0.30;
pub const HIT_SPRAY_COOLDOWN_RESET: f32 = 0.05;

pub const PRE_IMPACT_DT_STEPS: f32 = 1.2;
pub const SHOT_SPEED_TRIGGER: f32 = 500.0;

const PRIMARY_WEAPON_INDEX: usize = 0;

#[derive(SystemDesc, Default)]
pub struct CollisionWeaponFireHitboxSystem {
    pub hit_sound_cooldown_timer: f32,
    pub hit_spray_cooldown_timer: f32,
    pub arena_properties: ArenaProperties,
    pub global_weapon_spawner_cooldown_timer: f32,
}

impl<'s> System<'s> for CollisionWeaponFireHitboxSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, ArenaElement>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Player>,
        ReadStorage<'s, PlayerWeaponIcon>,
        WriteStorage<'s, Vehicle>,
        WriteStorage<'s, WeaponArray>,
        WriteStorage<'s, WeaponFire>,
        Read<'s, Time>,
        Read<'s, AssetStorage<Source>>,
        ReadExpect<'s, Sounds>,
        Option<Read<'s, Output>>,
        ReadExpect<'s, WeaponFireResource>,
        ReadExpect<'s, LazyUpdate>,
        ReadExpect<'s, GameModeSetup>,
        ReadExpect<'s, GameWeaponSetup>,
        ReadExpect<'s, WeaponStoreResource>,
    );

    fn setup(&mut self, world: &mut World) {
        self.hit_sound_cooldown_timer = -1.0;
        self.hit_spray_cooldown_timer = -1.0;

        let fetched_game_weapon_setup = world.try_fetch::<GameWeaponSetup>();

        if let Some(game_weapon_setup) = fetched_game_weapon_setup {
            self.global_weapon_spawner_cooldown_timer =
                game_weapon_setup.random_weapon_spawn_first_timer;
        } else {
            self.global_weapon_spawner_cooldown_timer = 20.0;
        }

        let arena_name;
        {
            let fetched_game_mode_setup = world.try_fetch::<GameModeSetup>();
            if let Some(game_mode_setup) = fetched_game_mode_setup {
                arena_name = game_mode_setup.arena_name.clone();
            } else {
                arena_name = ArenaNames::OpenEmptyMap;
            }
        }

        {
            let fetched_arena_store = world.try_fetch::<ArenaStoreResource>();
            if let Some(arena_store) = fetched_arena_store {
                self.arena_properties = match arena_store.properties.get(&arena_name) {
                    Some(arena_props_get) => (*arena_props_get).clone(),
                    _ => ArenaProperties::default(),
                };
            } else {
                self.arena_properties = ArenaProperties::default();
            }
        }
    }

    fn run(
        &mut self,
        (
            entities,
            mut arena_elements,
            mut transforms,
            mut players,
            player_weapon_icons,
            mut vehicles,
            mut weapon_arrays,
            mut weapon_fires,
            time,
            storage,
            sounds,
            audio_output,
            weapon_fire_resource,
            lazy_update,
            game_mode_setup,
            game_weapon_setup,
            weapon_store_resource,
        ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        if game_weapon_setup.random_weapon_spawns {
            self.global_weapon_spawner_cooldown_timer -= dt;
        }

        //weapon to non-moving hitbox collisions
        for (entity, mut arena_element, transform) in
            (&*entities, &mut arena_elements, &transforms).join()
        {
            if arena_element.obstacle_type == ObstacleType::Wall {
                let hitbox_x = transform.translation().x;
                let hitbox_y = transform.translation().y;

                for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                    (&*entities, &mut weapon_fires, &transforms).join()
                {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(
                        weapon_fire.width / 2.0,
                        weapon_fire.height / 2.0,
                    ));
                    let fire_collider_pos =
                        Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);

                    let collision;

                    let hitbox_collider_shape_circle = Ball::new(arena_element.hitbox.width / 2.0);
                    let hitbox_collider_shape_rect = Cuboid::new(Vector2::new(
                        arena_element.hitbox.width / 2.0,
                        arena_element.hitbox.height / 2.0,
                    )); //unused
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let weapon_fire_hit;
                    if weapon_fire.stats.shot_speed == 0.0 {
                        weapon_fire_hit = false; //assumes hitboxes don't move
                    } else {
                        //use ncollide algorithm
                        let margin;

                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        if abs_vel > SHOT_SPEED_TRIGGER {
                            //use pre-impact detection if within 3 time steps of radius of target
                            margin = abs_vel * dt * PRE_IMPACT_DT_STEPS;
                        } else {
                            margin = 0.0;
                        }

                        if arena_element.hitbox.shape == HitboxShape::Circle {
                            collision = query::proximity(
                                &fire_collider_pos,
                                &fire_collider_shape,
                                &hitbox_collider_pos,
                                &hitbox_collider_shape_circle,
                                margin,
                            );
                        } else {
                            //if hitbox.shape == HitboxShape::Rectangle {
                            collision = query::proximity(
                                &fire_collider_pos,
                                &fire_collider_shape,
                                &hitbox_collider_pos,
                                &hitbox_collider_shape_rect,
                                margin,
                            );
                        }

                        if collision == Proximity::Intersecting {
                            weapon_fire_hit = true;
                        } else if collision == Proximity::WithinMargin {
                            //if potentially on collision course, check time to impact
                            let fire_ray = Ray::new(
                                Point2::new(fire_x, fire_y),
                                Vector2::new(weapon_fire.dx, weapon_fire.dy),
                            );

                            let toi;
                            if arena_element.hitbox.shape == HitboxShape::Circle {
                                toi = hitbox_collider_shape_circle.toi_with_ray(
                                    &hitbox_collider_pos,
                                    &fire_ray,
                                    dt * PRE_IMPACT_DT_STEPS,
                                    true,
                                );
                            } else {
                                toi = hitbox_collider_shape_rect.toi_with_ray(
                                    &hitbox_collider_pos,
                                    &fire_ray,
                                    dt * PRE_IMPACT_DT_STEPS,
                                    true,
                                );
                            }

                            if let Some(toi_result) = toi {
                                if toi_result <= PRE_IMPACT_DT_STEPS * dt {
                                    weapon_fire_hit = true;
                                } else {
                                    weapon_fire_hit = false;
                                }
                            } else {
                                weapon_fire_hit = false;
                            }
                        } else {
                            weapon_fire_hit = false;
                        }
                    }

                    if weapon_fire_hit {
                        if !weapon_fire.stats.attached {
                            if weapon_fire.stats.bounces > 0 {
                                weapon_fire.stats.bounces -= 1;
                                weapon_fire.owner_player_id = None;

                                let contact_data;
                                if arena_element.hitbox.shape == HitboxShape::Circle {
                                    contact_data = query::contact(
                                        &fire_collider_pos,
                                        &fire_collider_shape,
                                        &hitbox_collider_pos,
                                        &hitbox_collider_shape_circle,
                                        0.0,
                                    );
                                } else {
                                    contact_data = query::contact(
                                        &fire_collider_pos,
                                        &fire_collider_shape,
                                        &hitbox_collider_pos,
                                        &hitbox_collider_shape_rect,
                                        0.0,
                                    );
                                }
                                let contact_pt = contact_data.unwrap().world2;

                                let (new_dx, new_dy, _new_angle) = calc_bounce_angle(
                                    contact_pt.x.clone(),
                                    contact_pt.y.clone(),
                                    hitbox_x,
                                    hitbox_y,
                                    arena_element.hitbox.shape.clone(),
                                    weapon_fire.dx.clone(),
                                    weapon_fire.dy.clone(),
                                );

                                weapon_fire.dx = new_dx;
                                weapon_fire.dy = new_dy;
                            } else {
                                let _ = entities.delete(weapon_fire_entity);
                                weapon_fire.active = false;
                            }
                        }
                    }
                }
            } else if arena_element.is_weapon_box {
                //delete old weapon_boxes if they do not have their own spawn timer
                if arena_element.spawn_time.is_none() {
                    if self.global_weapon_spawner_cooldown_timer <= 0.0 {
                        let _ = entities.delete(entity);
                    }
                }
            } else if arena_element.is_weapon_spawn_point {
                //check for special map-defined weapon box spawns here
                //the generic global-rules weapon box spawns will be handled below
                if game_weapon_setup.allow_map_specific_spawn_weapons {
                    if arena_element.is_weapon_spawn_point && !arena_element.spawn_time.is_none() {
                        arena_element.spawn_timer = Some(arena_element.spawn_timer.unwrap() - dt);

                        if arena_element.spawn_timer.unwrap() <= 0.0 {
                            arena_element.spawn_timer = arena_element.spawn_time;
                            spawn_weapon_box_from_spawner(
                                &entities,
                                &weapon_fire_resource,
                                &lazy_update,
                                arena_element,
                            );
                        }
                    }
                }
            }
        }

        //update weapon fire angle after potential bouncing
        for (weapon_fire, weapon_fire_transform) in (&weapon_fires, &mut transforms).join() {
            if !weapon_fire.stats.heat_seeking {
                let angle = clean_angle(weapon_fire.dy.atan2(weapon_fire.dx) - (PI / 2.0));

                weapon_fire_transform.set_rotation_2d(angle);
            }
        }

        //global rules weapon box spawns
        if game_weapon_setup.random_weapon_spawns {
            if self.global_weapon_spawner_cooldown_timer <= 0.0 {
                self.global_weapon_spawner_cooldown_timer =
                    game_weapon_setup.random_weapon_spawn_timer;

                let mut rng = rand::thread_rng();
                //Filter to only spawn from the spawners that don't have their own timer
                let mut random_weapon_spawn_boxes: Vec<&ArenaElement> = Vec::new();

                for arena_element in (&arena_elements).join() {
                    if arena_element.is_weapon_spawn_point && arena_element.spawn_time.is_none() {
                        random_weapon_spawn_boxes.push(arena_element);
                    }
                }
                let number_of_random_spawn_locations = random_weapon_spawn_boxes.len();

                let mut available_indices =
                    (0..number_of_random_spawn_locations).collect::<Vec<usize>>();

                for _idx in 0..game_weapon_setup
                    .random_weapon_spawn_count
                    .min(number_of_random_spawn_locations as u32)
                {
                    let remove_index = rng.gen_range(0, available_indices.len()) as usize;
                    available_indices.remove(remove_index);
                }

                log::debug!("available_locations: {}", number_of_random_spawn_locations);
                log::debug!(
                    "spawns_required: {}",
                    game_weapon_setup
                        .random_weapon_spawn_count
                        .min(number_of_random_spawn_locations as u32)
                );
                log::debug!("spawn_indices: {:?}", available_indices);

                for (idx, arena_element) in random_weapon_spawn_boxes.iter().enumerate() {
                    if available_indices.contains(&idx) {
                        spawn_weapon_box_from_spawner(
                            &entities,
                            &weapon_fire_resource,
                            &lazy_update,
                            arena_element,
                        );
                    }
                }
            }
        }

        //weapon to vehicle collisions
        let mut player_makes_hit_map = HashMap::new();
        let mut player_makes_kill_map = HashMap::new();
        let mut player_got_killed_map = HashMap::new();

        let mut explosion_map: Vec<(usize, WeaponFire, f32, f32)> = Vec::new();
        let mut chain_map: Vec<(WeaponFire, f32, f32)> = Vec::new();

        for (player, vehicle, vehicle_transform) in
            (&mut players, &mut vehicles, &transforms).join()
        {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            let vehicle_rotation = vehicle_transform.rotation();
            let (_, _, vehicle_angle) = vehicle_rotation.euler_angles();

            let vehicle_collider_shape =
                Cuboid::new(Vector2::new(vehicle.width / 2.0, vehicle.height / 2.0));
            let vehicle_collider_pos =
                Isometry2::new(Vector2::new(vehicle_x, vehicle_y), vehicle_angle);

            player.last_hit_timer += dt;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                (&*entities, &mut weapon_fires, &transforms).join()
            {
                if weapon_fire.owner_player_id.is_none()
                    || weapon_fire.owner_player_id.unwrap() != player.id
                {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(
                        weapon_fire.width / 2.0,
                        weapon_fire.height / 2.0,
                    ));
                    let fire_collider_pos =
                        Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);

                    //use for something like AoE damage
                    if weapon_fire.stats.trigger_immediately {
                        explosion_map.push((
                            player.id.clone(),
                            weapon_fire.clone(),
                            fire_x,
                            fire_y,
                        ));

                        let position = Vector3::new(fire_x, fire_y, 0.5);
                        explosion_shockwave(
                            &entities,
                            &weapon_fire_resource,
                            position,
                            weapon_fire.stats.damage_radius,
                            &lazy_update,
                        );

                        let _ = entities.delete(weapon_fire_entity);
                    }

                    let weapon_fire_hit;
                    if weapon_fire.stats.trigger_radius > 0.0 {
                        //use old lightweight detection algorithm
                        if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                            < (vehicle.width / 2.0 + weapon_fire.stats.trigger_radius).powi(2)
                        {
                            weapon_fire_hit = true;
                        } else {
                            weapon_fire_hit = false;
                        }
                    } else if weapon_fire.stats.shot_speed == 0.0 {
                        //use old lightweight detection algorithm
                        if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                            < vehicle.width.powi(2)
                        {
                            weapon_fire_hit = true;
                        } else {
                            weapon_fire_hit = false;
                        }
                    } else {
                        //use ncollide algorithm
                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        let margin;
                        if abs_vel > SHOT_SPEED_TRIGGER {
                            //use pre-impact detection if within 3 time steps of radius of target
                            margin = abs_vel * dt * PRE_IMPACT_DT_STEPS;
                        } else {
                            margin = 0.0;
                        }

                        let collision = query::proximity(
                            &fire_collider_pos,
                            &fire_collider_shape,
                            &vehicle_collider_pos,
                            &vehicle_collider_shape,
                            margin,
                        );

                        if collision == Proximity::Intersecting {
                            weapon_fire_hit = true;
                        } else if collision == Proximity::WithinMargin {
                            //if potentially on collision course, check time to impact
                            let fire_ray = Ray::new(
                                Point2::new(fire_x, fire_y),
                                Vector2::new(weapon_fire.dx, weapon_fire.dy),
                            );

                            let toi = vehicle_collider_shape.toi_with_ray(
                                &vehicle_collider_pos,
                                &fire_ray,
                                dt * PRE_IMPACT_DT_STEPS,
                                true,
                            );

                            if let Some(toi_result) = toi {
                                if toi_result <= PRE_IMPACT_DT_STEPS * dt {
                                    weapon_fire_hit = true;
                                } else {
                                    weapon_fire_hit = false;
                                }
                            } else {
                                weapon_fire_hit = false;
                            }
                        } else {
                            weapon_fire_hit = false;
                        }
                    }

                    if weapon_fire_hit {
                        player.last_hit_by_id = weapon_fire.owner_player_id.clone();
                        player.last_hit_timer = 0.0;

                        player_makes_hit_map.insert(weapon_fire.owner_player_id.clone(), player.id);

                        let damage: f32 = weapon_fire.stats.damage;
                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            weapon_fire.owner_player_id.clone(),
                            Some(weapon_fire.weapon_name),
                            damage,
                            weapon_fire.stats.piercing_damage_pct,
                            weapon_fire.stats.shield_damage_pct,
                            weapon_fire.stats.armor_damage_pct,
                            weapon_fire.stats.health_damage_pct,
                            weapon_fire.stats.duration_damage,
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

                        //Pass slow-down effect from weapon to vehicle
                        if weapon_fire.stats.slow_down_effect.timer > 0.0 {
                            vehicle.restricted_velocity_timer =
                                weapon_fire.stats.slow_down_effect.timer;
                            vehicle.restricted_max_velocity = vehicle.max_velocity
                                * (1.0
                                    - (weapon_fire.stats.slow_down_effect.slow_down_pct / 100.0));
                        }
                        //Pass stuck accel effect from weapon to vehicle
                        if weapon_fire.stats.stuck_accel_effect_timer > 0.0 {
                            vehicle.stuck_accel_effect_timer =
                                weapon_fire.stats.stuck_accel_effect_timer;
                        }
                        //Pass ion malfunction effect from weapon to vehicle
                        if vehicle.shield.value == 0.0
                            && weapon_fire.stats.ion_malfunction_pct > 0.0
                        {
                            vehicle.ion_malfunction_pct = weapon_fire.stats.ion_malfunction_pct;
                        }

                        if weapon_fire.stats.damage_radius > 0.0 {
                            //spawn explosion entity and sprite
                            //check for hits below in a new join loop on vehicles and explosions
                            explosion_map.push((
                                player.id.clone(),
                                weapon_fire.clone(),
                                fire_x,
                                fire_y,
                            ));

                            let position = Vector3::new(fire_x, fire_y, 0.5);
                            explosion_shockwave(
                                &entities,
                                &weapon_fire_resource,
                                position,
                                weapon_fire.stats.damage_radius,
                                &lazy_update,
                            );
                        }

                        if weapon_fire.stats.chaining_damage.jumps > 0 {
                            let mut weapon_fire_chain_prong = weapon_fire.clone();

                            weapon_fire_chain_prong
                                .chain_hit_ids
                                .push(player.id.clone());

                            chain_map.push((weapon_fire_chain_prong, vehicle_x, vehicle_y));
                        }

                        if self.hit_spray_cooldown_timer < 0.0
                            && vehicle.state == VehicleState::Active
                        {
                            let position = Vector3::new(fire_x, fire_y, 0.5);
                            let shields_up = vehicle.shield.value > 0.0;
                            hit_spray(
                                &entities,
                                &weapon_fire_resource,
                                shields_up,
                                position,
                                &lazy_update,
                            );
                            self.hit_spray_cooldown_timer = HIT_SPRAY_COOLDOWN_RESET;
                        }
                        if self.hit_sound_cooldown_timer < 0.0
                            && vehicle.state == VehicleState::Active
                        {
                            play_score_sound(&*sounds, &storage, audio_output.as_deref());
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                        if !weapon_fire.stats.attached {
                            let _ = entities.delete(weapon_fire_entity);
                            weapon_fire.active = false;
                        }
                    }
                }
            }
        }

        //apply chain effect
        for (weapon_fire, hit_x, hit_y) in &chain_map {
            let mut vehicles_within_chain_radius: Vec<(f32, f32, f32, f32)> = Vec::new();

            for (player, vehicle, vehicle_transform) in
                (&mut players, &mut vehicles, &transforms).join()
            {
                if vehicle.state == VehicleState::Active {
                    let vehicle_x = vehicle_transform.translation().x;
                    let vehicle_y = vehicle_transform.translation().y;

                    if !weapon_fire.chain_hit_ids.contains(&player.id)
                        && (weapon_fire.owner_player_id.is_none()
                            || weapon_fire.owner_player_id.unwrap() != player.id)
                    {
                        if (hit_x - vehicle_x).powi(2) + (hit_y - vehicle_y).powi(2)
                            < (vehicle.width / 2.0 + weapon_fire.stats.chaining_damage.radius)
                                .powi(2)
                        {
                            let dist =
                                ((hit_x - vehicle_x).powi(2) + (hit_y - vehicle_y).powi(2)).sqrt();

                            let vehicle_size_offset = vehicle.height.max(vehicle.width);

                            vehicles_within_chain_radius.push((
                                dist,
                                vehicle_x,
                                vehicle_y,
                                vehicle_size_offset,
                            ));
                        }
                    }
                }
            }

            let mut prongs_remaining = weapon_fire.stats.chaining_damage.prongs.clone();

            vehicles_within_chain_radius.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            for (_dist, vehicle_x, vehicle_y, vehicle_size_offset) in
                vehicles_within_chain_radius.iter()
            {
                let mut weapon_fire_chain_prong = weapon_fire.clone();

                if prongs_remaining > 0 {
                    //spawn chain prong
                    weapon_fire_chain_prong.stats.damage *=
                        weapon_fire_chain_prong.stats.chaining_damage.damage_pct / 100.;
                    weapon_fire_chain_prong.stats.chaining_damage.jumps -= 1;
                    weapon_fire_chain_prong.stats.shot_life_limit =
                        weapon_fire_chain_prong.stats.chaining_damage.radius
                            / weapon_fire_chain_prong.stats.shot_speed;

                    let diff_x = hit_x - vehicle_x;
                    let diff_y = hit_y - vehicle_y;

                    let fire_angle = diff_y.atan2(diff_x) + (PI / 2.0);
                    //rotate by PI/2 to line up with 0deg is pointed towards top

                    let angle_x_comp: f32 = -fire_angle.sin();
                    let angle_y_comp: f32 = fire_angle.cos();
                    let x_offset = vehicle_size_offset * angle_x_comp;
                    let y_offset = vehicle_size_offset * angle_y_comp;

                    let init_x = *hit_x;
                    let init_y = *hit_y;

                    let fire_position = Vector3::new(init_x + x_offset, init_y + y_offset, 0.0);

                    chain_fire_weapon(
                        &entities,
                        &weapon_fire_resource,
                        weapon_fire_chain_prong.clone(),
                        fire_position,
                        fire_angle,
                        0.0,
                        weapon_fire_chain_prong.owner_player_id.clone(),
                        &lazy_update,
                    );

                    prongs_remaining -= 1;
                }
            }
        }

        //apply splash explosion damage
        for (player, vehicle, vehicle_transform) in
            (&mut players, &mut vehicles, &transforms).join()
        {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            for (player_id_already_hit, weapon_fire, fire_x, fire_y) in &explosion_map {
                if player.id != *player_id_already_hit {
                    if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                        < (vehicle.width / 2.0 + weapon_fire.stats.damage_radius).powi(2)
                    {
                        player.last_hit_by_id = weapon_fire.owner_player_id.clone();
                        player.last_hit_timer = 0.0;

                        let damage: f32 = weapon_fire.stats.damage;
                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            weapon_fire.owner_player_id.clone(),
                            Some(weapon_fire.weapon_name),
                            damage,
                            weapon_fire.stats.piercing_damage_pct,
                            weapon_fire.stats.shield_damage_pct,
                            weapon_fire.stats.armor_damage_pct,
                            weapon_fire.stats.health_damage_pct,
                            weapon_fire.stats.duration_damage,
                        );
                        if vehicle_destroyed && vehicle.state == VehicleState::Active {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                            if let Some(owner_player_id) = weapon_fire.owner_player_id {
                                if owner_player_id != player.id {
                                    player_makes_kill_map.insert(
                                        weapon_fire.owner_player_id.clone(),
                                        weapon_fire.weapon_name.clone(),
                                    );
                                }
                            }
                            player_got_killed_map
                                .insert(player.id.clone(), weapon_fire.owner_player_id.clone());
                        }

                        //Pass slow-down effect from weapon to vehicle
                        if weapon_fire.stats.slow_down_effect.timer > 0.0 {
                            vehicle.restricted_velocity_timer =
                                weapon_fire.stats.slow_down_effect.timer;
                            vehicle.restricted_max_velocity = vehicle.max_velocity
                                * (1.0
                                    - (weapon_fire.stats.slow_down_effect.slow_down_pct / 100.0));
                        }
                        //Pass stuck accel effect from weapon to vehicle
                        if weapon_fire.stats.stuck_accel_effect_timer > 0.0 {
                            vehicle.stuck_accel_effect_timer =
                                weapon_fire.stats.stuck_accel_effect_timer;
                        }
                        //Pass ion malfunction effect from weapon to vehicle
                        if vehicle.shield.value == 0.0
                            && weapon_fire.stats.ion_malfunction_pct > 0.0
                        {
                            vehicle.ion_malfunction_pct = weapon_fire.stats.ion_malfunction_pct;
                        }

                        if self.hit_spray_cooldown_timer < 0.0
                            && vehicle.state == VehicleState::Active
                        {
                            let position = Vector3::new(*fire_x, *fire_y, 0.5);
                            let shields_up = vehicle.shield.value > 0.0;
                            hit_spray(
                                &entities,
                                &weapon_fire_resource,
                                shields_up,
                                position,
                                &lazy_update,
                            );
                            self.hit_spray_cooldown_timer = HIT_SPRAY_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        //Kill tracking and gun-game weapon hot swap logic
        let mut weapon_icons_old_map = HashMap::new();

        for (player, weapon_array, vehicle, transform) in
            (&mut players, &weapon_arrays, &mut vehicles, &mut transforms).join()
        {
            let hit_data = player_makes_hit_map.get(&Some(player.id));

            if let Some(_hit_data) = hit_data {
                player.last_made_hit_timer = 0.0;
            }

            let killer_data = player_makes_kill_map.get(&Some(player.id));

            if let Some(killer_data) = killer_data {
                let weapon_name = killer_data;

                if weapon_array.installed.len() > 0 {
                    let primary_weapon = &weapon_array.installed[0].weapon;

                    //classic gun-game rules: hot-swap upgrade weapon type for player who got the kill
                    if game_mode_setup.game_mode == GameModes::ClassicGunGame
                        && *weapon_name == primary_weapon.name.clone()
                    {
                        player.gun_game_kills += 1; //handle gun game kills in special way below
                    } else {
                        player.kills += 1; //in all other modes the kill always counts
                    }
                }
            }

            let killed_data = player_got_killed_map.get(&player.id);

            if let Some(_killed_data) = killed_data {
                player.deaths += 1;

                kill_restart_vehicle(player, vehicle, transform, game_mode_setup.stock_lives);
            }
        }

        //Apply duration damage, such as poison/burns
        let mut player_earned_duration_damage_kill: HashMap<(usize, WeaponNames), u32> =
            HashMap::new();

        for (player, vehicle, vehicle_transform) in
            (&mut players, &mut vehicles, &transforms).join()
        {
            let mut vehicle_destroyed = false;
            let mut duration_damage_list = vehicle.duration_damages.clone();

            for (damager_id, weapon_name, duration_damage) in duration_damage_list.iter() {
                if duration_damage.timer > 0.0 {
                    let duration_damage_vehicle_destroyed: bool = vehicle_damage_model(
                        vehicle,
                        *damager_id,
                        *weapon_name,
                        duration_damage.damage_per_second.clone() * dt,
                        duration_damage.piercing_damage_pct.clone(),
                        duration_damage.shield_damage_pct.clone(),
                        duration_damage.armor_damage_pct.clone(),
                        duration_damage.health_damage_pct.clone(),
                        DurationDamage::default(), //These are zero/default so as to not re-apply the effect to the vehicle.
                                                   //Otherwise this duration damage effect would stack continuously.
                    );

                    if duration_damage_vehicle_destroyed {
                        vehicle_destroyed = true;

                        if let Some(damager_id) = damager_id {
                            if let Some(weapon_name) = weapon_name {
                                *player_earned_duration_damage_kill
                                    .entry((*damager_id, *weapon_name))
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                }
            }

            //Remove duration damages that have expired
            let mut lasting_duration_damages = Vec::new();

            for (damager_id, weapon_name, duration_damage) in duration_damage_list.iter_mut() {
                duration_damage.timer -= dt;

                if duration_damage.timer > 0.0 {
                    lasting_duration_damages.push((*damager_id, *weapon_name, *duration_damage));
                }
            }

            vehicle.duration_damages = lasting_duration_damages;

            if vehicle_destroyed {
                player.deaths += 1;

                kill_restart_vehicle(
                    player,
                    vehicle,
                    vehicle_transform,
                    game_mode_setup.stock_lives,
                );
            }
        }

        //Check for gun-game kills that haven't been awarded yet
        for (player, mut weapon_array, vehicle) in
            (&mut players, &mut weapon_arrays, &mut vehicles).join()
        {
            if weapon_array.installed.len() > 0 {
                let primary_weapon = &weapon_array.installed[PRIMARY_WEAPON_INDEX].weapon;
                //Update kills from duration damage effects
                let kills_data =
                    player_earned_duration_damage_kill.get(&(player.id, primary_weapon.name));

                if let Some(kills) = kills_data {
                    if game_mode_setup.game_mode == GameModes::ClassicGunGame {
                        player.gun_game_kills += 1;
                    } else {
                        player.kills += *kills as i32;
                    }
                }

                if player.gun_game_kills > player.kills {
                    //if kill was using player's current weapon
                    player.kills += 1;
                    let new_weapon_name = get_next_gg_weapon_name(
                        primary_weapon.name.clone(),
                        &weapon_store_resource,
                        &game_weapon_setup,
                    );

                    if let Some(new_weapon_name) = new_weapon_name.clone() {
                        weapon_icons_old_map.insert(
                            player.id,
                            (
                                PRIMARY_WEAPON_INDEX,
                                primary_weapon.stats.weapon_fire_type.clone(),
                            ),
                        );

                        update_weapon_properties(
                            &mut weapon_array,
                            PRIMARY_WEAPON_INDEX,
                            0,
                            None,
                            Some(new_weapon_name),
                            &weapon_store_resource,
                            &entities,
                            &weapon_fire_resource,
                            Some(player.id),
                            &lazy_update,
                        );

                        if weapon_array.installed.len() > 0 {
                            vehicle.weapon_weight = weapon_array.installed[PRIMARY_WEAPON_INDEX]
                                .weapon
                                .stats
                                .weight;
                        }
                    } //else, keep current weapon installed, no kill in this mode
                }
            }
        }

        //Remove inactive Weapon UI Icons
        for (entity, player_icon) in (&*entities, &player_weapon_icons).join() {
            let weapon_icons_old = weapon_icons_old_map.get(&player_icon.player_id);

            if let Some(weapon_icons_old) = weapon_icons_old {
                let (weapon_id, weapon_fire_type) = weapon_icons_old;

                if *weapon_id == player_icon.weapon_id
                    && *weapon_fire_type == player_icon.weapon_fire_type
                {
                    let _ = entities.delete(entity);
                }
            }
        }

        self.hit_sound_cooldown_timer -= dt;
        self.hit_spray_cooldown_timer -= dt;
    }
}
