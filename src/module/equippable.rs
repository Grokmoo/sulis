use rules::Damage;
use module::item::Slot;

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct Equippable {
    pub slot: Slot,
    pub damage: Option<Damage>,
}

