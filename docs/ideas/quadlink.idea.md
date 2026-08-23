---
title: quadlink
type: idea
status: todo
tags:
  - idea
---

# quadlink (Idea)

- MAVLink communication and command framework for sending / receiving MAVLink messages and issuing commands
- Handles serialization, connection handling, heartbeats, and command conventions in one place
- Taken / adapted from a few sources: project-firefly, other sims, and rust projects in VictoryForPhil / AndreasLabs
- Integrates with `vs-wtf` for timestamps and could bridge to `vs-data-store` / `vs-broker` for telemetry and task control
- Aims to be the shared MAVLink layer so drones, sims, and ground tooling talk the same protocol
