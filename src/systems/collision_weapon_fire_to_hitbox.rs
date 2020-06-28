use amethyst::{
    assets::AssetStorage,
    audio::{output::Output, Source},
    core::{Time, Transform},
    core::math::Vector3,
    derive::SystemDesc,
    ecs::{
        Entities, Join, LazyUpdate, Read, ReadExpect, ReadStorage, System, SystemData, World,
        WriteStorage,
    },
};

use std::f32::consts::PI;
use std::collections::HashMap;

extern crate nalgebra as na;
use na::{Isometry2, Vector2, Point2};
use ncollide2d::query::{self, Proximity, Ray, RayCast};
use ncollide2d::shape::{Ball, Cuboid};

use crate::components::{
    get_next_weapon_name, kill_restart_vehicle, update_weapon_properties,
    vehicle_damage_model, Hitbox, HitboxShape, Player, PlayerWeaponIcon, Vehicle, VehicleState, WeaponArray,
    WeaponFire, WeaponStoreResource,
};

use crate::entities::{spawn_weapon_boxes, hit_spray, explosion_shockwave, chain_fire_weapon};

use crate::resources::{GameModeSetup, GameModes, GameWeaponSetup, WeaponFireResource};

use crate::audio::{play_bounce_sound, play_score_sound, Sounds};

pub const HIT_SOUND_COOLDOWN_RESET: f32 = 0.30;
pub const HIT_SPRAY_COOLDOWN_RESET: f32 = 0.05;

pub const PRE_IMPACT_DT_STEPS: f32 = 1.2;
pub const SHOT_SPEED_TRIGGER: f32 = 500.0;


#[derive(SystemDesc, Default)]
pub struct CollisionWeaponFireHitboxSystem {
    pub hit_sound_cooldown_timer: f32,
    pub hit_spray_cooldown_timer: f32,
    pub weapon_spawner_cooldown_timer: f32,
}

impl<'s> System<'s> for CollisionWeaponFireHitboxSystem {
    type SystemData = (
        Entities<'s>,
        ReadStorage<'s, Hitbox>,
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
            self.weapon_spawner_cooldown_timer = game_weapon_setup.weapon_spawn_first_timer;
        }
        else {
            self.weapon_spawner_cooldown_timer = 20.0;
        }
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

        if game_weapon_setup.random_weapon_spawns
            && game_mode_setup.game_mode != GameModes::ClassicGunGame
        {
            self.weapon_spawner_cooldown_timer -= dt;
        }

