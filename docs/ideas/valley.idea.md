---
title: valley
type: idea
status: todo
tags:
  - idea
---

# valley (Idea)

- Validation helper framework: extensible, scriptable checks with clear pass/fail results
- Imported from project-firefly's validation systems, adapted into a reusable crate
- Scriptable checks allow defining validation rules as data / scripts rather than hardcoded code
- Each check produces structured pass/fail + diagnostics rather than a bare boolean
- May need a backend: `vs-data-store` for storing results, plus a sim / task framework to drive checks
- Could compose with `forest` (scenario runner) for running validation suites against scenarios
