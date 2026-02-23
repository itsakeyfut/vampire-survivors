---
description: Commit changes and create PR (keep under 100 lines)
allowed-tools: ["bash", "read", "grep"]
argument-hint: "[file1] [file2] ..."
---

Complete the implementation workflow:

**Steps:**

1. **Check current status:**
   ```bash
   git status
   git diff --stat
   ```

2. **Verify changes are under 100 lines:**
   ```bash
   git diff | wc -l
   ```
   Check the diff size. If over 100 lines, consider splitting into multiple PRs.

3. **MANDATORY: Verify and move to working branch:**

   **CRITICAL**: NEVER commit directly to `main` branch!

   ```bash
   # Check current branch
   git branch --show-current
   ```

   **If currently on `main`:**
   - STOP and create/switch to a feature branch first
   - Example: `git checkout -b feat/issue-XXX` or `git checkout existing-branch`

   **If already on a feature branch:**
   - Verify the branch name is correct
   - Proceed to next step

   **Branch naming convention:**
   - `feat/<description>` for features
   - `fix/<description>` for bug fixes
   - `refactor/<description>` for refactoring
   - `docs/<description>` for documentation
   - `chore/<description>` for tooling/build

4. **Run quality checks:**
   ```bash
   # Using just (recommended)
   just fmt
   just clippy
   just test
   just build

   # Or using cargo directly
   cargo fmt --all
   cargo clippy --workspace -- -D warnings
   cargo test --workspace
   cargo build --release
   ```

5. **Test in-game (if applicable):**
   ```bash
   just dev
   # Verify the changes work correctly in-game
   ```

6. **Stage and commit changes:**

   **File selection:**
   - If specific files were provided as arguments: `$ARGUMENTS`
     → Use: `git add $ARGUMENTS` (commit only specified files)
   - If no arguments were provided:
     → Use: `git add .` (commit all changed files)

   **Commit guidelines:**
   - Create logical, atomic commits
   - Follow conventional commits format: `<type>(<scope>): <description>`
   - Reference issue numbers with "Closes #XXX"
   - Write commit message in English
   - Example: `feat(weapons): implement magic wand projectile system\n\nCloses #5`

   **Commit Scopes:**
   - `core`: Core game logic
   - `ui`: UI systems
   - `audio`: Audio systems
   - `assets`: Asset management
   - `weapons`: Weapon systems (spawning, firing, evolution)
   - `enemies`: Enemy systems (spawning, AI, damage)
   - `collision`: Collision detection & damage
   - `xp`: XP & level-up system
   - `meta`: Meta progression & gold shop
   - `docs`: Documentation
   - `chore`: Build/tooling

7. **Push changes:**
   ```bash
   git push -u origin <branch-name>
   ```

8. **Create PR using gh command:**
   ```bash
   gh pr create --title "..." --body "..."
   ```

**PR Guidelines:**

**MANDATORY: Write PR in Japanese (日本語で記述)**

**MANDATORY PR Body Limit: MAXIMUM 100 LINES**

- **Keep PR body concise** - MUST be under 100 lines
- Use clear, concise language in Japanese
- Include only essential information:
  - 概要 (Brief summary: 2-4 sentences)
  - 変更内容 (Key changes: 3-5 bullet points)
  - テスト (Test plan: brief checklist)
  - 関連Issue (Related issues: "Closes #XXX")
- Avoid verbose descriptions, excessive formatting, or redundant information
- If more details are needed, add them as issue comments instead

**PR Title:**
- Follow conventional commits format in English
- Include scope if applicable
- Example: `feat(weapons): マジックワンドの投射体システムを実装`
- Example: `fix(collision): 敵との衝突判定バグを修正`
- Example: `docs: ホットリロード手順をREADMEに追加`

**PR Body Template (in Japanese):**
```markdown
## 概要
[2-4 sentences describing the change]

## 変更内容
- [Key change 1]
- [Key change 2]
- [Key change 3]

## テスト
- [ ] ユニットテスト追加/更新
- [ ] ゲーム内で動作確認
- [ ] `just check` 実行済み

Closes #XXX
```

Please proceed with these steps.
