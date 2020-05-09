use amethyst::ecs::Entity;

#[derive(Clone)]
pub struct MatchTimer {
    pub time: f32,
    pub ui_entity: Entity,
}