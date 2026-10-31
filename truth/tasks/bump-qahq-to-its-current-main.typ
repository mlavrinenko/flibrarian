#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump qahq to its current main",
  status: done(2026, 10, 31),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

the pin c520b7d predates qahq 0.9.0, and 3feae8d is current main
