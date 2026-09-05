//! Authoritative whole-ADR event commands.

use adrs_core::{
    ClaimsMode, Config,
    decision::{DecisionRepository, GovernanceStatus, decision_event_schema},
};
use anyhow::{Result, bail};
use std::path::Path;

fn repository(root: &Path, config: &Config) -> Result<DecisionRepository> {
    if config.claims.mode != ClaimsMode::Authoritative {
        bail!(
            "authoritative decision events are not enabled; add [claims] mode = \"authoritative\" to adrs.toml"
        );
    }
    Ok(DecisionRepository::new(
        root,
        &config.claims.events_dir,
        &config.claims.render_dir,
    )?)
}

pub fn schema(selector: &str) -> Result<()> {
    if selector != "decision-event/v1" && selector != "adrs.decision/v1" {
        bail!("unknown schema selector {selector:?}; supported selector: decision-event/v1");
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&decision_event_schema())?
    );
    Ok(())
}

pub struct DecisionListOptions<'a> {
    pub status: Option<&'a str>,
    pub since: Option<&'a str>,
    pub until: Option<&'a str>,
    pub decider: Option<&'a str>,
    pub tag: Option<&'a str>,
    pub long: bool,
}

pub fn list(root: &Path, config: &Config, options: DecisionListOptions<'_>) -> Result<()> {
    let DecisionListOptions {
        status,
        since,
        until,
        decider,
        tag,
        long,
    } = options;
    if tag.is_some() {
        bail!("authoritative decision-event/v1 does not define tags");
    }
    let status = status.map(parse_status).transpose()?;
    validate_filter_date("since", since)?;
    validate_filter_date("until", until)?;
    let decider = decider.map(str::to_lowercase);
    let state = repository(root, config)?.validate()?;
    for item in &state.events {
        let event = item.event.v1();
        if status.is_some_and(|expected| event.governance.status != expected)
            || since.is_some_and(|value| event.governance.proposed_at.as_str() < value)
            || until.is_some_and(|value| event.governance.proposed_at.as_str() > value)
            || decider.as_ref().is_some_and(|name| {
                !event
                    .governance
                    .actors
                    .iter()
                    .any(|actor| actor.id.to_lowercase().contains(name))
            })
        {
            continue;
        }
        if long {
            println!(
                "{:4}  {:10}  {}  {}  {}",
                event.sequence,
                status_name(event.governance.status),
                event.governance.proposed_at,
                event.id,
                event.title
            );
        } else {
            println!("{}", event.id);
        }
    }
    Ok(())
}

pub fn doctor(root: &Path, config: &Config) -> Result<()> {
    let repository = repository(root, config)?;
    let state = repository.validate()?;
    repository.render(true)?;
    println!(
        "authoritative decision history is healthy: {} events, {} active claims, {} candidates, frontier {}",
        state.events.len(),
        state
            .claims
            .values()
            .filter(|claim| matches!(
                claim.disposition,
                adrs_core::decision::ClaimDisposition::Active
            ))
            .count(),
        state.candidates.len(),
        state.frontier
    );
    Ok(())
}

fn validate_filter_date(name: &str, value: Option<&str>) -> Result<()> {
    if let Some(value) = value {
        time::Date::parse(value, &time::format_description::well_known::Iso8601::DATE)
            .map_err(|_| anyhow::anyhow!("invalid {name} date {value:?}; expected YYYY-MM-DD"))?;
    }
    Ok(())
}

fn parse_status(value: &str) -> Result<GovernanceStatus> {
    match value.to_ascii_lowercase().as_str() {
        "proposed" => Ok(GovernanceStatus::Proposed),
        "accepted" => Ok(GovernanceStatus::Accepted),
        "rejected" => Ok(GovernanceStatus::Rejected),
        "withdrawn" => Ok(GovernanceStatus::Withdrawn),
        _ => bail!("invalid authoritative governance status {value:?}"),
    }
}

fn status_name(status: GovernanceStatus) -> &'static str {
    match status {
        GovernanceStatus::Proposed => "proposed",
        GovernanceStatus::Accepted => "accepted",
        GovernanceStatus::Rejected => "rejected",
        GovernanceStatus::Withdrawn => "withdrawn",
    }
}

pub fn validate(root: &Path, config: &Config) -> Result<()> {
    let state = repository(root, config)?.validate()?;
    println!(
        "validated {} decision events at {}",
        state.events.len(),
        state.frontier
    );
    Ok(())
}

pub fn fold_json(root: &Path, config: &Config) -> Result<()> {
    let state = repository(root, config)?.validate()?;
    println!("{}", serde_json::to_string_pretty(&state)?);
    Ok(())
}

pub fn render(root: &Path, config: &Config, check: bool) -> Result<()> {
    let paths = repository(root, config)?.render(check)?;
    if check {
        println!("checked {} generated decision documents", paths.len());
    } else {
        for path in paths {
            println!("{}", path.display());
        }
    }
    Ok(())
}
