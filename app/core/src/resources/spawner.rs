use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Fallback constants (used when RON config is not yet loaded)
// ---------------------------------------------------------------------------

/// Base enemy spawn interval in seconds.
const DEFAULT_ENEMY_SPAWN_BASE_INTERVAL: f32 = 0.5;

/// Controls enemy spawn timing and difficulty scaling.
#[derive(Resource, Debug)]
pub struct EnemySpawner {
    /// Accumulated time since the last spawn.
    pub spawn_timer: f32,
    /// Current interval between spawns (seconds). Decreases as difficulty grows.
    pub spawn_interval: f32,
    /// Multiplier applied to enemy HP and spawn frequency as time progresses.
    pub difficulty_multiplier: f32,
    /// When `false` the spawner is suspended (e.g. during LevelUp, boss phase).
    pub active: bool,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            spawn_interval: DEFAULT_ENEMY_SPAWN_BASE_INTERVAL,
            difficulty_multiplier: 1.0,
            active: true,
        }
    }
}

/// Controls treasure chest spawn timing.
#[derive(Resource, Debug)]
pub struct TreasureSpawner {
    /// Accumulated time since the last chest spawn.
    pub spawn_timer: f32,
}

impl Default for TreasureSpawner {
    fn default() -> Self {
        Self { spawn_timer: 0.0 }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_spawner_default_values() {
        let s = EnemySpawner::default();
        assert_eq!(s.spawn_timer, 0.0);
        assert_eq!(s.spawn_interval, DEFAULT_ENEMY_SPAWN_BASE_INTERVAL);
        assert_eq!(s.difficulty_multiplier, 1.0);
        assert!(s.active, "spawner should be active by default");
    }

    #[test]
    fn enemy_spawner_can_be_paused() {
        let mut s = EnemySpawner::default();
        s.active = false;
        assert!(!s.active);
    }

    #[test]
    fn enemy_spawner_difficulty_starts_at_one() {
        let s = EnemySpawner::default();
        assert!(
            (s.difficulty_multiplier - 1.0).abs() < f32::EPSILON,
            "difficulty must start at 1.0"
        );
    }
}
