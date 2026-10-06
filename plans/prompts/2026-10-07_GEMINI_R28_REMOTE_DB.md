You are assigned one Omera task: R28-remote-db / GitHub issue #289.
Repository: D:\dev\Omera
GitHub repository: BerryUIKI/Omera

R28 / #257 is already completed and merged in PR #320. Preserve that upload-only cloud-sync wording. This assignment concerns storage-backend claims only.

Read repository AGENTS.md, plans/AGENT_RUNBOOK.md, plans/tasks/R28-remote-db.md, the relevant STATUS.md row, and plans/COMPLETION_WORKFLOW.md. Read current issue #289 before editing; do not load the entire remediation register.

Deliver the smallest complete correction:
1. Trace the currently selectable/runtime database backend through AppState, configuration handling and Settings. Verify current behavior rather than trusting historical prose.
2. Correct README, supported translated READMEs, wiki/setup documentation and other directly related user claims to describe SQLite as the supported runtime. Mark MySQL/PostgreSQL team storage according to its actual unavailable/planned/experimental state.
3. Distinguish SQL export, backend traits and connection tests from a functioning selectable application backend. Do not implement a remote database or modify storage semantics to match marketing text.
4. Complete every acceptance item in R28-remote-db.md and its V5 verification. Check consistency across supported documentation languages and preserve the previous cloud-sync correction.

Workflow authorization and completion requirements:
- The user explicitly authorizes you to finish this assigned task end to end: update documentation, open and merge its PR into dev after required checks/reviews, delete its safely obsolete local/remote feature branches, and close its assigned issue after integrated-candidate acceptance. Do not ask for another routine confirmation for these authorized steps.
- Read planning files from D:\dev\Omera\plans even when working in an isolated checkout; this directory is currently untracked. Carry/stage only the intentional task documentation files, not the whole plans/reviews directory.
- Follow plans/COMPLETION_WORKFLOW.md. Preserve unrelated changes; use an isolated branch/worktree based on the latest origin/dev when another agent is active. Never target main, bypass branch protections, force-push, or delete unrelated branches/user data.
- Update affected documentation and this task's STATUS.md row, model-queue entry and DISPATCH_ORDER.md entry. Keep concurrent agents' changes and use a dated task handoff. Engineering/GitHub prose must be English; supported user translations must agree. Preserve frozen historical review evidence.
- Do not mark verified before the fix is merged and its acceptance is checked on integrated dev. Publish final PR/merge-SHA/closure evidence in the maintained docs, using a small documentation follow-up PR if needed.
- After successful integration and acceptance, post an English evidence summary and close ONLY the assigned issue. Delete ONLY safely merged branches belonging to this task, after verifying no unique/unpushed work or active worktree depends on them.
- If required CI, review, permissions or acceptance cannot be satisfied, report the exact blocker and leave the issue open. An unexecuted test is not a pass.
- End with the merged PR URL, exact integrated commit, issue status, verification results, documentation updates, branch cleanup and remaining limits. Stop; do not start another task.
