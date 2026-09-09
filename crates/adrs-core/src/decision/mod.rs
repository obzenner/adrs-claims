//! Authoritative, storage-independent whole-ADR decision events.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub const DECISION_SCHEMA_V1: &str = "adrs.decision/v1";
const STATEMENT_DOMAIN: &[u8] = b"adrs-statement-v1\0";
const EVENT_DOMAIN: &[u8] = b"adrs-decision-event-v1\0";
const FRONTIER_DOMAIN: &[u8] = b"adrs-decision-frontier-v1\0";

#[derive(Debug, thiserror::Error)]
pub enum DecisionError {
    #[error("decision event I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid decision event YAML: {0}")]
    Yaml(#[from] serde_yaml_neo::Error),
    #[error("invalid decision event: {0}")]
    Validation(String),
    #[error("decision fold failed: {0}")]
    Fold(String),
    #[error("generated Markdown drift: {0}")]
    RenderDrift(String),
}

pub type DecisionResult<T> = Result<T, DecisionError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "schema")]
pub enum DecisionEvent {
    #[serde(rename = "adrs.decision/v1")]
    V1(DecisionEventV1),
}

impl DecisionEvent {
    pub fn v1(&self) -> &DecisionEventV1 {
        match self {
            Self::V1(value) => value,
        }
    }

    pub fn id(&self) -> &str {
        &self.v1().id
    }

