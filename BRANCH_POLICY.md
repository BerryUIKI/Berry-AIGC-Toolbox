# Branch Management Policy

## Overview

As of 2026-08-27, the repository was restructured: the old C#/.NET codebase was
archived and development restarted from scratch (new tech stack: Tauri 2 + Rust
+ Vue 3). The branch workflow is:

> **All pull requests target `dev` first. `dev` is merged into `rewrite` only
> as a unified release PR. `rewrite` carries official releases only.**

## Branch Structure

### `rewrite` — Official Release Branch (default, protected)

- 🔒 **Protected branch** — direct pushes are **FORBIDDEN**
- **Accepts release PRs from `dev` only** (no feature branches target `rewrite`)
- Requires 1 approval before merging
- Linear history enforced (no merge commits)
- Force pushes and deletions are blocked
- Conversation resolution required
- Enforced for administrators
- Every merge into `rewrite` is a formal release (tag accordingly)

### `dev` — Development Integration Branch

- All feature / fix / chore / docs branches target `dev`
- Acts as the staging area; code lands here first and is integration-tested
- When `dev` reaches a release-ready state, a single release PR merges `dev` → `rewrite`

### Feature / Fix / Chore / Docs Branches

- **Naming**: `feature/<name>`, `fix/<name>`, `chore/<name>`, `docs/<name>`
  - Example: `feature/metadata-parser`
  - Example: `chore/branch-policy-update`
- **Workflow**:
  1. Create from `dev`: `git checkout dev && git pull && git checkout -b feature/your-feature`
  2. Make commits with clear messages
  3. Push and open a PR **targeting `dev`** (NOT `rewrite`)
  4. After approval and merge to `dev`, the feature branch can be deleted

### `old/main` — Legacy Archive (read-only)

- Renamed from `main` on 2026-08-27; archive of the legacy C#/.NET codebase
  (WPF v1.x + Avalonia v2.0), full history preserved
- Reference only; no development happens here; may be deleted when no longer needed

## Release Process

1. Feature development happens in `feature/*` (etc.) branches
2. PRs merge into `dev` for integration testing
3. When `dev` is release-ready, open the release PR: `dev` → `rewrite`
4. After approval, merge the release PR and tag the release on `rewrite`
5. `rewrite` always reflects the latest official release

## Branch Protection Rules

### `rewrite` (enabled via GitHub API, 2026-08-27)

✅ **Enabled Rules**:
- Require pull request reviews (1 approval required)
- Dismiss stale reviews when new commits are pushed
- Require linear history (rebase only, no merge commits)
- Require conversation resolution before merging
- Enforce for administrators
- Block force pushes
- Block deletions

❌ **Direct pushes to `rewrite` are BLOCKED** (including admins)
❌ **Feature branches must NOT target `rewrite`** — only `dev` → `rewrite` release PRs

## Workflow Example

```bash
# Start a feature branch from dev
git checkout dev
git pull origin dev
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push and create PR targeting dev
git push -u origin feature/my-feature
gh pr create --base dev --title "feat: my feature"

# After merge to dev, sync local dev
git checkout dev
git pull origin dev

# When dev is release-ready, open the release PR to rewrite
gh pr create --base rewrite --head dev --title "release: merge dev to rewrite"
```

## Questions?

If you have questions about this policy, please open an issue or reach out to the maintainers.

---

**Last Updated**: 2026-08-27
**Policy Version**: 3.0
