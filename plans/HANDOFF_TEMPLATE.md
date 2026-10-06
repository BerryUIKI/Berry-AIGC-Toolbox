# Single-task handoff template

Copy into a task-specific work log or final handoff. Keep source evidence immutable.

```text
Review ID / issue / package:
Assigned owner / lead reviewer:
Starting branch / HEAD / unrelated dirty state:
Prerequisite integration commits and approved contracts:
Reproduced current production path or verified already-fixed result:
Concrete before -> after behavior:
Changed files / production entry points:
Compatibility, source-preservation and partial outcomes:
Targeted acceptance cases: actual result for each:
Verification profile / commands / exact outcomes:
Native GUI / provider / platform cases executed:
Not-run cases, failures and material limits:
Documentation / IPC inventory / independent DTO checks:
Implementation commit / PR, if authorized:
Integrated candidate commit / acceptance evidence:
Remaining dependencies / smallest next step:
Recommended status: unverified | reproduced | in-progress |
                    ready-for-review | verified | blocked | deferred
```

`verified` requires integrated-candidate acceptance. `ready-for-review` is not issue closure. A deferred/waived issue remains unresolved. A test that cannot run has an explicit reason and follow-up; it is never reported as passed.
