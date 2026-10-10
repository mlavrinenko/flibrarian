#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Adopt MindTape as the task tracker",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[a malformed task fails `just check` like any other gate],
      docs-match-code: helps[the backlog says what each change is for, next to the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `ls truth` in a fresh clone.
+ 🔴 It prints `No such file or directory`: flibrarian has no task tracker, so work lives in chat logs and heads.


== TO-BE

- 🟢 `mt ls` lists the backlog, and `just check` validates it with every other gate.
- 🟢 A commit names its task in a `Refs:` footer, and the `commit-msg` hook refuses one that does not.


== Changes

- `.mindtape/config.toml`: statuses, ICE priority, difficulty, src-files estimate, kind, assignee, WIP caps, goal check, views, commit rules.
- `.mindtape/templates/*.typ`: task, feat, fix, dev, docs, finding, tweak, with this repo's goals.
- `truth/lib.typ`, `truth/goals.typ`, `truth/process-goals.typ`: the goal registries the check reads. The product goals are a draft for the maintainer to review.
- `Justfile`, `flake.nix`: `just check-tasks` in `just check`, and `just install-hooks` run by the dev shell.
- `.githooks/commit-msg`: `mt check --commit-msg`.


== Rollback

- Revert the commit. No crate code depends on it.


== Out of scope

- A close gate. flibrarian has no mmz, so nothing proves `just check` passed when a task flips to `done`.


== Dogfood

This repo's own work is filed and closed through it from the next task on.

