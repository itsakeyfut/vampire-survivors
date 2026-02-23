# understand - Quick Codebase Overview

## Overview

Provides a **concise, high-level overview** of specific files, structs, or features. Perfect for quick reference and getting oriented.

For comprehensive analysis with detailed examples and dependency graphs, use `understand-deep` instead.

## Usage

```bash
understand <target>
```

**target** can be any of:
- **File path**: `app/core/src/weapons.rs`
- **Module name**: `collision`, `enemies`, `weapons`
- **Struct/Enum name**: `WeaponType`, `Enemy`, `GameState`
- **Feature name**: `weapon-evolution`, `xp-system`, `boss-fight`

## Output Format

Generates concise documentation in Markdown format (1-2 pages) containing:

### 1. Quick Summary
- **Purpose**: One-liner description
- **Location**: File path
- **Phase**: Implementation phase (from docs/06_implementation_plan.md)

### 2. Type Definition
- **Struct/Enum signature**: Just the type and field names (no doc comments)
- **Derived traits**: List only

### 3. Key Methods (Top 5-10)
- **Constructor**: Creation methods
- **Core methods**: Most commonly used 3-5 methods
- **Important helpers**: 2-3 utility methods
- Signature only, no implementation details

### 4. Dependencies (Simple List)
- **Direct imports**: 3-5 key dependencies
- **Used by**: 3-5 major usage locations
- Simple bullet points only (no diagrams)

### 5. Quick Example
- **One real usage example** from the codebase (5-10 lines)

### 6. Related Files
- **See also**: 2-3 related files to explore
- **Deep dive**: Link to `understand-deep` for full details

## Processing Steps

1. **Identify target**
   - Quick Glob/Grep search
   - Confirm if multiple candidates

2. **Extract essentials**
   - Read target file
   - Extract type definition (structure only)
   - Identify 5-10 most important methods
   - Skip detailed doc comments

3. **Minimal dependency analysis**
   - List 3-5 direct imports
   - Find 3-5 main usage locations (not exhaustive)
   - Skip comprehensive search

4. **Single usage example**
   - Find ONE clear, simple usage example
   - Prefer test code or simple initialization

5. **Output concise Markdown**
   - Keep to 1-2 pages maximum
   - No diagrams (keep it simple)
   - Focus on "what" not "how"

6. **Save to file**
   - Create `.claude/tmp/` directory if needed
   - Save to `.claude/tmp/understand_<target>.md`
   - Display file path after completion

## Output File

After generating the documentation, save it to:
```
.claude/tmp/understand_<sanitized_target>.md
```

**Examples**:
- `understand WeaponType` → `.claude/tmp/understand_weapontype.md`
- `understand collision` → `.claude/tmp/understand_collision.md`
- `understand xp_system` → `.claude/tmp/understand_xp_system.md`

## Example Output

```markdown
# `WeaponType` - Quick Overview

## Summary

**Purpose**: Enum representing all weapon types available in the game
**Location**: `app/core/src/weapons.rs`
**Phase**: Phase 4 (Weapon Systems)

---

## Type Definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WeaponType {
    Whip,          // 水平スイング
    MagicWand,     // 最近敵に弾発射
    Knife,         // 移動方向に高速貫通
    Garlic,        // 周囲にオーラ
    Bible,         // 周回攻撃
    ThunderRing,   // ランダム雷撃
    Cross,         // ブーメラン
    FireWand,      // 最大HPの敵に火球
}
```

**Derived**: `Debug, Clone, Copy, PartialEq, Eq, Hash`

---

## Key Methods

**Evolution**:
```rust
pub fn can_evolve_with(&self, passive: PassiveType) -> Option<WeaponType>
pub fn evolved_name(&self) -> &'static str
```

**Properties**:
```rust
pub fn base_damage(&self) -> f32
pub fn base_cooldown(&self) -> f32
pub fn max_level(&self) -> u32
```

---

## Dependencies

**Used by**:
- `systems/weapons.rs` - Weapon firing systems
- `systems/level_up.rs` - Level-up card generation
- `systems/treasure.rs` - Treasure content determination

---

## Quick Example

```rust
let weapon_type = WeaponType::MagicWand;
let damage = weapon_type.base_damage(); // 20.0
let cooldown = weapon_type.base_cooldown(); // 0.5
```

---

## Related Files

**See also**:
- `components.rs` - WeaponState component definition
- `systems/weapons.rs` - Weapon firing logic

**For detailed analysis**: Run `understand-deep WeaponType`

---

**Generated**: 2026-02-23
**Command**: `understand WeaponType`
```

## Important Notes

- **Brevity First**: Keep output to 1-2 pages maximum
- **Skip Details**: No comprehensive method lists, no full doc comments
- **Quick Reference**: Optimized for fast lookup, not learning
- **Direct Users**: When more detail is needed, suggest `understand-deep`
- **Save Output**: Always save to `.claude/tmp/understand_*.md`

## Technical Guidelines

### What to Include
- Type signatures (fields, enums)
- Top 5-10 methods only
- 3-5 key dependencies
- 1 simple usage example
- 2-3 related files

### What to Skip
- Detailed doc comments (use first sentence only)
- All methods (just the important ones)
- Exhaustive dependency search
- Multiple code examples
- Diagrams (ASCII or otherwise)
- Test code details
- Implementation explanations

### Selection Criteria for Methods

**Include**:
1. Constructor (`new`, `default`, `from_*`)
2. Most commonly called methods (check usage count)
3. Core API methods (public, non-helper)
4. Methods mentioned in doc comments
5. Methods used in examples

**Skip**:
- Internal helpers
- Rarely used utilities
- Simple getters/setters
- Private methods

---

**Purpose**: Provide quick orientation for developers who just need to know "what is this" without deep diving into "how it works".
