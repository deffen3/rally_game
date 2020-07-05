use amethyst::{
    core::{Time, Transform},
    derive::SystemDesc,
    ecs::{Entities, Join, Read, ReadStorage, System, SystemData, WriteStorage, World}
};

use crate::components::{
    Player, Vehicle, VehicleState, WeaponArray, WeaponFire,
    ArenaProperties, ArenaNames, ArenaStoreResource
};
use crate::resources::{GameModeSetup};
use crate::systems::{clean_angle};

use log::debug;
use std::collections::HashMap;
use std::f32::consts::PI;

#[derive(SystemDesc, Default)]
pub struct MoveWeaponFireSystem {
    pub arena_properties: ArenaProperties,
}


impl<'s> System<'s> for MoveWeaponFireSystem {
    type SystemData = (
        Entities<'s>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, WeaponFire>,
        ReadStorage<'s, Vehicle>,
        ReadStorage<'s, WeaponArray>,
        ReadStorage<'s, Player>,
        Read<'s, Time>,
    );

    fn setup(&mut self, world: &mut World) {
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
            }
            else {
                self.arena_properties = ArenaProperties::default();
            }
        }
    }

    fn run(
        &mut self,
        (entities, mut transforms, mut weapon_fires, vehicles, weapon_arrays, players, time): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        let mut vehicle_owner_map = HashMap::new();
        let mut heat_seeking_angle_map = HashMap::new();

        for (entity, weapon_fire, transform) in (&entities, &mut weapon_fires, &transforms).join() {
            weapon_fire.shot_life_timer += dt;
            weapon_fire.stats.damage = (weapon_fire.stats.damage - (weapon_fire.stats.damage*(weapon_fire.stats.damage_reduction_pct_rate/100.0) * dt)).max(0.0);

            if weapon_fire.stats.shot_life_limit >= 0.0
                && weapon_fire.shot_life_timer >= weapon_fire.stats.shot_life_limit
            {
                let _ = entities.delete(entity);
            } else {
                let fire_x = transform.translation().x;
                let fire_y = transform.translation().y;

                if weapon_fire.stats.heat_seeking {
                    let mut closest_vehicle_x_diff = 0.0;
                    let mut closest_vehicle_y_diff = 0.0;
                    let mut closest_vehicle_dist = 1_000_000_000.0;

                    for (vehicle, vehicle_transform, player) in
                        (&vehicles, &transforms, &players).join()
                    {
                        if vehicle.state == VehicleState::Active {
                            if let Some(owner_player_id) = weapon_fire.owner_player_id {
                                if owner_player_id != player.id {
                                    let vehicle_x = vehicle_transform.translation().x;
                                    let vehicle_y = vehicle_transform.translation().y;

                                    // let weapon_rotation = transform.rotation();
                                    // let (_, _, weapon_angle) = weapon_rotation.euler_angles();

                                    let dist = ((vehicle_x - fire_x).powi(2)
                                        + (vehicle_y - fire_y).powi(2))
                                    .sqrt();

                                    if dist < closest_vehicle_dist {
                                        closest_vehicle_dist = dist;
                                        closest_vehicle_x_diff = fire_x - vehicle_x;
                                        closest_vehicle_y_diff = fire_y - vehicle_y;
                                    }
                                }
                            }
                        }
                    }

                    let target_angle = clean_angle(
                        closest_vehicle_y_diff.atan2(closest_vehicle_x_diff) + (PI / 2.0)
                    ); //rotate by PI/2 to line up with yaw angle

                    heat_seeking_angle_map.insert(entity.id(), target_angle);
                }

                if weapon_fire.stats.attached {
                    for (_vehicle, vehicle_transform, weapon_array, player) in
                        (&vehicles, &transforms, &weapon_arrays, &players).join()
                    {
                        if let Some(owner_player_id) = weapon_fire.owner_player_id {
                            if owner_player_id == player.id {
                                let vehicle_rotation = vehicle_transform.rotation();
                                let (_, _, yaw) = vehicle_rotation.euler_angles();

                                for (weapon_idx, weapon_install) in weapon_array.installed.iter().enumerate() {
                                    let weapon = &weapon_install.weapon;

                                    //undeploy old attached weapons
                                    if weapon_fire.weapon_array_id == weapon_idx 
                                            && weapon.name != weapon_fire.weapon_name {
                                        weapon_fire.deployed = false;
                                    }

                                    if weapon.name == weapon_fire.weapon_name && weapon.stats.fire_stats.attached {
                                        //pass on deployed status
                                        if weapon.deployed == false {
                                            weapon_fire.deployed = false;
                                            let _ = entities.delete(entity);
                                        }
                                        else if weapon.deployed == true {
                                            weapon_fire.deployed = true;
                                        }
                                    }

                                    vehicle_owner_map.insert(
                                        weapon_fire.owner_player_id,
                                        (
                                            vehicle_transform.translation().x,
                                            vehicle_transform.translation().y,
                                            yaw,
                                        ),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        for (entity, weapon_fire, transform) in
            (&*entities, &mut weapon_fires, &mut transforms).join()
        {
            if weapon_fire.active {
                if weapon_fire.stats.heat_seeking {
                    let heat_seeking_data = heat_seeking_angle_map.get(&entity.id());

                    if let Some(heat_seeking_data) = heat_seeking_data {
                        let angle = heat_seeking_data;

                        transform.set_rotation_2d(*angle);

                        let velocity_x_comp = -angle.sin(); //left is -, right is +
                        let velocity_y_comp = angle.cos(); //up is +, down is -

                        let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                        let abs_vel = sq_vel.sqrt();

                        weapon_fire.dx += weapon_fire.stats.heat_seeking_agility * velocity_x_comp * dt;
                        weapon_fire.dx *= weapon_fire.stats.shot_speed / abs_vel;

                        weapon_fire.dy += weapon_fire.stats.heat_seeking_agility * velocity_y_comp * dt;
                        weapon_fire.dy *= weapon_fire.stats.shot_speed / abs_vel;
                    }
                }

                if weapon_fire.stats.attached {
                    if weapon_fire.deployed {
                        let vehicle_owner_data = vehicle_owner_map.get(&weapon_fire.owner_player_id);

                        if let Some(vehicle_owner_data) = vehicle_owner_data {
                            let (x, y, vehicle_angle) = vehicle_owner_data;

                            let angle = vehicle_angle + weapon_fire.spawn_angle;

                            let yaw_x_comp = -angle.sin(); //left is -, right is +
                            let yaw_y_comp = angle.cos(); //up is +, down is -

                            debug!("attached: {}, {}, {}", x, y, angle);

                            transform.set_rotation_2d(angle - PI);
                            transform.set_translation_x(x + yaw_x_comp * 14.0);
                            transform.set_translation_y(y + yaw_y_comp * 14.0);
                        }
                    } else {
                        let _ = entities.delete(entity);
                    }
                } else { //move to updated position based on velocity
                    if weapon_fire.stats.shot_speed > 0.0 {
                        if weapon_fire.stats.accel_rate.abs() > 0.0 {
                            let sq_vel = weapon_fire.dx.powi(2) + weapon_fire.dy.powi(2);
                            let abs_vel = sq_vel.sqrt();

                            let new_speed = (abs_vel + weapon_fire.stats.accel_rate * dt).max(0.0);
                            weapon_fire.stats.shot_speed = new_speed;

                            let scalar = new_speed / abs_vel;
                            weapon_fire.dx *= scalar;
                            weapon_fire.dy *= scalar;
                        }

                        transform.prepend_translation_x(weapon_fire.dx * dt);
                        transform.prepend_translation_y(weapon_fire.dy * dt);

                        let fire_x = transform.translation().x;
                        let fire_y = transform.translation().y;

                        //out of arena logic
                        if (fire_x > self.arena_properties.width)
                            || (fire_x < 0.0)
                            || (fire_y > self.arena_properties.height)
                            || (fire_y < 0.0)
                        {
                            if !weapon_fire.stats.attached {
                                if weapon_fire.stats.bounces > 0 {
                                    weapon_fire.stats.bounces -= 1;
                                    weapon_fire.owner_player_id = None;

                                    if (fire_x > self.arena_properties.width)
                                        || (fire_x < 0.0) 
                                    {
                                        weapon_fire.dx *= -1.0;

                                        let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                        //rotate by PI/2 to line up with 0deg is pointed towards top
                                        
                                        transform.set_rotation_2d(new_angle);
                                    }
                                    else if (fire_y > self.arena_properties.height)
                                        || (fire_y < 0.0)
                                    {
                                        weapon_fire.dy *= -1.0;

                                        let new_angle = weapon_fire.dy.atan2(weapon_fire.dx) + (PI / 2.0); 
                                        //rotate by PI/2 to line up with 0deg is pointed towards top
                                        
                                        transform.set_rotation_2d(new_angle);
                                    }
                                }
                                else {
                                    let _ = entities.delete(entity);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
