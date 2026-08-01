# Branch Management Policy

## Branch Structure

Berry-AIGC-Toolbox follows a structured branching strategy to ensure code quality and stable releases.

### Main Branches

- **`main`** - Production-ready code only
  - 🔒 **Protected branch** - Direct pushes are **FORBIDDEN**
  - All changes must come via Pull Requests from `dev`
  - Requires 1 approval before merging
  - Linear history enforced (no merge commits)
  - Force pushes and deletions are blocked
  - Conversation resolution required

- **`dev`** - Development integration branch
  - All feature branches should target this branch
  - Acts as a staging area before merging to `main`
  - Pull Requests should be submitted here first

### Feature Branches

- **Naming Convention**: `feature/<feature-name>`
  - Example: `feature/data-layer-migration`
  - Example: `feature/domain-layer-implementation`

- **Workflow**:
  1. Create from `dev`: `git checkout dev && git pull && git checkout -b feature/your-feature`
  2. Make commits with clear messages
  3. Push to remote: `git push -u origin feature/your-feature`
  4. Create PR targeting `dev` (NOT `main`)
  5. After approval and merge to `dev`, create PR from `dev` to `main`

### Bugfix Branches

- **Naming Convention**: `fix/<bug-name>`
  - Example: `fix/rating-serialization`

### Release Process

1. Feature development happens in `feature/*` branches
2. PRs merge into `dev` for integration testing
3. When ready for release, PR from `dev` to `main`
4. Tag releases on `main` branch

## Branch Protection Rules

### main Branch Protection

✅ **Enabled Rules**:
- Require pull request reviews (1 approval required)
- Dismiss stale reviews when new commits are pushed
- Require linear history (rebase only, no merge commits)
- Require conversation resolution before merging
- Enforce for administrators
- Block force pushes
- Block deletions

❌ **Direct pushes to main are BLOCKED**

## Workflow Example

```bash
# Start new feature
git checkout dev
git pull origin dev
git checkout -b feature/my-feature

# Make changes and commit
git add .
git commit -m "feat: add new feature"

# Push and create PR
git push -u origin feature/my-feature
gh pr create --base dev --title "feat: my feature"

# After merge to dev, sync local dev
git checkout dev
git pull origin dev

# When ready for release, create PR to main
gh pr create --base main --head dev --title "release: merge dev to main"
```

## Questions?

If you have questions about this policy, please open an issue or reach out to the maintainers.

---

**Last Updated**: 2026-08-01
**Policy Version**: 1.0