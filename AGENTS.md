# Agent Instructions

## Building

- Never use `--features bundled-duckdb` during development. It hogs RAM and is only needed for production builds. Just use `cargo build` / `cargo run` without it.

## File Size Limits

1. For Rust files with inline tests: run `ejectest src/path/to/file.rs`
2. For other cases: refactor into smaller modules

## Build & Test Commands

- Build: `cargo build`
- Test all: `cargo test`
- Test single: `cargo test test_name`
- Lint: `cargo clippy --all-targets`
- Format: `cargo fmt`
- Check: `just check` (runs fmt, lint, test)

## Code Style

- Follow Rust 2024 edition idioms
- No code comments unless they explain a non-obvious "why"
- Keep functions small and focused (~5-25 lines ideal, 26-50 acceptable, 51+ too long)
- Avoid deeply nested code, extract into new functions
- All lints from workspace Cargo.toml must pass (clippy pedantic + nursery)
- Use `anyhow` for application error handling
- Prefer `thiserror` for library error types

## Tasks

- Tasks live in `truth/tasks/` under [MindTape](https://github.com/mlavrinenko/mindtape): load the `mindtape` skill and use `mt` for every task change. `mt ls @agent` lists open agent work.
- File with `mt add -t feat` (or `fix`, `dev`, `docs`, `finding`, `tweak`). Every open task gives each goal in `truth/goals.typ` and `truth/process-goals.typ` a verdict.
- Flip a task with the commit that carries its work (`mt wip <stem>` in the first, `mt done <stem>` in the last) and name it in a `Refs: <stem>` footer. The `commit-msg` hook (`.githooks/`, wired by `just install-hooks`) runs `mt check --commit-msg`.
- `just check-tasks` validates the backlog and is part of `just check`.

## Worktrees

- Delegated work forks with `just fork <slug>` (a sibling worktree on a new branch, dev shell allowed, `frontend/node_modules` installed and `frontend/dist` built, so `just check` passes there with nothing set up by hand) and lands with `just land` run in that worktree: it gates, checks no named task is left `wip`, fast-forwards main and removes the worktree. Both wrap [worktrunk](https://worktrunk.dev); [.config/wt.toml](.config/wt.toml) holds the hooks. Never run a bare `wt merge`: it squashes away the `Refs:` footers.
