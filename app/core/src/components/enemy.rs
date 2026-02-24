use bevy::prelude::*;

use crate::types::{AIType, EnemyType};

/// Core enemy stats. Attached to every enemy entity.
#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub enemy_type: EnemyType,
    pub max_hp: f32,
    pub current_hp: f32,
    /// Movement speed in pixels/second.
    pub move_speed: f32,
    /// Contact damage dealt to the player per hit.
    pub damage: f32,
    /// XP gem value dropped on death.
    pub xp_value: u32,
    /// Probability (0.0–1.0) of dropping a gold coin on death.
    pub gold_chance: f32,
}

impl Enemy {
    /// Construct an `Enemy` from its type and the current difficulty multiplier.
    ///
    /// HP is scaled by `difficulty`; all other stats remain at their base values.
    /// `difficulty` should be ≥ 1.0 (1.0 = start of run, no scaling).
    pub fn from_type(enemy_type: EnemyType, difficulty: f32) -> Self {
        use crate::constants::*;
        use crate::types::EnemyType::*;
        let (base_hp, speed, damage, xp, gold) = match enemy_type {
            Bat => ENEMY_STATS_BAT,
            Skeleton => ENEMY_STATS_SKELETON,
            Zombie => ENEMY_STATS_ZOMBIE,
            Ghost => ENEMY_STATS_GHOST,
            Demon => ENEMY_STATS_DEMON,
            Medusa => ENEMY_STATS_MEDUSA,
            Dragon => ENEMY_STATS_DRAGON,
            BossDeath => ENEMY_STATS_BOSS_DEATH,
        };
        let max_hp = base_hp * difficulty.max(1.0);
        Self {
            enemy_type,
            max_hp,
            current_hp: max_hp,
            move_speed: speed,
            damage,
            xp_value: xp,
            gold_chance: gold,
        }
    }

    /// Returns `true` if the enemy has been reduced to zero HP.
    pub fn is_dead(&self) -> bool {
        self.current_hp <= 0.0
    }

    /// Apply `amount` damage, clamping `current_hp` to zero.
    ///
    /// Negative values are treated as zero (no healing side-effect).
    pub fn take_damage(&mut self, amount: f32) {
        let dmg = amount.max(0.0);
        self.current_hp = (self.current_hp - dmg).max(0.0);
    }
}

/// Drives enemy movement and attack behavior.
#[derive(Component, Debug)]
pub struct EnemyAI {
    pub ai_type: AIType,
    /// Timer between ranged attacks (used by KeepDistance AI).
    pub attack_timer: f32,
    /// Maximum distance at which this enemy will attack.
    pub attack_range: f32,
}

/// Brief color flash applied when an enemy takes damage.
#[derive(Component, Debug)]
pub struct DamageFlash {
    /// Remaining flash duration in seconds.
    pub timer: f32,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_from_type_bat_base_stats() {
        use crate::types::EnemyType;
        let e = Enemy::from_type(EnemyType::Bat, 1.0);
        assert_eq!(e.enemy_type, EnemyType::Bat);
        assert_eq!(e.max_hp, crate::constants::ENEMY_STATS_BAT.0);
        assert_eq!(e.current_hp, e.max_hp);
        assert_eq!(e.move_speed, crate::constants::ENEMY_STATS_BAT.1);
        assert_eq!(e.xp_value, crate::constants::ENEMY_STATS_BAT.3);
    }

    #[test]
    fn enemy_from_type_all_variants_construct() {
        use crate::types::EnemyType::*;
        for et in [
            Bat, Skeleton, Zombie, Ghost, Demon, Medusa, Dragon, BossDeath,
        ] {
            let e = Enemy::from_type(et, 1.0);
            assert!(e.max_hp > 0.0, "{et:?} must have positive HP");
            assert!(e.move_speed > 0.0, "{et:?} must have positive speed");
            assert_eq!(e.current_hp, e.max_hp, "{et:?} starts at full HP");
        }
    }

    #[test]
    fn enemy_difficulty_scales_hp() {
        use crate::types::EnemyType;
        let base = Enemy::from_type(EnemyType::Skeleton, 1.0);
        let hard = Enemy::from_type(EnemyType::Skeleton, 2.0);
        assert!(
            (hard.max_hp - base.max_hp * 2.0).abs() < base.max_hp * 1e-6,
            "HP should double at difficulty 2"
        );
        // Speed is unaffected by difficulty
        assert_eq!(base.move_speed, hard.move_speed);
    }

    #[test]
    fn enemy_difficulty_clamped_to_one() {
        use crate::types::EnemyType;
        let normal = Enemy::from_type(EnemyType::Bat, 1.0);
        let sub_one = Enemy::from_type(EnemyType::Bat, 0.1);
        assert_eq!(
            normal.max_hp, sub_one.max_hp,
            "difficulty below 1.0 should clamp to 1.0"
        );
    }

    #[test]
    fn enemy_is_dead_when_hp_zero() {
        use crate::types::EnemyType;
        let mut e = Enemy::from_type(EnemyType::Bat, 1.0);
        assert!(!e.is_dead());
        e.current_hp = 0.0;
        assert!(e.is_dead());
    }

    #[test]
    fn enemy_take_damage_reduces_hp() {
        use crate::types::EnemyType;
        let mut e = Enemy::from_type(EnemyType::Skeleton, 1.0);
        let initial = e.current_hp;
        e.take_damage(10.0);
        assert_eq!(e.current_hp, initial - 10.0);
    }

    #[test]
    fn enemy_take_damage_clamps_to_zero() {
        use crate::types::EnemyType;
        let mut e = Enemy::from_type(EnemyType::Bat, 1.0);
        e.take_damage(9999.0);
        assert_eq!(e.current_hp, 0.0);
        assert!(e.is_dead());
    }

    #[test]
    fn enemy_take_damage_negative_is_noop() {
        use crate::types::EnemyType;
        let mut e = Enemy::from_type(EnemyType::Bat, 1.0);
        let before = e.current_hp;
        e.take_damage(-5.0); // must not heal
        assert_eq!(e.current_hp, before);
    }
}
