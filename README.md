# Berry-AIGC-Toolbox

> **⚠️ Clean-Slate Rewrite (Orphan Branch: `rewrite`)**
> This branch starts from zero with a new tech stack. All previous code
> (WPF v1.x in `legacy/`, Avalonia v2.0 in `src/`) lives on the old branches
> (`main`, `dev`, etc.) and is intentionally NOT carried over.

## 简介 / About

Berry-AIGC-Toolbox 是一个 AI 生成图像的元数据索引器与查看器，帮助用户组织、
搜索和管理不断增长的 AI 生成内容集合。

Berry-AIGC-Toolbox is an AI-generated image metadata indexer and viewer that
helps you organize, search, and manage your growing collection of AI-generated
content.

## 技术栈 / Tech Stack (Planned)

| 组件 Component | 技术 Technology |
|---|---|
| Desktop Shell | [Tauri 2](https://tauri.app/) (Rust) |
| Frontend | Web frontend (to be decided) |
| Backend / Core | Rust |

> Stack details are a placeholder — final choices will be recorded here as the
> rewrite progresses.

## 开发状态 / Status

- 🟡 **In progress**: initial scaffolding on the `rewrite` orphan branch
- Not yet runnable

## 开发 / Development

Prerequisites and build instructions will be added once the initial scaffold
is in place.

## 分支策略 / Branch Policy

See [BRANCH_POLICY.md](BRANCH_POLICY.md) for the full branching strategy.
`main` is protected: all changes flow through PRs via `dev`.

## 许可 / License

[MIT](LICENSE)

---

**Project Status**: 🟡 Clean-slate rewrite (bootstrap)
**Branch**: `rewrite` (orphan, no shared history)
**Last Updated**: 2026-08-27
