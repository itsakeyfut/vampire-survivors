use bevy::prelude::Color;

/// Visual tier of an XP gem, derived from its XP value at spawn time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GemTier {
    #[default]
    Small, // value 1–4
    Medium, // value 5–24
    Large,  // value 25–99
    Boss,   // value 100+
}

impl GemTier {
    pub fn from_value(v: u32) -> Self {
        match v {
            0..=4 => Self::Small,
            5..=24 => Self::Medium,
            25..=99 => Self::Large,
            _ => Self::Boss,
        }
    }

    pub fn color(self) -> Color {
        match self {
            Self::Small => Color::srgb(0.3, 0.5, 1.0),
            Self::Medium => Color::srgb(0.2, 0.9, 0.3),
            Self::Large => Color::srgb(1.0, 0.3, 0.2),
            Self::Boss => Color::srgb(1.0, 0.9, 0.1),
        }
    }

    pub fn radius(self) -> f32 {
        match self {
            Self::Small => 4.0,
            Self::Medium => 6.0,
            Self::Large => 9.0,
            Self::Boss => 13.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value_boundaries() {
        assert_eq!(GemTier::from_value(0), GemTier::Small);
        assert_eq!(GemTier::from_value(4), GemTier::Small);
        assert_eq!(GemTier::from_value(5), GemTier::Medium);
        assert_eq!(GemTier::from_value(24), GemTier::Medium);
        assert_eq!(GemTier::from_value(25), GemTier::Large);
        assert_eq!(GemTier::from_value(99), GemTier::Large);
        assert_eq!(GemTier::from_value(100), GemTier::Boss);
        assert_eq!(GemTier::from_value(999), GemTier::Boss);
    }

    #[test]
    fn default_is_small() {
        assert_eq!(GemTier::default(), GemTier::Small);
    }
}
