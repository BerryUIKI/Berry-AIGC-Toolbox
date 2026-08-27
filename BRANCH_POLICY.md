# Branch Management Policy

## Overview

As of 2026-08-27, the repository was restructured: the old C#/.NET codebase was
archived and development restarted from scratch on the orphan branch `rewrite`
(new tech stack: Tauri/Rust). All legacy branches (`main`, `dev`, `feature/*`,
etc.) have been deleted. This policy describes the current structure.

## Branch Structure

### Main Branch: `rewrite`

- **`rewrite`** - Production-ready / active development line (default branch)
  - 🔒 **Protected branch** - Direct pushes are **FORBIDDEN**
  - All changes must come via Pull Requests
  - Requires 1 approval before merging
  - Linear history enforced (no merge commits)
  - Force pushes and deletions are blocked
  - Conversation resolution required
  - Enforced for administrators

### Archive Branch: `old/main`

- **`old/main`** - Read-only archive of the legacy C#/.NET codebase
  - Renamed from `main` on 2026-08-27
  - Contains the full history of the old project (WPF v1.x + Avalonia v2.0)
  - Kept for reference only; no new development happens here
  - May be deleted at any time once no longer needed

### Feature Branches

- **Naming Convention**: `feature/<feature-name>`
  - Example: `feature/initial-scaffold`
  - Example: `feature/metadata-parser`

- **Workflow**:
  1. Create from `rewrite`: `git checkout rewrite && git pull && git checkout -b feature/your-feature`
  2. Make commits with clear messages
  3. Push to remote: `git push -u origin feature/your-feature`
  4. Create PR targeting `rewrite` (NOT `old/main`)
  5. After approval and merge to `rewrite`, the feature branch can be deleted

### Bugfix Branches

- **Naming Convention**: `fix/<bug-name>`
  - Example: `fix/rating-serialization`

### Chore / Docs Branches

- **Naming Convention**: `chore/<topic>` or `docs/<topic>`
  - Example: `chore/branch-policy-update`
  - Same workflow as feature branches

## Branch Protection Rules (rewrite)

✅ **Enabled Rules** (configured via GitHub API, 2026-08-27):
- Require pull request reviews (1 approval required)
- Dismiss stale reviews when new commits are pushed
- Require linear history (rebase only, no merge commits)
- Require conversation resolution before merging
- Enforce for administrators
- Block force pushes
- Block deletions

❌ **Direct pushes to rewrite are BLOCKED** (including admins)

## Release Process

1. Development happens in `feature/*` (or `fix/*`, `chore/*`) branches
2. PRs merge into `rewrite` for integration testing
3. When ready for release, tag releases on `rewrite` branch
4. (Optional) If a release branch is ever needed, create `release/<version>` from `rewrite`

## Workflow Example

```bash
# Start new feature
git checkout rewrite
git pull origin rewrite
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push and create PR
git push -u origin feature/my-feature
gh pr create --base rewrite --title "feat: my feature"

# After merge, sync local rewrite
git checkout rewrite
git pull origin rewrite
```

## Questions?

If you have questions about this policy, please open an issue or reach out to the maintainers.

---

**Last Updated**: 2026-08-27
**Policy Version**: 2.0
