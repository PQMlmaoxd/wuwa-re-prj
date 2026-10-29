//! Bounded campaign metadata, append-only batch index loading, and copy preflight.
//!
//! This module deliberately does not create or mutate an index on import. The
//! public loader only consumes a validated v5 base and an already-present
//! activation index. Copy and activation helpers are exercised by tests until a
//! later phase is explicitly authorized.

use super::{
    ActiveReceipt, CurationStatus, CurrentMethod, EvidenceReference, EvidenceStatus, ExportStatus,
    ImportedFileView, Manifest, MethodIdentity, Methods, Record, RecordValidation, SourceArtifact,
    SourceStatus, Supersession, ValidationError, inspect_relative_inner, parse_strict_value,
    read_relative, sha256_hex, valid_lower_hex, validate_method_token, validate_relative_path,
    validate_type_token,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub const CATALOG_REL: &str = "metadata/curation/bulk-curate01.json";
pub const INDEX_REL: &str = "metadata/curation-batches.jsonl";
pub const CAMPAIGN_ID: &str = "bulk-curate01";
pub const CATALOG_SCHEMA: &str = "wuwa-reconstruction-bulk-curate-catalog-v1";
pub const INDEX_SCHEMA: &str = "wuwa-reconstruction-curation-batch-index-v1";
pub const ACTIVATION_SCHEMA: &str = "wuwa-reconstruction-curation-batch-activation-v1";
pub const BASELINE_V5_RECEIPT_SHA256: &str =
    "7c990cc73ea4050f7b20b00b7c8de2aef3f51bd179c516f12e0102b4bc5d11fd";
pub const FROZEN_SNAPSHOT_ID: &str =
    "d805365e60cf1b1c156dceeb1b496779de082744e6d0c82dd126b1b5f452fcf6";
pub const FROZEN_REGISTRY_RECEIPT_SHA256: &str =
    "a44f9d23a143973f5d452a16c2240deeddd75fcd8b9193c7b47fa7d9af90ae7c";
pub const CATALOG_SHA256: &str = "405f27fc0dffddb5ce6556233fb46107985ca2109f41341cdb4e017de48dbb99";
pub const AUTHORITY_PREFIX: &str = "src/managed/cs-e3fc67839851/";
pub const GAMEPLAY_COHORT: &str = "gameplay66";
pub const PERFORMANCE_COHORT: &str = "performance25";
pub const EXPECTED_RECORDS: usize = 91;
pub const EXPECTED_DESCRIPTOR_METHODS: usize = 429;
pub const EXPECTED_TOTAL_METHODS: usize = 431;
pub const EXPECTED_GAMEPLAY_FILES: usize = 66;
pub const EXPECTED_GAMEPLAY_METHODS: usize = 310;
pub const EXPECTED_GAMEPLAY_ORDINARY: usize = 292;
pub const EXPECTED_GAMEPLAY_ASSOCIATED: usize = 18;
pub const EXPECTED_PERFORMANCE_FILES: usize = 25;
pub const EXPECTED_PERFORMANCE_METHODS: usize = 121;
pub const EXPECTED_PERFORMANCE_ORDINARY: usize = 102;
pub const EXPECTED_PERFORMANCE_ASSOCIATED: usize = 19;
pub const EXPECTED_P_DESCRIPTOR_METHODS: usize = 0;
pub const EXPECTED_P_RECORD_METHODS: usize = 2;

const GP_PRIMARY_FRESH_ONLY_METHODS: [&str; 7] = [
    "0x0601A016",
    "0x0601A01E",
    "0x0601A01F",
    "0x0601A031",
    "0x0601A032",
    "0x0601A035",
    "0x06019FFB",
];

const DURABLE_FULL_LABELS: [&str; 14] = [
    "FULL_CIL",
    "INDEPENDENT_FULL_CIL_REVIEWED",
    "PRIMARY_FRESH_FULL_CIL_ANCHOR",
    "FRESH_FULL_CIL_CALLBACK",
    "INDEPENDENT_FRESH_UNTRUNCATED_CIL",
    "DIRECT_IL_VERIFIED",
    "DIRECT_CIL_FULL",
    "DIRECT_CIL_COMPLETE=true",
    "COMPLETE_DIRECT_CIL_METHOD_COUNT_9",
    "FULL_ACCEPTED_PRIOR_CIL",
    "CIL12_DOC",
    "CIL15_DOC",
    "CIL16_DOC",
    "P11_DOC",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignCatalog {
    pub schema: String,
    pub version: u64,
    pub campaign: String,
    pub baseline_v5_sha256: String,
    pub frozen_snapshot_id: String,
    pub frozen_registry_receipt_sha256: String,
    pub approved_cohorts: Vec<CohortSpec>,
    pub records: Vec<CampaignRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CohortSpec {
    pub id: String,
    pub files: u64,
    pub methods: u64,
    pub ordinary_methods: u64,
    pub associated_methods: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignRecord {
    pub id: String,
    pub cohort: String,
    pub existing_record_id: Option<String>,
    pub source_path: String,
    pub source_size: u64,
    pub source_sha256: String,
    pub destination_path: String,
    pub authority_id: String,
    pub authority_sha256: String,
    pub authority_mvid: String,
    #[serde(rename = "type_def")]
    pub type_def: Option<String>,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
    pub verification_basis: String,
    pub evidence_references: Vec<EvidenceReference>,
    pub scope: String,
    pub completeness: String,
    pub representation: String,
    pub namespace_context: Option<String>,
    pub relationship: String,
    pub explanation: String,
    pub dependencies: Vec<String>,
    pub editorial_changes: Vec<String>,
    pub source_quality: String,
    pub curation_status: CurationStatus,
    pub methods: Vec<CampaignMethod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignMethod {
    #[serde(rename = "type_def")]
    pub type_def: String,
    #[serde(rename = "method_def")]
    pub method_def: String,
    pub root_type: String,
    pub role: String,
    pub coverage: String,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
    pub verification_basis: String,
    pub rva: Option<String>,
    pub il_range: Option<String>,
    pub code_size: Option<u64>,
    pub map_line: Option<u64>,
    pub source_lines: Option<String>,
    pub evidence_references: Vec<EvidenceReference>,
    pub reported_labels: Vec<String>,
    pub association: String,
    pub representation: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchHeader {
    pub schema: String,
    pub version: u64,
    pub campaign: String,
    pub baseline_v5_sha256: String,
    pub frozen_snapshot_id: String,
    pub frozen_registry_receipt_sha256: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub approved_cohorts: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct BatchActivation {
    pub schema: String,
    pub version: u64,
    pub sequence: u64,
    pub cohort: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub descriptor_count: u64,
    pub method_count: u64,
    pub raw_previous_prefix_sha256: String,
}

#[derive(Debug, Clone)]
pub struct BatchIndex {
    pub header: BatchHeader,
    pub activations: Vec<BatchActivation>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CopiedFile {
    pub id: String,
    pub source_path: String,
    pub destination_path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CopyReceipt {
    pub campaign: String,
    pub cohort: String,
    pub files: Vec<CopiedFile>,
    pub complete: bool,
    pub failed_attempt: Option<FailedAttempt>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FailedAttempt {
    pub id: String,
    pub destination_path: String,
    pub stage: String,
    pub created: bool,
    pub partial: bool,
    pub observed_size: Option<u64>,
    pub observed_sha256: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CopyFailure {
    pub code: String,
    pub completed: Vec<CopiedFile>,
    pub failed_attempt: Option<FailedAttempt>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplyReceipt {
    pub copy: CopyReceipt,
    pub index_before_sha256: Option<String>,
    pub index_after_sha256: String,
    pub activation: BatchActivation,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ApplyFailure {
    pub code: String,
    pub copy: Option<CopyReceipt>,
    pub copy_failure: Option<CopyFailure>,
    pub index_touched: bool,
}

fn err(code: &str) -> ValidationError {
    ValidationError::new(code)
}

fn err_at(code: &str, path: &str) -> ValidationError {
    ValidationError::at(code, path)
}

fn require_lower_hex(value: &str, code: &str) -> Result<(), ValidationError> {
    if valid_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(err(code))
    }
}

fn validate_token_pair(type_def: &str, method_def: &str) -> Result<(), ValidationError> {
    if !validate_type_token(type_def) || !validate_method_token(method_def) {
        return Err(err("BULK_METHOD_TOKEN_INVALID"));
    }
    Ok(())
}

fn validate_method_roles(method: &CampaignMethod) -> Result<(), ValidationError> {
    if method.role != "ORDINARY" && method.role != "ASSOCIATED" {
        return Err(err("BULK_METHOD_ROLE_INVALID"));
    }
    if method.coverage.is_empty()
        || method.verification_basis.is_empty()
        || method.association.is_empty()
        || method.representation.is_empty()
    {
        return Err(err("BULK_METHOD_METADATA_INCOMPLETE"));
    }
    if method.evidence_references.is_empty() {
        return Err(err("BULK_METHOD_PROOF_MISSING"));
    }
    for reference in &method.evidence_references {
        validate_relative_path(&reference.path).map_err(|_| err("BULK_EVIDENCE_PATH_INVALID"))?;
        if reference.locator.is_empty() {
            return Err(err("BULK_EVIDENCE_LOCATOR_EMPTY"));
        }
        if let Some(hash) = &reference.artifact_sha256 {
            require_lower_hex(hash, "BULK_EVIDENCE_SHA_INVALID")?;
        }
    }
    if let Some(rva) = &method.rva {
        if rva.is_empty() {
            return Err(err("BULK_METHOD_RVA_EMPTY"));
        }
    }
    if let Some(il_range) = &method.il_range {
        if il_range.is_empty() {
            return Err(err("BULK_METHOD_IL_RANGE_EMPTY"));
        }
    }
    Ok(())
}

/// Structural validation. Exact campaign cardinalities are checked separately by
/// the approved-catalog loader.
pub fn validate_campaign_catalog(catalog: &CampaignCatalog) -> Result<(), ValidationError> {
    if catalog.schema != CATALOG_SCHEMA
        || catalog.version != 1
        || catalog.campaign != CAMPAIGN_ID
        || catalog.baseline_v5_sha256 != BASELINE_V5_RECEIPT_SHA256
        || catalog.frozen_snapshot_id != FROZEN_SNAPSHOT_ID
        || catalog.frozen_registry_receipt_sha256 != FROZEN_REGISTRY_RECEIPT_SHA256
    {
        return Err(err("BULK_CATALOG_ROOT_INVALID"));
    }
    if catalog.approved_cohorts.is_empty() {
        return Err(err("BULK_COHORT_SET_EMPTY"));
    }

    let mut cohort_ids = BTreeSet::new();
    for cohort in &catalog.approved_cohorts {
        if !cohort_ids.insert(cohort.id.clone()) {
            return Err(err("BULK_DUPLICATE_COHORT"));
        }
        if cohort.files == 0
            || cohort.methods == 0
            || cohort.ordinary_methods + cohort.associated_methods != cohort.methods
        {
            return Err(err("BULK_COHORT_COUNT_INVALID"));
        }
    }
    if !cohort_ids.contains(GAMEPLAY_COHORT) || !cohort_ids.contains(PERFORMANCE_COHORT) {
        return Err(err("BULK_COHORT_SET_INVALID"));
    }

    let mut ids = BTreeSet::new();
    let mut source_paths = BTreeSet::new();
    let mut destination_paths = BTreeSet::new();
    let mut method_keys = BTreeSet::new();
    for record in &catalog.records {
        if !cohort_ids.contains(&record.cohort) {
            return Err(err("BULK_RECORD_COHORT_UNKNOWN"));
        }
        if record.id.is_empty()
            || !record
                .id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(err("BULK_RECORD_ID_INVALID"));
        }
        if !ids.insert(record.id.clone()) {
            return Err(err("BULK_DUPLICATE_RECORD_ID"));
        }
        validate_relative_path(&record.source_path).map_err(|_| err("BULK_SOURCE_PATH_INVALID"))?;
        validate_relative_path(&record.destination_path)
            .map_err(|_| err("BULK_DESTINATION_PATH_INVALID"))?;
        if !record.source_path.starts_with("src/reconstructed/") {
            return Err(err("BULK_SOURCE_ROOT_INVALID"));
        }
        if !record.destination_path.starts_with(AUTHORITY_PREFIX) {
            return Err(err("BULK_DESTINATION_AUTHORITY_INVALID"));
        }
        if !source_paths.insert(record.source_path.to_ascii_lowercase()) {
            return Err(err("BULK_SOURCE_PATH_COLLISION"));
        }
        if !destination_paths.insert(record.destination_path.to_ascii_lowercase()) {
            return Err(err("BULK_DESTINATION_COLLISION"));
        }
        if record.source_size == 0 {
            return Err(err("BULK_SOURCE_SIZE_ZERO"));
        }
        require_lower_hex(&record.source_sha256, "BULK_SOURCE_SHA_INVALID")?;
        if record.authority_id != super::MANAGED_AUTHORITY_ID
            || record.authority_sha256 != super::MANAGED_AUTHORITY_SHA256
            || record.authority_mvid != super::MANAGED_MVID
        {
            return Err(err("BULK_AUTHORITY_INVALID"));
        }
        if record.verification_basis.is_empty()
            || record.scope.is_empty()
            || record.completeness.is_empty()
            || record.representation.is_empty()
            || record.relationship.is_empty()
            || record.explanation.is_empty()
            || record.source_quality.is_empty()
            || record
                .namespace_context
                .as_deref()
                .is_some_and(str::is_empty)
        {
            return Err(err("BULK_RECORD_TEXT_INCOMPLETE"));
        }
        for reference in &record.evidence_references {
            validate_relative_path(&reference.path)
                .map_err(|_| err("BULK_RECORD_EVIDENCE_PATH_INVALID"))?;
            if reference.locator.is_empty() {
                return Err(err("BULK_RECORD_EVIDENCE_LOCATOR_EMPTY"));
            }
            if let Some(hash) = &reference.artifact_sha256 {
                require_lower_hex(hash, "BULK_RECORD_EVIDENCE_SHA_INVALID")?;
            }
        }

        if let Some(existing) = &record.existing_record_id {
            if existing != "P" || record.id != "P" || record.cohort != PERFORMANCE_COHORT {
                return Err(err("BULK_EXISTING_RECORD_INVALID"));
            }
            if !record.methods.is_empty() {
                return Err(err("BULK_EXISTING_RECORD_METHODS_NOT_NEW"));
            }
            continue;
        }
        if record.id == "P" {
            return Err(err("BULK_NEW_RECORD_USURPS_EXISTING_ID"));
        }
        if record.methods.is_empty() {
            return Err(err("BULK_NEW_RECORD_METHODS_EMPTY"));
        }
        if let Some(type_def) = &record.type_def {
            if !validate_type_token(type_def) {
                return Err(err("BULK_RECORD_TYPEDEF_INVALID"));
            }
        }
        let mut record_method_keys = BTreeSet::new();
        for method in &record.methods {
            validate_token_pair(&method.type_def, &method.method_def)?;
            if method.root_type.is_empty() || !validate_type_token(&method.root_type) {
                return Err(err("BULK_METHOD_ROOT_TYPE_INVALID"));
            }
            validate_method_roles(method)?;
            if record.source_status == SourceStatus::InferredConditional
                && (method.source_status != SourceStatus::InferredConditional
                    || method.evidence_status != EvidenceStatus::SourceOnly)
            {
                return Err(err("BULK_INFERRED_EVIDENCE_PROMOTION"));
            }
            if method.evidence_status == EvidenceStatus::DirectCilFull
                && !method
                    .reported_labels
                    .iter()
                    .any(|label| DURABLE_FULL_LABELS.contains(&label.as_str()))
            {
                return Err(err("BULK_DIRECT_EVIDENCE_WITHOUT_DURABLE_LABEL"));
            }
            if record.source_path.ends_with("BaseSkillComponent.cs")
                && GP_PRIMARY_FRESH_ONLY_METHODS.contains(&method.method_def.as_str())
                && method.evidence_status == EvidenceStatus::DirectCilFull
            {
                return Err(err("BULK_GP_PRIMARY_ONLY_PROMOTION"));
            }
            if let Some(type_def) = &record.type_def {
                if type_def != &method.type_def {
                    return Err(err("BULK_METHOD_TYPEDEF_MISMATCH"));
                }
            }
            let key = format!("{}/{}", method.type_def, method.method_def);
            if !record_method_keys.insert(key.clone()) {
                return Err(err("BULK_DUPLICATE_RECORD_METHOD"));
            }
            if !method_keys.insert(key) {
                return Err(err("BULK_DUPLICATE_QUALIFIED_METHOD"));
            }
        }
    }
    let p_records = catalog
        .records
        .iter()
        .filter(|record| record.existing_record_id.as_deref() == Some("P"))
        .count();
    if p_records != 1 {
        return Err(err("BULK_P_RECORD_COUNT_INVALID"));
    }
    Ok(())
}

fn validate_approved_catalog_shape(catalog: &CampaignCatalog) -> Result<(), ValidationError> {
    validate_campaign_catalog(catalog)?;
    if catalog.records.len() != EXPECTED_RECORDS {
        return Err(err("BULK_APPROVED_RECORD_COUNT_INVALID"));
    }
    let gameplay = catalog
        .approved_cohorts
        .iter()
        .find(|cohort| cohort.id == GAMEPLAY_COHORT)
        .ok_or_else(|| err("BULK_GAMEPLAY_COHORT_MISSING"))?;
    let performance = catalog
        .approved_cohorts
        .iter()
        .find(|cohort| cohort.id == PERFORMANCE_COHORT)
        .ok_or_else(|| err("BULK_PERFORMANCE_COHORT_MISSING"))?;
    if gameplay.files != EXPECTED_GAMEPLAY_FILES as u64
        || gameplay.methods != EXPECTED_GAMEPLAY_METHODS as u64
        || gameplay.ordinary_methods != EXPECTED_GAMEPLAY_ORDINARY as u64
        || gameplay.associated_methods != EXPECTED_GAMEPLAY_ASSOCIATED as u64
        || performance.files != EXPECTED_PERFORMANCE_FILES as u64
        || performance.methods != EXPECTED_PERFORMANCE_METHODS as u64
        || performance.ordinary_methods != EXPECTED_PERFORMANCE_ORDINARY as u64
        || performance.associated_methods != EXPECTED_PERFORMANCE_ASSOCIATED as u64
    {
        return Err(err("BULK_APPROVED_COHORT_COUNTS_INVALID"));
    }
    let gp_records = catalog
        .records
        .iter()
        .filter(|record| record.cohort == GAMEPLAY_COHORT)
        .count();
    let perf_records = catalog
        .records
        .iter()
        .filter(|record| record.cohort == PERFORMANCE_COHORT)
        .count();
    if gp_records != EXPECTED_GAMEPLAY_FILES || perf_records != EXPECTED_PERFORMANCE_FILES {
        return Err(err("BULK_APPROVED_FILE_COUNTS_INVALID"));
    }
    let mut total_descriptor_methods = 0_usize;
    let mut gp_ordinary = 0_usize;
    let mut gp_associated = 0_usize;
    let mut perf_ordinary = 0_usize;
    let mut perf_associated = 0_usize;
    let mut p_descriptor_methods = 0_usize;
    for record in &catalog.records {
        match record.existing_record_id.as_deref() {
            Some("P") => p_descriptor_methods += record.methods.len(),
            _ => {
                total_descriptor_methods += record.methods.len();
                for method in &record.methods {
                    match (record.cohort.as_str(), method.role.as_str()) {
                        (GAMEPLAY_COHORT, "ORDINARY") => gp_ordinary += 1,
                        (GAMEPLAY_COHORT, "ASSOCIATED") => gp_associated += 1,
                        (PERFORMANCE_COHORT, "ORDINARY") => perf_ordinary += 1,
                        (PERFORMANCE_COHORT, "ASSOCIATED") => perf_associated += 1,
                        _ => return Err(err("BULK_APPROVED_METHOD_ROLE_INVALID")),
                    }
                }
            }
        }
    }
    if total_descriptor_methods != EXPECTED_DESCRIPTOR_METHODS
        || total_descriptor_methods + EXPECTED_P_RECORD_METHODS != EXPECTED_TOTAL_METHODS
        || p_descriptor_methods != EXPECTED_P_DESCRIPTOR_METHODS
        || gp_ordinary != EXPECTED_GAMEPLAY_ORDINARY
        || gp_associated != EXPECTED_GAMEPLAY_ASSOCIATED
        || perf_ordinary != 100
        || perf_associated != EXPECTED_PERFORMANCE_ASSOCIATED
    {
        return Err(err("BULK_APPROVED_METHOD_COUNTS_INVALID"));
    }
    Ok(())
}

pub fn parse_campaign_catalog(bytes: &[u8]) -> Result<CampaignCatalog, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let catalog: CampaignCatalog =
        serde_json::from_value(value).map_err(|_| err("BULK_CATALOG_SCHEMA_INVALID"))?;
    validate_campaign_catalog(&catalog)?;
    Ok(catalog)
}

pub fn load_campaign_catalog(root: &Path) -> Result<CampaignCatalog, ValidationError> {
    let bytes =
        read_relative(root, CATALOG_REL).map_err(|failure| err_at(failure.code, CATALOG_REL))?;
    if sha256_hex(&bytes) != CATALOG_SHA256 {
        return Err(err("BULK_CATALOG_ANCHOR_MISMATCH"));
    }
    let catalog = parse_campaign_catalog(&bytes)?;
    validate_approved_catalog_shape(&catalog)?;
    Ok(catalog)
}

fn load_approved_catalog(root: &Path) -> Result<CampaignCatalog, ValidationError> {
    load_campaign_catalog(root)
}

fn build_record(record: &CampaignRecord) -> Result<Record, ValidationError> {
    let mut current = Vec::with_capacity(record.methods.len());
    for method in &record.methods {
        validate_token_pair(&method.type_def, &method.method_def)?;
        current.push(CurrentMethod {
            method_def: method.method_def.clone(),
            coverage: format!("{}; {}", method.role, method.coverage),
            evidence_status: method.evidence_status.clone(),
            source_status: method.source_status.clone(),
            verification_basis: method.verification_basis.clone(),
            symbol_key: MethodIdentity {
                full_authority_sha256: record.authority_sha256.clone(),
                mvid: record.authority_mvid.clone(),
                type_def: method.type_def.clone(),
                method_def: method.method_def.clone(),
            },
            rva: method.rva.clone(),
            il_range: method.il_range.clone(),
            evidence_references: method.evidence_references.clone(),
            source_provenance: None,
        });
    }
    Ok(Record {
        id: record.id.clone(),
        unit_id: None,
        namespace_context: record.namespace_context.clone(),
        original_path: record.source_path.clone(),
        original_size: record.source_size,
        original_sha256: record.source_sha256.clone(),
        hash_basis: "raw_source_file_bytes".to_owned(),
        evidence_status: record.evidence_status.clone(),
        source_status: record.source_status.clone(),
        verification_basis: record.verification_basis.clone(),
        evidence_references: record.evidence_references.clone(),
        proposed_path: Some(record.destination_path.clone()),
        proposed_path_reason:
            "Approved bulk-curation campaign destination; copy and activation receipts determine import state."
                .to_owned(),
        authority_id: record.authority_id.clone(),
        authority_binary_sha256: Some(record.authority_sha256.clone()),
        authority_sha256_kind: "managed_assembly".to_owned(),
        authority_mvid: Some(record.authority_mvid.clone()),
        build_reference: None,
        type_def: record.type_def.clone(),
        methods: Methods {
            historical: Vec::new(),
            current,
        },
        scope: record.scope.clone(),
        completeness: record.completeness.clone(),
        representation: record.representation.clone(),
        curation_status: record.curation_status.clone(),
        export_status: ExportStatus::PublicationReviewRequired,
        duplicate_group: None,
        relationship: record.relationship.clone(),
        supersession: Supersession {
            same_authority: true,
            qualified: false,
            scope: Vec::new(),
            supersedes: Vec::new(),
            superseded_by: Vec::new(),
        },
        explanation: record.explanation.clone(),
        dependencies: record.dependencies.clone(),
        editorial_changes: record.editorial_changes.clone(),
        validation: RecordValidation {
            source_hash: "CATALOG_DECLARED_EXACT_SOURCE_BYTES".to_owned(),
            authority_identity: "MANAGED_AUTHORITY_SHA_MVID_ACTUAL_METHOD_TOKENS".to_owned(),
            path_collision: "CHECKED_AGAINST_BASE_AND_BATCH".to_owned(),
            equivalence: "NOT_ESTABLISHED".to_owned(),
            compilation: "NOT_BUILDABLE_WITH_CURRENT_CONTEXT".to_owned(),
            runtime: "PARKED_NOT_RUN".to_owned(),
            publication: "PUBLICATION_REVIEW_REQUIRED".to_owned(),
            final_receipt: "EXTERNAL_PRIMARY_RECEIPT_REQUIRED".to_owned(),
        },
    })
}

fn file_view(record: &Record, destination_path: &str) -> ImportedFileView {
    ImportedFileView {
        id: record.id.clone(),
        original_path: record.original_path.clone(),
        imported: SourceArtifact {
            path: destination_path.to_owned(),
            size: record.original_size,
            sha256: record.original_sha256.clone(),
        },
        methods: record
            .methods
            .current
            .iter()
            .map(|method| method.symbol_key.clone())
            .collect(),
    }
}

/// Merge only the activated cohorts. No record or file is mutated in the base
/// v5 context; P is reused by ID and method set, never cloned.
pub fn apply_activated_catalog(
    manifest: &mut Manifest,
    active: &mut Option<ActiveReceipt>,
    catalog: &CampaignCatalog,
    activated: &[String],
) -> Result<(), ValidationError> {
    validate_campaign_catalog(catalog)?;
    let Some(base_active) = active.as_ref() else {
        return Err(err("BULK_INDEX_REQUIRES_V5"));
    };
    if base_active.version != 5 {
        return Err(err("BULK_INDEX_REQUIRES_V5"));
    }
    let mut activated_set = BTreeSet::new();
    for cohort in activated {
        if cohort != GAMEPLAY_COHORT && cohort != PERFORMANCE_COHORT {
            return Err(err("BULK_ACTIVATION_COHORT_UNKNOWN"));
        }
        if !activated_set.insert(cohort.as_str()) {
            return Err(err("BULK_ACTIVATION_COHORT_DUPLICATE"));
        }
    }
    if !activated_set.is_empty() && activated.first().map(String::as_str) != Some(GAMEPLAY_COHORT) {
        return Err(err("BULK_ACTIVATION_ORDER_INVALID"));
    }
    if activated.len() == 2 && activated[1] != PERFORMANCE_COHORT {
        return Err(err("BULK_ACTIVATION_ORDER_INVALID"));
    }

    let mut base_ids = BTreeSet::new();
    let mut base_sources = BTreeSet::new();
    let mut base_destinations = BTreeSet::new();
    for record in &manifest.records {
        base_ids.insert(record.id.clone());
        base_sources.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            base_destinations.insert(path.to_ascii_lowercase());
        }
    }
    let mut active_ids = BTreeSet::new();
    let mut active_methods = BTreeSet::new();
    for file in &base_active.files {
        active_ids.insert(file.id.clone());
        for method in &file.methods {
            active_methods.insert(format!(
                "{}|{}|{}|{}",
                method.full_authority_sha256, method.mvid, method.type_def, method.method_def
            ));
        }
    }

    let mut new_records = Vec::new();
    let mut new_files = Vec::new();
    let mut seen_sources = base_sources.clone();
    let mut seen_destinations = base_destinations.clone();
    let mut seen_methods = active_methods.clone();
    let mut seen_record_ids = base_ids.clone();
    let mut seen_file_ids = active_ids.clone();

    for descriptor in catalog
        .records
        .iter()
        .filter(|record| activated_set.contains(record.cohort.as_str()))
    {
        if let Some(existing) = &descriptor.existing_record_id {
            if existing != "P" {
                return Err(err("BULK_EXISTING_RECORD_INVALID"));
            }
            let Some(record) = manifest.records.iter().find(|record| record.id == "P") else {
                return Err(err("BULK_P_RECORD_MISSING"));
            };
            if record.original_path != descriptor.source_path
                || record.original_size != descriptor.source_size
                || record.original_sha256 != descriptor.source_sha256
                || record.proposed_path.as_deref() != Some(descriptor.destination_path.as_str())
            {
                return Err(err("BULK_P_DESCRIPTOR_MISMATCH"));
            }
            if descriptor.methods.is_empty() {
                // P's existing methods remain the sole authority.
            } else if descriptor.methods.len() != record.methods.current.len() {
                return Err(err("BULK_P_METHOD_SET_MISMATCH"));
            }
            for method in &record.methods.current {
                let key = format!(
                    "{}|{}|{}|{}",
                    method.symbol_key.full_authority_sha256,
                    method.symbol_key.mvid,
                    method.symbol_key.type_def,
                    method.symbol_key.method_def
                );
                if !seen_methods.insert(key) {
                    return Err(err("BULK_QUALIFIED_METHOD_OVERLAP"));
                }
            }
            if !seen_file_ids.insert(record.id.clone()) {
                return Err(err("BULK_DUPLICATE_ACTIVE_FILE_ID"));
            }
            new_files.push(file_view(record, &descriptor.destination_path));
            continue;
        }

        if seen_record_ids.contains(&descriptor.id) {
            return Err(err("BULK_DUPLICATE_RECORD_ID_ACTIVE"));
        }
        seen_record_ids.insert(descriptor.id.clone());
        if !seen_sources.insert(descriptor.source_path.to_ascii_lowercase()) {
            return Err(err("BULK_SOURCE_PATH_COLLISION_BASE"));
        }
        if !seen_destinations.insert(descriptor.destination_path.to_ascii_lowercase()) {
            return Err(err("BULK_DESTINATION_COLLISION_BASE"));
        }
        let record = build_record(descriptor)?;
        for method in &record.methods.current {
            let key = format!(
                "{}|{}|{}|{}",
                method.symbol_key.full_authority_sha256,
                method.symbol_key.mvid,
                method.symbol_key.type_def,
                method.symbol_key.method_def
            );
            if !seen_methods.insert(key) {
                return Err(err("BULK_QUALIFIED_METHOD_OVERLAP"));
            }
        }
        if !seen_file_ids.insert(record.id.clone()) {
            return Err(err("BULK_DUPLICATE_ACTIVE_FILE_ID"));
        }
        new_files.push(file_view(&record, &descriptor.destination_path));
        new_records.push(record);
    }

    manifest.records.extend(new_records);
    let mut files = base_active.files.clone();
    files.extend(new_files);
    *active = Some(ActiveReceipt {
        ledger_path: INDEX_REL.to_owned(),
        version: 6,
        files,
    });
    Ok(())
}

fn index_lines(bytes: &[u8]) -> Result<Vec<(usize, usize)>, ValidationError> {
    if bytes.is_empty() || bytes.last() != Some(&b'\n') {
        return Err(err("BULK_INDEX_TRUNCATED"));
    }
    let mut lines = Vec::new();
    let mut start = 0;
    for (position, byte) in bytes.iter().enumerate() {
        if *byte == b'\n' {
            if position == start {
                return Err(err("BULK_INDEX_EMPTY_LINE"));
            }
            lines.push((start, position));
            start = position + 1;
        }
    }
    if start != bytes.len() {
        return Err(err("BULK_INDEX_TRUNCATED"));
    }
    Ok(lines)
}

fn parse_line<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    start: usize,
    end: usize,
    code: &str,
) -> Result<T, ValidationError> {
    let value = parse_strict_value(&bytes[start..end])?;
    let canonical = canonical_json_line(&value)?;
    if canonical.as_slice() != &bytes[start..=end] {
        return Err(err("BULK_INDEX_NONCANONICAL_LINE"));
    }
    serde_json::from_value(value).map_err(|_| err(code))
}

fn canonicalize_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = BTreeMap::new();
            for (key, value) in map {
                sorted.insert(key, canonicalize_value(value));
            }
            let mut object = Map::new();
            for (key, value) in sorted {
                object.insert(key, value);
            }
            Value::Object(object)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize_value).collect()),
        value => value,
    }
}

/// Canonical compact JSON line used for both header and activation records.
pub fn canonical_json_line(value: &Value) -> Result<Vec<u8>, ValidationError> {
    let normalized = canonicalize_value(value.clone());
    let mut bytes =
        serde_json::to_vec(&normalized).map_err(|_| err("BULK_INDEX_SERIALIZE_FAILED"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn canonical_struct_line<T: Serialize>(value: &T) -> Result<Vec<u8>, ValidationError> {
    let value = serde_json::to_value(value).map_err(|_| err("BULK_INDEX_SERIALIZE_FAILED"))?;
    canonical_json_line(&value)
}

pub fn parse_index_bytes(bytes: &[u8]) -> Result<BatchIndex, ValidationError> {
    let lines = index_lines(bytes)?;
    if lines.is_empty() {
        return Err(err("BULK_INDEX_HEADER_MISSING"));
    }
    let header: BatchHeader =
        parse_line(bytes, lines[0].0, lines[0].1, "BULK_INDEX_SCHEMA_INVALID")?;
    if header.schema != INDEX_SCHEMA
        || header.version != 1
        || header.campaign != CAMPAIGN_ID
        || header.baseline_v5_sha256 != BASELINE_V5_RECEIPT_SHA256
        || header.frozen_snapshot_id != FROZEN_SNAPSHOT_ID
        || header.frozen_registry_receipt_sha256 != FROZEN_REGISTRY_RECEIPT_SHA256
        || header.catalog_path != CATALOG_REL
        || header.catalog_sha256 != CATALOG_SHA256
        || header.approved_cohorts
            != vec![GAMEPLAY_COHORT.to_owned(), PERFORMANCE_COHORT.to_owned()]
    {
        return Err(err("BULK_INDEX_HEADER_INVALID"));
    }

    let mut activations = Vec::new();
    for (line_index, (start, end)) in lines.iter().enumerate().skip(1) {
        let activation: BatchActivation =
            parse_line(bytes, *start, *end, "BULK_INDEX_ACTIVATION_SCHEMA_INVALID")?;
        let expected_sequence = line_index as u64;
        if activation.schema != ACTIVATION_SCHEMA
            || activation.version != 1
            || activation.sequence != expected_sequence
            || activation.catalog_path != CATALOG_REL
            || activation.catalog_sha256 != CATALOG_SHA256
            || activation.cohort
                != match expected_sequence {
                    1 => GAMEPLAY_COHORT,
                    2 => PERFORMANCE_COHORT,
                    _ => return Err(err("BULK_INDEX_ACTIVATION_ORDER_INVALID")),
                }
            || activation.descriptor_count == 0
            || activation.method_count == 0
            || sha256_hex(&bytes[..*start]) != activation.raw_previous_prefix_sha256
        {
            return Err(err("BULK_INDEX_ACTIVATION_INVALID"));
        }
        activations.push(activation);
    }
    if activations.len() > 2 {
        return Err(err("BULK_INDEX_ACTIVATION_ORDER_INVALID"));
    }
    Ok(BatchIndex {
        header,
        activations,
    })
}

pub fn append_activation_bytes(
    index_bytes: &[u8],
    desired_cohort: &str,
) -> Result<Vec<u8>, ValidationError> {
    let index = parse_index_bytes(index_bytes)?;
    if index.activations.len() >= 2 {
        return Err(err("BULK_INDEX_ALREADY_COMPLETE"));
    }
    let expected_sequence = index.activations.len() as u64 + 1;
    let expected_cohort = match expected_sequence {
        1 => GAMEPLAY_COHORT,
        2 => PERFORMANCE_COHORT,
        _ => return Err(err("BULK_INDEX_ACTIVATION_ORDER_INVALID")),
    };
    if desired_cohort != expected_cohort {
        return Err(err("BULK_INDEX_ACTIVATION_COHORT_MISMATCH"));
    }
    let (descriptor_count, method_count) = match expected_cohort {
        GAMEPLAY_COHORT => (
            EXPECTED_GAMEPLAY_FILES as u64,
            EXPECTED_GAMEPLAY_METHODS as u64,
        ),
        PERFORMANCE_COHORT => (
            EXPECTED_PERFORMANCE_FILES as u64,
            EXPECTED_PERFORMANCE_METHODS as u64,
        ),
        _ => return Err(err("BULK_INDEX_ACTIVATION_ORDER_INVALID")),
    };
    let activation = BatchActivation {
        schema: ACTIVATION_SCHEMA.to_owned(),
        version: 1,
        sequence: expected_sequence,
        cohort: expected_cohort.to_owned(),
        catalog_path: CATALOG_REL.to_owned(),
        catalog_sha256: CATALOG_SHA256.to_owned(),
        descriptor_count,
        method_count,
        raw_previous_prefix_sha256: sha256_hex(index_bytes),
    };
    let line = canonical_struct_line(&activation)?;
    let mut next = index_bytes.to_vec();
    next.extend_from_slice(&line);
    parse_index_bytes(&next)?;
    Ok(next)
}

fn batch_header() -> BatchHeader {
    BatchHeader {
        schema: INDEX_SCHEMA.to_owned(),
        version: 1,
        campaign: CAMPAIGN_ID.to_owned(),
        baseline_v5_sha256: BASELINE_V5_RECEIPT_SHA256.to_owned(),
        frozen_snapshot_id: FROZEN_SNAPSHOT_ID.to_owned(),
        frozen_registry_receipt_sha256: FROZEN_REGISTRY_RECEIPT_SHA256.to_owned(),
        catalog_path: CATALOG_REL.to_owned(),
        catalog_sha256: CATALOG_SHA256.to_owned(),
        approved_cohorts: vec![GAMEPLAY_COHORT.to_owned(), PERFORMANCE_COHORT.to_owned()],
    }
}

fn create_index_header(root: &Path) -> Result<(), ValidationError> {
    let state = inspect_relative_inner(root, INDEX_REL)
        .map_err(|failure| err_at(failure.code, INDEX_REL))?;
    if state == super::SafePathState::File {
        return Ok(());
    }
    if state != super::SafePathState::Missing {
        return Err(err_at("BULK_INDEX_NOT_REGULAR", INDEX_REL));
    }
    let bytes = canonical_struct_line(&batch_header())?;
    let absolute = root.join(INDEX_REL);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&absolute)
        .map_err(|_| err_at("BULK_INDEX_CREATE_FAILED", INDEX_REL))?;
    file.write_all(&bytes)
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|_| err_at("BULK_INDEX_WRITE_FAILED", INDEX_REL))?;
    Ok(())
}

fn append_activation_disk(
    root: &Path,
    cohort: &str,
    expected_before: Option<&[u8]>,
) -> Result<BatchActivation, ValidationError> {
    match expected_before {
        Some(expected) => {
            let current = read_relative(root, INDEX_REL)
                .map_err(|failure| err_at(failure.code, INDEX_REL))?;
            if current != expected {
                return Err(err_at("BULK_INDEX_PREFIX_CHANGED", INDEX_REL));
            }
        }
        None => {
            if inspect_relative_inner(root, INDEX_REL)
                .map_err(|failure| err_at(failure.code, INDEX_REL))?
                != super::SafePathState::Missing
            {
                return Err(err_at("BULK_INDEX_UNEXPECTED_EXISTING", INDEX_REL));
            }
        }
    }
    create_index_header(root)?;
    let before =
        read_relative(root, INDEX_REL).map_err(|failure| err_at(failure.code, INDEX_REL))?;
    let next = append_activation_bytes(&before, cohort)?;
    let index = parse_index_bytes(&next)?;
    let activation = index
        .activations
        .last()
        .ok_or_else(|| err("BULK_ACTIVATION_MISSING"))?
        .clone();
    let absolute = root.join(INDEX_REL);
    let mut file = OpenOptions::new()
        .append(true)
        .open(&absolute)
        .map_err(|_| err_at("BULK_INDEX_APPEND_OPEN_FAILED", INDEX_REL))?;
    file.write_all(&next[before.len()..])
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|_| err_at("BULK_INDEX_APPEND_FAILED", INDEX_REL))?;
    let after =
        read_relative(root, INDEX_REL).map_err(|failure| err_at(failure.code, INDEX_REL))?;
    if !after.starts_with(&before) || after != next {
        return Err(err_at("BULK_INDEX_PREFIX_MISMATCH", INDEX_REL));
    }
    parse_index_bytes(&after)?;
    Ok(activation)
}

fn activation_counts(
    catalog: &CampaignCatalog,
    cohort: &str,
    manifest: &Manifest,
) -> Result<(u64, u64), ValidationError> {
    let spec = catalog
        .approved_cohorts
        .iter()
        .find(|spec| spec.id == cohort)
        .ok_or_else(|| err("BULK_ACTIVATION_COHORT_UNKNOWN"))?;
    let records = catalog
        .records
        .iter()
        .filter(|record| record.cohort == cohort)
        .count() as u64;
    let mut methods = 0_u64;
    for record in catalog
        .records
        .iter()
        .filter(|record| record.cohort == cohort)
    {
        if let Some(existing) = &record.existing_record_id {
            if existing == "P" {
                let Some(existing_record) = manifest.records.iter().find(|record| record.id == "P")
                else {
                    return Err(err("BULK_P_RECORD_MISSING"));
                };
                methods += existing_record.methods.current.len() as u64;
            }
        } else {
            methods += record.methods.len() as u64;
        }
    }
    if records != spec.files || methods != spec.methods {
        return Err(err("BULK_ACTIVATION_COUNTS_INVALID"));
    }
    Ok((records, methods))
}

/// Read and validate the activation index if it exists. Absence is byte-for-byte
/// compatible with the old v5 loader. A present index must be backed by v5 and
/// the approved catalog; there is no invalid-index fallback.
pub fn apply_index(
    root: &Path,
    manifest: &mut Manifest,
    active: &mut Option<ActiveReceipt>,
) -> Result<(), ValidationError> {
    let state = inspect_relative_inner(root, INDEX_REL)
        .map_err(|failure| err_at(failure.code, INDEX_REL))?;
    if state == super::SafePathState::Missing {
        return Ok(());
    }
    if state != super::SafePathState::File {
        return Err(err_at("BULK_INDEX_NOT_REGULAR", INDEX_REL));
    }
    let Some(base_active) = active.as_ref() else {
        return Err(err("BULK_INDEX_REQUIRES_V5"));
    };
    if base_active.version != 5 {
        return Err(err("BULK_INDEX_REQUIRES_V5"));
    }
    let v5_bytes = read_relative(root, super::RECEIPT_V5_REL)
        .map_err(|failure| err_at(failure.code, super::RECEIPT_V5_REL))?;
    if sha256_hex(&v5_bytes) != BASELINE_V5_RECEIPT_SHA256 {
        return Err(err("BULK_INDEX_V5_ANCHOR_MISMATCH"));
    }
    let index_bytes =
        read_relative(root, INDEX_REL).map_err(|failure| err_at(failure.code, INDEX_REL))?;
    let index = parse_index_bytes(&index_bytes)?;
    let catalog = load_approved_catalog(root)?;
    let mut cohorts = Vec::new();
    for activation in &index.activations {
        let (records, methods) = activation_counts(&catalog, &activation.cohort, manifest)?;
        if activation.descriptor_count != records || activation.method_count != methods {
            return Err(err("BULK_ACTIVATION_COUNTS_INVALID"));
        }
        cohorts.push(activation.cohort.clone());
    }
    apply_activated_catalog(manifest, active, &catalog, &cohorts)
}

fn copy_failure(
    code: &str,
    completed: &[CopiedFile],
    record: Option<&CampaignRecord>,
    curated_root: &Path,
    created: bool,
    stage: &str,
) -> CopyFailure {
    let failed_attempt = record.map(|record| {
        let observed = read_relative(curated_root, &record.destination_path).ok();
        let observed_size = observed.as_ref().map(|bytes| bytes.len() as u64);
        let observed_sha256 = observed.as_ref().map(|bytes| sha256_hex(bytes));
        let partial = observed_size.is_some_and(|size| size != record.source_size)
            || observed_sha256
                .as_deref()
                .is_some_and(|hash| hash != record.source_sha256);
        FailedAttempt {
            id: record.id.clone(),
            destination_path: record.destination_path.clone(),
            stage: stage.to_owned(),
            created,
            partial,
            observed_size,
            observed_sha256,
        }
    });
    CopyFailure {
        code: code.to_owned(),
        completed: completed.to_vec(),
        failed_attempt,
    }
}

fn copy_one<F>(
    source_root: &Path,
    curated_root: &Path,
    record: &CampaignRecord,
    bytes: &[u8],
    completed: &[CopiedFile],
    mut fault: Option<&mut F>,
) -> Result<CopiedFile, CopyFailure>
where
    F: FnMut(&str, &mut fs::File) -> Result<(), ()>,
{
    let before = match inspect_relative_inner(curated_root, &record.destination_path) {
        Ok(super::SafePathState::Missing) => (),
        Ok(_) => {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_COLLISION",
                completed,
                Some(record),
                curated_root,
                false,
                "precreate",
            ));
        }
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_UNSAFE",
                completed,
                Some(record),
                curated_root,
                false,
                "precreate",
            ));
        }
    };
    let _ = before;
    let destination = curated_root.join(&record.destination_path);
    let Some(parent) = destination.parent() else {
        return Err(copy_failure(
            "BULK_COPY_DESTINATION_PARENT_MISSING",
            completed,
            Some(record),
            curated_root,
            false,
            "parent",
        ));
    };
    if fs::create_dir_all(parent).is_err() {
        return Err(copy_failure(
            "BULK_COPY_PARENT_CREATE_FAILED",
            completed,
            Some(record),
            curated_root,
            false,
            "parent",
        ));
    }
    match inspect_relative_inner(curated_root, &record.destination_path) {
        Ok(super::SafePathState::Missing) => (),
        Ok(_) => {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_COLLISION",
                completed,
                Some(record),
                curated_root,
                false,
                "postparent",
            ));
        }
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_UNSAFE",
                completed,
                Some(record),
                curated_root,
                false,
                "postparent",
            ));
        }
    }
    let mut file = match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
    {
        Ok(file) => file,
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_COLLISION",
                completed,
                Some(record),
                curated_root,
                false,
                "create",
            ));
        }
    };
    if let Some(callback) = fault.as_deref_mut() {
        if callback(&record.destination_path, &mut file).is_err() {
            drop(file);
            return Err(copy_failure(
                "BULK_COPY_INJECTED_FAULT",
                completed,
                Some(record),
                curated_root,
                true,
                "write",
            ));
        }
    }
    if file.write_all(bytes).is_err() || file.flush().is_err() {
        drop(file);
        return Err(copy_failure(
            "BULK_COPY_WRITE_FAILED",
            completed,
            Some(record),
            curated_root,
            true,
            "write",
        ));
    }
    drop(file);
    match inspect_relative_inner(curated_root, &record.destination_path) {
        Ok(super::SafePathState::File) => (),
        Ok(_) => {
            return Err(copy_failure(
                "BULK_COPY_READBACK_NOT_REGULAR",
                completed,
                Some(record),
                curated_root,
                true,
                "readback",
            ));
        }
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_READBACK_UNSAFE",
                completed,
                Some(record),
                curated_root,
                true,
                "readback",
            ));
        }
    }
    let copied = match read_relative(curated_root, &record.destination_path) {
        Ok(copied) => copied,
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_READBACK_FAILED",
                completed,
                Some(record),
                curated_root,
                true,
                "readback",
            ));
        }
    };
    if copied != bytes {
        return Err(copy_failure(
            "BULK_COPY_READBACK_MISMATCH",
            completed,
            Some(record),
            curated_root,
            true,
            "readback",
        ));
    }
    let source_after = match read_relative(source_root, &record.source_path) {
        Ok(source_after) => source_after,
        Err(_) => {
            return Err(copy_failure(
                "BULK_COPY_SOURCE_READBACK_FAILED",
                completed,
                Some(record),
                curated_root,
                true,
                "sourcecheck",
            ));
        }
    };
    if sha256_hex(&source_after) != record.source_sha256
        || source_after.len() as u64 != record.source_size
    {
        return Err(copy_failure(
            "BULK_COPY_SOURCE_MUTATED",
            completed,
            Some(record),
            curated_root,
            true,
            "sourcecheck",
        ));
    }
    Ok(CopiedFile {
        id: record.id.clone(),
        source_path: record.source_path.clone(),
        destination_path: record.destination_path.clone(),
        size: record.source_size,
        sha256: record.source_sha256.clone(),
    })
}

