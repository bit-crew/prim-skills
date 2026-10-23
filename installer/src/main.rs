//! prim-skills-install — installs the agent-agnostic skills into the agent.
//!
//! Two kinds of skill, one destination:
//!   - OWN skills (this repo's `skills/*`): copied into ~/.agents/skills.
//!   - EXTERNAL skills (Archify, grill-me): installed from source with the
//!     `skills` CLI (`npx skills add ... --agent pi`), which also writes to
//!     ~/.agents/skills. Installing from source keeps them updatable.
//!
//! ~/.agents/skills is a scan root for Pi (verified in gentle-pi's
//! skill-registry), so everything lands in one place Pi reads.
//!
//! Target agent: Pi (for now). The set of skills is universal (no profiles).
//!
//! Subcommands:
//!   (none)   install/refresh all skills (idempotent)
//!   update   update external skills to latest (`skills update`)
//!
//! Dependencies: Node/npx (for the `skills` CLI). If missing, external skills
//! are skipped with a note — the environment installer (devlab) owns Node.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const BLUE: &str = "\x1b[1;34m";
const GREEN: &str = "\x1b[1;32m";
const RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

/// This content repo, cloned at install time to get the OWN skills.
const SKILLS_REPO: &str = "https://github.com/bit-crew/prim-skills.git";

/// External skills installed from source via the `skills` CLI.
/// (repo, skill-name-or-"*")
const EXTERNAL_SKILLS: &[(&str, &str)] = &[
    ("tt-a1i/archify", "archify"),
    ("mattpocock/skills", "grilling"),
];

fn log(msg: &str) {
    println!("{BLUE}[prim-skills]{RESET} {msg}");
}
fn ok(msg: &str) {
    println!("{GREEN}[prim-skills]{RESET} {msg}");
}
fn err(msg: &str) {
    eprintln!("{RED}[prim-skills]{RESET} {msg}");
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    let result = match arg.as_str() {
        "update" => run_update(),
        _ => run_install(),
    };
    if let Err(e) = result {
        err(&format!("fallo: {e}"));
        std::process::exit(1);
    }
}

/// Install/refresh all skills into ~/.agents/skills.
fn run_install() -> io::Result<()> {
    let home = dirs::home_dir().ok_or_else(|| io::Error::other("sin HOME"))?;
    let dest = agents_skills_dir(&home);
    std::fs::create_dir_all(&dest)?;

    install_own_skills(&dest)?;
    install_external_skills();

    ok("Skills instaladas/actualizadas en ~/.agents/skills.");
    Ok(())
}

/// Update external skills to their latest version (`skills update`).
fn run_update() -> io::Result<()> {
    if !has_npx() {
        err("npx no disponible; no se pueden actualizar skills externas.");
        return Ok(());
    }
    log("Actualizando skills externas (skills update)...");
    let _ = run_npx(&["-y", "skills", "update", "--global", "--yes"]);
    ok("Actualizacion de skills externas completa.");
    Ok(())
}

/// ~/.agents/skills — the unified destination (a Pi scan root).
fn agents_skills_dir(home: &Path) -> PathBuf {
    home.join(".agents").join("skills")
}

/// Clone this repo to a temp dir and copy its `skills/*` into the destination.
/// The clone is removed afterward so nothing lingers.
fn install_own_skills(dest: &Path) -> io::Result<()> {
    let tmp = std::env::temp_dir().join(format!("prim-skills-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    log("Clonando prim-skills (skills propias)...");
    if !run_ok(
        "git",
        &["clone", "--depth", "1", SKILLS_REPO, tmp.to_str().unwrap()],
    ) {
        err("no se pudo clonar prim-skills; se omiten las skills propias.");
        return Ok(());
    }
    let from_root = tmp.join("skills");
    let mut n = 0;
    if let Ok(entries) = std::fs::read_dir(&from_root) {
        for entry in entries.filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_dir() {
                let to = dest.join(entry.file_name());
                let _ = std::fs::remove_dir_all(&to); // refresh, don't merge stale
                if copy_dir(&p, &to).is_ok() {
                    n += 1;
                }
            }
        }
    }
    let _ = std::fs::remove_dir_all(&tmp);
    log(&format!(
        "{n} skills propias instaladas en {}",
        dest.display()
    ));
    Ok(())
}

/// Install external skills from source with the `skills` CLI (updatable).
fn install_external_skills() {
    if !has_npx() {
        err("npx no disponible; se omiten las skills externas (Node lo instala devlab).");
        return;
    }
    for (repo, skill) in EXTERNAL_SKILLS {
        log(&format!("Instalando skill externa {repo} ({skill})..."));
        let ok_install = run_npx(&[
            "-y", "skills", "add", repo, "--skill", skill, "--agent", "pi", "--global", "--copy",
            "--yes",
        ]);
        if !ok_install {
            err(&format!("fallo instalando {repo} ({skill}) (continuo)"));
        }
    }
}

// --- helpers ----------------------------------------------------------------

fn has_npx() -> bool {
    which("npx")
}

fn run_npx(args: &[&str]) -> bool {
    run_ok("npx", args)
}

fn run_ok(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Is `bin` on PATH? (adds .exe/.cmd on Windows for local dev/testing).
fn which(bin: &str) -> bool {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let exts: &[&str] = if cfg!(windows) {
        &["", ".exe", ".cmd", ".bat"]
    } else {
        &[""]
    };
    std::env::split_paths(&path).any(|dir| {
        exts.iter().any(|ext| {
            let mut p = dir.join(bin);
            if !ext.is_empty() {
                p.set_extension(&ext[1..]);
            }
            p.is_file()
        })
    })
}

/// Recursively copy a directory tree.
fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir(&src, &dst)?;
        } else {
            std::fs::copy(&src, &dst)?;
        }
    }
    Ok(())
}
