use amethyst::{
    assets::Handle,
    ecs::prelude::World,
    renderer::{SpriteRender, SpriteSheet},
};

#[derive(Clone)]
pub struct WeaponFireResource {
    /// The render that locates the sprite in a sprite sheet resource
    pub laser_double_sprite_render: SpriteRender,
    pub laser_beam_sprite_render: SpriteRender,
    pub laser_burst_sprite_render: SpriteRender,
    pub projectile_cannon_sprite_render: SpriteRender,
    pub projectile_burst_render: SpriteRender,
    pub projectile_rapid_render: SpriteRender,
    pub mine_p1_sprite_render: SpriteRender,
    pub mine_p2_sprite_render: SpriteRender,
    pub mine_p3_sprite_render: SpriteRender,
    pub mine_p4_sprite_render: SpriteRender,
    pub trap_p1_sprite_render: SpriteRender,
    pub trap_p2_sprite_render: SpriteRender,
    pub trap_p3_sprite_render: SpriteRender,
    pub trap_p4_sprite_render: SpriteRender,
    pub grenade_sprite_render: SpriteRender,
    pub missile_sprite_render: SpriteRender,
    pub rockets_sprite_render: SpriteRender,
    pub laser_sword_sprite_render: SpriteRender,
    pub flame_sprite_render: SpriteRender,
    pub weapon_box_sprite_render: SpriteRender,
    pub sparking_sprite_render: SpriteRender,
    pub rocket_spray_sprite_render: SpriteRender,
    pub shield_hit_spray_sprite_render: SpriteRender,
    pub hull_hit_spray_sprite_render: SpriteRender,
    pub smoke_spray_sprite_render: SpriteRender,
    pub ion_sprite_render: SpriteRender,
    pub shockwave_sprite_render: SpriteRender,
    pub bio_spike_sprite_render: SpriteRender,
    pub light_bolt_sprite_render: SpriteRender,
    pub slime_ball_sprite_render: SpriteRender,
}

pub fn initialize_weapon_fire_resource(
    world: &mut World,
    sprite_sheet_handle: Handle<SpriteSheet>,
) -> WeaponFireResource {
    let weapon_fire_resource = WeaponFireResource {
        laser_double_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 4,
        },
        laser_beam_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 5,
        },
        laser_burst_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 6,
        },
        projectile_cannon_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 7,
        },
        projectile_burst_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 8,
        },
        projectile_rapid_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 9,
        },
        mine_p1_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 10,
        },
        mine_p2_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 16,
        },
        mine_p3_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 17,
        },
        mine_p4_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 18,
        },
        grenade_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 23,
        },
        missile_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 11,
        },
        rockets_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 12,
        },
        laser_sword_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 15,
        },
        trap_p1_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 25,
        },
        trap_p2_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 26,
        },
        trap_p3_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 27,
        },
        trap_p4_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 28,
        },
        flame_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 34,
        },
        weapon_box_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 35,
        },
        sparking_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 37,
        },
        rocket_spray_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 38,
        },
        shield_hit_spray_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 39,
        },
        hull_hit_spray_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 40,
        },
        smoke_spray_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 41,
        },
        ion_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 42,
        },
        shockwave_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 43,
        },
        bio_spike_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 75,
        },
        light_bolt_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 76,
        },
        slime_ball_sprite_render: SpriteRender {
            sprite_sheet: sprite_sheet_handle.clone(),
            sprite_number: 77,
        },
        
    };
    world.insert(weapon_fire_resource.clone());

    weapon_fire_resource
}