fn copy_cohort_with_hook<F>(
    source_root: &Path,
    curated_root: &Path,
    catalog: &CampaignCatalog,
    cohort: &str,
    mut fault: Option<&mut F>,
) -> Result<CopyReceipt, CopyFailure>
where
    F: FnMut(&str, &mut fs::File) -> Result<(), ()>,
{
    if cohort != GAMEPLAY_COHORT && cohort != PERFORMANCE_COHORT {
        return Err(copy_failure(
            "BULK_COPY_COHORT_UNKNOWN",
            &[],
            None,
            curated_root,
            false,
            "cohort",
        ));
    }
    if validate_campaign_catalog(catalog).is_err() {
        return Err(copy_failure(
            "BULK_COPY_CATALOG_INVALID",
            &[],
            None,
            curated_root,
            false,
            "catalog",
        ));
    }
    let mut descriptors: Vec<&CampaignRecord> = catalog
        .records
        .iter()
        .filter(|record| record.cohort == cohort)
        .collect();
    if descriptors.is_empty() {
        return Err(copy_failure(
            "BULK_COPY_COHORT_EMPTY",
            &[],
            None,
            curated_root,
            false,
            "cohort",
        ));
    }
    descriptors.sort_by(|left, right| left.destination_path.cmp(&right.destination_path));
    let mut preflight_sources = BTreeSet::new();
    let mut preflight_destinations = BTreeSet::new();
    let mut source_bytes = Vec::with_capacity(descriptors.len());
    for record in &descriptors {
        if !preflight_sources.insert(record.source_path.to_ascii_lowercase())
            || !preflight_destinations.insert(record.destination_path.to_ascii_lowercase())
        {
            return Err(copy_failure(
                "BULK_COPY_BATCH_COLLISION",
                &[],
                Some(record),
                curated_root,
                false,
                "preflight",
            ));
        }
        if validate_relative_path(&record.destination_path).is_err()
            || !record.destination_path.starts_with(AUTHORITY_PREFIX)
        {
            return Err(copy_failure(
                "BULK_COPY_DESTINATION_UNSAFE",
                &[],
                Some(record),
                curated_root,
                false,
                "preflight",
            ));
        }
        match inspect_relative_inner(curated_root, &record.destination_path) {
            Ok(super::SafePathState::Missing) => (),
            Ok(_) => {
                return Err(copy_failure(
                    "BULK_COPY_DESTINATION_COLLISION",
                    &[],
                    Some(record),
                    curated_root,
                    false,
                    "preflight",
                ));
            }
            Err(_) => {
                return Err(copy_failure(
                    "BULK_COPY_DESTINATION_UNSAFE",
                    &[],
                    Some(record),
                    curated_root,
                    false,
                    "preflight",
                ));
            }
        }
        let bytes = match read_relative(source_root, &record.source_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(copy_failure(
                    "BULK_COPY_SOURCE_MISSING",
                    &[],
                    Some(record),
                    curated_root,
                    false,
                    "sourcepreflight",
                ));
            }
        };
        if bytes.len() as u64 != record.source_size || sha256_hex(&bytes) != record.source_sha256 {
            return Err(copy_failure(
                "BULK_COPY_SOURCE_MISMATCH",
                &[],
                Some(record),
                curated_root,
                false,
                "sourcepreflight",
            ));
        }
        source_bytes.push(bytes);
    }

    let mut completed = Vec::with_capacity(descriptors.len());
    for (record, bytes) in descriptors.iter().zip(source_bytes.iter()) {
        match copy_one(
            source_root,
            curated_root,
            record,
            bytes,
            &completed,
            fault.as_deref_mut(),
        ) {
            Ok(copied) => completed.push(copied),
            Err(failure) => return Err(failure),
        }
    }
    Ok(CopyReceipt {
        campaign: CAMPAIGN_ID.to_owned(),
        cohort: cohort.to_owned(),
        files: completed,
        complete: true,
        failed_attempt: None,
    })
}

