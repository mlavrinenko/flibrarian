#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep a green check-frontend quiet",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling", "frontend"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[a red linter stands out instead of hiding among green summaries],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 11),
)

== Problem

+ Run `just check-frontend` on a clean main.
+ 🔴 It prints about nine lines (Prettier, ESLint, svelte-check and linecop summaries) although it passed, and `just check` repeats them on every green run.


== TO-BE

- 🟢 A passing `just check-frontend` prints nothing; a failing linter prints its full output.


== Changes

- `flake.nix`: `moreutils` in the dev shell, for `chronic`.
- `Justfile`: each `check-frontend` command runs under `chronic`.


== Rollback

- Revert the commit.


== Out of scope

- The nix "untrusted flake configuration" lines `direnv exec` prints: a host setting, not this repo's.


== Dogfood

`just check` here prints nothing on a green run from the frontend arm.

