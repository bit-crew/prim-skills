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

This repo holds **content** (the skills + instructions + manifest) **and its own
installer** (`installer/`, a Rust binary `prim-skills-install`).

- **`prim-skills-install`** installs every skill into `~/.agents/skills` (a Pi
  scan root), unified in one place:
  - **own skills** (this repo's `skills/*`) are copied in;
  - **external skills** (Archify, grill-me) are installed from source with the
    `skills` CLI so they stay updatable.
  Re-running it refreshes; `prim-skills-install update` updates the external
  skills. Target agent: **Pi** (for now). Needs Node/npx (the environment
  installer, devlab, owns Node).

- Install / bootstrap:
  ```sh
  curl -fsSL https://raw.githubusercontent.com/bit-crew/prim-skills/main/install.sh | sh
  ```

- Update external skills to their latest version:
  ```sh
  curl -fsSL .../install.sh | sh -s -- update
  ```

- Uninstall every managed skill from `~/.agents/skills` (own + external):
  ```sh
  curl -fsSL .../install.sh | sh -s -- uninstall
  ```

  Running the binary directly (if already downloaded): `prim-skills-install`
  (install/refresh), `prim-skills-install update`, `prim-skills-install uninstall`.

- **devlab** (`bit-crew/devlab`) provisions the environment + Pi/gentle-ai and,
  on the VPS profile, calls `prim-skills-install` automatically. devlab no longer
  copies skills itself; it still curates its own instructions into Pi's AGENTS.md
  (that steering is specific to devlab, not universal skills).

Keeping skills here (content + installer) means the same set works for any agent,
with low coupling: update a skill once, re-run the installer, every consumer
picks it up.
