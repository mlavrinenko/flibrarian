#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fork and land worktrees with in-repo worktrunk hooks that prepare the frontend",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "frontend"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[every worktree gates the same way, with nothing set up by hand],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 11),
)

== Problem

+ `wt switch --create x` in flibrarian, then `cd ../flibrarian-x && just check`.
+ 🔴 `flibrarian-web` fails to compile: `#[folder = "../../frontend/dist/"]` in crates/flibrarian-web/src/main.rs embeds a gitignored directory a fresh worktree lacks.
+ 🔴 With `frontend/dist` present but no `frontend/node_modules`, `just check-frontend` hangs on `bunx`. On 2026-10-11 an agent got past both only by copying dist and symlinking node_modules into the primary checkout.
+ 🔴 The repo has no `.config/wt.toml`, and ~/.nix no longer ships worktrunk user hooks, so a worktree also needs a manual `direnv allow`, and `wt merge` runs no gate.


== TO-BE

- 🟢 `just fork <slug>` gives a worktree whose `just check` passes with no hand setup: the dev shell is allowed, `bun install` and `bun run build` have run in `frontend/`.
- 🟢 `just land` runs `just check` and `mt check --commits {{ target }}..HEAD` before fast-forwarding, then imports gate records with `mmz --import` if the repo uses mmz.
- 🟢 `worktrunk` comes from the flake's dev shell, so a clone needs only nix and direnv.


== Changes

- `.config/wt.toml`: pre-start (direnv allow, then `bun install --frozen-lockfile` and `bun run build` in `frontend/`), pre-merge (gate, then the wip check), pre-remove (record import), as in canibump's `.config/wt.toml` (commits 753011a, 1a77431).
- `flake.nix`: `worktrunk` in the dev shell.
- `Justfile`: `fork <slug>` runs `wt switch --create <slug> --yes`; `land` runs `wt merge --no-squash --no-commit --yes`.
- AGENTS.md and CONTRIBUTING.md: delegate with `just fork`, land with `just land`.


== Rollback

- Revert the commit. Worktrees then need `direnv allow` and a frontend build by hand again.


== Out of scope

- Committing `frontend/dist`: a build output stays generated.
- The other repos' hooks, each filed in its own repo.


== Dogfood

The next delegated task here forks with `just fork`, gates with no hand setup and lands with `just land`.