        //weapon to non-moving hitbox collisions
        for (entity, hitbox, transform) in (&*entities, &hitboxes, &transforms).join() {
            if hitbox.is_wall {
                let hitbox_x = transform.translation().x;
                let hitbox_y = transform.translation().y;

                for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                    (&*entities, &mut weapon_fires, &transforms).join()
                {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(weapon_fire.width/2.0, weapon_fire.height/2.0));
                    let fire_collider_pos = Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);

                    let collision;

                    let hitbox_collider_shape_circle = Ball::new(hitbox.width / 2.0);
                    let hitbox_collider_shape_rect = Cuboid::new(Vector2::new(hitbox.width/2.0, hitbox.height/2.0)); //unused
                    let hitbox_collider_pos = Isometry2::new(Vector2::new(hitbox_x, hitbox_y), 0.0);

                    let weapon_fire_hit;
                    if weapon_fire.shot_speed == 0.0 {
                        weapon_fire_hit = false; //assumes hitboxes don't move
                    }
                    else { //use ncollide algorithm
                        let margin;

                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        if abs_vel > SHOT_SPEED_TRIGGER {
                            //use pre-impact detection if within 3 time steps of radius of target
                            margin = abs_vel * dt * PRE_IMPACT_DT_STEPS;
                        }
                        else {
                            margin = 0.0;
                        }

                        if hitbox.shape == HitboxShape::Circle {
                            collision = query::proximity(
                                &fire_collider_pos, &fire_collider_shape,
                                &hitbox_collider_pos, &hitbox_collider_shape_circle,
                                margin,
                            );
                        } else { //if hitbox.shape == HitboxShape::Rectangle {
                            collision = query::proximity(
                                &fire_collider_pos, &fire_collider_shape,
                                &hitbox_collider_pos, &hitbox_collider_shape_rect,
                                margin,
                            );
                        }

                        
                        if collision == Proximity::Intersecting {
                            weapon_fire_hit = true;
                        }
                        else if collision == Proximity::WithinMargin {
                            //if potentially on collision course, check time to impact
                            let fire_ray = Ray::new(Point2::new(fire_x, fire_y), Vector2::new(weapon_fire.dx, weapon_fire.dy));

                            let toi;
                            if hitbox.shape == HitboxShape::Circle {
                                toi = hitbox_collider_shape_circle.toi_with_ray(&hitbox_collider_pos, &fire_ray, dt * PRE_IMPACT_DT_STEPS, true);
                            }
                            else {
                                toi = hitbox_collider_shape_rect.toi_with_ray(&hitbox_collider_pos, &fire_ray, dt * PRE_IMPACT_DT_STEPS, true);
                            }

                            if let Some(toi_result) = toi {
                                if toi_result <= PRE_IMPACT_DT_STEPS*dt {
                                    weapon_fire_hit = true;
                                }
                                else {
                                    weapon_fire_hit = false;
                                }
                            }
                            else {
                                weapon_fire_hit = false;
                            }
                        }
                        else {
                            weapon_fire_hit = false;
                        }
                    }

                    if weapon_fire_hit {
                        if !weapon_fire.attached {
                            if weapon_fire.bounces > 0 {
                                weapon_fire.bounces -= 1;

                                let contact_data;
                                if hitbox.shape == HitboxShape::Circle 
                                {
                                    contact_data = query::contact(
                                        &fire_collider_pos, &fire_collider_shape,
                                        &hitbox_collider_pos, &hitbox_collider_shape_circle,
                                        0.0);
                                }
                                else {
                                    contact_data = query::contact(
                                        &fire_collider_pos, &fire_collider_shape,
                                        &hitbox_collider_pos, &hitbox_collider_shape_rect,
                                        0.0);
                                }
        
                                let contact_pt = contact_data.unwrap().world2;

                                //Input:
                                //contact_pt.x, contact_pt.y
                                //hitbox_x, hitbox_y

                                //Output:
                                //weapon_fire.dx, weapon_fire.dy
                                //weapon_fire_transform.set_rotation_2d

                                
                                let x_diff = hitbox_x - contact_pt.x;
                                let y_diff = hitbox_y - contact_pt.y;

                                let mut new_angle;
                                if hitbox.shape == HitboxShape::Circle 
                                {
                                    let mut contact_perp_angle = y_diff.atan2(x_diff) + PI/2.0;
                                    if contact_perp_angle > PI {
                                        contact_perp_angle -= 2.0*PI;
                                    }

                                    log::info!("fire_angle: {}", fire_angle/PI*180.0);
                                    log::info!("contact_perp_angle: {}", contact_perp_angle/PI*180.0);

                                    new_angle = contact_perp_angle - fire_angle - PI/2.0;

                                    log::info!("new_angle: {}", new_angle/PI*180.0);

                                    if new_angle > PI {
                                        new_angle -= 2.0*PI;
                                    }
                                    else if new_angle < -PI {
                                        new_angle += 2.0*PI;
                                    }

                                    log::info!("new_angle: {}", new_angle/PI*180.0);
                                }
                                else 
                                {
                                    let contact_perp_angle = y_diff.atan2(x_diff);

                                    new_angle = contact_perp_angle - fire_angle;
                                }

                                let weapon_fire_speed = (weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2)).sqrt();

                                weapon_fire.dx = weapon_fire_speed*-new_angle.sin();
                                weapon_fire.dy = weapon_fire_speed*new_angle.cos();
                                
                                //weapon_fire_transform.set_rotation_2d(new_angle);


                                // if (fire_x > (ARENA_WIDTH + 2.0 * weapon_fire.width))
                                //     || (fire_x < (-2.0 * weapon_fire.width)) 
                                // {
                                //     weapon_fire.dx *= -1.0;

                                //     let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                //     //rotate by PI/2 to line up with 0deg is pointed towards top
                                    
                                //     transform.set_rotation_2d(new_angle);
                                // }
                                // else if (fire_y > (ARENA_HEIGHT + 2.0 * weapon_fire.width))
                                //     || (fire_y < (UI_HEIGHT - 2.0 * weapon_fire.width))
                                // {
                                //     weapon_fire.dy *= -1.0;

                                //     let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                //     //rotate by PI/2 to line up with 0deg is pointed towards top
                                    
                                //     transform.set_rotation_2d(new_angle);
                                // }
                            }
                            else {
                                let _ = entities.delete(weapon_fire_entity);
                                weapon_fire.active = false;
                            }   
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


        //update weapon fire angle after potential bouncing
        for (weapon_fire, weapon_fire_transform) in
            (&weapon_fires, &mut transforms).join()
        {
            let angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI/2.0);

            weapon_fire_transform.set_rotation_2d(angle);
        }
        


        //weapon box spawns
        if game_weapon_setup.random_weapon_spawns
            && game_mode_setup.game_mode != GameModes::ClassicGunGame
        {
            if self.weapon_spawner_cooldown_timer <= 0.0 {
                self.weapon_spawner_cooldown_timer = game_weapon_setup.weapon_spawn_timer;

                spawn_weapon_boxes(
                    &entities,
                    &weapon_fire_resource,
                    &lazy_update,
                    game_weapon_setup.weapon_spawn_count.clone(),
                    game_mode_setup.game_mode.clone(),
                );
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

            let vehicle_collider_shape = Cuboid::new(Vector2::new(vehicle.width/2.0, vehicle.height/2.0));
            let vehicle_collider_pos = Isometry2::new(Vector2::new(vehicle_x, vehicle_y), vehicle_angle);

            player.last_hit_timer += dt;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                (&*entities, &mut weapon_fires, &transforms).join()
            {
                if weapon_fire.owner_player_id != player.id {
                    let fire_x = weapon_fire_transform.translation().x;
                    let fire_y = weapon_fire_transform.translation().y;

                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();

                    let fire_collider_shape = Cuboid::new(Vector2::new(weapon_fire.width/2.0, weapon_fire.height/2.0));
                    let fire_collider_pos = Isometry2::new(Vector2::new(fire_x, fire_y), fire_angle);


                    //use for something like AoE damage
                    if weapon_fire.trigger_immediately {
                        explosion_map
                            .push((player.id.clone(), weapon_fire.clone(), fire_x, fire_y));

                        let position = Vector3::new(fire_x, fire_y, 0.5);
                        
                        explosion_shockwave(
                            &entities,
                            &weapon_fire_resource,
                            position,
                            weapon_fire.damage_radius, 
                            &lazy_update,
                        );

                        let _ = entities.delete(weapon_fire_entity);
                    }

                    let weapon_fire_hit;
                    if weapon_fire.trigger_radius > 0.0 {
                        //use old lightweight detection algorithm
                        if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                                < (vehicle.width/2.0 + weapon_fire.trigger_radius).powi(2) {
                            weapon_fire_hit = true;
                        }
                        else {
                            weapon_fire_hit = false;
                        }
                    }
                    else if weapon_fire.shot_speed == 0.0 {
                        //use old lightweight detection algorithm
                        if (fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                                < vehicle.width.powi(2) {
                            weapon_fire_hit = true;
                        }
                        else {
                            weapon_fire_hit = false;
                        }
                    }
                    else {
                        //use ncollide algorithm
                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        let margin;
                        if abs_vel > SHOT_SPEED_TRIGGER {
                            //use pre-impact detection if within 3 time steps of radius of target
                            margin = abs_vel * dt * PRE_IMPACT_DT_STEPS;
                        }
                        else {
                            margin = 0.0;
                        }

                        let collision = query::proximity(
                            &fire_collider_pos, &fire_collider_shape,
                            &vehicle_collider_pos, &vehicle_collider_shape,
                            margin,
                        );

                        
                        if collision == Proximity::Intersecting {
                            weapon_fire_hit = true;
                        }
                        else if collision == Proximity::WithinMargin {
                            //if potentially on collision course, check time to impact
                            let fire_ray = Ray::new(Point2::new(fire_x, fire_y), Vector2::new(weapon_fire.dx, weapon_fire.dy));

                            let toi = vehicle_collider_shape.toi_with_ray(&vehicle_collider_pos, &fire_ray, dt * PRE_IMPACT_DT_STEPS, true);

                            if let Some(toi_result) = toi {
                                if toi_result <= PRE_IMPACT_DT_STEPS*dt {
                                    weapon_fire_hit = true;
                                }
                                else {
                                    weapon_fire_hit = false;
                                }
                            }
                            else {
                                weapon_fire_hit = false;
                            }
                        }
                        else {
                            weapon_fire_hit = false;
                        }
                    }

                    if weapon_fire_hit {
                        player.last_hit_by_id = Some(weapon_fire.owner_player_id.clone());
                        player.last_hit_timer = 0.0;

                        player_makes_hit_map.insert(weapon_fire.owner_player_id.clone(), player.id);


                        let damage: f32 = weapon_fire.damage;
                    
                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            weapon_fire.piercing_damage_pct,
                            weapon_fire.shield_damage_pct,
                            weapon_fire.armor_damage_pct,
                            weapon_fire.health_damage_pct,
                            weapon_fire.duration_damage,
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
                        if weapon_fire.slow_down_effect.timer > 0.0 {
                            vehicle.restricted_velocity_timer = weapon_fire.slow_down_effect.timer;
                            vehicle.restricted_max_velocity = vehicle.max_velocity * (1.0 - (weapon_fire.slow_down_effect.slow_down_pct/100.0));
                        }
                    
                        //Pass stuck accel effect from weapon to vehicle
                        if weapon_fire.stuck_accel_effect_timer > 0.0 {
                            vehicle.stuck_accel_effect_timer = weapon_fire.stuck_accel_effect_timer;
                        }
                    
                        //Pass ion malfunction effect from weapon to vehicle
                        if vehicle.shield.value == 0.0 && weapon_fire.ion_malfunction_pct > 0.0 {
                            vehicle.ion_malfunction_pct = weapon_fire.ion_malfunction_pct;
                        }

                        if weapon_fire.damage_radius > 0.0 {
                            //spawn explosion entity and sprite
                            //check for hits below in a new join loop on vehicles and explosions
                            explosion_map
                                .push((player.id.clone(), weapon_fire.clone(), fire_x, fire_y));

                            let position = Vector3::new(fire_x, fire_y, 0.5);
                            
                            explosion_shockwave(
                                &entities,
                                &weapon_fire_resource,
                                position,
                                weapon_fire.damage_radius, 
                                &lazy_update,
                            );
                        }


                        if weapon_fire.chaining_damage.jumps > 0 {
                            let mut weapon_fire_chain_prong = weapon_fire.clone();

                            weapon_fire_chain_prong.chain_hit_ids.push(player.id.clone());

                            chain_map.push((
                                weapon_fire_chain_prong,
                                vehicle_x,
                                vehicle_y,
                            ));
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
                    
                        if !weapon_fire.attached {
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

                    if !weapon_fire.chain_hit_ids.contains(&player.id) && player.id != weapon_fire.owner_player_id {
                        if (hit_x - vehicle_x).powi(2) + (hit_y - vehicle_y).powi(2)
                            < (vehicle.width/2.0 + weapon_fire.chaining_damage.radius).powi(2)
                        {
                            let dist = ((hit_x - vehicle_x).powi(2) + (hit_y - vehicle_y).powi(2)).sqrt();

                            let vehicle_size_offset = vehicle.height.max(vehicle.width);

                            vehicles_within_chain_radius.push((dist, vehicle_x, vehicle_y, vehicle_size_offset));
                        }
                    }
                }
            }

            let mut prongs_remaining = weapon_fire.chaining_damage.prongs.clone();

            vehicles_within_chain_radius.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            for (_dist, vehicle_x, vehicle_y, vehicle_size_offset) in vehicles_within_chain_radius.iter() {
                let mut weapon_fire_chain_prong = weapon_fire.clone();

                if prongs_remaining > 0 {
                    //spawn chain prong
                    weapon_fire_chain_prong.damage *= weapon_fire_chain_prong.chaining_damage.damage_pct/100.;
                    weapon_fire_chain_prong.chaining_damage.jumps -= 1;
                    weapon_fire_chain_prong.shot_life_limit = weapon_fire_chain_prong.chaining_damage.radius / weapon_fire_chain_prong.shot_speed;

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

                    let fire_position = Vector3::new(
                        init_x + x_offset,
                        init_y + y_offset,
                        0.0,
                    );

                    chain_fire_weapon(
                        &entities,
                        &weapon_fire_resource,
                        weapon_fire_chain_prong.clone(),
                        fire_position,
                        fire_angle,
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
                        < (vehicle.width/2.0 + weapon_fire.damage_radius).powi(2) 
                    {
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
                            weapon_fire.duration_damage,
                        );
                    
                        if vehicle_destroyed && vehicle.state == VehicleState::Active {
                            play_bounce_sound(&*sounds, &storage, audio_output.as_deref());
                    
                            if player.id != weapon_fire.owner_player_id.clone() {
                                player_makes_kill_map.insert(
                                    weapon_fire.owner_player_id.clone(),
                                    weapon_fire.weapon_name.clone(),
                                );
                            }
                    
                            player_got_killed_map
                                .insert(player.id.clone(), weapon_fire.owner_player_id.clone());
                        }


                        //Pass slow-down effect from weapon to vehicle
                        if weapon_fire.slow_down_effect.timer > 0.0 {
                            vehicle.restricted_velocity_timer = weapon_fire.slow_down_effect.timer;
                            vehicle.restricted_max_velocity = vehicle.max_velocity * (1.0 - (weapon_fire.slow_down_effect.slow_down_pct/100.0));
                        }
                        
                        //Pass stuck accel effect from weapon to vehicle
                        if weapon_fire.stuck_accel_effect_timer > 0.0 {
                            vehicle.stuck_accel_effect_timer = weapon_fire.stuck_accel_effect_timer;
                        }
                    
                        //Pass ion malfunction effect from weapon to vehicle
                        if vehicle.shield.value == 0.0 && weapon_fire.ion_malfunction_pct > 0.0 {
                            vehicle.ion_malfunction_pct = weapon_fire.ion_malfunction_pct;
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

        for (player, mut weapon_array, vehicle, transform) in
            (&mut players, &mut weapon_arrays, &mut vehicles, &mut transforms).join()
        {
            let hit_data = player_makes_hit_map.get(&player.id);

            if let Some(_hit_data) = hit_data {
                player.last_made_hit_timer = 0.0;
            }

            let killer_data = player_makes_kill_map.get(&player.id);

            if let Some(killer_data) = killer_data {
                let weapon_name = killer_data;

                if let Some(primary_weapon) = &weapon_array.weapons[0] {
                    //classic gun-game rules: hot-swap upgrade weapon type for player who got the kill
                    if game_mode_setup.game_mode == GameModes::ClassicGunGame
                        && *weapon_name == primary_weapon.name.clone()
                    {
                        //if kill was using player's current weapon
                        player.kills += 1;
                        let new_weapon_name = get_next_weapon_name(
                            primary_weapon.name.clone(),
                            &weapon_store_resource,
                        );

                        if let Some(new_weapon_name) = new_weapon_name.clone() {
                            weapon_icons_old_map.insert(player.id, primary_weapon.stats.weapon_type.clone());

                            update_weapon_properties(
                                &mut weapon_array,
                                0,
                                new_weapon_name,
                                &weapon_store_resource,
                                &entities,
                                &weapon_fire_resource,
                                player.id,
                                &lazy_update,
                            );

                            if let Some(new_primary_weapon) = &weapon_array.weapons[0] {
                                vehicle.weapon_weight = new_primary_weapon.stats.weight;
                            }
                        } //else, keep current weapon installed, no kill in this mode
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
        self.hit_spray_cooldown_timer -= dt;
    }
}