    pub fn sequence(&self) -> u64 {
        self.v1().sequence
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionEventV1 {
    pub id: String,
    pub sequence: u64,
    pub title: String,
    pub governance: GovernanceV1,
    pub context: ContextV1,
    pub drivers: Vec<DecisionDriverV1>,
    pub options: Vec<DecisionOptionV1>,
    pub outcome: DecisionOutcomeV1,
    pub consequences: ConsequencesV1,
    pub verification: VerificationPlanV1,
    pub relationships: RelationshipsV1,
    pub sources: Vec<DecisionSourceV1>,
    pub honest_scope: Vec<LimitationV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum GovernanceStatus {
    Proposed,
    Accepted,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GovernanceV1 {
    pub status: GovernanceStatus,
    pub proposed_at: String,
    #[serde(default)]
    pub decided_at: Option<String>,
    #[serde(default)]
    pub prior_frontier: Option<String>,
    pub actors: Vec<GovernanceActorV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GovernanceActorV1 {
    pub id: String,
    pub role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextV1 {
    pub problem: String,
    pub prior_state: Vec<ClaimReferenceV1>,
    pub constraints: Vec<NamedStatementV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaimReferenceV1 {
    pub claim_id: String,
    pub statement_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NamedStatementV1 {
    pub id: String,
    pub statement: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DriverPriority {
    Required,
    Preferred,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionDriverV1 {
    pub id: String,
    pub statement: String,
    pub priority: DriverPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum OptionDisposition {
    Selected,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum DriverAssessment {
    Satisfies,
    Conflicts,
    Neutral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionOptionV1 {
    pub id: String,
    pub title: String,
    pub description: String,
    pub disposition: OptionDisposition,
    pub reasons: Vec<OptionReasonV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OptionReasonV1 {
    pub driver: String,
    pub assessment: DriverAssessment,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionOutcomeV1 {
    pub claims: Vec<ClaimEffectV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ClaimEffectV1 {
    Declare { declaration: ClaimDeclarationV1 },
    Retract { retraction: ClaimRetractionV1 },
    Supersede { supersession: ClaimSupersessionV1 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaimDeclarationV1 {
    pub claim_id: String,
    pub statement: String,
    pub statement_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaimRetractionV1 {
    pub claim_id: String,
    pub statement_digest: String,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClaimSupersessionV1 {
    pub claim_id: String,
    pub statement_digest: String,
    pub replacement: ClaimDeclarationV1,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConsequencesV1 {
    pub positive: Vec<ConsequenceV1>,
    pub negative: Vec<ConsequenceV1>,
    pub risks: Vec<RiskV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConsequenceV1 {
    pub id: String,
    pub statement: String,
    pub caused_by: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RiskV1 {
    pub id: String,
    pub statement: String,
    pub mitigations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerificationPlanV1 {
    pub obligations: Vec<VerificationObligationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerificationObligationV1 {
    pub id: String,
    pub subject_claims: Vec<String>,
    pub method: VerificationMethodV1,
    pub expected: VerificationExpectedV1,
    pub interpretation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case", deny_unknown_fields)]
pub enum VerificationMethodV1 {
    Command { command: String },
    Observation { procedure: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerificationExpectedV1 {
    #[serde(default)]
    pub exit_status: Option<i32>,
    #[serde(default)]
    pub statement: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RelationshipsV1 {
    pub supersedes: Vec<DecisionRelationV1>,
    pub depends_on: Vec<DecisionRelationV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelationV1 {
    pub decision: String,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecisionSourceV1 {
    pub id: String,
    pub kind: String,
    pub locator: String,
    pub relevance: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LimitationV1 {
    pub id: String,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedDecisionEvent {
    pub event: DecisionEvent,
    pub event_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum ClaimDisposition {
    Active,
    Retracted {
        by: String,
        rationale: String,
    },
    Superseded {
        by: String,
        replacement_claim_id: String,
        replacement_statement_digest: String,
        rationale: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClaimRecord {
    pub claim_id: String,
    pub statement: String,
    pub statement_digest: String,
    pub declared_by: String,
    pub disposition: ClaimDisposition,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionState {
    pub frontier: String,
    pub events: Vec<ValidatedDecisionEvent>,
    pub claims: BTreeMap<String, ClaimRecord>,
    pub candidates: Vec<ValidatedDecisionEvent>,
}

impl Default for DecisionState {
    fn default() -> Self {
        Self {
            frontier: digest(FRONTIER_DOMAIN, &[]),
            events: Vec::new(),
            claims: BTreeMap::new(),
            candidates: Vec::new(),
        }
    }
}

pub fn decision_event_schema() -> serde_json::Value {
    serde_json::to_value(schemars::schema_for!(DecisionEvent))
        .expect("generated DecisionEvent schema must serialize")
}

pub fn statement_digest(statement: &str) -> String {
    digest(STATEMENT_DOMAIN, statement.as_bytes())
}

pub fn event_digest(event: &DecisionEvent) -> DecisionResult<String> {
    let canonical = serde_json::to_vec(event)
        .map_err(|error| DecisionError::Validation(format!("canonical serialization: {error}")))?;
    Ok(digest(EVENT_DOMAIN, &canonical))
}

fn digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
    format!("sha256:{:x}", hasher.finalize())
}

fn advance_frontier(prior: &str, event_digest: &str) -> String {
    let mut bytes = Vec::with_capacity(prior.len() + event_digest.len() + 16);
    bytes.extend_from_slice(&(prior.len() as u64).to_be_bytes());
    bytes.extend_from_slice(prior.as_bytes());
    bytes.extend_from_slice(&(event_digest.len() as u64).to_be_bytes());
    bytes.extend_from_slice(event_digest.as_bytes());
    digest(FRONTIER_DOMAIN, &bytes)
}

fn validate_date(name: &str, value: &str) -> DecisionResult<()> {
    time::Date::parse(value, &time::format_description::well_known::Iso8601::DATE)
        .map(|_| ())
        .map_err(|_| {
            DecisionError::Validation(format!(
                "{name} must be an ISO 8601 date in YYYY-MM-DD form"
            ))
        })
}

fn required(name: &str, value: &str) -> DecisionResult<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("todo") {
        return Err(DecisionError::Validation(format!(
            "{name} must be non-empty and non-placeholder"
        )));
    }
    Ok(())
}

fn validate_declaration(value: &ClaimDeclarationV1) -> DecisionResult<()> {
    required("claim_id", &value.claim_id)?;
    required("claim statement", &value.statement)?;
    let expected = statement_digest(&value.statement);
    if value.statement_digest != expected {
        return Err(DecisionError::Validation(format!(
            "claim {} statement digest mismatch: expected {expected}",
            value.claim_id
        )));
    }
    Ok(())
}

pub fn validate_event(event: DecisionEvent) -> DecisionResult<ValidatedDecisionEvent> {
    let value = event.v1();
    required("decision id", &value.id)?;
    required("title", &value.title)?;
    required("context.problem", &value.context.problem)?;
    required("governance.proposed_at", &value.governance.proposed_at)?;
    validate_date("governance.proposed_at", &value.governance.proposed_at)?;
    if value.sequence == 0 {
        return Err(DecisionError::Validation(
            "sequence must be non-zero".into(),
        ));
    }
    if value.governance.actors.is_empty() {
        return Err(DecisionError::Validation(
            "governance.actors must not be empty".into(),
        ));
    }
    for actor in &value.governance.actors {
        required("actor id", &actor.id)?;
        required("actor role", &actor.role)?;
    }
    match value.governance.status {
        GovernanceStatus::Proposed if value.governance.decided_at.is_some() => {
            return Err(DecisionError::Validation(
                "proposed decision must not have decided_at".into(),
            ));
        }
        GovernanceStatus::Accepted | GovernanceStatus::Rejected | GovernanceStatus::Withdrawn
            if value.governance.decided_at.is_none() =>
        {
            return Err(DecisionError::Validation(
                "decided decision requires decided_at".into(),
            ));
        }
        GovernanceStatus::Accepted | GovernanceStatus::Rejected | GovernanceStatus::Withdrawn => {
            validate_date(
                "governance.decided_at",
                value.governance.decided_at.as_deref().unwrap_or_default(),
            )?;
        }
        _ => {}
    }

    let driver_ids: BTreeSet<_> = value.drivers.iter().map(|item| item.id.as_str()).collect();
    if driver_ids.len() != value.drivers.len() {
        return Err(DecisionError::Validation("duplicate driver id".into()));
    }
    for driver in &value.drivers {
        required("driver id", &driver.id)?;
        required("driver statement", &driver.statement)?;
    }
    let option_ids: BTreeSet<_> = value.options.iter().map(|item| item.id.as_str()).collect();
    if option_ids.len() != value.options.len() {
        return Err(DecisionError::Validation("duplicate option id".into()));
    }
    let selected = value
        .options
        .iter()
        .filter(|item| item.disposition == OptionDisposition::Selected)
        .count();
    if value.governance.status == GovernanceStatus::Accepted && selected != 1 {
        return Err(DecisionError::Validation(
            "accepted decision requires exactly one selected option".into(),
        ));
    }
    for option in &value.options {
        required("option id", &option.id)?;
        required("option title", &option.title)?;
        required("option description", &option.description)?;
        if option.disposition == OptionDisposition::Rejected && option.reasons.is_empty() {
            return Err(DecisionError::Validation(format!(
                "rejected option {} requires a reason",
                option.id
            )));
        }
        for reason in &option.reasons {
            if !driver_ids.contains(reason.driver.as_str()) {
                return Err(DecisionError::Validation(format!(
                    "option {} references unknown driver {}",
                    option.id, reason.driver
                )));
            }
            required("option reason", &reason.explanation)?;
        }
    }
    if value.governance.status == GovernanceStatus::Accepted && value.outcome.claims.is_empty() {
        return Err(DecisionError::Validation(
            "accepted decision requires at least one claim effect".into(),
        ));
    }
    for effect in &value.outcome.claims {
        match effect {
            ClaimEffectV1::Declare { declaration } => validate_declaration(declaration)?,
            ClaimEffectV1::Retract { retraction } => {
                required("retraction claim_id", &retraction.claim_id)?;
                required("retraction statement_digest", &retraction.statement_digest)?;
                required("retraction rationale", &retraction.rationale)?;
            }
            ClaimEffectV1::Supersede { supersession } => {
                required("supersession claim_id", &supersession.claim_id)?;
                required(
                    "supersession statement_digest",
                    &supersession.statement_digest,
                )?;
                required("supersession rationale", &supersession.rationale)?;
                validate_declaration(&supersession.replacement)?;
            }
        }
    }

    validate_complete_references(value)?;
    let event_digest = event_digest(&event)?;
    Ok(ValidatedDecisionEvent {
        event,
        event_digest,
    })
}

fn validate_complete_references(value: &DecisionEventV1) -> DecisionResult<()> {
    for prior in &value.context.prior_state {
        required("prior_state claim_id", &prior.claim_id)?;
        required("prior_state statement_digest", &prior.statement_digest)?;
    }
    for constraint in &value.context.constraints {
        required("constraint id", &constraint.id)?;
        required("constraint statement", &constraint.statement)?;
    }
    let local_claims: BTreeSet<&str> = value
        .outcome
        .claims
        .iter()
        .filter_map(|effect| match effect {
            ClaimEffectV1::Declare { declaration } => Some(declaration.claim_id.as_str()),
            ClaimEffectV1::Supersede { supersession } => {
                Some(supersession.replacement.claim_id.as_str())
            }
            ClaimEffectV1::Retract { .. } => None,
        })
        .collect();
    let allowed_claim = |claim: &str| {
        local_claims.contains(claim)
            || value
                .context
                .prior_state
                .iter()
                .any(|prior| prior.claim_id == claim)
    };
    for consequence in value
        .consequences
        .positive
        .iter()
        .chain(&value.consequences.negative)
    {
        required("consequence id", &consequence.id)?;
        required("consequence statement", &consequence.statement)?;
        for claim in &consequence.caused_by {
            if !allowed_claim(claim) {
                return Err(DecisionError::Validation(format!(
                    "consequence {} references claim {} absent from this event or prior_state",
                    consequence.id, claim
                )));
            }
        }
    }
    let obligation_ids: BTreeSet<&str> = value
        .verification
        .obligations
        .iter()
        .map(|item| item.id.as_str())
        .collect();
    if obligation_ids.len() != value.verification.obligations.len() {
        return Err(DecisionError::Validation(
            "duplicate verification obligation id".into(),
        ));
    }
    for obligation in &value.verification.obligations {
        required("verification id", &obligation.id)?;
        required("verification interpretation", &obligation.interpretation)?;
        match &obligation.method {
            VerificationMethodV1::Command { command } => required("verification command", command)?,
            VerificationMethodV1::Observation { procedure } => {
                required("verification procedure", procedure)?
            }
        }
        if obligation.expected.exit_status.is_none() && obligation.expected.statement.is_none() {
            return Err(DecisionError::Validation(format!(
                "verification {} requires an expected exit_status or statement",
                obligation.id
            )));
        }
        if let Some(statement) = &obligation.expected.statement {
            required("verification expected statement", statement)?;
        }
        for claim in &obligation.subject_claims {
            if !allowed_claim(claim) {
                return Err(DecisionError::Validation(format!(
                    "verification {} references claim {} absent from this event or prior_state",
                    obligation.id, claim
                )));
            }
        }
    }
    for risk in &value.consequences.risks {
        required("risk id", &risk.id)?;
        required("risk statement", &risk.statement)?;
        for mitigation in &risk.mitigations {
            if !obligation_ids.contains(mitigation.as_str()) {
                return Err(DecisionError::Validation(format!(
                    "risk {} references unknown verification {}",
                    risk.id, mitigation
                )));
            }
        }
    }
    for relation in value
        .relationships
        .supersedes
        .iter()
        .chain(&value.relationships.depends_on)
    {
        required("related decision", &relation.decision)?;
        for claim in &relation.claims {
            if !allowed_claim(claim) {
                return Err(DecisionError::Validation(format!(
                    "relationship to {} references claim {} absent from this event or prior_state",
                    relation.decision, claim
                )));
            }
        }
    }
    for source in &value.sources {
        required("source id", &source.id)?;
        required("source kind", &source.kind)?;
        required("source locator", &source.locator)?;
        required("source relevance", &source.relevance)?;
    }
    for limitation in &value.honest_scope {
        required("honest_scope id", &limitation.id)?;
        required("honest_scope statement", &limitation.statement)?;
    }
    Ok(())
}

pub fn fold(events: Vec<DecisionEvent>) -> DecisionResult<DecisionState> {
    let mut validated = events
        .into_iter()
        .map(validate_event)
        .collect::<DecisionResult<Vec<_>>>()?;
    validated.sort_by_key(|item| item.event.sequence());

    let mut state = DecisionState::default();
    let mut ids = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    for item in validated {
        let event = item.event.v1();
        if !ids.insert(event.id.clone()) {
            return Err(DecisionError::Fold(format!(
                "duplicate decision id {}",
                event.id
            )));
        }
        if !sequences.insert(event.sequence) {
            return Err(DecisionError::Fold(format!(
                "duplicate decision sequence {}",
                event.sequence
            )));
        }
        for prior in &event.context.prior_state {
            let claim = state.claims.get(&prior.claim_id).ok_or_else(|| {
                DecisionError::Fold(format!(
                    "decision {} prior_state references unknown claim {}",
                    event.id, prior.claim_id
                ))
            })?;
            if claim.statement_digest != prior.statement_digest {
                return Err(DecisionError::Fold(format!(
                    "decision {} prior_state has stale digest for claim {}",
                    event.id, prior.claim_id
                )));
            }
        }
        for relation in event
            .relationships
            .supersedes
            .iter()
            .chain(&event.relationships.depends_on)
        {
            if relation.decision == event.id || !ids.contains(&relation.decision) {
                return Err(DecisionError::Fold(format!(
                    "decision {} references unknown or non-prior decision {}",
                    event.id, relation.decision
                )));
            }
        }
        if let Some(expected) = &event.governance.prior_frontier
            && expected != &state.frontier
        {
            return Err(DecisionError::Fold(format!(
                "decision {} prior frontier mismatch: expected {}, found {}",
                event.id, expected, state.frontier
            )));
        }
        if event.governance.status == GovernanceStatus::Accepted {
            apply_claims(&mut state.claims, event)?;
        } else if event.governance.status == GovernanceStatus::Proposed {
            // Candidate effects must be applicable to the accepted state even though
            // they do not mutate it. Otherwise an unknown or stale target could sit
            // in the roadmap until acceptance and make the candidate projection a lie.
            let mut candidate_claims = state.claims.clone();
            apply_claims(&mut candidate_claims, event)?;
            state.candidates.push(item.clone());
        }
        state.frontier = advance_frontier(&state.frontier, &item.event_digest);
        state.events.push(item);
    }
    Ok(state)
}

fn active_target<'a>(
    claims: &'a mut BTreeMap<String, ClaimRecord>,
    claim_id: &str,
    expected_digest: &str,
) -> DecisionResult<&'a mut ClaimRecord> {
    let target = claims
        .get_mut(claim_id)
        .ok_or_else(|| DecisionError::Fold(format!("unknown claim target {claim_id}")))?;
    if target.statement_digest != expected_digest {
        return Err(DecisionError::Fold(format!(
            "stale statement digest for claim {claim_id}"
        )));
    }
    if target.disposition != ClaimDisposition::Active {
        return Err(DecisionError::Fold(format!(
            "claim {claim_id} is already inactive"
        )));
    }
    Ok(target)
}

fn declare(
    claims: &mut BTreeMap<String, ClaimRecord>,
    declaration: &ClaimDeclarationV1,
    decision_id: &str,
) -> DecisionResult<()> {
    if let Some(existing) = claims.get(&declaration.claim_id) {
        let detail = if existing.statement_digest == declaration.statement_digest {
            "already declared"
        } else {
            "already identifies a different proposition"
        };
        return Err(DecisionError::Fold(format!(
            "claim {} {detail}",
            declaration.claim_id
        )));
    }
    claims.insert(
        declaration.claim_id.clone(),
        ClaimRecord {
            claim_id: declaration.claim_id.clone(),
            statement: declaration.statement.clone(),
            statement_digest: declaration.statement_digest.clone(),
            declared_by: decision_id.to_string(),
            disposition: ClaimDisposition::Active,
        },
    );
    Ok(())
}

fn apply_claims(
    claims: &mut BTreeMap<String, ClaimRecord>,
    event: &DecisionEventV1,
) -> DecisionResult<()> {
    for effect in &event.outcome.claims {
        match effect {
            ClaimEffectV1::Declare { declaration } => {
                declare(claims, declaration, &event.id)?;
            }
            ClaimEffectV1::Retract { retraction } => {
                let target =
                    active_target(claims, &retraction.claim_id, &retraction.statement_digest)?;
                target.disposition = ClaimDisposition::Retracted {
                    by: event.id.clone(),
                    rationale: retraction.rationale.clone(),
                };
            }
            ClaimEffectV1::Supersede { supersession } => {
                active_target(
                    claims,
                    &supersession.claim_id,
                    &supersession.statement_digest,
                )?;
                declare(claims, &supersession.replacement, &event.id)?;
                claims
                    .get_mut(&supersession.claim_id)
                    .expect("target checked")
                    .disposition = ClaimDisposition::Superseded {
                    by: event.id.clone(),
                    replacement_claim_id: supersession.replacement.claim_id.clone(),
                    replacement_statement_digest: supersession.replacement.statement_digest.clone(),
                    rationale: supersession.rationale.clone(),
                };
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct DecisionRepository {
    root: PathBuf,
    events_dir: PathBuf,
    render_dir: PathBuf,
}

impl DecisionRepository {
    pub fn new(root: &Path, events_dir: &Path, render_dir: &Path) -> DecisionResult<Self> {
        validate_relative_dir("events_dir", events_dir)?;
        validate_relative_dir("render_dir", render_dir)?;
        if events_dir == render_dir {
            return Err(DecisionError::Validation(
                "events_dir and render_dir must differ".into(),
            ));
        }
        Ok(Self {
            root: root.to_path_buf(),
            events_dir: events_dir.to_path_buf(),
            render_dir: render_dir.to_path_buf(),
        })
    }

    pub fn load_events(&self) -> DecisionResult<Vec<DecisionEvent>> {
        let directory = self.root.join(&self.events_dir);
        let mut paths = Vec::new();
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            let is_event = path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".adr.yaml"));
            if !is_event {
                return Err(DecisionError::Validation(format!(
                    "authoritative event directory contains unsupported entry {}",
                    path.display()
                )));
            }
            paths.push(path);
        }
        if paths.is_empty() {
            return Err(DecisionError::Validation(format!(
                "authoritative event directory {} contains no *.adr.yaml events",
                directory.display()
            )));
        }
        paths.sort();
        paths
            .into_iter()
            .map(|path| {
                let source = fs::read_to_string(&path)?;
                serde_yaml_neo::from_str(&source).map_err(DecisionError::from)
            })
            .collect()
    }

    pub fn validate(&self) -> DecisionResult<DecisionState> {
        fold(self.load_events()?)
    }

    pub fn render(&self, check: bool) -> DecisionResult<Vec<PathBuf>> {
        let state = self.validate()?;
        let render_dir = self.root.join(&self.render_dir);
        if !check {
            fs::create_dir_all(&render_dir)?;
        }

        let expected_paths: BTreeSet<PathBuf> = state
            .events
            .iter()
            .map(|item| {
                let value = item.event.v1();
                render_dir.join(format!("{:04}-{}.md", value.sequence, slug(&value.title)))
            })
            .collect();
        if render_dir.exists() {
            for entry in fs::read_dir(&render_dir)? {
                let path = entry?.path();
                if !expected_paths.contains(&path) {
                    return Err(DecisionError::RenderDrift(format!(
                        "unexpected entry {} is not generated by the authoritative event set",
                        path.display()
                    )));
                }
            }
        }

        let mut rendered = Vec::new();
        for item in &state.events {
            let value = item.event.v1();
            let filename = format!("{:04}-{}.md", value.sequence, slug(&value.title));
            let path = render_dir.join(filename);
            let expected = render_markdown(item, &state.frontier);
            if check {
                let actual = fs::read_to_string(&path).map_err(|_| {
                    DecisionError::RenderDrift(format!("missing {}", path.display()))
                })?;
                if actual != expected {
                    return Err(DecisionError::RenderDrift(format!(
                        "{} differs from authoritative event",
                        path.display()
                    )));
                }
            } else {
                fs::write(&path, expected)?;
            }
            rendered.push(path);
        }
        Ok(rendered)
    }
}

fn validate_relative_dir(name: &str, path: &Path) -> DecisionResult<()> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(DecisionError::Validation(format!(
            "{name} must be a non-empty repository-relative path without parent traversal"
        )));
    }
    Ok(())
}

fn slug(title: &str) -> String {
    let mut result = String::new();
    let mut dash = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            dash = false;
        } else if !dash && !result.is_empty() {
            result.push('-');
            dash = true;
        }
    }
    result.trim_end_matches('-').to_string()
}

pub fn render_markdown(event: &ValidatedDecisionEvent, frontier: &str) -> String {
    let value = event.event.v1();
    let mut out = String::new();
    let _ = writeln!(out, "<!-- Generated by cladrs from {DECISION_SCHEMA_V1}.");
    let _ = writeln!(out, "Event digest: {}", event.event_digest);
    let _ = writeln!(out, "Frontier: {frontier}");
    let _ = writeln!(out, "Do not edit this file directly. -->\n");
    let _ = writeln!(out, "# {} — {}\n", value.id, value.title);
    let _ = writeln!(out, "## Governance\n");
    let _ = writeln!(out, "- Status: `{:?}`", value.governance.status);
    let _ = writeln!(out, "- Proposed: {}", value.governance.proposed_at);
    if let Some(date) = &value.governance.decided_at {
        let _ = writeln!(out, "- Decided: {date}");
    }
    if let Some(prior) = &value.governance.prior_frontier {
        let _ = writeln!(out, "- Prior frontier: `{prior}`");
    }
    for actor in &value.governance.actors {
        let _ = writeln!(out, "- Actor: **{}** ({})", actor.id, actor.role);
    }
    let _ = writeln!(out, "\n## Context\n\n{}\n", value.context.problem);
    let _ = writeln!(out, "## Prior State\n");
    for prior in &value.context.prior_state {
        let _ = writeln!(
            out,
            "- **`{}`** at `{}`",
            prior.claim_id, prior.statement_digest
        );
    }
    let _ = writeln!(out);
    render_named(&mut out, "Constraints", &value.context.constraints);
    let _ = writeln!(out, "## Decision Drivers\n");
    for driver in &value.drivers {
        let _ = writeln!(
            out,
            "- **{}** ({:?}): {}",
            driver.id, driver.priority, driver.statement
        );
    }
    let _ = writeln!(out, "\n## Considered Options\n");
    for option in &value.options {
        let _ = writeln!(
            out,
            "### {} — {:?}\n\n{}\n",
            option.title, option.disposition, option.description
        );
        for reason in &option.reasons {
            let _ = writeln!(
                out,
                "- `{}` / `{:?}`: {}",
                reason.driver, reason.assessment, reason.explanation
            );
        }
    }
    let _ = writeln!(out, "\n## Decision Outcome\n");
    for effect in &value.outcome.claims {
        match effect {
            ClaimEffectV1::Declare { declaration } => {
                let _ = writeln!(
                    out,
                    "- Declares **`{}`** (`{}`): {}",
                    declaration.claim_id, declaration.statement_digest, declaration.statement
                );
            }
            ClaimEffectV1::Retract { retraction } => {
                let _ = writeln!(
                    out,
                    "- Retracts **`{}`** at `{}`: {}",
                    retraction.claim_id, retraction.statement_digest, retraction.rationale
                );
            }
            ClaimEffectV1::Supersede { supersession } => {
                let _ = writeln!(
                    out,
                    "- Supersedes **`{}`** at `{}` with **`{}`** at `{}`: {}",
                    supersession.claim_id,
                    supersession.statement_digest,
                    supersession.replacement.claim_id,
                    supersession.replacement.statement_digest,
                    supersession.rationale
                );
                let _ = writeln!(out, "  - {}", supersession.replacement.statement);
            }
        }
    }
    render_consequences(
        &mut out,
        "Positive Consequences",
        &value.consequences.positive,
    );
    render_consequences(
        &mut out,
        "Negative Consequences",
        &value.consequences.negative,
    );
    let _ = writeln!(out, "## Risks\n");
    for risk in &value.consequences.risks {
        let _ = writeln!(out, "- **{}**: {}", risk.id, risk.statement);
        if !risk.mitigations.is_empty() {
            let _ = writeln!(out, "  - Mitigations: {}", risk.mitigations.join(", "));
        }
    }
    let _ = writeln!(out, "\n## Verification\n");
    for obligation in &value.verification.obligations {
        let method = match &obligation.method {
            VerificationMethodV1::Command { command } => format!("command `{command}`"),
            VerificationMethodV1::Observation { procedure } => format!("observation: {procedure}"),
        };
        let expected = match (
            &obligation.expected.exit_status,
            &obligation.expected.statement,
        ) {
            (Some(status), Some(statement)) => format!("exit {status}; {statement}"),
            (Some(status), None) => format!("exit {status}"),
            (None, Some(statement)) => statement.clone(),
            (None, None) => "no machine expectation".to_string(),
        };
        let _ = writeln!(
            out,
            "- **{}** ({method}; expects {expected}; claims: {}): {}",
            obligation.id,
            obligation.subject_claims.join(", "),
            obligation.interpretation
        );
    }
    let _ = writeln!(out, "\n## Relationships\n");
    for relation in &value.relationships.supersedes {
        let _ = writeln!(
            out,
            "- Supersedes `{}` ({})",
            relation.decision,
            relation.claims.join(", ")
        );
    }
    for relation in &value.relationships.depends_on {
        let _ = writeln!(
            out,
            "- Depends on `{}` ({})",
            relation.decision,
            relation.claims.join(", ")
        );
    }
    let _ = writeln!(out, "\n## Sources\n");
    for source in &value.sources {
        let _ = writeln!(
            out,
            "- **{}** (`{}`: `{}`): {}",
            source.id, source.kind, source.locator, source.relevance
        );
    }
    let _ = writeln!(out, "\n## Honest Scope\n");
    for limitation in &value.honest_scope {
        let _ = writeln!(out, "- **{}**: {}", limitation.id, limitation.statement);
    }
    out
}

fn render_named(out: &mut String, title: &str, values: &[NamedStatementV1]) {
    let _ = writeln!(out, "## {title}\n");
    for value in values {
        let _ = writeln!(out, "- **{}**: {}", value.id, value.statement);
    }
    let _ = writeln!(out);
}

fn render_consequences(out: &mut String, title: &str, values: &[ConsequenceV1]) {
    let _ = writeln!(out, "\n## {title}\n");
    for value in values {
        let causes = if value.caused_by.is_empty() {
            "none".to_string()
        } else {
            value.caused_by.join(", ")
        };
        let _ = writeln!(
            out,
            "- **{}** (caused by: {}): {}",
            value.id, causes, value.statement
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declaration(id: &str, statement: &str) -> ClaimDeclarationV1 {
        ClaimDeclarationV1 {
            claim_id: id.into(),
            statement: statement.into(),
            statement_digest: statement_digest(statement),
        }
    }

    fn event(id: &str, sequence: u64, status: GovernanceStatus) -> DecisionEvent {
        DecisionEvent::V1(DecisionEventV1 {
            id: id.into(),
            sequence,
            title: format!("Decision {sequence}"),
            governance: GovernanceV1 {
                status,
                proposed_at: "2026-01-01".into(),
                decided_at: (status != GovernanceStatus::Proposed).then(|| "2026-01-02".into()),
                prior_frontier: None,
                actors: vec![GovernanceActorV1 {
                    id: "team".into(),
                    role: "owner".into(),
                }],
            },
            context: ContextV1 {
                problem: "A complete problem".into(),
                prior_state: vec![],
                constraints: vec![NamedStatementV1 {
                    id: "bounded".into(),
                    statement: "Remain bounded".into(),
                }],
            },
            drivers: vec![DecisionDriverV1 {
                id: "driver".into(),
                statement: "One authority".into(),
                priority: DriverPriority::Required,
            }],
            options: vec![DecisionOptionV1 {
                id: "selected".into(),
                title: "Selected".into(),
                description: "Selected approach".into(),
                disposition: OptionDisposition::Selected,
                reasons: vec![OptionReasonV1 {
                    driver: "driver".into(),
                    assessment: DriverAssessment::Satisfies,
                    explanation: "It fits".into(),
                }],
            }],
            outcome: DecisionOutcomeV1 {
                claims: vec![ClaimEffectV1::Declare {
                    declaration: declaration(
                        &format!("claim-{sequence}"),
                        &format!("Statement {sequence}"),
                    ),
                }],
            },
            consequences: ConsequencesV1 {
                positive: vec![ConsequenceV1 {
                    id: "positive".into(),
                    statement: "It helps".into(),
                    caused_by: vec![format!("claim-{sequence}")],
                }],
                negative: vec![],
                risks: vec![],
            },
            verification: VerificationPlanV1 {
                obligations: vec![VerificationObligationV1 {
                    id: "verify".into(),
                    subject_claims: vec![format!("claim-{sequence}")],
                    method: VerificationMethodV1::Command {
                        command: "true".into(),
                    },
                    expected: VerificationExpectedV1 {
                        exit_status: Some(0),
                        statement: None,
                    },
                    interpretation: "The check passes".into(),
                }],
            },
            relationships: RelationshipsV1 {
                supersedes: vec![],
                depends_on: vec![],
            },
            sources: vec![DecisionSourceV1 {
                id: "source".into(),
                kind: "repository-path".into(),
                locator: "README.md".into(),
                relevance: "Background".into(),
            }],
            honest_scope: vec![LimitationV1 {
                id: "truth".into(),
                statement: "Decision does not prove implementation".into(),
            }],
        })
    }

    #[test]
    fn generated_schema_is_closed_and_versioned() {
        let schema = decision_event_schema();
        let text = serde_json::to_string(&schema).unwrap();
        assert!(text.contains("adrs.decision/v1"));
        assert!(text.contains("additionalProperties"));
        assert!(text.contains("honest_scope"));
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let yaml = "schema: adrs.decision/v1\nid: ADR-1\nsequence: 1\ntitle: x\nunknown: true\ngovernance: {}\ncontext: {}\ndrivers: []\noptions: []\noutcome: {}\nconsequences: {}\nverification: {}\nrelationships: {}\nsources: []\nhonest_scope: []\n";
        assert!(serde_yaml_neo::from_str::<DecisionEvent>(yaml).is_err());
    }

    #[test]
    fn digest_mismatch_is_rejected() {
        let mut decision = event("ADR-1", 1, GovernanceStatus::Accepted);
        let DecisionEvent::V1(value) = &mut decision;
        let ClaimEffectV1::Declare { declaration } = &mut value.outcome.claims[0] else {
            unreachable!()
        };
        declaration.statement_digest = "sha256:wrong".into();
        assert!(validate_event(decision).is_err());
    }

    #[test]
    fn every_top_level_section_affects_event_digest() {
        let base = event("ADR-1", 1, GovernanceStatus::Accepted);
        let base_digest = event_digest(&base).unwrap();
        for index in 0..11 {
            let mut changed = base.clone();
            let DecisionEvent::V1(value) = &mut changed;
            match index {
                0 => value.title.push('!'),
                1 => value.governance.actors[0].role.push('!'),
                2 => value.context.problem.push('!'),
                3 => value.drivers[0].statement.push('!'),
                4 => value.options[0].description.push('!'),
                5 => value.outcome.claims.push(ClaimEffectV1::Declare {
                    declaration: declaration("other", "Other"),
                }),
                6 => value.consequences.positive[0].statement.push('!'),
                7 => value.verification.obligations[0].interpretation.push('!'),
                8 => value.relationships.depends_on.push(DecisionRelationV1 {
                    decision: "ADR-0".into(),
                    claims: vec![],
                }),
                9 => value.sources[0].relevance.push('!'),
                10 => value.honest_scope[0].statement.push('!'),
                _ => unreachable!(),
            }
            assert_ne!(
                base_digest,
                event_digest(&changed).unwrap(),
                "section {index}"
            );
        }
    }

    #[test]
    fn proposed_does_not_mutate_current_claims() {
        let state = fold(vec![event("ADR-1", 1, GovernanceStatus::Proposed)]).unwrap();
        assert!(state.claims.is_empty());
        assert_eq!(state.candidates.len(), 1);
    }

    #[test]
    fn retract_and_supersede_are_exact_and_deterministic() {
        let first = event("ADR-1", 1, GovernanceStatus::Accepted);
        let old_digest = statement_digest("Statement 1");
        let mut second = event("ADR-2", 2, GovernanceStatus::Accepted);
        let DecisionEvent::V1(value) = &mut second;
        value.outcome.claims = vec![ClaimEffectV1::Supersede {
            supersession: ClaimSupersessionV1 {
                claim_id: "claim-1".into(),
                statement_digest: old_digest,
                replacement: declaration("claim-2", "Replacement"),
                rationale: "Better".into(),
            },
        }];
        value.verification.obligations[0].subject_claims = vec!["claim-2".into()];
        value.consequences.positive[0].caused_by = vec!["claim-2".into()];
        let a = fold(vec![second.clone(), first.clone()]).unwrap();
        let b = fold(vec![first, second]).unwrap();
        assert_eq!(a.frontier, b.frontier);
        assert_eq!(a.claims, b.claims);
        assert!(matches!(
            a.claims["claim-1"].disposition,
            ClaimDisposition::Superseded { .. }
        ));
        assert_eq!(a.claims["claim-2"].disposition, ClaimDisposition::Active);
    }

    #[test]
    fn stale_target_digest_fails() {
        let first = event("ADR-1", 1, GovernanceStatus::Accepted);
        let mut second = event("ADR-2", 2, GovernanceStatus::Accepted);
        let DecisionEvent::V1(value) = &mut second;
        value.outcome.claims = vec![ClaimEffectV1::Retract {
            retraction: ClaimRetractionV1 {
                claim_id: "claim-1".into(),
                statement_digest: "sha256:stale".into(),
                rationale: "No longer applies".into(),
            },
        }];
        value.verification.obligations.clear();
        assert!(fold(vec![first, second]).is_err());
    }

    #[test]
    fn proposed_effects_must_apply_to_the_accepted_state() {
        let first = event("ADR-1", 1, GovernanceStatus::Accepted);
        let mut candidate = event("ADR-2", 2, GovernanceStatus::Proposed);
        let DecisionEvent::V1(value) = &mut candidate;
        value.outcome.claims = vec![ClaimEffectV1::Retract {
            retraction: ClaimRetractionV1 {
                claim_id: "claim-1".into(),
                statement_digest: "sha256:stale".into(),
                rationale: "Candidate replacement".into(),
            },
        }];
        value.consequences.positive.clear();
        value.verification.obligations.clear();
        assert!(fold(vec![first, candidate]).is_err());
    }

    #[test]
    fn authoritative_directory_fails_closed_on_empty_or_unsupported_content() {
        let root = tempfile::tempdir().unwrap();
        let events = root.path().join("events");
        fs::create_dir(&events).unwrap();
        let repository =
            DecisionRepository::new(root.path(), Path::new("events"), Path::new("rendered"))
                .unwrap();
        assert!(repository.load_events().is_err());

        fs::write(events.join("notes.md"), "opaque prose").unwrap();
        assert!(repository.load_events().is_err());
    }
}
