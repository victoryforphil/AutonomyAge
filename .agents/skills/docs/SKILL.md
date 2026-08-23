---
name: docs
description: Conventions for this repo's docs/ folder — idea tickets, Obsidian-style frontmatter, the status lifecycle, and the {doc_name}.{type}.md filename pattern. Load when creating, updating, or reviewing docs under docs/.
---

# Docs

Conventions for the `docs/` folder. Keep docs scannable, uniform, and machine-friendly.

## Filename pattern

Every doc encodes its **type** in the filename suffix, and the suffix must match the `type` field in frontmatter.

- `{doc_name}.idea.md` — idea ticket
- `{doc_name}.plan.md` — plan
- `{doc_name}.design.md` — design doc

Other types follow the same shape: `{doc_name}.{type}.md`.

## Frontmatter

Every doc starts with Obsidian-style YAML frontmatter (`---` delimited):

```yaml
---
title: Human-readable Name
type: idea        # must match the filename suffix
status: todo      # lifecycle state
tags:
  - idea
---
```

- `type` must equal the filename suffix.
- `status` drives the lifecycle below.

## Status lifecycle

Docs move through `status` states. Add frontmatter to every new doc.

```yaml
status: todo | done | active | deprecated
```

- `todo` — planned, not built. Lives in the type folder (`docs/ideas/`).
- `done` — complete. Move the file into the type folder's `done/` subdir (e.g. `docs/ideas/done/`).
- `active` / `deprecated` — living docs and retired ones.

## Ideas

Idea tickets live in `docs/ideas/` as `{doc_name}.idea.md`. They are short notes, not full specs: a one-line purpose, sourcing notes (e.g. "import from project-firefly / AndreasLabs"), and a bullet feature list. When an idea becomes real work, promote it to a `plan` or `design` doc and flip its `status` to `done`.

## Generic rules

- Lead with the decision or purpose. Prefer short sections and bullets.
- Add frontmatter to every new doc; keep `type` and `status` correct.
- `docs/TODO.md` links each idea to its `docs/ideas/{name}.idea.md` entry.
- Don't duplicate deeper specs — link to them.
