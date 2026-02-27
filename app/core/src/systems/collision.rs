//! Manual circle collision detection.
//!
//! No physics engine is used in this project. All collision detection is
//! performed with simple geometric checks against [`CircleCollider`] radii.
//!
//! # Usage
//!
//! Call [`check_circle_collision`] with the world-space positions and radii of
//! two circles. Returns `true` when the circles overlap (penetration depth > 0).
//!
//! ```
//! use bevy::math::Vec2;
//! use vs_core::systems::collision::check_circle_collision;
//!
//! let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0);
//! assert!(hit); // circles overlap by 5 px
//! ```

use bevy::prelude::Vec2;

/// Returns `true` when two circles overlap (penetration depth > 0).
///
/// Uses a squared-distance comparison to avoid a square-root operation,
/// which is important for systems that check many entity pairs per frame.
///
/// Circles that are exactly touching (distance == r1 + r2) return `false`
/// because there is zero penetration depth.
///
/// # Arguments
///
/// * `pos1` — centre of the first circle in world space (pixels)
/// * `r1`   — radius of the first circle (pixels); **must be non-negative**
/// * `pos2` — centre of the second circle in world space (pixels)
/// * `r2`   — radius of the second circle (pixels); **must be non-negative**
///
/// # Panics (debug builds only)
///
/// Panics via [`debug_assert!`] if either radius is negative.
pub fn check_circle_collision(pos1: Vec2, r1: f32, pos2: Vec2, r2: f32) -> bool {
    debug_assert!(r1 >= 0.0, "r1 must be non-negative, got {r1}");
    debug_assert!(r2 >= 0.0, "r2 must be non-negative, got {r2}");
    let combined_radius = r1 + r2;
    pos1.distance_squared(pos2) < combined_radius * combined_radius
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Circles with centres 15 px apart and combined radius 20 px overlap.
    #[test]
    fn overlapping_circles_return_true() {
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 10.0);
        assert!(hit, "circles overlapping by 5 px should collide");
    }

    /// Circles with centres 30 px apart and combined radius 20 px do not overlap.
    #[test]
    fn separated_circles_return_false() {
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(30.0, 0.0), 10.0);
        assert!(!hit, "circles 10 px apart should not collide");
    }

    /// Circles touching at exactly one point have penetration depth 0 — no collision.
    #[test]
    fn touching_circles_return_false() {
        // distance = 20.0, combined radius = 20.0 → touching but not overlapping.
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(20.0, 0.0), 10.0);
        assert!(
            !hit,
            "circles touching at a single point should not collide"
        );
    }

    /// Collision detection is symmetric: swapping the two circles gives the same result.
    #[test]
    fn collision_is_symmetric() {
        let pos_a = Vec2::new(0.0, 0.0);
        let pos_b = Vec2::new(15.0, 0.0);
        let r_a = 8.0_f32;
        let r_b = 10.0_f32;

        let ab = check_circle_collision(pos_a, r_a, pos_b, r_b);
        let ba = check_circle_collision(pos_b, r_b, pos_a, r_a);
        assert_eq!(ab, ba, "collision must be symmetric");
    }

    /// Concentric circles (same position, both radius > 0) always overlap.
    #[test]
    fn concentric_circles_overlap() {
        let hit = check_circle_collision(Vec2::ZERO, 5.0, Vec2::ZERO, 1.0);
        assert!(hit, "concentric circles should always collide");
    }

    /// A zero-radius circle whose centre lies inside the other circle overlaps.
    #[test]
    fn zero_radius_inside_other_circle_overlaps() {
        // pos2 is 5 px from pos1; r1 = 10 px, r2 = 0 → point inside circle.
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(5.0, 0.0), 0.0);
        assert!(hit, "point inside a circle should collide");
    }

    /// A zero-radius circle outside the other circle does not overlap.
    #[test]
    fn zero_radius_outside_other_circle_no_collision() {
        // pos2 is 15 px from pos1; r1 = 10 px, r2 = 0 → point outside circle.
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(15.0, 0.0), 0.0);
        assert!(!hit, "point outside a circle should not collide");
    }

    /// Works correctly for circles offset on the Y-axis (not just X-axis).
    #[test]
    fn vertical_offset_overlap() {
        let hit = check_circle_collision(Vec2::ZERO, 10.0, Vec2::new(0.0, 15.0), 10.0);
        assert!(hit, "vertically offset overlapping circles should collide");
    }

    /// Works correctly for circles at a diagonal offset.
    #[test]
    fn diagonal_offset_no_overlap() {
        // Diagonal distance = sqrt(100 + 100) ≈ 14.14; combined radius = 10 → no hit.
        let hit = check_circle_collision(Vec2::ZERO, 5.0, Vec2::new(10.0, 10.0), 5.0);
        assert!(!hit, "diagonally separated circles should not collide");
    }
}
