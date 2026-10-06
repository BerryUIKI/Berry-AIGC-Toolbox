<!-- omera-review-2026-10-05:R29-table -->

## Problem and evidence

During native inspection at a normal 1280×850 window, the filename column occupied substantial fixed space while prompt/model information required horizontal scrolling. This is a usability proposal; no incorrect pixel rendering or performance regression is claimed.

Reviewed on Omera 0.4.3, `dev` at [`bdac8fb`](https://github.com/BerryUIKI/Omera/commit/bdac8fbcdbac5535075e091453b77c3afe518ba5), Windows development app launched with `pnpm run tauri dev`. Source: [FileList.vue](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/src/components/FileList.vue).

## Validation scenario

1. Open Table at a normal window width with both side panes visible.
2. Compare visible filename, prompt and model information.
3. Evaluate a compact/default column set, resizing and saved column visibility.

## Acceptance criteria

- [ ] Allow relevant columns to be shown/hidden/resized with accessible controls.
- [ ] Provide a usable compact default at narrow/normal widths and persist user preferences.
- [ ] Verify keyboard navigation, long names, horizontal scrolling and narrow/wide windows.

## Scope

This is the layout/navigation portion of review finding R29, split from localization so it can be discussed and delivered independently. Preserve gallery card-width/virtualization rules and keep all new user text in locale files. Any implementation PR should target `dev`.
