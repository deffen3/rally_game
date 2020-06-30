use amethyst::ecs::prelude::{Component, DenseVecStorage};

#[derive(Clone, Debug, PartialEq)]
pub enum HitboxShape {
    Rectangle,
    Circle,
    InnerQuarterCircle,
    OuterQuarterCircle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RaceCheckpointType {
    NotCheckpoint,
    Checkpoint,
    Lap,
}

#[derive(Clone)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub shape: HitboxShape,
}

impl Component for Hitbox {
    type Storage = DenseVecStorage<Self>;
}

impl Hitbox {
    pub fn new(
        width: f32,
        height: f32,
        angle: f32,
        shape: HitboxShape,
    ) -> Hitbox {
        Hitbox {
            width,
            height,
            angle,
            shape,
        }
    }
}


#[derive(Clone)]
pub struct Arena {
    pub is_wall: bool,
    pub is_hill: bool,
    pub checkpoint: RaceCheckpointType,
    pub checkpoint_id: i32,
    pub is_weapon_box: bool,
    pub hitbox: Hitbox,
}

impl Component for Arena {
    type Storage = DenseVecStorage<Self>;
}

impl Arena {
    pub fn new(
        is_wall: bool,
        is_hill: bool,
        checkpoint: RaceCheckpointType,
        checkpoint_id: i32,
        is_weapon_box: bool,
        hitbox: Hitbox,
    ) -> Arena {
        Arena {
            is_wall,
            is_hill,
            checkpoint,
            checkpoint_id,
            is_weapon_box,
            hitbox,
        }
    }
}