use amethyst::ecs::prelude::{Component, DenseVecStorage};

#[derive(Clone, Debug, PartialEq)]
pub enum HitboxShape {
    Rectangle,
    Circle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RaceCheckpointType {
    NotCheckpoint,
    CheckpointStart,
    CheckpointFinish,
    LapStart,
    LapFinish
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
}

impl Component for Hitbox {
    type Storage = DenseVecStorage<Self>;
}

impl Hitbox {
    pub fn new(width: f32, height: f32, angle: f32, 
            shape: HitboxShape,
            is_wall: bool, is_hill: bool,
            checkpoint: RaceCheckpointType, checkpoint_id: i32,
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
        }
    }
}
