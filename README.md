# prim-skills

Agent-agnostic skills and instructions — a single canonical source any coding
agent can consume (gentle-ai / Pi today; Claude, Kiro, OpenCode, etc. via
adapters, without changing this repo).

## Layout

```text
prim-skills/
├── manifest.toml       # canonical, agent-agnostic definition (instructions + roles + skills)
├── instructions/       # always-on / conditional steering, neutral markdown
│   ├── tech.md
│   ├── code-conventions.md
│   ├── odd.md
│   └── visual-style.md
└── skills/             # one dir per skill, universal SKILL.md format
    ├── architect-decisions/
    ├── knowledge-base/
    ├── logging/
    ├── migration/
    ├── project-setup/
    ├── skill-design/
    └── testing/
```

## Format

- **Skills** follow the near-universal `SKILL.md` convention: a directory whose
  `SKILL.md` has YAML frontmatter (`name`, `description`, triggers) plus a
  Markdown body, with optional `scripts/`, `assets/`, `references/`.
- **Instructions** are plain markdown (optionally with frontmatter). They map to
  what Kiro calls "steering" and what Pi/gentle inject into the system prompt.
- **`manifest.toml`** is the single source of truth. It describes three axes:
  content (instructions + skills + roles), while WHERE/HOW to install and which
  capabilities each agent supports are owned by each consumer's adapter.

## How it is consumed

This repo holds **content only**. Installation and curation into a specific
agent live elsewhere:

- **devlab** (`bit-crew/devlab`) clones this repo during provisioning and curates
  it into the Pi / gentle-ai install (copies skills into `~/.pi/agent/skills`,
  concatenates instructions, refreshes the skill registry), then removes the
  clone. The content is never vendored into devlab.

Keeping content here means the same skills work for any agent, with low coupling
and low tech debt: update a skill once, every consumer picks it up.
