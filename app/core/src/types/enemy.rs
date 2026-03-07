use serde::{Deserialize, Serialize};

/// All enemy types, ordered by earliest appearance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyType {
    /// Appears from 0 min. Fast but fragile flier.
    Bat,
    /// Appears from 0 min. Basic melee enemy.
    Skeleton,
    /// Appears from 5 min. Slow but tanky.
    Zombie,
    /// Appears from 10 min. Passes through walls.
    Ghost,
    /// Appears from 15 min. High damage.
    Demon,
    /// Appears from 20 min. Ranged attacker, keeps distance.
    Medusa,
    /// Appears from 25 min. Charge attack.
    Dragon,
    /// Appears at 30 min. Final boss, multi-phase.
    BossDeath,
    /// Summoned by Boss Death at Phase2 (HP < 60%). Low HP, normal speed.
    MiniDeath,
}

/// Enemy AI behavior mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AIType {
    /// Moves directly toward the player each frame.
    ChasePlayer,
    /// Maintains attack distance and fires ranged projectiles.
    KeepDistance,
    /// Charges at the player in a straight line.
    ChargeAttack,
    /// Multi-phase boss behavior.
    BossMultiPhase,
}

/// Boss fight phases.
///
/// Attached as a component to the Boss Death entity.  Systems that implement
/// multi-phase boss behavior query for this component to determine which
/// attack patterns and movement rules apply.
#[derive(bevy::prelude::Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossPhase {
    Phase1,
    Phase2,
    Phase3,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_type_is_copy() {
        let e = EnemyType::Bat;
        let _copy = e;
        let _original = e; // should not move
    }

    #[test]
    fn enemy_type_all_nine_variants_exist() {
        // Ensure every variant listed in the spec compiles and is distinct.
        let variants = [
            EnemyType::Bat,
            EnemyType::Skeleton,
            EnemyType::Zombie,
            EnemyType::Ghost,
            EnemyType::Demon,
            EnemyType::Medusa,
            EnemyType::Dragon,
            EnemyType::BossDeath,
            EnemyType::MiniDeath,
        ];
        assert_eq!(variants.len(), 9);
        // All variants must be distinct (PartialEq).
        for i in 0..variants.len() {
            for j in 0..variants.len() {
                if i == j {
                    assert_eq!(variants[i], variants[j]);
                } else {
                    assert_ne!(variants[i], variants[j]);
                }
            }
        }
    }

    #[test]
    fn ai_type_is_copy() {
        let a = AIType::ChasePlayer;
        let _copy = a;
        let _original = a;
    }
}