fn copy_cohort_unchecked(
    source_root: &Path,
    curated_root: &Path,
    catalog: &CampaignCatalog,
    cohort: &str,
) -> Result<CopyReceipt, CopyFailure> {
    type Fault = fn(&str, &mut fs::File) -> Result<(), ()>;
    copy_cohort_with_hook::<Fault>(source_root, curated_root, catalog, cohort, None)
}

/// Persist a create-new copy receipt under an already-safe relative path.
/// The leaf must be missing; ancestors are inspected by the shared no-follow
/// path guard, and no arbitrary absolute destination is accepted.
pub fn write_copy_receipt(
    root: &Path,
    relative: &str,
    receipt: &CopyReceipt,
) -> Result<(), ValidationError> {
    validate_relative_path(relative).map_err(|_| err("BULK_COPY_RECEIPT_PATH_UNSAFE"))?;
    match inspect_relative_inner(root, relative) {
        Ok(super::SafePathState::Missing) => (),
        Ok(_) => return Err(err("BULK_COPY_RECEIPT_EXISTS")),
        Err(failure) => return Err(err_at(failure.code, relative)),
    }
    let json =
        serde_json::to_string(receipt).map_err(|_| err("BULK_COPY_RECEIPT_SERIALIZE_FAILED"))?;
    let path = root.join(relative);
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|_| err("BULK_COPY_RECEIPT_CREATE_FAILED"))?;
    file.write_all(json.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.flush())
        .and_then(|_| file.sync_all())
        .map_err(|_| err("BULK_COPY_RECEIPT_WRITE_FAILED"))
}

