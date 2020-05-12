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
    CheckpointStart,
    CheckpointFinish,
    LapStart,
    LapFinish,
}

#[derive(Clone)]
pub struct Hitbox {
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub shape: HitboxShape,
    pub is_wall: bool,
    pub is_hill: bool,
    pub checkpoint: RaceCheckpointType,
    pub checkpoint_id: i32,
    pub is_weapon_box: bool,
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
        is_wall: bool,
        is_hill: bool,
        checkpoint: RaceCheckpointType,
        checkpoint_id: i32,
        is_weapon_box: bool,
    ) -> Hitbox {
        Hitbox {
            width,
            height,
            angle,
            shape,
            is_wall,
            is_hill,
            checkpoint,
            checkpoint_id,
            is_weapon_box,
        }
    }
}
