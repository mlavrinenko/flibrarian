#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump qahq past the naersk anchor move",
  status: done(2026, 10, 31),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

qahq main moved the shared naersk anchor from mindtape to linecop, so the lock follows it
