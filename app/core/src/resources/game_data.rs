use bevy::prelude::*;

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
            xp_to_next_level: crate::constants::XP_LEVEL_BASE,
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
        assert_eq!(gd.xp_to_next_level, crate::constants::XP_LEVEL_BASE);
        assert_eq!(gd.kill_count, 0);
        assert_eq!(gd.gold_earned, 0);
        assert!(!gd.boss_spawned);
    }
}
