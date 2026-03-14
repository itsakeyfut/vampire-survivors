/// All available stages, each with distinct enemy pools and difficulty modifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StageType {
    /// Mad Forest — basic enemies (Bat, Skeleton). Baseline difficulty.
    #[default]
    MadForest,
    /// Inlaid Library — medium enemies (Zombie, Ghost). HP ×1.2, speed ×1.1.
    InlaidLibrary,
    /// Dairy Plant — hard enemies (Demon, Medusa). HP ×1.5, speed ×1.2.
    DairyPlant,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_type_default_is_mad_forest() {
        assert_eq!(StageType::default(), StageType::MadForest);
    }

    #[test]
    fn stage_type_is_copy() {
        let s = StageType::MadForest;
        let _copy = s;
        let _original = s; // must not move
    }

    #[test]
    fn all_three_variants_are_distinct() {
        assert_ne!(StageType::MadForest, StageType::InlaidLibrary);
        assert_ne!(StageType::MadForest, StageType::DairyPlant);
        assert_ne!(StageType::InlaidLibrary, StageType::DairyPlant);
    }
}
