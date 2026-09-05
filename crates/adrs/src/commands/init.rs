//! Initialize command.

use adrs_core::{
    CONFIG_FILE, ClaimsConfig, ClaimsMode, Config, ConfigMode, HIDDEN_CONFIG_FILE,
    LEGACY_CONFIG_FILE, Repository, decision::DecisionRepository, discover,
};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::path::PathBuf;

/// Resolve the root to initialize in and the ADR directory to use.
///
/// An explicit `directory` argument always wins. Otherwise the ADR
/// directory is resolved through the same config discovery every other
/// command uses (`adrs.toml`, then `.adrs.toml`, then `.adr-dir`, then
/// `ADR_DIRECTORY`, then the global config), falling back to `doc/adr`
/// only when nothing is configured.
fn resolve_init_target(root: &Path, directory: Option<PathBuf>) -> (PathBuf, PathBuf) {
    match directory {
        Some(dir) => (root.to_path_buf(), dir),
        None => match discover(root) {
            Ok(discovered) => {
                crate::warn_unknown_config_keys(&discovered);
                (discovered.root, discovered.config.adr_dir)
            }
            Err(_) => (root.to_path_buf(), Config::default().adr_dir),
        },
    }
}

pub fn init_authoritative(root: &Path, events_dir: PathBuf, render_dir: PathBuf) -> Result<()> {
    // Construction performs the same absolute/traversal/separation checks used
    // by every authoritative command before anything is written.
    DecisionRepository::new(root, &events_dir, &render_dir)?;

    let markers = [CONFIG_FILE, HIDDEN_CONFIG_FILE, LEGACY_CONFIG_FILE];
    if markers.iter().any(|name| root.join(name).exists()) {
        let existing = Config::load(root).context("failed to load existing ADR configuration")?;
        if existing.claims.mode == ClaimsMode::Authoritative
            && existing.claims.events_dir == events_dir
            && existing.claims.render_dir == render_dir
        {
            std::fs::create_dir_all(root.join(&events_dir))?;
            std::fs::create_dir_all(root.join(&render_dir))?;
            println!("{}", root.join(CONFIG_FILE).display());
            return Ok(());
        }
        bail!(
            "an ADR repository already exists with different settings; authoritative init refuses to overwrite it"
        );
    }

    let config = Config {
        mode: ConfigMode::NextGen,
        adr_dir: render_dir.clone(),
        claims: ClaimsConfig {
            mode: ClaimsMode::Authoritative,
            events_dir: events_dir.clone(),
            render_dir: render_dir.clone(),
        },
        ..Config::default()
    };

    std::fs::create_dir_all(root.join(&events_dir))?;
    std::fs::create_dir_all(root.join(&render_dir))?;
    config.save(root)?;
    println!("{}", root.join(CONFIG_FILE).display());
    println!("events: {}", root.join(events_dir).display());
    println!("generated Markdown: {}", root.join(render_dir).display());
    Ok(())
}

pub fn init(root: &Path, directory: Option<PathBuf>, ng: bool) -> Result<()> {
    let (init_root, adr_dir) = resolve_init_target(root, directory);

    let repo = Repository::init(&init_root, Some(adr_dir.clone()), ng).with_context(|| {
        format!(
            "Failed to initialize ADR repository in {}",
            adr_dir.display()
        )
    })?;

    // Check how many ADRs exist
    let adr_count = repo.list().map(|cladrs| cladrs.len()).unwrap_or(0);

    if adr_count > 1 {
        // More than just the initial ADR means we found existing ADRs
        println!(
            "{} ({} existing ADRs found)",
            repo.adr_path().display(),
            adr_count
        );
    } else {
        println!("{}", repo.adr_path().display());
    }

    Ok(())
}