fn method_key(method: &MethodIdentity) -> String {
    format!(
        "{}|{}|{}|{}",
        method.full_authority_sha256, method.mvid, method.type_def, method.method_def
    )
}

fn expected_active_identity_sets(
    base_active: &ActiveReceipt,
    manifest: &Manifest,
    catalog: &CampaignCatalog,
    activated: &[String],
) -> Result<(BTreeSet<String>, BTreeSet<String>), ValidationError> {
    let mut ids = BTreeSet::new();
    let mut methods = BTreeSet::new();
    for file in &base_active.files {
        ids.insert(file.id.clone());
        for method in &file.methods {
            methods.insert(method_key(method));
        }
    }
    for descriptor in catalog
        .records
        .iter()
        .filter(|record| activated.contains(&record.cohort))
    {
        if let Some(existing) = &descriptor.existing_record_id {
            if existing != "P" {
                return Err(err("BULK_EXISTING_RECORD_INVALID"));
            }
            let Some(record) = manifest.records.iter().find(|record| record.id == "P") else {
                return Err(err("BULK_P_RECORD_MISSING"));
            };
            if descriptor.source_path != record.original_path
                || descriptor.source_size != record.original_size
                || descriptor.source_sha256 != record.original_sha256
                || descriptor.destination_path
                    != record.proposed_path.as_deref().unwrap_or_default()
            {
                return Err(err("BULK_P_DESCRIPTOR_MISMATCH"));
            }
            ids.insert(record.id.clone());
            for method in &record.methods.current {
                methods.insert(method_key(&method.symbol_key));
            }
        } else {
            let record = build_record(descriptor)?;
            ids.insert(record.id.clone());
            for method in &record.methods.current {
                methods.insert(method_key(&method.symbol_key));
            }
        }
    }
    Ok((ids, methods))
}

