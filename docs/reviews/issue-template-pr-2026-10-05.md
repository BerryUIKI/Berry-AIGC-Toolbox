The repository has no issue forms, so reports do not consistently capture the version, reproduction, expected/actual behavior, or acceptance criteria. Add English Bug report and Feature request forms using existing labels, and document focused reporting and follow-up conventions in CONTRIBUTING.md.

Validation:
- Both forms pass strict YAML parsing.
- Required metadata, supported field types, unique field IDs, and field labels were checked.
- `git diff --check` passes.

This change only adds repository contribution documentation and forms. The review baseline has an unrelated `cargo fmt --check` failure in `src-tauri/src/commands.rs`; this PR does not modify that file.

Activation: GitHub displays issue templates from the default branch, currently `main`. This PR targets `dev` per the repository integration policy. The forms become available in the issue chooser when the normal release flow carries them into `main`. See [GitHub's issue-template documentation](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository).

