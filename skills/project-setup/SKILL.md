



---
name: setup
description: Initialize new Python projects with complete tooling. Use when starting a new project or setting up environment from scratch.
allowed-tools: Read, Grep, Glob, Write, Edit, Bash
---

# Setup Skill

Complete project initialization workflow for Python 3.12+ projects.

## Workflow Sequence

Execute steps in order:

### 1. Initialize Git
```bash
git init
git add .
git commit -m "Initial commit"
```

### 2. Create .gitignore
Copy from `skills/devops/assets/.gitignore` or create minimal:
```
__pycache__/
*.py[cod]
.env
.venv/
*.egg-info/
dist/
build/
.pytest_cache/
.ruff_cache/
.secrets.baseline
```

### 3. Copy Configuration Files
From `skills/devops/assets/`:
- `pyproject.toml` - project metadata + tool configs
- `.pre-commit-config.yaml` - pre-commit hooks

### 4. Install Dependencies
```bash
python -m pip install -e ".[dev]"
pre-commit install
pre-commit install --hook-type commit-msg --hook-type pre-push
```

### 5. Validate Setup
```bash
makevalidate  # or ruff check && ruff format --check && bandit -c pyproject.toml -r skills/
```

## Configuration Templates Location

```
skills/devops/assets/
├── pyproject.toml          # Ruff, codespell, pytest config
├── .pre-commit-config.yaml # All hooks (security, lint, format)
├── .gitignore             # Python project ignores
└── Makefile               # Common targets (optional)
```

## Makefile Target Template

```makefile
.PHONY: install test lint format validate clean

install:
    python -m pip install -e ".[dev]"
    pre-commit install

test:
    pytest

lint:
    ruff check .

format:
    ruff format .

validate:
    ruff check . && ruff format --check . && bandit -c pyproject.toml -r skills/

clean:
    rm -rf __pycache__/ .pytest_cache/ .ruff_cache/ *.egg-info/
```

## Tool Version Checks

Ensure installed versions match config:
- Python >= 3.12
- ruff >= 0.9.4
- bandit >= 1.8.2
- semgrep >= 1.97.0
- safety >= 3.3.0

## Post-Setup Validation

After initial setup, verify:
1. `ruff check .` passes
2. `ruff format --check .` passes
3. `bandit -r skills/` passes
4. `pre-commit run --all-files` passes

## Asset Copy Commands

```bash
cp skills/devops/assets/pyproject.toml .
cp skills/devops/assets/.pre-commit-config.yaml .
cp skills/devops/assets/.gitignore .
```

---

## Per-Project Isolation (uv + direnv + gcloud)

Rule the agent MUST enforce: each project keeps its dependencies, environment
variables, and cloud credentials isolated. Correct the user if a project mixes
global state (e.g. switching gcloud account globally, installing deps outside a
venv, or sharing one venv across projects).

### Dependencies by language

| Language | Isolation | Needs direnv? |
|---|---|---|
| Python | per-project `.venv` created by `uv` — NOT automatic, must be activated | Yes — to activate the venv for the editor/LSP |
| Rust | `Cargo.toml` + local `target/` — automatic, never shared | No (only for env vars) |
| Go | `go.mod` per project — automatic, never shared | No (only for env vars) |
| Text-only | nothing to isolate | No |

Python is the ONLY language that is not auto-isolated. Rust and Go isolate
dependencies natively through their package manager; do not add a venv-style
layer for them.

### uv vs direnv — do not confuse them

- `uv` manages the venv and dependencies. `uv run <cmd>` already executes inside
  the project venv WITHOUT activating anything. For pure terminal work, `uv run`
  alone is enough and direnv is redundant for dependencies.
- `direnv` loads/unloads environment variables per directory via `.envrc`. It is
  language-agnostic and knows nothing about Python.
- Keep direnv for TWO jobs uv does not do:
  1. Activate the venv so the editor/LSP (Helix + basedpyright/ruff) sees the
     project's dependencies on entry. Helix launches the LSP with the PATH
     `python`, not `uv run`, so without activation imports resolve as missing.
  2. Project-scoped environment variables and cloud context (below).

### Python project `.envrc`

```bash
# .envrc — activate the uv venv on entry (official direnv Python#uv pattern)
source .venv/bin/activate
```

Create the venv first with `uv venv`, then `direnv allow` once (direnv blocks a
new `.envrc` until you authorize it — security by design).

### gcloud: multiple accounts, one per project (no global clobber)

Problem: `gcloud config set account ...` writes to the GLOBAL config
(`~/.config/gcloud/`). Two projects setting different accounts overwrite each
other — switching folders does NOT switch account. This is the trap to avoid.

Correct pattern = named configurations + a per-project env var:

1. Authenticate each account once (credentials live in a shared store):
   ```bash
   gcloud auth login   # repeat for each account
   ```
2. Create one named configuration per client:
   ```bash
   gcloud config configurations create client-a
   gcloud config set account a@company-a.com
   gcloud config set project project-a

   gcloud config configurations create client-b
   gcloud config set account b@company-b.com
   gcloud config set project project-b
   ```
3. In each project's `.envrc`, select the configuration by env var (per-process,
   does NOT touch global active config):
   ```bash
   # project A
   export CLOUDSDK_ACTIVE_CONFIG_NAME=client-a
   ```
   ```bash
   # project B
   export CLOUDSDK_ACTIVE_CONFIG_NAME=client-b
   ```

`CLOUDSDK_ACTIVE_CONFIG_NAME` overrides the global active config for that shell
only, so entering a folder switches the gcloud account with no risk of one
project clobbering the other. Same idea applies to `AWS_PROFILE`, `GOOGLE_...`,
database URLs, and secrets: project-scoped vars belong in `.envrc`, never set
globally.
