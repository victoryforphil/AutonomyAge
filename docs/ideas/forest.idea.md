---
title: forest
type: idea
status: todo
tags:
  - idea
---

# forest (Idea)

- Scenario runner framework: run file-based scenarios / steps with an extendable framework
- Scenarios defined as files (markup/data) describing a sequence of steps, so scenario content and runner logic stay separate
- Steps are pluggable: register step handlers and compose scenarios from reusable building blocks
- Taken from project-firefly, adapted into a reusable crate
- Likely composes with `vs-broker` / `vs-data-store` to drive task steps, and with `valley` for validating scenario outcomes
- Useful for sims, hardware-in-the-loop, and replay of recorded runs
