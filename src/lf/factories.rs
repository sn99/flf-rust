//! Object factory by type
use crate::lf::character::Character;
use crate::lf::data::ObjectData;
use crate::lf::effect::EffectObj;
use crate::lf::specialattack::SpecialAttack;
use crate::lf::weapon::Weapon;

pub enum GameObject {
    Character(Character),
    Weapon(Weapon),
    Special(SpecialAttack),
    Effect(EffectObj),
}

pub fn create(uid: u32, data: ObjectData, team: i32, x: f64, z: f64) -> GameObject {
    match data.obj_type.as_str() {
        "character" => GameObject::Character(Character::new(uid, data, team, x, z)),
        "lightweapon" | "heavyweapon" | "drink" | "broken" => {
            GameObject::Weapon(Weapon::new(uid, data, x, z))
        }
        "specialattack" => GameObject::Special(SpecialAttack::new(uid, data, team, x, 0.0, z, 1)),
        "effect" => GameObject::Effect(EffectObj::new(uid, data, x, 0.0, z)),
        _ => GameObject::Weapon(Weapon::new(uid, data, x, z)),
    }
}
