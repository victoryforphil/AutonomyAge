---
title: vs-viz
type: idea
status: todo
tags:
  - idea
---

# vs-viz (Idea)

- Visualization utilities / helpers crate, starting with Rerun but designed to be extensible to more backends
- Thin, ergonomic wrappers over Rerun's API: log helpers, timestamps, scene building, standard telemetry payloads
- Backend-agnostic core so other visualization sinks (e.g. custom viewers, logs) can be added later
- Integrates with `vs-wtf` time primitives and `vs-data-store` topics so data can be logged as it flows
- Source material: Rerun usage scattered around VictoryForPhil/AndreasLabs, consolidate and adapt from there
