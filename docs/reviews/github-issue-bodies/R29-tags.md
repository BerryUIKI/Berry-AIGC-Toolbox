<!-- omera-review-2026-10-05:R29-tags -->

## Problem and evidence

The current Sidebar exposes a long alphabetical tag list without a direct tag search/filter affordance. Finding a particular tag requires scanning or scrolling. This is a usability proposal observed during native inspection.

Reviewed on Omera 0.4.3, `dev` at [`bdac8fb`](https://github.com/BerryUIKI/Omera/commit/bdac8fbcdbac5535075e091453b77c3afe518ba5), Windows development app launched with `pnpm run tauri dev`. Source: [Sidebar.vue](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/src/components/Sidebar.vue).

## Validation scenario

1. Populate an isolated library with many distinct tags.
2. Locate a particular tag in the Sidebar using current controls.
3. Compare with a searchable/filterable tag list while preserving current selected-tag behavior.

## Acceptance criteria

- [ ] Provide a localized, keyboard-operable tag search/filter control.
- [ ] Preserve selected tags and make empty/no-match states clear.
- [ ] Keep rendering/query work bounded for large tag collections and test rapid input.

## Scope

This is the layout/navigation portion of review finding R29, split from localization so it can be discussed and delivered independently. Preserve gallery card-width/virtualization rules and keep all new user text in locale files. Any implementation PR should target `dev`.
