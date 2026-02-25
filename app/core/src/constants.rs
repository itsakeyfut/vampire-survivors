// ---------------------------------------------------------------------------
// Window
// ---------------------------------------------------------------------------

pub const WINDOW_WIDTH: u32 = 1280;
pub const WINDOW_HEIGHT: u32 = 720;

// ---------------------------------------------------------------------------
// Player base stats
// ---------------------------------------------------------------------------

pub const PLAYER_BASE_HP: f32 = 100.0;
pub const PLAYER_BASE_SPEED: f32 = 200.0;
pub const PLAYER_BASE_DAMAGE_MULT: f32 = 1.0;
pub const PLAYER_BASE_COOLDOWN_REDUCTION: f32 = 0.0;
pub const PLAYER_BASE_PROJECTILE_SPEED: f32 = 1.0;
pub const PLAYER_BASE_DURATION_MULT: f32 = 1.0;
pub const PLAYER_BASE_AREA_MULT: f32 = 1.0;
pub const PLAYER_BASE_LUCK: f32 = 1.0;
pub const PLAYER_BASE_HP_REGEN: f32 = 0.0;
pub const PLAYER_PICKUP_RADIUS: f32 = 80.0;

/// Duration of invincibility frames after taking damage (seconds).
pub const PLAYER_INVINCIBILITY_TIME: f32 = 0.5;

// ---------------------------------------------------------------------------
// Collision radii (pixels)
// ---------------------------------------------------------------------------

pub const COLLIDER_PLAYER: f32 = 12.0;
pub const COLLIDER_PROJECTILE_SMALL: f32 = 5.0;
pub const COLLIDER_PROJECTILE_LARGE: f32 = 10.0;
pub const COLLIDER_XP_GEM: f32 = 6.0;
pub const COLLIDER_GOLD_COIN: f32 = 6.0;
pub const COLLIDER_TREASURE: f32 = 20.0;

// Enemy colliders
pub const COLLIDER_BAT: f32 = 8.0;
pub const COLLIDER_SKELETON: f32 = 12.0;
pub const COLLIDER_ZOMBIE: f32 = 14.0;
pub const COLLIDER_GHOST: f32 = 10.0;
pub const COLLIDER_DEMON: f32 = 14.0;
pub const COLLIDER_MEDUSA: f32 = 12.0;
pub const COLLIDER_DRAGON: f32 = 20.0;
pub const COLLIDER_BOSS_DEATH: f32 = 30.0;

// ---------------------------------------------------------------------------
// Enemy base stats
//
// HP values are multiplied by the runtime `difficulty_multiplier` before use.
// Speed, damage, XP, and gold chance remain constant regardless of difficulty.
// ---------------------------------------------------------------------------

/// (base_hp, speed px/s, contact_damage, xp_value, gold_drop_chance 0–1)
pub const ENEMY_STATS_BAT: (f32, f32, f32, u32, f32) = (10.0, 150.0, 5.0, 3, 0.05);
pub const ENEMY_STATS_SKELETON: (f32, f32, f32, u32, f32) = (30.0, 80.0, 8.0, 5, 0.08);
pub const ENEMY_STATS_ZOMBIE: (f32, f32, f32, u32, f32) = (80.0, 40.0, 12.0, 8, 0.10);
pub const ENEMY_STATS_GHOST: (f32, f32, f32, u32, f32) = (40.0, 70.0, 10.0, 6, 0.08);
pub const ENEMY_STATS_DEMON: (f32, f32, f32, u32, f32) = (100.0, 120.0, 15.0, 10, 0.12);
pub const ENEMY_STATS_MEDUSA: (f32, f32, f32, u32, f32) = (60.0, 60.0, 12.0, 8, 0.10);
pub const ENEMY_STATS_DRAGON: (f32, f32, f32, u32, f32) = (200.0, 80.0, 20.0, 15, 0.15);
pub const ENEMY_STATS_BOSS_DEATH: (f32, f32, f32, u32, f32) = (5000.0, 30.0, 50.0, 500, 1.0);

// ---------------------------------------------------------------------------
// Projectile defaults
// ---------------------------------------------------------------------------

/// Default projectile travel speed in pixels/second (before `projectile_speed_mult`).
pub const BASE_PROJECTILE_SPEED: f32 = 300.0;
/// Default projectile lifetime in seconds.
pub const BASE_PROJECTILE_LIFETIME: f32 = 5.0;

// ---------------------------------------------------------------------------
// Weapon limits
// ---------------------------------------------------------------------------

/// Maximum number of weapons the player can carry simultaneously.
pub const MAX_WEAPONS: usize = 6;
/// Maximum number of passive items the player can carry.
pub const MAX_PASSIVES: usize = 6;
/// Maximum weapon upgrade level.
pub const MAX_WEAPON_LEVEL: u8 = 8;
/// Maximum passive item upgrade level.
pub const MAX_PASSIVE_LEVEL: u8 = 5;

// ---------------------------------------------------------------------------
// Game rules
// ---------------------------------------------------------------------------

/// Time at which Boss Death spawns (30 minutes, in seconds).
pub const BOSS_SPAWN_TIME: f32 = 30.0 * 60.0;
/// Interval between treasure chest spawns (3 minutes, in seconds).
pub const TREASURE_SPAWN_INTERVAL: f32 = 180.0;

// ---------------------------------------------------------------------------
// Enemy spawning
// ---------------------------------------------------------------------------

/// Base spawn interval between enemies (seconds).
pub const ENEMY_SPAWN_BASE_INTERVAL: f32 = 0.5;
/// Maximum number of simultaneous enemies before spawning is throttled.
pub const ENEMY_MAX_COUNT: usize = 500;
/// Distance from the player (pixels) beyond which enemies are silently despawned.
///
/// Enemies culled this way do **not** drop XP gems — they simply disappear.
/// This prevents unbounded memory growth when the player moves far from
/// a cluster of enemies.
pub const ENEMY_CULL_DISTANCE: f32 = 2000.0;
/// Hard ceiling for the difficulty multiplier (≈ 90 minutes of play).
///
/// Prevents the spawn interval from shrinking to sub-frame values in
/// extreme runtimes beyond the intended 30-minute game length.
pub const DIFFICULTY_MAX: f32 = 10.0;

// ---------------------------------------------------------------------------
// XP / levelling
// ---------------------------------------------------------------------------

/// XP required for the first level-up.
pub const XP_LEVEL_BASE: u32 = 20;
/// Multiplier applied to XP threshold each level.
pub const XP_LEVEL_MULTIPLIER: f32 = 1.2;

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// Higher value = tighter / faster camera follow.
pub const CAMERA_LERP_SPEED: f32 = 10.0;

// ---------------------------------------------------------------------------
// Spatial grid
// ---------------------------------------------------------------------------

/// Grid cell size for spatial partitioning (pixels). Should be roughly 2×
/// the largest enemy collision radius to avoid excessive multi-cell lookups.
pub const SPATIAL_GRID_CELL_SIZE: f32 = 64.0;
