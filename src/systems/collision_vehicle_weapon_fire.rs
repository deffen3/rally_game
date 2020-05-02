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

use crate::components::{
    get_next_weapon_name, kill_restart_vehicle, update_weapon_icon, update_weapon_properties,
    Hitbox, Player, PlayerWeaponIcon, Vehicle, Weapon, WeaponFire, WeaponTypes,
};

use crate::rally::vehicle_damage_model;
use crate::resources::WeaponFireResource;

use crate::audio::{play_bounce_sound, play_score_sound, Sounds};

pub const HIT_SOUND_COOLDOWN_RESET: f32 = 0.25;

#[derive(SystemDesc, Default)]
pub struct CollisionVehicleWeaponFireSystem {
    pub hit_sound_cooldown_timer: f32,
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
    );

    fn setup(&mut self, _world: &mut World) {
        self.hit_sound_cooldown_timer = -1.0;
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
        ): Self::SystemData,
    ) {
        let dt = time.delta_seconds();

        for (hitbox, transform) in (&hitboxes, &transforms).join() {
            let hitbox_x = transform.translation().x;
            let hitbox_y = transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                (&*entities, &weapon_fires, &transforms).join()
            {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if (fire_x - hitbox_x).powi(2) + (fire_y - hitbox_y).powi(2)
                    < (hitbox.width / 2.0).powi(2)
                {
                    if !weapon_fire.attached {
                        let _ = entities.delete(weapon_fire_entity);
                    }
                }
            }
        }

        //let mut player_makes_kill: Vec<(usize, usize, WeaponNames)> = Vec::new();

        let mut player_makes_kill_map = HashMap::new();
        let mut player_got_killed_map = HashMap::new();

        for (player, vehicle, _weapon, vehicle_transform) in
            (&players, &mut vehicles, &mut weapons, &transforms).join()
        {
            let vehicle_x = vehicle_transform.translation().x;
            let vehicle_y = vehicle_transform.translation().y;

            for (weapon_fire_entity, weapon_fire, weapon_fire_transform) in
                (&*entities, &weapon_fires, &transforms).join()
            {
                let fire_x = weapon_fire_transform.translation().x;
                let fire_y = weapon_fire_transform.translation().y;

                if weapon_fire.owner_player_id != player.id {
                    let fire_rotation = weapon_fire_transform.rotation();
                    let (_, _, fire_angle) = fire_rotation.euler_angles();
                    let fire_x_comp = -fire_angle.sin(); //left is -, right is +
                    let fire_y_comp = fire_angle.cos(); //up is +, down is -

                    if ((fire_x - vehicle_x).powi(2) + (fire_y - vehicle_y).powi(2)
                        < vehicle.width.powi(2))
                        || ((fire_x + fire_x_comp * weapon_fire.height / 2.0 - vehicle_x).powi(2)
                            + (fire_y + fire_y_comp * weapon_fire.height / 2.0 - vehicle_y).powi(2)
                            < vehicle.width.powi(2))
                        || ((fire_x - fire_x_comp * weapon_fire.height / 2.0 - vehicle_x).powi(2)
                            + (fire_y - fire_y_comp * weapon_fire.height / 2.0 - vehicle_y).powi(2)
                            < vehicle.width.powi(2))
                    {
                        if !weapon_fire.attached {
                            let _ = entities.delete(weapon_fire_entity);
                        }

                        let damage: f32 = weapon_fire.damage;

                        let vehicle_destroyed: bool = vehicle_damage_model(
                            vehicle,
                            damage,
                            weapon_fire.piercing_damage_pct,
                            weapon_fire.shield_damage_pct,
                            weapon_fire.armor_damage_pct,
                            weapon_fire.health_damage_pct,
                        );

                        if vehicle_destroyed {
                            play_bounce_sound(
                                &*sounds,
                                &storage,
                                audio_output.as_deref(),
                            );

                            player_makes_kill_map.insert(
                                weapon_fire.owner_player_id.clone(),
                                weapon_fire.weapon_name.clone(),
                            );

                            player_got_killed_map.insert(
                                player.id.clone(),
                                weapon_fire.owner_player_id.clone(),
                            );
                        }

                        if self.hit_sound_cooldown_timer < 0.0 {
                            play_score_sound(
                                &*sounds,
                                &storage,
                                audio_output.as_deref(),
                            );
                            self.hit_sound_cooldown_timer = HIT_SOUND_COOLDOWN_RESET;
                        }
                    }
                }
            }
        }

        let mut weapon_icons_old: Vec<(usize, WeaponTypes)> = Vec::new();

        for (player, mut weapon, vehicle, transform) in
            (&mut players, &mut weapons, &mut vehicles, &mut transforms).join()
        {

            let killer_data = player_makes_kill_map.get(&player.id);

            if let Some(killer_data) = killer_data {
                let weapon_name = killer_data;

                if *weapon_name == weapon.name { //if kill was using player's current weapon

                    //classic gun-game rules: upgrade weapon type for player who got the kill
                    let new_weapon_name = get_next_weapon_name(weapon.name.clone());

                    player.kills += 1;

                    if let Some(new_weapon_name) = new_weapon_name.clone() {
                        weapon_icons_old.push((player.id, weapon.stats.weapon_type));

                        update_weapon_properties(weapon, new_weapon_name);
                        update_weapon_icon(
                            &entities,
                            &mut weapon,
                            &weapon_fire_resource,
                            player.id,
                            &lazy_update,
                        );
                    }
                }
            }

            let killed_data = player_got_killed_map.get(&player.id);

            if let Some(_killed_data) = killed_data {
                kill_restart_vehicle(vehicle, transform);
            }
        }

        for (entity, player_icon) in (&*entities, &player_icons).join() {
            for (player_id, weapon_type) in &weapon_icons_old {
                if *player_id == player_icon.id && *weapon_type == player_icon.weapon_type {
                    let _ = entities.delete(entity);
                }
            }
        }

        self.hit_sound_cooldown_timer -= dt;
    }
}