/// Production entry: load only the pinned catalog from the curated root,
/// validate the current v5/v6 context and every active imported file, copy the
/// complete cohort, then append the activation suffix. No arbitrary catalog is
/// accepted and no mutation occurs before copy/readback success.
pub fn apply_approved_cohort(
    source_root: &Path,
    curated_root: &Path,
    cohort: &str,
) -> Result<ApplyReceipt, ApplyFailure> {
    let fail = |code: &str| ApplyFailure {
        code: code.to_owned(),
        copy: None,
        copy_failure: None,
        index_touched: false,
    };
    let catalog = load_campaign_catalog(curated_root).map_err(|error| ApplyFailure {
        code: error.code,
        copy: None,
        copy_failure: None,
        index_touched: false,
    })?;
    let context = super::load_context_at(curated_root).map_err(|error| ApplyFailure {
        code: error.issue.code,
        copy: None,
        copy_failure: None,
        index_touched: false,
    })?;
    let (manifest, active) = (context.manifest, context.active);
    let (_, _base_manifest, base_active) =
        super::load_context_baseline(curated_root).map_err(|error| ApplyFailure {
            code: error.issue.code,
            copy: None,
            copy_failure: None,
            index_touched: false,
        })?;
    let Some(base_active) = base_active else {
        return Err(fail("BULK_APPLY_REQUIRES_V5"));
    };
    if base_active.version != 5 {
        return Err(fail("BULK_APPLY_REQUIRES_V5"));
    }
    let v5_bytes =
        read_relative(curated_root, super::RECEIPT_V5_REL).map_err(|error| fail(error.code))?;
    if sha256_hex(&v5_bytes) != BASELINE_V5_RECEIPT_SHA256 {
        return Err(fail("BULK_APPLY_V5_ANCHOR_MISMATCH"));
    }
    let index_state =
        inspect_relative_inner(curated_root, INDEX_REL).map_err(|error| fail(error.code))?;
    let index_before = match index_state {
        super::SafePathState::Missing => None,
        super::SafePathState::File => {
            Some(read_relative(curated_root, INDEX_REL).map_err(|error| fail(error.code))?)
        }
        super::SafePathState::Directory => return Err(fail("BULK_INDEX_NOT_REGULAR")),
    };
    let activated = if let Some(bytes) = &index_before {
        parse_index_bytes(bytes)
            .map_err(|error| ApplyFailure {
                code: error.code,
                copy: None,
                copy_failure: None,
                index_touched: false,
            })?
            .activations
            .into_iter()
            .map(|activation| activation.cohort)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let expected_cohort = match activated.len() {
        0 => GAMEPLAY_COHORT,
        1 if activated[0] == GAMEPLAY_COHORT => PERFORMANCE_COHORT,
        1 => return Err(fail("BULK_APPLY_ACTIVATION_ORDER_INVALID")),
        _ => return Err(fail("BULK_APPLY_ALREADY_COMPLETE")),
    };
    if cohort != expected_cohort {
        return Err(fail("BULK_APPLY_COHORT_ORDER_MISMATCH"));
    }
    let expected_version = if index_before.is_some() { 6 } else { 5 };
    let Some(active) = active else {
        return Err(fail("BULK_APPLY_REQUIRES_V5"));
    };
    if active.version != expected_version || (index_before.is_none() && active.version != 5) {
        return Err(fail("BULK_APPLY_ACTIVE_VERSION_MISMATCH"));
    }
    let (expected_ids, expected_methods) =
        expected_active_identity_sets(&base_active, &manifest, &catalog, &activated).map_err(
            |error| ApplyFailure {
                code: error.code,
                copy: None,
                copy_failure: None,
                index_touched: false,
            },
        )?;
    let mut actual_ids = BTreeSet::new();
    let mut actual_methods = BTreeSet::new();
    for file in &active.files {
        if !actual_ids.insert(file.id.clone()) {
            return Err(fail("BULK_APPLY_DUPLICATE_ACTIVE_FILE"));
        }
        for method in &file.methods {
            if !actual_methods.insert(method_key(method)) {
                return Err(fail("BULK_APPLY_DUPLICATE_ACTIVE_METHOD"));
            }
        }
    }
    if actual_ids != expected_ids || actual_methods != expected_methods {
        return Err(fail("BULK_APPLY_ACTIVE_IDENTITY_MISMATCH"));
    }
    for file in &active.files {
        super::check_imported_file_view(curated_root, file)
            .map_err(|_| fail("BULK_APPLY_ACTIVE_IMPORTED_FILE_INVALID"))?;
    }

    let copy =
        copy_cohort_unchecked(source_root, curated_root, &catalog, cohort).map_err(|copy| {
            ApplyFailure {
                code: "BULK_APPLY_COPY_FAILED".to_owned(),
                copy: None,
                copy_failure: Some(copy),
                index_touched: false,
            }
        })?;
    let before_sha = index_before.as_deref().map(sha256_hex);
    let activation = append_activation_disk(curated_root, cohort, index_before.as_deref())
        .map_err(|error| ApplyFailure {
            code: error.code,
            copy: Some(copy.clone()),
            copy_failure: None,
            index_touched: true,
        })?;
    let after = read_relative(curated_root, INDEX_REL).map_err(|error| ApplyFailure {
        code: error.code.to_owned(),
        copy: Some(copy.clone()),
        copy_failure: None,
        index_touched: true,
    })?;
    if index_before
        .as_deref()
        .is_some_and(|before| !after.starts_with(before))
    {
        return Err(ApplyFailure {
            code: "BULK_INDEX_PREFIX_MISMATCH".to_owned(),
            copy: Some(copy),
            copy_failure: None,
            index_touched: true,
        });
    }
    Ok(ApplyReceipt {
        copy,
        index_before_sha256: before_sha,
        index_after_sha256: sha256_hex(&after),
        activation,
    })
}

#[cfg(test)]
mod unit_contracts {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn partial_copy_preserves_first_and_failed_attempt_without_activation() {
        let source_root = crate::curated_root()
            .parent()
            .expect("research root")
            .to_path_buf();
        let catalog = load_campaign_catalog(&crate::curated_root()).expect("catalog");
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = crate::curated_root()
            .join("target")
            .join("bulk-unit-fixtures")
            .join(format!("{}-{n}", std::process::id()));
        std::fs::create_dir_all(&root).expect("unit fixture");
        let mut calls = 0_u32;
        let mut fault = |_relative: &str, file: &mut std::fs::File| {
            calls += 1;
            if calls == 2 {
                file.write_all(b"partial").expect("write partial");
                return Err(());
            }
            Ok(())
        };
        let failure = copy_cohort_with_hook(
            &source_root,
            &root,
            &catalog,
            GAMEPLAY_COHORT,
            Some(&mut fault),
        )
        .expect_err("injected second write fault");
        assert_eq!(failure.code, "BULK_COPY_INJECTED_FAULT");
        assert_eq!(failure.completed.len(), 1);
        let failed = failure.failed_attempt.as_ref().expect("failed attempt");
        assert!(failed.created);
        assert!(failed.partial);
        assert_eq!(failed.stage, "write");
        assert_eq!(failed.observed_size, Some(7));
        assert!(root.join(&failed.destination_path).is_file());
        assert!(!root.join(INDEX_REL).exists());
        let receipt = CopyReceipt {
            campaign: CAMPAIGN_ID.to_owned(),
            cohort: GAMEPLAY_COHORT.to_owned(),
            files: failure.completed.clone(),
            complete: false,
            failed_attempt: failure.failed_attempt.clone(),
        };
        write_copy_receipt(&root, "partial-copy.receipt.json", &receipt).expect("receipt");
        let read = std::fs::read(root.join("partial-copy.receipt.json")).expect("receipt bytes");
        let value: Value = serde_json::from_slice(&read).expect("receipt json");
        assert_eq!(value["files"].as_array().expect("completed").len(), 1);
        assert_eq!(value["failed_attempt"]["created"], true);
        assert_eq!(value["failed_attempt"]["stage"], "write");
    }
}
