pub use self::hitbox::{Hitbox, HitboxShape};
pub use self::players::{BotMode, Player, PlayerWeaponIcon};
pub use self::vehicles::{check_respawn_vehicle, kill_restart_vehicle, Vehicle};
pub use self::weapons::{
    build_standard_weapon, get_next_weapon_type, update_weapon_icon, update_weapon_properties,
    weapon_type_from_u8, Weapon, WeaponFire, WeaponTypes,
};
pub use self::shields::Shield;
pub use self::armor::Armor;
pub use self::health::Health;

mod hitbox;
mod players;
mod vehicles;
mod weapons;
mod shields;
mod armor;
mod health;