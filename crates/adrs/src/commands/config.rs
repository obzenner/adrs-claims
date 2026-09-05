//! Config command.

use adrs_core::{ConfigSource, DiscoveredConfig};
use anyhow::Result;
use std::path::Path;

/// Show configuration with discovery information.
pub fn config_with_discovery(start_dir: &Path, discovered: Option<DiscoveredConfig>) -> Result<()> {
    match discovered {
        Some(disc) => {
            println!("Project root: {}", disc.root.display());
            println!(
                "Config source: {}",
                match &disc.source {
                    ConfigSource::Project(path) => format!("{}", path.display()),
                    ConfigSource::Global(path) => format!("{} (global)", path.display()),
                    ConfigSource::Environment => "environment variable".to_string(),
                    ConfigSource::Default => "defaults".to_string(),
                }
            );
            println!("ADR directory: {}", disc.config.adr_dir.display());
            println!(
                "Full path: {}",
                disc.root.join(&disc.config.adr_dir).display()
            );
            println!("Mode: {:?}", disc.config.mode);
            println!("Claims mode: {:?}", disc.config.claims.mode);
            if disc.config.claims.mode == adrs_core::ClaimsMode::Authoritative {
                println!(
                    "Decision events: {}",
                    disc.root.join(&disc.config.claims.events_dir).display()
                );
                println!(
                    "Generated Markdown: {}",
                    disc.root.join(&disc.config.claims.render_dir).display()
                );
            }
            if let Some(ref default_status) = disc.config.default_status {
                println!("Default ADR status: {}", default_status);
            }
            if disc.config.no_edit {
                println!("No edit: true");
            }
            if let Some(ref format) = disc.config.templates.format {
                println!("Template format: {}", format);
            }
            if let Some(ref variant) = disc.config.templates.variant {
                println!("Template variant: {}", variant);
            }
            if let Some(ref custom) = disc.config.templates.custom {
                println!("Custom template: {}", custom.display());
            }
        }
        None => {
            println!("No ADR repository found.");
            println!("Search started from: {}", start_dir.display());
            println!();
            println!("Run 'cladrs init' to create a new repository.");
        }
    }

    Ok(())
}
