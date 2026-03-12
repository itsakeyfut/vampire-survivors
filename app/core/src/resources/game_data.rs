use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// XP required for the first level-up.
const DEFAULT_XP_LEVEL_BASE: u32 = 20;

/// Global game-session data. Reset at the start of each run.
#[derive(Resource, Debug)]
pub struct GameData {
    /// Seconds elapsed since the run started (paused during LevelUp/Paused).
    pub elapsed_time: f32,
    /// Current player level (1-indexed).
    pub current_level: u32,
    /// Accumulated XP within the current level.
    pub current_xp: u32,
    /// XP required to reach the next level.
    pub xp_to_next_level: u32,
    /// Fractional XP carried over across gem absorptions.
    ///
    /// `xp_multiplier` can produce non-integer XP per gem (e.g. 3 × 1.1 = 3.3).
    /// Rather than rounding each gem in isolation (which silently discards the
    /// bonus), the fractional remainder is accumulated here and converted to whole
    /// XP only once it reaches ≥ 1.0.
    pub xp_fractional_accumulator: f32,
    /// Total enemies defeated this run.
    pub kill_count: u32,
    /// Gold collected this run (added to MetaProgress on run end).
    pub gold_earned: u32,
    /// True once Boss Death has been spawned.
    pub boss_spawned: bool,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            elapsed_time: 0.0,
            current_level: 1,
            current_xp: 0,
            xp_to_next_level: DEFAULT_XP_LEVEL_BASE,
            xp_fractional_accumulator: 0.0,
            kill_count: 0,
            gold_earned: 0,
            boss_spawned: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_data_default() {
        let gd = GameData::default();
        assert_eq!(gd.elapsed_time, 0.0);
        assert_eq!(gd.current_level, 1);
        assert_eq!(gd.current_xp, 0);
        assert_eq!(gd.xp_to_next_level, DEFAULT_XP_LEVEL_BASE);
        assert_eq!(gd.xp_fractional_accumulator, 0.0);
        assert_eq!(gd.kill_count, 0);
        assert_eq!(gd.gold_earned, 0);
        assert!(!gd.boss_spawned);
    }
}
