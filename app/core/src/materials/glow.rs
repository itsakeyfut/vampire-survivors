//! Generic radial-glow [`Material2d`] driven entirely by a WGSL fragment shader.
//!
//! The shader reads `globals.time` (provided by Bevy's render pipeline) to
//! animate a gentle pulse, so no per-frame CPU updates are required.
//!
//! ## Usage
//!
//! Spawn a quad mesh sized to the desired glow area, attach
//! `MeshMaterial2d(materials.add(GlowMaterial { params: GlowParams { .. } }))`,
//! and set its [`AlphaMode2d::Blend`] (already the default).
//!
//! ```ignore
//! let glow_size = radius * 3.0;
//! commands.spawn((
//!     Mesh2d(meshes.add(Rectangle::new(glow_size * 2.0, glow_size * 2.0))),
//!     MeshMaterial2d(materials.add(GlowMaterial::for_treasure(radius))),
//!     Transform::default(),
//!     Visibility::Hidden,
//! ));
//! ```

use bevy::math::{Vec2, Vec4};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

// ---------------------------------------------------------------------------
// Fallback constant
// ---------------------------------------------------------------------------

/// Default glow colour for treasure chests (yellow, matching the placeholder
/// sprite).  Phase 17 will replace the placeholder sprite; at that point this
/// colour should be revisited.
///
/// Format: `[R, G, B, A]` in linear sRGB, all channels in 0.0–1.0.
const DEFAULT_TREASURE_GLOW_COLOR: [f32; 4] = [1.0, 0.85, 0.15, 0.85];

// ---------------------------------------------------------------------------
// Shader params struct
// ---------------------------------------------------------------------------

/// Uniform data passed to `shaders/glow.wgsl`.
///
/// Layout (explicit `_pad` ensures a 32-byte struct across all WGSL backends;
/// vec4=16 bytes, two f32=8 bytes, Vec2 pad=8 bytes):
///
/// | Field          | WGSL type    | Offset | Size |
/// |----------------|--------------|--------|------|
/// | `glow_color`   | `vec4<f32>`  | 0      | 16   |
/// | `inner_radius` | `f32`        | 16     | 4    |
/// | `outer_radius` | `f32`        | 20     | 4    |
/// | `_pad`         | `vec2<f32>`  | 24     | 8    |
#[derive(ShaderType, Clone, Debug, Default)]
pub struct GlowParams {
    /// RGBA colour of the glow ring (A controls overall opacity).
    pub glow_color: Vec4,
    /// Inner radius in UV space (0–0.5) — start of the glow ring.
    pub inner_radius: f32,
    /// Outer radius in UV space (0–0.5) — end of the glow ring.
    pub outer_radius: f32,
    /// Padding to make the struct a multiple of 16 bytes, matching the WGSL
    /// uniform buffer alignment requirement.  Do not remove.
    pub _pad: Vec2,
}

// ---------------------------------------------------------------------------
// Material
// ---------------------------------------------------------------------------

/// 2D material that renders a radial glow ring with a GPU-driven time pulse.
///
/// Register [`GlowMaterialPlugin`] once in your app, then use
/// `Assets<GlowMaterial>` to create instances.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct GlowMaterial {
    #[uniform(0)]
    pub params: GlowParams,
}

impl GlowMaterial {
    /// Convenience constructor for the yellow treasure-chest glow.
    ///
    /// `chest_radius` is the [`CircleCollider`] radius of the chest (pixels).
    /// The glow mesh should be sized to `chest_radius * 3.0 * 2.0` on each axis.
    pub fn for_treasure(chest_radius: f32) -> Self {
        let [r, g, b, a] = DEFAULT_TREASURE_GLOW_COLOR;
        // UV space: the mesh covers [0,1]×[0,1].
        // The chest body occupies the inner circle up to 1/3 of the mesh half-size.
        let mesh_half = chest_radius * 3.0;
        let inner = chest_radius / mesh_half * 0.5; // ≈ 0.167
        let outer = 0.5; // full mesh radius in UV space

        Self {
            params: GlowParams {
                glow_color: Vec4::new(r, g, b, a),
                inner_radius: inner,
                outer_radius: outer,
                _pad: Vec2::ZERO,
            },
        }
    }
}

impl Material2d for GlowMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glow.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers [`GlowMaterial`] with Bevy's render pipeline.
///
/// Add this to your [`App`] once (via [`GameCorePlugin`]).
pub struct GlowMaterialPlugin;

impl Plugin for GlowMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<GlowMaterial>::default());
    }
}
