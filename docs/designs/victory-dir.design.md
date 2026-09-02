---
title: Victory Dir
type: design
status: active
tags:
  - design
  - dir
---

# Victory Dir

`vs-dir` provides one small struct-based path locator for Victory Suite tools.

## Resolution

`DirLocator::app_dir()` uses the same order for the application, logs, and data
paths:

```text
configured environment base/.dot
→ current working directory/.dot
→ Git repository root/.dot
→ Cargo crate root/.dot
→ HOME/.dot
```

The configured environment variable is explicit and names a base directory. The
locator always appends the configured dot component. Empty values are ignored.
Project candidates are checked without creating directories; only the final home
fallback or a selected subdirectory is created.

## API boundary

`DirConfig` owns the literal dot component and optional environment variable.
`DirLocator` owns that configuration plus the launch cwd. Directory resolution
and creation are member methods, keeping logging and other paths on one
resolution path.

The crate does not parse configuration, load agent instructions, initialize
logging, or manage artifacts. Those concerns remain with consumers.

## Layout

```text
libs/vs-dir/src/
├── lib.rs
├── config.rs
└── path/
    ├── mod.rs
    ├── locator.rs
    ├── utils.rs
    └── tests.rs
```

Tests are high-level scenarios in `src/path/tests.rs`. New behavior should
extend an existing scenario rather than create a one-case test for every edge.
