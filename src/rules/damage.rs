#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields)]
pub struct Damage {
    pub min: u32,
    pub max: u32,
}

impl Damage {
    pub fn max(this: Damage, other: Damage) -> Damage {
        if other.average() > this.average() {
            other
        } else {
            this
        }
    }

    pub fn average(&self) -> f32 {
        (self.min as f32 + self.max as f32) / 2.0
    }
}

impl Default for Damage {
    fn default() -> Damage {
        Damage {
            min: 0,
            max: 0,
        }
    }
}
