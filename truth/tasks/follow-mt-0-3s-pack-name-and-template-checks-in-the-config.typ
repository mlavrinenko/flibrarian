#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Follow mt 0.3's pack name and template checks in the config",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      instant-search: neutral,
      any-fb2: neutral,
      safe-index: neutral,
      one-core: neutral,
      usable-files: neutral,
    ),
    process: (
      sound-codebase: helps[every `mt` command and the commit hook run again],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 29),
)

== Problem

+ Run `mt check` here with `mt` 0.3.0.
+ 🔴 It fails: `extends: unknown pack: 'mindtape-tasks'`, and once renamed, finding.typ's `context` and `observation` and tweak.typ's `why` are `text` slots whose examples hold code spans, which 0.3.0 refuses. The commit hook runs the same check, so no commit lands.


== TO-BE

- 🟢 `mt check` passes on `mt` 0.3.0 with the same rules as before.


== Changes

- `.mindtape/config.toml`: `extends = ["mindtape:tasks"]`.
- `.mindtape/templates/finding.typ`, `.mindtape/templates/tweak.typ`: the three slots become `raw`.


== Rollback

- Revert the commit and use an `mt` older than 0.3.0.


== Out of scope

- Any rule change. The local rules still layer on the same pack.


== Dogfood

The commit hook checks this commit with `mt` 0.3.0.

