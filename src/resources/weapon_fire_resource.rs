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
    pub mine_neutral_sprite_render: SpriteRender,
    pub missile_sprite_render: SpriteRender,
    pub rockets_sprite_render: SpriteRender,
    pub laser_sword_sprite_render: SpriteRender,
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
        mine_neutral_sprite_render: SpriteRender {
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
            sprite_sheet: sprite_sheet_handle,
            sprite_number: 15,
        },
    };
    world.insert(weapon_fire_resource.clone());

    weapon_fire_resource
}
