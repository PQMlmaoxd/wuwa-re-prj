use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub mod bulk;
pub mod provenance;
pub mod registry;
pub const MANIFEST_REL: &str = "metadata/migration.json";
pub const RECEIPT_REL: &str = "metadata/imports.v1.json";
pub const RECEIPT_V2_REL: &str = "metadata/imports.v2.json";
pub const RECEIPT_V3_REL: &str = "metadata/imports.v3.json";
pub const RECEIPT_V4_REL: &str = "metadata/imports.v4.json";
pub const RECEIPT_V5_REL: &str = "metadata/imports.v5.json";
pub const PERFORMANCE_SOURCE_REL: &str = "metadata/performance-source.v1.json";
pub const SCHEDULING_SOURCE_REL: &str = "metadata/scheduling-source.v1.json";
pub const TASK_EXECUTION_SOURCE_REL: &str = "metadata/task-execution-source.v1.json";
pub const BASELINE_MANIFEST_SHA256: &str =
    "7da6c8c7b2d95b8a488adaba7ef46d5775d434b75dfdbee55b5e8e9613a2899b";
pub const BASELINE_RECEIPT_V1_SHA256: &str =
    "de837114c5105ff9307caa3c02bc2e6ed776605cfd7abbe8a27aa895b5495c07";
pub const BASELINE_RECEIPT_V2_SHA256: &str =
    "24f3b419274648fbbc22a064623d7ff18953ca4e3ad3ab15a5ac243d520086ce";
pub const BASELINE_RECEIPT_V3_SHA256: &str =
    "22898a66f260d88b6f6295149fbf23fb47be73ac5c91e13418f615248f5421c9";
pub const BASELINE_RECEIPT_V4_SHA256: &str =
    "b11d0d405f8dc49c3f9e93c6f91626f6062d8e2c97563b1d9ca3199f82a2076d";
pub const PERFORMANCE_SOURCE_V1_SHA256: &str =
    "bd8b2ef69be52e2c4f18fb9080e049d1b2e9da6bc377777b43e3c984cc6aa040";
pub const SCHEDULING_SOURCE_V1_SHA256: &str =
    "2d6ab3fbff603761c82dfcadeb2c32e37e82d1e85523de21e12bf157edab7c54";
pub const TASK_EXECUTION_SOURCE_V1_SHA256: &str =
    "46e312246a5d5d870da293bb95c500e2c03e8d4960ed6cfb29d952a193bd509e";
pub const MANAGED_AUTHORITY_ID: &str = "MANAGED-CS-E3FC67839851";
pub const MANAGED_AUTHORITY_SHA256: &str =
    "e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db";
pub const MANAGED_MVID: &str = "2d6d37cb-76e5-4075-8220-3eeca41471e3";
pub const BUILD_AUTHORITY_ID: &str = "BLD-WUWA-0001";
pub const REQUIRED_IMPORT_IDS: [&str; 3] = ["E13", "E17", "E18"];
pub const REQUIRED_V2_IMPORT_IDS: [&str; 5] = ["D", "E13", "E17", "E18", "T"];
pub const REQUIRED_V3_IMPORT_IDS: [&str; 7] =
    ["D", "E13", "E17", "E18", "PTIMER", "PTIMER_INSTANCE", "T"];
pub const REQUIRED_V4_IMPORT_IDS: [&str; 10] = [
    "D",
    "E13",
    "E17",
    "E18",
    "PSCHED_GRAPH",
    "PSCHED_HEAP",
    "PSCHED_UTILS",
    "PTIMER",
    "PTIMER_INSTANCE",
    "T",
];
pub const REQUIRED_V5_IMPORT_IDS: [&str; 13] = [
    "D",
    "E13",
    "E17",
    "E18",
    "PEXEC_BASE",
    "PEXEC_QUEUE",
    "PEXEC_SYSTEM",
    "PSCHED_GRAPH",
    "PSCHED_HEAP",
    "PSCHED_UTILS",
    "PTIMER",
    "PTIMER_INSTANCE",
    "T",
];
const PERFORMANCE_SOURCE_IDS: [&str; 2] = ["PTIMER", "PTIMER_INSTANCE"];
const SCHEDULING_SOURCE_IDS: [&str; 3] = ["PSCHED_GRAPH", "PSCHED_UTILS", "PSCHED_HEAP"];
const V3_QUALIFIED_METHOD_COUNT: usize = 18;
const V4_QUALIFIED_METHOD_COUNT: usize = 51;
const V4_NEW_FULL_METHOD_COUNT: usize = 12;
const V4_NEW_SOURCE_ONLY_METHOD_COUNT: usize = 21;
const TASK_EXECUTION_SOURCE_IDS: [&str; 3] = ["PEXEC_SYSTEM", "PEXEC_BASE", "PEXEC_QUEUE"];
const V5_QUALIFIED_METHOD_COUNT: usize = 102;
const V5_NEW_FULL_METHOD_COUNT: usize = 23;
const V5_NEW_SOURCE_ONLY_METHOD_COUNT: usize = 27;
const V5_NEW_METADATA_ONLY_METHOD_COUNT: usize = 1;
const V2_D_METHOD_DEFS: [&str; 8] = [
    "0x06018865",
    "0x06018867",
    "0x06018868",
    "0x06018869",
    "0x0601886A",
    "0x0601886B",
    "0x0601886C",
    "0x06018873",
];
const V2_T_METHOD_DEFS: [&str; 1] = ["0x0601886E"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub path: Option<String>,
    pub record_id: Option<String>,
}

impl ValidationError {
    fn new(code: &str) -> Self {
        Self {
            code: code.to_owned(),
            path: None,
            record_id: None,
        }
    }

    fn at(code: &str, path: &str) -> Self {
        Self {
            code: code.to_owned(),
            path: Some(safe_path_label(path)),
            record_id: None,
        }
    }

    fn record(code: &str, id: &str) -> Self {
        Self {
            code: code.to_owned(),
            path: None,
            record_id: Some(id.to_owned()),
        }
    }

    fn with_record(mut self, id: &str) -> Self {
        self.record_id = Some(id.to_owned());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BinaryAuthority {
    pub authority_id: String,
    pub authority_binary_sha256: Option<String>,
    pub mvid: Option<String>,
    pub kind: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MethodIdentity {
    pub full_authority_sha256: String,
    pub mvid: String,
    #[serde(rename = "typeDef")]
    pub type_def: String,
    #[serde(rename = "methodDef")]
    pub method_def: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceArtifact {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFileView {
    pub id: String,
    pub original_path: String,
    pub imported: SourceArtifact,
    pub methods: Vec<MethodIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveReceipt {
    pub ledger_path: String,
    pub version: u64,
    pub files: Vec<ImportedFileView>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum EvidenceStatus {
    #[serde(rename = "SOURCE_ONLY")]
    SourceOnly,
    #[serde(rename = "DIRECT_CIL_PARTIAL")]
    DirectCilPartial,
    #[serde(rename = "DIRECT_CIL_FULL")]
    DirectCilFull,
    #[serde(rename = "METADATA_ONLY")]
    MetadataOnly,
    #[serde(rename = "BLOCKED")]
    Blocked,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl EvidenceStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::SourceOnly => "SOURCE_ONLY",
            Self::DirectCilPartial => "DIRECT_CIL_PARTIAL",
            Self::DirectCilFull => "DIRECT_CIL_FULL",
            Self::MetadataOnly => "METADATA_ONLY",
            Self::Blocked => "BLOCKED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum SourceStatus {
    #[serde(rename = "RECONSTRUCTED_FROM_DIRECT_CIL")]
    ReconstructedFromDirectCil,
    #[serde(rename = "SOURCE_ONLY")]
    SourceOnly,
    #[serde(rename = "GENERATED_DIAGNOSTIC")]
    GeneratedDiagnostic,
    #[serde(rename = "METADATA_ONLY")]
    MetadataOnly,
    #[serde(rename = "INFERRED_CONDITIONAL")]
    InferredConditional,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl SourceStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::ReconstructedFromDirectCil => "RECONSTRUCTED_FROM_DIRECT_CIL",
            Self::SourceOnly => "SOURCE_ONLY",
            Self::GeneratedDiagnostic => "GENERATED_DIAGNOSTIC",
            Self::MetadataOnly => "METADATA_ONLY",
            Self::InferredConditional => "INFERRED_CONDITIONAL",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum CurationStatus {
    #[serde(rename = "CURATED_READY")]
    CuratedReady,
    #[serde(rename = "NEEDS_EDITORIAL_REVIEW")]
    NeedsEditorialReview,
    #[serde(rename = "NEEDS_PROVENANCE_REVIEW")]
    NeedsProvenanceReview,
    #[serde(rename = "PARTIAL_OR_SOURCE_ONLY")]
    PartialOrSourceOnly,
    #[serde(rename = "HISTORICAL_ONLY")]
    HistoricalOnly,
    #[serde(rename = "INTERNAL_ONLY")]
    InternalOnly,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl CurationStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::CuratedReady => "CURATED_READY",
            Self::NeedsEditorialReview => "NEEDS_EDITORIAL_REVIEW",
            Self::NeedsProvenanceReview => "NEEDS_PROVENANCE_REVIEW",
            Self::PartialOrSourceOnly => "PARTIAL_OR_SOURCE_ONLY",
            Self::HistoricalOnly => "HISTORICAL_ONLY",
            Self::InternalOnly => "INTERNAL_ONLY",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ExportStatus {
    #[serde(rename = "INTERNAL_ONLY")]
    InternalOnly,
    #[serde(rename = "HANDOFF_READY")]
    HandoffReady,
    #[serde(rename = "PUBLICATION_REVIEW_REQUIRED")]
    PublicationReviewRequired,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl ExportStatus {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::InternalOnly => "INTERNAL_ONLY",
            Self::HandoffReady => "HANDOFF_READY",
            Self::PublicationReviewRequired => "PUBLICATION_REVIEW_REQUIRED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum VerificationResult {
    #[serde(rename = "PASS")]
    Pass,
    #[serde(rename = "FAIL")]
    Fail,
    #[serde(rename = "NOT_CHECKED")]
    NotChecked,
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
}

impl VerificationResult {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::NotChecked => "NOT_CHECKED",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportStatus {
    Imported,
    NotImported,
    NotApplicable,
}

impl ImportStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Imported => "IMPORTED",
            Self::NotImported => "NOT_IMPORTED",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub manifest_version: u64,
    pub description: String,
    pub foundation_status: FoundationStatus,
    pub authority_policy: AuthorityPolicy,
    pub symbol_policy: SymbolPolicy,
    pub non_managed_identity_policy: NonManagedIdentityPolicy,
    pub architecture_identity: ArchitectureIdentity,
    pub schema_contract: SchemaContract,
    pub allowed_curation_statuses: Vec<String>,
    pub allowed_export_statuses: Vec<String>,
    pub benchmark: Benchmark,
    pub preservation: Preservation,
    pub records: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoundationStatus {
    pub stage: String,
    pub writer_checkpoint: bool,
    pub primary_final_receipt: String,
    pub publication: String,
    pub runtime: String,
    pub game_version: String,
    pub curated_ready_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityPolicy {
    pub managed: ManagedAuthority,
    pub build: BuildAuthority,
    pub join_rule: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedAuthority {
    pub authority_id: String,
    pub authority_sha256_kind: String,
    pub authority_binary_sha256: String,
    pub mvid: String,
    pub assembly_version_label: String,
    pub game_version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildAuthority {
    pub authority_id: String,
    pub authority_sha256_kind: String,
    pub local_version: String,
    pub historical_client_sha256: String,
    pub managed_build_claim: bool,
    pub association_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SymbolPolicy {
    pub managed_key: Vec<String>,
    pub filename_or_display_name_is_key: bool,
    pub method_is_migration_unit: bool,
    pub whole_class_coverage_by_method_overlap: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NonManagedIdentityPolicy {
    pub identity: String,
    pub method_tokens: Option<Value>,
    pub binary_authority: String,
    pub cross_join: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureIdentity {
    pub managed_build_alias: String,
    pub namespace_policy: String,
    pub type_method_identity: String,
    pub compile_proof: String,
    pub model_proof: String,
    pub binary_equivalence: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaContract {
    pub version: String,
    pub required_record_fields: Vec<String>,
    pub field_types: BTreeMap<String, String>,
    pub nullable_fields: Vec<String>,
    pub allowed_evidence_statuses: Vec<String>,
    pub allowed_source_statuses: Vec<String>,
    pub orthogonality: String,
    pub path_rule: String,
    pub current_method_required_fields: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    pub document: String,
    pub status: String,
    pub retrieval_date: String,
    pub pinned_branch_count: u64,
    pub root_html_count: u64,
    pub commit_feed_count: u64,
    pub targeted_fetch_count: u64,
    pub cited_url_count: u64,
    pub github_rest_result: String,
    pub dolphin_filename_correction: String,
    pub faulting_upstream_commands_run: bool,
    pub status_legend: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preservation {
    pub scope: String,
    pub prior_21_file_snapshot_sha256: String,
    pub added_after_snapshot: String,
    pub accepted_closure: AcceptedClosure,
    pub final_verification: String,
    pub whole_repository_claim: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptedClosure {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: String,
    pub unit_id: Option<String>,
    pub namespace_context: Option<String>,
    pub original_path: String,
    pub original_size: u64,
    pub original_sha256: String,
    pub hash_basis: String,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
    pub verification_basis: String,
    pub evidence_references: Vec<EvidenceReference>,
    pub proposed_path: Option<String>,
    pub proposed_path_reason: String,
    pub authority_id: String,
    pub authority_binary_sha256: Option<String>,
    pub authority_sha256_kind: String,
    pub authority_mvid: Option<String>,
    pub build_reference: Option<BuildReference>,
    #[serde(rename = "typeDef")]
    pub type_def: Option<String>,
    pub methods: Methods,
    pub scope: String,
    pub completeness: String,
    pub representation: String,
    pub curation_status: CurationStatus,
    pub export_status: ExportStatus,
    pub duplicate_group: Option<String>,
    pub relationship: String,
    pub supersession: Supersession,
    pub explanation: String,
    pub dependencies: Vec<String>,
    pub editorial_changes: Vec<String>,
    pub validation: RecordValidation,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildReference {
    pub build_id: String,
    pub local_version: String,
    pub historical_client_sha256: String,
    pub association_status: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Methods {
    pub historical: Vec<HistoricalMethod>,
    pub current: Vec<CurrentMethod>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoricalMethod {
    pub symbol_key: MethodIdentity,
    pub name: Option<String>,
    pub coverage: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CurrentMethod {
    #[serde(rename = "methodDef")]
    pub method_def: String,
    pub coverage: String,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
    pub verification_basis: String,
    pub symbol_key: MethodIdentity,
    pub rva: Option<String>,
    pub il_range: Option<String>,
    pub evidence_references: Vec<EvidenceReference>,
    #[serde(skip, default)]
    pub source_provenance: Option<provenance::SourceProvenance>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub path: String,
    pub locator: String,
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supersession {
    pub same_authority: bool,
    pub qualified: bool,
    pub scope: Vec<String>,
    pub supersedes: Vec<String>,
    pub superseded_by: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordValidation {
    pub source_hash: String,
    pub authority_identity: String,
    pub path_collision: String,
    pub equivalence: String,
    pub compilation: String,
    pub runtime: String,
    pub publication: String,
    pub final_receipt: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportReceipt {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub imports: Vec<ImportEntry>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportEntry {
    pub id: String,
    pub authority_id: String,
    pub method_identity: MethodIdentity,
    pub original_path: String,
    pub imported_path: String,
    pub original_size: u64,
    pub imported_size: u64,
    pub original_sha256: String,
    pub imported_sha256: String,
    pub exact_byte_comparison: VerificationResult,
    pub implementation_scope: String,
    pub source_completeness: String,
    pub unresolved_dependencies: Vec<String>,
    pub editorial_status: ReceiptEditorialStatus,
    pub compilation_status: ReceiptCompilationStatus,
    pub equivalence_status: ReceiptEquivalenceStatus,
    pub publication_status: ReceiptPublicationStatus,
    pub runtime_status: ReceiptRuntimeStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportedMethod {
    pub method_identity: MethodIdentity,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MappedImportEntry {
    pub id: String,
    pub authority_id: String,
    pub methods: Vec<ImportedMethod>,
    pub original_path: String,
    pub imported_path: String,
    pub original_size: u64,
    pub imported_size: u64,
    pub original_sha256: String,
    pub imported_sha256: String,
    pub exact_byte_comparison: VerificationResult,
    pub implementation_scope: String,
    pub source_completeness: String,
    pub unresolved_dependencies: Vec<String>,
    pub evidence_status: EvidenceStatus,
    pub source_status: SourceStatus,
    pub editorial_status: ReceiptEditorialStatus,
    pub compilation_status: ReceiptCompilationStatus,
    pub equivalence_status: ReceiptEquivalenceStatus,
    pub publication_status: ReceiptPublicationStatus,
    pub runtime_status: ReceiptRuntimeStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ImportEntryV2 {
    Event(ImportEntry),
    Mapped(MappedImportEntry),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportReceiptV2 {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub previous_receipt_sha256: String,
    pub imports: Vec<ImportEntryV2>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerformanceSourceCatalog {
    pub schema: String,
    pub version: u64,
    pub base_manifest_sha256: String,
    pub records: Vec<Record>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportReceiptV3 {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub previous_receipt_sha256: String,
    pub performance_source_sha256: String,
    pub imports: Vec<ImportEntryV2>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskExecutionSourceCatalog {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub performance_source_sha256: String,
    pub scheduling_source_sha256: String,
    pub records: Vec<SchedulingSourceRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulingSourceCatalog {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub performance_source_sha256: String,
    pub records: Vec<SchedulingSourceRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulingSourceRecord {
    pub record: Record,
    pub associated_types: Vec<AssociatedType>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssociatedType {
    #[serde(rename = "typeDef")]
    pub type_def: String,
    #[serde(rename = "parent_typeDef")]
    pub parent_type_def: String,
    pub evidence_references: Vec<EvidenceReference>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportReceiptV4 {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub previous_receipt_sha256: String,
    pub performance_source_sha256: String,
    pub scheduling_source_sha256: String,
    pub imports: Vec<ImportEntryV2>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportReceiptV5 {
    pub schema: String,
    pub version: u64,
    pub manifest_sha256: String,
    pub previous_receipt_sha256: String,
    pub performance_source_sha256: String,
    pub scheduling_source_sha256: String,
    pub task_execution_source_sha256: String,
    pub imports: Vec<ImportEntryV2>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ReceiptEditorialStatus {
    #[serde(rename = "NEEDS_EDITORIAL_REVIEW")]
    NeedsEditorialReview,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ReceiptCompilationStatus {
    #[serde(rename = "NOT_RUN")]
    NotRun,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ReceiptEquivalenceStatus {
    #[serde(rename = "NOT_ESTABLISHED")]
    NotEstablished,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ReceiptPublicationStatus {
    #[serde(rename = "DISABLED")]
    Disabled,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub enum ReceiptRuntimeStatus {
    #[serde(rename = "PARKED")]
    Parked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafePathState {
    Missing,
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FsFailure {
    code: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErrorKind {
    Schema,
    Execution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AppError {
    kind: ErrorKind,
    issue: ValidationError,
}

impl AppError {
    fn schema(code: &str) -> Self {
        Self {
            kind: ErrorKind::Schema,
            issue: ValidationError::new(code),
        }
    }

    fn execution(code: &str) -> Self {
        Self {
            kind: ErrorKind::Execution,
            issue: ValidationError::new(code),
        }
    }

    fn execution_at(code: &str, path: &str) -> Self {
        Self {
            kind: ErrorKind::Execution,
            issue: ValidationError::at(code, path),
        }
    }

    fn from_validation(error: ValidationError) -> Self {
        Self {
            kind: ErrorKind::Schema,
            issue: error,
        }
    }
}

struct StrictValue(Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}

struct StrictValueVisitor;

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON value")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::Bool(value)))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::Number(value.into())))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::Number(value.into())))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !value.is_finite() {
            return Err(E::custom("__RE_NONFINITE_NUMBER__"));
        }
        let number = serde_json::Number::from_f64(value)
            .ok_or_else(|| E::custom("__RE_NONFINITE_NUMBER__"))?;
        Ok(StrictValue(Value::Number(number)))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::String(value.to_owned())))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::String(value)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::Null))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(StrictValue(Value::Null))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<StrictValue>()? {
            values.push(value.0);
        }
        Ok(StrictValue(Value::Array(values)))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("__RE_DUPLICATE_JSON_KEY__"));
            }
            let value = map.next_value::<StrictValue>()?;
            values.insert(key, value.0);
        }
        Ok(StrictValue(Value::Object(values)))
    }
}

pub fn parse_strict_value(bytes: &[u8]) -> Result<Value, ValidationError> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValue::deserialize(&mut deserializer).map_err(json_parse_error)?;
    deserializer.end().map_err(json_parse_error)?;
    Ok(value.0)
}

fn json_parse_error(error: serde_json::Error) -> ValidationError {
    let message = error.to_string();
    if message.contains("__RE_DUPLICATE_JSON_KEY__") {
        ValidationError::new("DUPLICATE_JSON_KEY")
    } else if message.contains("__RE_NONFINITE_NUMBER__") {
        ValidationError::new("NONFINITE_JSON_NUMBER")
    } else {
        ValidationError::new("MALFORMED_JSON")
    }
}

pub fn parse_manifest_bytes(bytes: &[u8]) -> Result<Manifest, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let manifest: Manifest =
        serde_json::from_value(value).map_err(|_| ValidationError::new("SCHEMA_INVALID"))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn parse_receipt_bytes(
    bytes: &[u8],
    manifest: &Manifest,
) -> Result<ImportReceipt, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let receipt: ImportReceipt = serde_json::from_value(value)
        .map_err(|_| ValidationError::new("RECEIPT_SCHEMA_INVALID"))?;
    validate_receipt(&receipt, manifest)?;
    Ok(receipt)
}

pub fn parse_receipt_v2_bytes(
    bytes: &[u8],
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
) -> Result<ImportReceiptV2, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let receipt: ImportReceiptV2 =
        serde_json::from_value(value).map_err(|_| ValidationError::new("V2_SCHEMA_INVALID"))?;
    validate_receipt_v2(&receipt, manifest, historical_v1)?;
    Ok(receipt)
}

pub fn parse_performance_source_bytes(
    bytes: &[u8],
    baseline: &Manifest,
) -> Result<PerformanceSourceCatalog, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let catalog: PerformanceSourceCatalog = serde_json::from_value(value)
        .map_err(|_| ValidationError::new("CATALOG_SCHEMA_INVALID"))?;
    validate_performance_source(&catalog, baseline)?;
    Ok(catalog)
}

pub fn parse_receipt_v3_bytes(
    bytes: &[u8],
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    catalog: &PerformanceSourceCatalog,
) -> Result<ImportReceiptV3, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let receipt: ImportReceiptV3 =
        serde_json::from_value(value).map_err(|_| ValidationError::new("V3_SCHEMA_INVALID"))?;
    validate_receipt_v3(&receipt, manifest, historical_v1, historical_v2, catalog)?;
    Ok(receipt)
}

pub fn parse_scheduling_source_bytes(
    bytes: &[u8],
    baseline: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
) -> Result<SchedulingSourceCatalog, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let catalog: SchedulingSourceCatalog = serde_json::from_value(value)
        .map_err(|_| ValidationError::new("SCHEDULING_CATALOG_SCHEMA_INVALID"))?;
    validate_scheduling_source(&catalog, baseline, performance_catalog)?;
    Ok(catalog)
}

pub fn parse_receipt_v4_bytes(
    bytes: &[u8],
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    historical_v3: &ImportReceiptV3,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
) -> Result<ImportReceiptV4, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let receipt: ImportReceiptV4 =
        serde_json::from_value(value).map_err(|_| ValidationError::new("V4_SCHEMA_INVALID"))?;
    validate_receipt_v4(
        &receipt,
        manifest,
        historical_v1,
        historical_v2,
        historical_v3,
        performance_catalog,
        scheduling_catalog,
    )?;
    Ok(receipt)
}

pub fn parse_task_execution_source_bytes(
    bytes: &[u8],
    manifest: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
) -> Result<TaskExecutionSourceCatalog, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let catalog: TaskExecutionSourceCatalog = serde_json::from_value(value)
        .map_err(|_| ValidationError::new("TASK_CATALOG_SCHEMA_INVALID"))?;
    validate_task_execution_source(&catalog, manifest, performance_catalog, scheduling_catalog)?;
    Ok(catalog)
}

pub fn parse_receipt_v5_bytes(
    bytes: &[u8],
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    historical_v3: &ImportReceiptV3,
    historical_v4: &ImportReceiptV4,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
    task_catalog: &TaskExecutionSourceCatalog,
) -> Result<ImportReceiptV5, ValidationError> {
    let value = parse_strict_value(bytes)?;
    let receipt: ImportReceiptV5 =
        serde_json::from_value(value).map_err(|_| ValidationError::new("V5_SCHEMA_INVALID"))?;
    validate_receipt_v5(
        &receipt,
        manifest,
        historical_v1,
        historical_v2,
        historical_v3,
        historical_v4,
        performance_catalog,
        scheduling_catalog,
        task_catalog,
    )?;
    Ok(receipt)
}

pub fn curated_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| manifest_dir.to_path_buf())
}

fn absolute_path_from_current_dir(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .map(|current| current.join(&path))
            .unwrap_or(path)
    }
}

fn nearest_metadata_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join("metadata").join("migration.json").is_file())
        .map(Path::to_path_buf)
}

fn cli_curated_root(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return absolute_path_from_current_dir(path);
    }
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    nearest_metadata_root(&current).unwrap_or(current)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn safe_path_label(path: &str) -> String {
    if validate_relative_path_inner(path).is_ok() {
        path.to_owned()
    } else {
        "<invalid-relative-path>".to_owned()
    }
}

fn validate_relative_path_inner(path: &str) -> Result<(), ()> {
    if path.is_empty() || !path.is_ascii() || path.starts_with('/') || path.ends_with('/') {
        return Err(());
    }
    if path.contains('\\') || path.contains(':') {
        return Err(());
    }
    for component in path.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            return Err(());
        }
        if component.ends_with('.') || component.ends_with(' ') {
            return Err(());
        }
        if component
            .chars()
            .any(|character| character.is_ascii_control() || "<>\"|?*".contains(character))
        {
            return Err(());
        }
        let stem = component.split('.').next().unwrap_or(component);
        let upper = stem.to_ascii_uppercase();
        if matches!(
            upper.as_str(),
            "CON"
                | "PRN"
                | "AUX"
                | "NUL"
                | "COM1"
                | "COM2"
                | "COM3"
                | "COM4"
                | "COM5"
                | "COM6"
                | "COM7"
                | "COM8"
                | "COM9"
                | "LPT1"
                | "LPT2"
                | "LPT3"
                | "LPT4"
                | "LPT5"
                | "LPT6"
                | "LPT7"
                | "LPT8"
                | "LPT9"
                | "COM¹"
                | "COM²"
                | "COM³"
                | "LPT¹"
                | "LPT²"
                | "LPT³"
        ) {
            return Err(());
        }
    }
    Ok(())
}

pub fn validate_relative_path(path: &str) -> Result<(), ValidationError> {
    validate_relative_path_inner(path).map_err(|_| ValidationError::new("UNSAFE_RELATIVE_PATH"))
}

fn absolute_path(path: &Path) -> Result<PathBuf, FsFailure> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        env::current_dir()
            .map(|current| current.join(path))
            .map_err(|_| FsFailure {
                code: "CURRENT_DIR_ERROR",
            })
    }
}

#[cfg(windows)]
fn metadata_is_unsafe(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_type().is_symlink() || metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn metadata_is_unsafe(metadata: &fs::Metadata) -> bool {
    metadata.file_type().is_symlink()
}

pub fn reparse_attributes_rejected(file_attributes: u32) -> bool {
    file_attributes & 0x400 != 0
}

fn check_chain(path: &Path) -> Result<SafePathState, FsFailure> {
    let mut ancestors: Vec<PathBuf> = path.ancestors().map(Path::to_path_buf).collect();
    ancestors.reverse();
    for ancestor in ancestors {
        let metadata = match fs::symlink_metadata(&ancestor) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(SafePathState::Missing);
            }
            Err(_) => {
                return Err(FsFailure {
                    code: "LSTAT_ERROR",
                });
            }
        };
        if metadata_is_unsafe(&metadata) {
            return Err(FsFailure {
                code: "UNSAFE_REPARSE_OR_SYMLINK",
            });
        }
        if ancestor != path && !metadata.is_dir() {
            return Err(FsFailure {
                code: "PATH_PARENT_NOT_DIRECTORY",
            });
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            FsFailure {
                code: "MISSING_PATH",
            }
        } else {
            FsFailure {
                code: "LSTAT_ERROR",
            }
        }
    })?;
    if metadata_is_unsafe(&metadata) {
        return Err(FsFailure {
            code: "UNSAFE_REPARSE_OR_SYMLINK",
        });
    }
    if metadata.is_dir() {
        Ok(SafePathState::Directory)
    } else if metadata.is_file() {
        Ok(SafePathState::File)
    } else {
        Err(FsFailure {
            code: "PATH_NOT_REGULAR",
        })
    }
}

fn inspect_relative_inner(root: &Path, relative: &str) -> Result<SafePathState, FsFailure> {
    validate_relative_path_inner(relative).map_err(|_| FsFailure {
        code: "UNSAFE_RELATIVE_PATH",
    })?;
    let absolute_root = absolute_path(root)?;
    let root_state = check_chain(&absolute_root)?;
    if root_state == SafePathState::Missing {
        return Ok(SafePathState::Missing);
    }
    if root_state != SafePathState::Directory {
        return Err(FsFailure {
            code: "RESEARCH_ROOT_NOT_DIRECTORY",
        });
    }
    check_chain(&absolute_root.join(relative))
}

pub fn inspect_relative(root: &Path, relative: &str) -> Result<SafePathState, ValidationError> {
    inspect_relative_inner(root, relative)
        .map_err(|failure| ValidationError::at(failure.code, &safe_path_label(relative)))
}

fn read_relative(root: &Path, relative: &str) -> Result<Vec<u8>, FsFailure> {
    match inspect_relative_inner(root, relative)? {
        SafePathState::File => {
            let absolute_root = absolute_path(root)?;
            fs::read(absolute_root.join(relative)).map_err(|_| FsFailure { code: "READ_ERROR" })
        }
        SafePathState::Missing => Err(FsFailure {
            code: "MISSING_PATH",
        }),
        SafePathState::Directory => Err(FsFailure {
            code: "PATH_NOT_REGULAR",
        }),
    }
}

fn fs_error(kind: ErrorKind, code: &'static str, path: &str) -> AppError {
    let issue = ValidationError::at(code, path);
    AppError { kind, issue }
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_uuid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 36
        && [8, 13, 18, 23]
            .iter()
            .all(|index| bytes.get(*index) == Some(&b'-'))
        && bytes.iter().enumerate().all(|(index, byte)| {
            [8, 13, 18, 23].contains(&index)
                || byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()
        })
}

fn valid_token(value: &str, prefix: &str) -> bool {
    value.len() == 10
        && value.starts_with(prefix)
        && value[4..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'A'..=b'F').contains(&byte))
        && value[4..].bytes().any(|byte| byte != b'0')
}

fn valid_authority_syntax(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes.len() <= 80
        && bytes[0].is_ascii_alphabetic()
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-')
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
}

pub fn validate_authority_syntax(value: &str) -> bool {
    valid_authority_syntax(value)
}

pub fn validate_method_token(value: &str) -> bool {
    valid_token(value, "0x06")
}

fn validate_type_token(value: &str) -> bool {
    valid_token(value, "0x02")
}

fn method_key(identity: &MethodIdentity) -> String {
    format!(
        "{}|{}|{}|{}",
        identity.full_authority_sha256, identity.mvid, identity.type_def, identity.method_def
    )
}

fn qualified_key_set(record: &Record) -> BTreeSet<String> {
    record
        .methods
        .historical
        .iter()
        .map(|method| method_key(&method.symbol_key))
        .chain(
            record
                .methods
                .current
                .iter()
                .map(|method| method_key(&method.symbol_key)),
        )
        .collect()
}

fn record_by_id<'a>(manifest: &'a Manifest, id: &str) -> Option<&'a Record> {
    manifest.records.iter().find(|record| record.id == id)
}

fn expected_evidence_statuses() -> Vec<String> {
    [
        "SOURCE_ONLY",
        "DIRECT_CIL_PARTIAL",
        "DIRECT_CIL_FULL",
        "METADATA_ONLY",
        "BLOCKED",
        "UNKNOWN",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn expected_source_statuses() -> Vec<String> {
    [
        "RECONSTRUCTED_FROM_DIRECT_CIL",
        "SOURCE_ONLY",
        "GENERATED_DIAGNOSTIC",
        "METADATA_ONLY",
        "INFERRED_CONDITIONAL",
        "UNKNOWN",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn expected_curation_statuses() -> Vec<String> {
    [
        "CURATED_READY",
        "NEEDS_EDITORIAL_REVIEW",
        "NEEDS_PROVENANCE_REVIEW",
        "PARTIAL_OR_SOURCE_ONLY",
        "HISTORICAL_ONLY",
        "INTERNAL_ONLY",
        "UNKNOWN",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn expected_export_statuses() -> Vec<String> {
    [
        "INTERNAL_ONLY",
        "HANDOFF_READY",
        "PUBLICATION_REVIEW_REQUIRED",
        "UNKNOWN",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn validate_evidence_reference(reference: &EvidenceReference) -> Result<(), ValidationError> {
    validate_relative_path(&reference.path)
        .map_err(|_| ValidationError::at("UNSAFE_MANIFEST_PATH", &reference.path))?;
    if reference.locator.is_empty() {
        return Err(ValidationError::new("EMPTY_EVIDENCE_LOCATOR"));
    }
    if let Some(hash) = &reference.artifact_sha256 {
        if !valid_lower_hex(hash, 64) {
            return Err(ValidationError::new("INVALID_EVIDENCE_SHA256"));
        }
    }
    Ok(())
}

fn validate_method_identity(
    record: &Record,
    identity: &MethodIdentity,
    method_def: Option<&str>,
) -> Result<(), ValidationError> {
    if record.authority_id != MANAGED_AUTHORITY_ID
        || identity.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
        || identity.mvid != MANAGED_MVID
    {
        return Err(ValidationError::record(
            "INCONSISTENT_MANAGED_SYMBOL",
            &record.id,
        ));
    }
    let type_def = record
        .type_def
        .as_deref()
        .ok_or_else(|| ValidationError::record("MISSING_MANAGED_TYPEDEF", &record.id))?;
    if identity.type_def != type_def || !validate_type_token(&identity.type_def) {
        return Err(ValidationError::record("INCONSISTENT_TYPEDEF", &record.id));
    }
    if !validate_method_token(&identity.method_def) {
        return Err(ValidationError::record("INVALID_METHODDEF", &record.id));
    }
    if method_def.is_some_and(|value| value != identity.method_def) {
        return Err(ValidationError::record("METHOD_KEY_MISMATCH", &record.id));
    }
    Ok(())
}

fn has_qualifying_load01_reference(method: &CurrentMethod) -> bool {
    let paths: BTreeSet<&str> = method
        .evidence_references
        .iter()
        .map(|reference| reference.path.as_str())
        .collect();
    paths.contains("docs/performance/PERF_LOAD01_GAMEMODE_PROMISE_GETTER.md")
        && paths.contains("evidence/performance/PERF_LOAD01_DIRECT_CIL.txt")
        && method.verification_basis.contains("LOAD01")
}

#[derive(Clone, Copy)]
struct ExpectedPerformanceMethod {
    method_def: &'static str,
    rva: &'static str,
    il_range: &'static str,
}

#[derive(Clone, Copy)]
struct ExpectedPerformanceRecord {
    type_def: &'static str,
    original_path: &'static str,
    original_size: u64,
    original_sha256: &'static str,
    proposed_path: &'static str,
    methods: &'static [ExpectedPerformanceMethod],
}

const PERF03_REPORT_PATH: &str = "docs/performance/PERF03_SCRIPT_RUNTIME_BRIDGE.md";
const PERF03_RAW_PATH: &str = "evidence/performance/PERF03_DIRECT_CIL.txt";
const PERF03_EVIDENCE_PATH: &str = "evidence/performance/PERF03_EVIDENCE.json";
const PERF03_REPORT_SHA256: &str =
    "f3d785ffe8a024fb4a09f47ad23e9a3fa469d9d3f97f27e6a2e438b8cf069fde";
const PERF03_RAW_SHA256: &str = "cd24738a2e66e11f13aaf0c3c0fca7d4323ea6f6821c8a27aba276a2dbc33127";
const PERF03_EVIDENCE_SHA256: &str =
    "3a5b97eaace2880d319e1e3941f333901654c624bacb31df03c30982676110ad";

const PTIMER_METHODS: [ExpectedPerformanceMethod; 2] = [
    ExpectedPerformanceMethod {
        method_def: "0x06003267",
        rva: "0x226FC",
        il_range: "[0x0000,0x0109)",
    },
    ExpectedPerformanceMethod {
        method_def: "0x06003269",
        rva: "0x22898",
        il_range: "[0x0000,0x0016)",
    },
];

const PTIMER_INSTANCE_METHODS: [ExpectedPerformanceMethod; 4] = [
    ExpectedPerformanceMethod {
        method_def: "0x06003271",
        rva: "0x22AB8",
        il_range: "[0x0000,0x0015)",
    },
    ExpectedPerformanceMethod {
        method_def: "0x0600327E",
        rva: "0x23140",
        il_range: "[0x0000,0x00D8)",
    },
    ExpectedPerformanceMethod {
        method_def: "0x0600326B",
        rva: "0x228C4",
        il_range: "[0x0000,0x0116)",
    },
    ExpectedPerformanceMethod {
        method_def: "0x0600327F",
        rva: "0x23224",
        il_range: "[0x0000,0x002E)",
    },
];

fn expected_performance_record(id: &str) -> Option<ExpectedPerformanceRecord> {
    match id {
        "PTIMER" => Some(ExpectedPerformanceRecord {
            type_def: "0x02000BEC",
            original_path: "src/reconstructed/performance/managed/Timer.cs",
            original_size: 2779,
            original_sha256: "c5d437c8f4c80de3a163fbb80fc6c4968b3467cb280d8bf4c1f9d310e09ecbfa",
            proposed_path: "src/managed/cs-e3fc67839851/timer/Timer.cs",
            methods: &PTIMER_METHODS,
        }),
        "PTIMER_INSTANCE" => Some(ExpectedPerformanceRecord {
            type_def: "0x02000BED",
            original_path: "src/reconstructed/performance/managed/TimerSystemInstance.cs",
            original_size: 4931,
            original_sha256: "2607a87cd72841399c0f6159ea08a9bd2587317c57e7b92327dee1f819b12201",
            proposed_path: "src/managed/cs-e3fc67839851/timer/TimerSystemInstance.cs",
            methods: &PTIMER_INSTANCE_METHODS,
        }),
        _ => None,
    }
}

#[derive(Clone, Copy)]
struct ExpectedSchedulingMethod {
    type_def: &'static str,
    method_def: &'static str,
    rva: &'static str,
    il_range: &'static str,
    full: bool,
}

#[derive(Clone, Copy)]
struct ExpectedAssociatedType {
    type_def: &'static str,
    parent_type_def: &'static str,
}

#[derive(Clone, Copy)]
struct ExpectedSchedulingRecord {
    root_type: &'static str,
    original_path: &'static str,
    original_size: u64,
    original_sha256: &'static str,
    proposed_path: &'static str,
    methods: &'static [ExpectedSchedulingMethod],
    associated_types: &'static [ExpectedAssociatedType],
}

const PSCHED_GRAPH_METHODS: [ExpectedSchedulingMethod; 12] = [
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C916",
        rva: "0x88EBF0",
        il_range: "[0x0000,0x001E)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C917",
        rva: "0x88EC10",
        il_range: "[0x0000,0x0022)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C918",
        rva: "0x88EC40",
        il_range: "[0x0000,0x01F2)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C919",
        rva: "0x88EE5C",
        il_range: "[0x0000,0x003F)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C91A",
        rva: "0x88EEA7",
        il_range: "[0x0000,0x0001)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D4",
        method_def: "0x0601C91B",
        rva: "0x88EEA9",
        il_range: "[0x0000,0x0007)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200968E",
        method_def: "0x0604A4F0",
        rva: "0x14296C8",
        il_range: "[0x0000,0x0037)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200968E",
        method_def: "0x0604A4F1",
        rva: "0x142970C",
        il_range: "[0x0000,0x00F6)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200968E",
        method_def: "0x0604A4F2",
        rva: "0x142980E",
        il_range: "[0x0000,0x0007)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200968E",
        method_def: "0x0604A4F3",
        rva: "0x1429818",
        il_range: "[0x0000,0x003D)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200968F",
        method_def: "0x0604A4F4",
        rva: "0x1429864",
        il_range: "[0x0000,0x01D1)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200CE0E",
        method_def: "0x060506E6",
        rva: "0x164EA6C",
        il_range: "[0x0000,0x0282)",
        full: true,
    },
];

const PSCHED_UTILS_METHODS: [ExpectedSchedulingMethod; 6] = [
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C940",
        rva: "0x88FD58",
        il_range: "[0x0000,0x0052)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C941",
        rva: "0x88FDC8",
        il_range: "[0x0000,0x0050)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C942",
        rva: "0x88FE34",
        il_range: "[0x0000,0x009E)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C943",
        rva: "0x88FEF0",
        il_range: "[0x0000,0x00C7)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C944",
        rva: "0x88FFD4",
        il_range: "[0x0000,0x01FF)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x020034D9",
        method_def: "0x0601C945",
        rva: "0x8901FC",
        il_range: "[0x0000,0x008D)",
        full: false,
    },
];

const PSCHED_HEAP_METHODS: [ExpectedSchedulingMethod; 15] = [
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x0600026B",
        rva: "0xD708",
        il_range: "[0x0000,0x0024)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x0600026C",
        rva: "0xD730",
        il_range: "[0x0000,0x008A)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x0600026D",
        rva: "0xD7E4",
        il_range: "[0x0000,0x000C)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x0600026E",
        rva: "0xD7F1",
        il_range: "[0x0000,0x000F)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x0600026F",
        rva: "0xD804",
        il_range: "[0x0000,0x0040)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000270",
        rva: "0xD850",
        il_range: "[0x0000,0x0017)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000271",
        rva: "0xD868",
        il_range: "[0x0000,0x000D)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000272",
        rva: "0xD878",
        il_range: "[0x0000,0x002E)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000273",
        rva: "0xD8B4",
        il_range: "[0x0000,0x00B0)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000274",
        rva: "0xD970",
        il_range: "[0x0000,0x00C6)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000275",
        rva: "0xDA44",
        il_range: "[0x0000,0x0040)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000276",
        rva: "0xDA90",
        il_range: "[0x0000,0x0023)",
        full: false,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000277",
        rva: "0xDAC0",
        il_range: "[0x0000,0x0044)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000278",
        rva: "0xDB10",
        il_range: "[0x0000,0x0095)",
        full: true,
    },
    ExpectedSchedulingMethod {
        type_def: "0x0200006B",
        method_def: "0x06000279",
        rva: "0xDBB4",
        il_range: "[0x0000,0x0063)",
        full: true,
    },
];

const PSCHED_GRAPH_ASSOCIATED: [ExpectedAssociatedType; 3] = [
    ExpectedAssociatedType {
        type_def: "0x0200968E",
        parent_type_def: "0x020034D4",
    },
    ExpectedAssociatedType {
        type_def: "0x0200968F",
        parent_type_def: "0x020034D4",
    },
    ExpectedAssociatedType {
        type_def: "0x0200CE0E",
        parent_type_def: "0x0200968E",
    },
];

const NO_ASSOCIATED_TYPES: [ExpectedAssociatedType; 0] = [];

const PSCHED_GRAPH_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/6ef1ac9f3b7c3584d8dc5d548be0fd71b292e9c09e2351ff8af9df966a957a93/evidence/raw/methods.jsonl";
const PSCHED_GRAPH_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/6ef1ac9f3b7c3584d8dc5d548be0fd71b292e9c09e2351ff8af9df966a957a93/evidence/raw/types.jsonl";
const PSCHED_UTILS_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/e7fbd9dbdcdbe310d1a3ae7baae4fc8d44e71464a3e7f0a25d8993b1a4e1ef9e/evidence/raw/methods.jsonl";
const PSCHED_UTILS_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/e7fbd9dbdcdbe310d1a3ae7baae4fc8d44e71464a3e7f0a25d8993b1a4e1ef9e/evidence/raw/types.jsonl";
const PSCHED_HEAP_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/3e1facc4e6f297ee4666b05aa99bf905c9c0f54eaf0502dac773a1d9bf1a3721/evidence/raw/methods.jsonl";
const PSCHED_HEAP_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/3e1facc4e6f297ee4666b05aa99bf905c9c0f54eaf0502dac773a1d9bf1a3721/evidence/raw/types.jsonl";

fn expected_scheduling_raw_paths(id: &str) -> Option<(&'static str, &'static str)> {
    match id {
        "PSCHED_GRAPH" => Some((PSCHED_GRAPH_RAW_METHODS, PSCHED_GRAPH_RAW_TYPES)),
        "PSCHED_UTILS" => Some((PSCHED_UTILS_RAW_METHODS, PSCHED_UTILS_RAW_TYPES)),
        "PSCHED_HEAP" => Some((PSCHED_HEAP_RAW_METHODS, PSCHED_HEAP_RAW_TYPES)),
        _ => None,
    }
}

fn expected_scheduling_record(id: &str) -> Option<ExpectedSchedulingRecord> {
    match id {
        "PSCHED_GRAPH" => Some(ExpectedSchedulingRecord {
            root_type: "0x020034D4",
            original_path: "src/reconstructed/performance/TaskGraph.cs",
            original_size: 10715,
            original_sha256: "2a6b17d8e10c287437b450d8fa87f717c008fa6571fcd3bd26c61eb603b5373a",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/TaskGraph.cs",
            methods: &PSCHED_GRAPH_METHODS,
            associated_types: &PSCHED_GRAPH_ASSOCIATED,
        }),
        "PSCHED_UTILS" => Some(ExpectedSchedulingRecord {
            root_type: "0x020034D9",
            original_path: "src/reconstructed/performance/TaskUtils.cs",
            original_size: 9108,
            original_sha256: "f156234df333c606e4d3ea92445f39954f7beb9ed1797f44f55f70751edfc4fd",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/TaskUtils.cs",
            methods: &PSCHED_UTILS_METHODS,
            associated_types: &NO_ASSOCIATED_TYPES,
        }),
        "PSCHED_HEAP" => Some(ExpectedSchedulingRecord {
            root_type: "0x0200006B",
            original_path: "src/reconstructed/performance/PriorityQueue.cs",
            original_size: 7853,
            original_sha256: "48d2102543b23a10077b86b6621b0f805f60c1dc8b3773b85a0baf2b46b1bb4d",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/PriorityQueue.cs",
            methods: &PSCHED_HEAP_METHODS,
            associated_types: &NO_ASSOCIATED_TYPES,
        }),
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TaskProof {
    Full,
    Source,
    Metadata,
}

#[derive(Clone, Copy)]
struct ExpectedTaskMethod {
    type_def: &'static str,
    method_def: &'static str,
    rva: &'static str,
    il_range: Option<&'static str>,
    proof: TaskProof,
}

#[derive(Clone, Copy)]
struct ExpectedTaskRecord {
    root_type: &'static str,
    original_path: &'static str,
    original_size: u64,
    original_sha256: &'static str,
    proposed_path: &'static str,
    methods: &'static [ExpectedTaskMethod],
    associated_types: &'static [ExpectedAssociatedType],
}

const PEXEC_SYSTEM_METHODS: [ExpectedTaskMethod; 18] = [
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C931",
        rva: "0x88F8BC",
        il_range: Some("[0x0000,0x001F)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C932",
        rva: "0x88F8E7",
        il_range: Some("[0x0000,0x0009)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C933",
        rva: "0x88F8F4",
        il_range: Some("[0x0000,0x0037)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C934",
        rva: "0x88F938",
        il_range: Some("[0x0000,0x00DA)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C935",
        rva: "0x88FA20",
        il_range: Some("[0x0000,0x002A)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C936",
        rva: "0x88FA58",
        il_range: Some("[0x0000,0x006F)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C937",
        rva: "0x88FAD4",
        il_range: Some("[0x0000,0x0027)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C938",
        rva: "0x88FB08",
        il_range: Some("[0x0000,0x0025)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C939",
        rva: "0x88FB3C",
        il_range: Some("[0x0000,0x002C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C93A",
        rva: "0x88FB74",
        il_range: Some("[0x0000,0x00B3)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C93B",
        rva: "0x88FC34",
        il_range: Some("[0x0000,0x0051)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C93C",
        rva: "0x88FC91",
        il_range: Some("[0x0000,0x0018)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C93D",
        rva: "0x88FCAC",
        il_range: Some("[0x0000,0x002F)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D8",
        method_def: "0x0601C93E",
        rva: "0x88FCE8",
        il_range: Some("[0x0000,0x0037)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02009691",
        method_def: "0x0604A4F7",
        rva: "0x1429A8E",
        il_range: Some("[0x0000,0x000D)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02009692",
        method_def: "0x0604A4F8",
        rva: "0x1429A9C",
        il_range: Some("[0x0000,0x019E)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02009693",
        method_def: "0x0604A4FA",
        rva: "0x1429C74",
        il_range: Some("[0x0000,0x0417)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02009694",
        method_def: "0x0604A4FC",
        rva: "0x142A0DC",
        il_range: Some("[0x0000,0x00E8)"),
        proof: TaskProof::Full,
    },
];

const PEXEC_BASE_METHODS: [ExpectedTaskMethod; 14] = [
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C909",
        rva: "0x88EA5C",
        il_range: Some("[0x0000,0x0046)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90A",
        rva: "0x88EAAE",
        il_range: Some("[0x0000,0x0007)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90B",
        rva: "0x88EAB6",
        il_range: Some("[0x0000,0x0008)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90C",
        rva: "0x88EABF",
        il_range: Some("[0x0000,0x000C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90D",
        rva: "0x88EACC",
        il_range: Some("[0x0000,0x000C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90E",
        rva: "0x88EADC",
        il_range: Some("[0x0000,0x0043)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C90F",
        rva: "0x88EB2C",
        il_range: Some("[0x0000,0x0042)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C910",
        rva: "0x88EB7A",
        il_range: Some("[0x0000,0x001E)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C911",
        rva: "0x88EB99",
        il_range: Some("[0x0000,0x000C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C912",
        rva: "0x88EBA8",
        il_range: Some("[0x0000,0x0037)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C913",
        rva: "0x0",
        il_range: None,
        proof: TaskProof::Metadata,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C914",
        rva: "0x88EBEB",
        il_range: Some("[0x0000,0x0002)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x020034D3",
        method_def: "0x0601C915",
        rva: "0x88EBEE",
        il_range: Some("[0x0000,0x0001)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x0200968D",
        method_def: "0x0604A4ED",
        rva: "0x1429504",
        il_range: Some("[0x0000,0x0151)"),
        proof: TaskProof::Full,
    },
];

const PEXEC_QUEUE_METHODS: [ExpectedTaskMethod; 19] = [
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A3",
        rva: "0xE142",
        il_range: Some("[0x0000,0x001A)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A4",
        rva: "0xE15D",
        il_range: Some("[0x0000,0x0007)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A5",
        rva: "0xE168",
        il_range: Some("[0x0000,0x0080)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A6",
        rva: "0xE1F4",
        il_range: Some("[0x0000,0x00CC)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A7",
        rva: "0xE2CC",
        il_range: Some("[0x0000,0x00B4)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A8",
        rva: "0xE38C",
        il_range: Some("[0x0000,0x003F)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002A9",
        rva: "0xE3D7",
        il_range: Some("[0x0000,0x002A)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AA",
        rva: "0xE404",
        il_range: Some("[0x0000,0x00C4)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AB",
        rva: "0xE4D4",
        il_range: Some("[0x0000,0x028B)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AC",
        rva: "0xE76B",
        il_range: Some("[0x0000,0x000A)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AD",
        rva: "0xE776",
        il_range: Some("[0x0000,0x0038)"),
        proof: TaskProof::Full,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AE",
        rva: "0xE7AF",
        il_range: Some("[0x0000,0x000C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02000071",
        method_def: "0x060002AF",
        rva: "0xE7BC",
        il_range: Some("[0x0000,0x0007)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x0604670C",
        rva: "0x12A9B8F",
        il_range: Some("[0x0000,0x002C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x0604670D",
        rva: "0x12A9BBC",
        il_range: Some("[0x0000,0x0057)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x0604670E",
        rva: "0x12A9C1F",
        il_range: Some("[0x0000,0x0008)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x0604670F",
        rva: "0x12A9C28",
        il_range: Some("[0x0000,0x0007)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x06046710",
        rva: "0x12A9C30",
        il_range: Some("[0x0000,0x000C)"),
        proof: TaskProof::Source,
    },
    ExpectedTaskMethod {
        type_def: "0x02007184",
        method_def: "0x06046711",
        rva: "0x12A9C3D",
        il_range: Some("[0x0000,0x0001)"),
        proof: TaskProof::Source,
    },
];

const PEXEC_SYSTEM_ASSOCIATED: [ExpectedAssociatedType; 4] = [
    ExpectedAssociatedType {
        type_def: "0x02009691",
        parent_type_def: "0x020034D8",
    },
    ExpectedAssociatedType {
        type_def: "0x02009692",
        parent_type_def: "0x020034D8",
    },
    ExpectedAssociatedType {
        type_def: "0x02009693",
        parent_type_def: "0x020034D8",
    },
    ExpectedAssociatedType {
        type_def: "0x02009694",
        parent_type_def: "0x020034D8",
    },
];

const PEXEC_BASE_ASSOCIATED: [ExpectedAssociatedType; 1] = [ExpectedAssociatedType {
    type_def: "0x0200968D",
    parent_type_def: "0x020034D3",
}];

const PEXEC_QUEUE_ASSOCIATED: [ExpectedAssociatedType; 1] = [ExpectedAssociatedType {
    type_def: "0x02007184",
    parent_type_def: "0x02000071",
}];

fn expected_task_record(id: &str) -> Option<ExpectedTaskRecord> {
    match id {
        "PEXEC_SYSTEM" => Some(ExpectedTaskRecord {
            root_type: "0x020034D8",
            original_path: "src/reconstructed/performance/TaskSystem.cs",
            original_size: 15075,
            original_sha256: "0567e1ec4cff3885fc5add61ff4288fd1f19e69894445630f7dc0f1afce792d9",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/TaskSystem.cs",
            methods: &PEXEC_SYSTEM_METHODS,
            associated_types: &PEXEC_SYSTEM_ASSOCIATED,
        }),
        "PEXEC_BASE" => Some(ExpectedTaskRecord {
            root_type: "0x020034D3",
            original_path: "src/reconstructed/performance/TaskBase.cs",
            original_size: 5418,
            original_sha256: "50dd7ff2779ce004601ee862ffbf5b578c0dc46823df61f68f8d00fbf76771c3",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/TaskBase.cs",
            methods: &PEXEC_BASE_METHODS,
            associated_types: &PEXEC_BASE_ASSOCIATED,
        }),
        "PEXEC_QUEUE" => Some(ExpectedTaskRecord {
            root_type: "0x02000071",
            original_path: "src/reconstructed/performance/Queue.cs",
            original_size: 10163,
            original_sha256: "439ce33738812997ad56772218f108eb82873f34f105f6ed677f4b9cb6b4ad3a",
            proposed_path: "src/managed/cs-e3fc67839851/scheduling/Queue.cs",
            methods: &PEXEC_QUEUE_METHODS,
            associated_types: &PEXEC_QUEUE_ASSOCIATED,
        }),
        _ => None,
    }
}

const PEXEC_SYSTEM_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/b61135f089b80d51f0d49f22c104a39ef53d3482d405b84313535feae831ef24/evidence/raw/methods.jsonl";
const PEXEC_SYSTEM_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/b61135f089b80d51f0d49f22c104a39ef53d3482d405b84313535feae831ef24/evidence/raw/types.jsonl";
const PEXEC_BASE_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/3661efec61bceb606b5698a533e0470e34ae41e7da60d5e21f63a6c3e65483ee/evidence/raw/methods.jsonl";
const PEXEC_BASE_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/3661efec61bceb606b5698a533e0470e34ae41e7da60d5e21f63a6c3e65483ee/evidence/raw/types.jsonl";
const PEXEC_QUEUE_RAW_METHODS: &str = "src/reconstructed/managed/full/CSharpScript/shards/bcdf009b92e6421939d65a3ec383b16d2ce16f584be51beed146e9e6f0bf0acb/evidence/raw/methods.jsonl";
const PEXEC_QUEUE_RAW_TYPES: &str = "src/reconstructed/managed/full/CSharpScript/shards/bcdf009b92e6421939d65a3ec383b16d2ce16f584be51beed146e9e6f0bf0acb/evidence/raw/types.jsonl";
const PERF02_EVIDENCE_PATH: &str = "evidence/performance/PERF02_EVIDENCE.json";
const PERF02_RAW_PATH: &str = "evidence/performance/PERF02_DIRECT_CIL.txt";
const PERF02_MANIFEST_PATH: &str = "evidence/performance/PERF02_DIRECT_CIL_MANIFEST.json";
const PERF02_REPORT_PATH: &str = "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md";
const PERF_TASK01_PATH: &str = "docs/performance/PERF_TASK01_CLEAR_PROMISE_LIFECYCLE.md";

fn expected_task_raw_paths(id: &str) -> Option<(&'static str, &'static str)> {
    match id {
        "PEXEC_SYSTEM" => Some((PEXEC_SYSTEM_RAW_METHODS, PEXEC_SYSTEM_RAW_TYPES)),
        "PEXEC_BASE" => Some((PEXEC_BASE_RAW_METHODS, PEXEC_BASE_RAW_TYPES)),
        "PEXEC_QUEUE" => Some((PEXEC_QUEUE_RAW_METHODS, PEXEC_QUEUE_RAW_TYPES)),
        _ => None,
    }
}

fn validate_record_methods(record: &Record) -> Result<(), ValidationError> {
    let managed = record.authority_id == MANAGED_AUTHORITY_ID;
    if managed {
        if record.type_def.is_none()
            && (!record.methods.historical.is_empty() || !record.methods.current.is_empty())
        {
            return Err(ValidationError::record(
                "MISSING_MANAGED_TYPEDEF",
                &record.id,
            ));
        }
        if let Some(type_def) = &record.type_def {
            if !validate_type_token(type_def) {
                return Err(ValidationError::record("INVALID_TYPEDEF", &record.id));
            }
        }
    } else if !record.methods.historical.is_empty() || !record.methods.current.is_empty() {
        return Err(ValidationError::record(
            "NON_MANAGED_METHOD_TOKEN",
            &record.id,
        ));
    }

    let mut historical = BTreeSet::new();
    for method in &record.methods.historical {
        validate_method_identity(record, &method.symbol_key, None)?;
        if !historical.insert(method_key(&method.symbol_key)) {
            return Err(ValidationError::record(
                "DUPLICATE_HISTORICAL_METHOD",
                &record.id,
            ));
        }
        if method.coverage.is_empty() {
            return Err(ValidationError::record("EMPTY_METHOD_COVERAGE", &record.id));
        }
    }

    let mut current = BTreeSet::new();
    for method in &record.methods.current {
        validate_method_identity(record, &method.symbol_key, Some(&method.method_def))?;
        if !current.insert(method_key(&method.symbol_key)) {
            return Err(ValidationError::record(
                "DUPLICATE_CURRENT_METHOD",
                &record.id,
            ));
        }
        if method.coverage.is_empty() || method.verification_basis.is_empty() {
            return Err(ValidationError::record(
                "INCOMPLETE_CURRENT_METHOD",
                &record.id,
            ));
        }
        if let Some(rva) = &method.rva {
            if rva.is_empty() {
                return Err(ValidationError::record("EMPTY_METHOD_RVA", &record.id));
            }
        }
        if let Some(il_range) = &method.il_range {
            if il_range.is_empty() {
                return Err(ValidationError::record("EMPTY_METHOD_IL_RANGE", &record.id));
            }
        }
        for reference in &method.evidence_references {
            validate_evidence_reference(reference)
                .map_err(|error| error.with_record(&record.id))?;
        }
        if matches!(method.source_status, SourceStatus::SourceOnly)
            && matches!(
                method.evidence_status,
                EvidenceStatus::DirectCilPartial | EvidenceStatus::DirectCilFull
            )
            && !(record.id == "P"
                && method.method_def == "0x0601C6AF"
                && matches!(method.evidence_status, EvidenceStatus::DirectCilFull)
                && has_qualifying_load01_reference(method))
        {
            return Err(ValidationError::record(
                "SOURCE_ONLY_PROMOTION_UNPROVEN",
                &record.id,
            ));
        }
    }
    Ok(())
}

fn relation_mentions(relation: &str, id: &str) -> bool {
    relation
        .split(|character: char| !character.is_ascii_alphanumeric())
        .any(|part| part == id)
}

fn supersession_has_relation(a: &Record, b: &Record) -> bool {
    (a.supersession.supersedes.iter().any(|id| id == &b.id)
        && b.supersession.superseded_by.iter().any(|id| id == &a.id))
        || (a.supersession.superseded_by.iter().any(|id| id == &b.id)
            && b.supersession.supersedes.iter().any(|id| id == &a.id))
}

fn qualified_relation_covers(a: &Record, b: &Record, overlap: &BTreeSet<String>) -> bool {
    if a.authority_id != b.authority_id || !supersession_has_relation(a, b) {
        return false;
    }
    let (left, right) = if a.supersession.supersedes.iter().any(|id| id == &b.id) {
        (&a.supersession, &b.supersession)
    } else {
        (&b.supersession, &a.supersession)
    };
    if !left.same_authority || !left.qualified || !right.same_authority || !right.qualified {
        return false;
    }
    let scope: BTreeSet<String> = left.scope.iter().cloned().collect();
    !scope.is_empty()
        && overlap.iter().all(|key| {
            let token = key.rsplit('|').next().unwrap_or_default();
            scope.contains(token)
        })
}

fn permitted_representation_overlap(a: &Record, b: &Record) -> bool {
    let ids = BTreeSet::from([a.id.as_str(), b.id.as_str()]);
    ids == BTreeSet::from(["P", "G"])
        && a.authority_id == b.authority_id
        && a.type_def == b.type_def
        && a.duplicate_group.as_deref() == Some("gamemodepromise-layer-overlap")
        && b.duplicate_group.as_deref() == Some("gamemodepromise-layer-overlap")
        && a.representation != b.representation
        && relation_mentions(&a.relationship, &b.id)
        && relation_mentions(&b.relationship, &a.id)
}

fn validate_supersession(manifest: &Manifest) -> Result<(), ValidationError> {
    let records: BTreeMap<&str, &Record> = manifest
        .records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect();
    for record in &manifest.records {
        let mut all_ids = BTreeSet::new();
        for id in record
            .supersession
            .supersedes
            .iter()
            .chain(record.supersession.superseded_by.iter())
        {
            if id == &record.id || !all_ids.insert(id.clone()) {
                return Err(ValidationError::record(
                    "INVALID_SUPERSESSION_REFERENCE",
                    &record.id,
                ));
            }
            let target = records
                .get(id.as_str())
                .ok_or_else(|| ValidationError::record("UNKNOWN_SUPERSESSION_ID", &record.id))?;
            if record.supersession.same_authority != (record.authority_id == target.authority_id) {
                return Err(ValidationError::record(
                    "SUPERSESSION_AUTHORITY_MISMATCH",
                    &record.id,
                ));
            }
            if record.supersession.qualified {
                let overlap: BTreeSet<String> = qualified_key_set(record)
                    .intersection(&qualified_key_set(target))
                    .cloned()
                    .collect();
                if !record.supersession.same_authority
                    || record.supersession.scope.is_empty()
                    || overlap.is_empty()
                    || !record
                        .supersession
                        .scope
                        .iter()
                        .all(|token| valid_token(token, "0x06"))
                    || !overlap.iter().all(|key| {
                        let token = key.rsplit('|').next().unwrap_or_default();
                        record.supersession.scope.iter().any(|scope| scope == token)
                    })
                    || !supersession_has_relation(record, target)
                {
                    return Err(ValidationError::record(
                        "INVALID_QUALIFIED_SUPERSESSION",
                        &record.id,
                    ));
                }
            } else if !record.supersession.scope.is_empty() {
                return Err(ValidationError::record(
                    "UNQUALIFIED_SUPERSESSION_SCOPE",
                    &record.id,
                ));
            }
        }
    }

    for left_index in 0..manifest.records.len() {
        for right_index in left_index + 1..manifest.records.len() {
            let left = &manifest.records[left_index];
            let right = &manifest.records[right_index];
            let overlap: BTreeSet<String> = qualified_key_set(left)
                .intersection(&qualified_key_set(right))
                .cloned()
                .collect();
            if overlap.is_empty() {
                continue;
            }
            if !qualified_relation_covers(left, right, &overlap)
                && !permitted_representation_overlap(left, right)
            {
                return Err(ValidationError::record(
                    "CONFLICTING_QUALIFIED_OVERLAP",
                    &left.id,
                ));
            }
        }
    }
    Ok(())
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), ValidationError> {
    if manifest.schema != "wuwa-reconstruction-migration-v1" || manifest.manifest_version != 1 {
        return Err(ValidationError::new("MANIFEST_VERSION_INVALID"));
    }
    if manifest.authority_policy.managed.authority_id != MANAGED_AUTHORITY_ID
        || manifest.authority_policy.managed.authority_binary_sha256 != MANAGED_AUTHORITY_SHA256
        || manifest.authority_policy.managed.mvid != MANAGED_MVID
        || !valid_uuid(&manifest.authority_policy.managed.mvid)
        || manifest.authority_policy.managed.authority_sha256_kind != "managed_assembly"
        || manifest.authority_policy.build.authority_id != BUILD_AUTHORITY_ID
        || manifest
            .authority_policy
            .build
            .historical_client_sha256
            .len()
            != 64
        || !valid_lower_hex(
            &manifest.authority_policy.build.historical_client_sha256,
            64,
        )
    {
        return Err(ValidationError::new("AUTHORITY_POLICY_INVALID"));
    }
    if manifest.symbol_policy.managed_key
        != ["full_authority_sha256", "mvid", "typeDef", "methodDef"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    {
        return Err(ValidationError::new("SYMBOL_POLICY_INVALID"));
    }
    if manifest.schema_contract.allowed_evidence_statuses != expected_evidence_statuses()
        || manifest.schema_contract.allowed_source_statuses != expected_source_statuses()
        || manifest.allowed_curation_statuses != expected_curation_statuses()
        || manifest.allowed_export_statuses != expected_export_statuses()
    {
        return Err(ValidationError::new("ENUM_POLICY_INVALID"));
    }
    if manifest.preservation.accepted_closure.sha256.len() != 64
        || !valid_lower_hex(&manifest.preservation.accepted_closure.sha256, 64)
    {
        return Err(ValidationError::new("INVALID_PRESERVATION_SHA256"));
    }
    validate_relative_path(&manifest.benchmark.document)
        .map_err(|_| ValidationError::at("UNSAFE_MANIFEST_PATH", &manifest.benchmark.document))?;
    validate_relative_path(&manifest.preservation.accepted_closure.path).map_err(|_| {
        ValidationError::at(
            "UNSAFE_MANIFEST_PATH",
            &manifest.preservation.accepted_closure.path,
        )
    })?;

    let mut ids = BTreeSet::new();
    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    for record in &manifest.records {
        if record.id.is_empty() || !ids.insert(record.id.clone()) {
            return Err(ValidationError::new("DUPLICATE_RECORD_ID"));
        }
        validate_relative_path(&record.original_path)
            .map_err(|_| ValidationError::at("UNSAFE_MANIFEST_PATH", &record.original_path))?;
        if !original_paths.insert(record.original_path.to_ascii_lowercase()) {
            return Err(ValidationError::record(
                "DUPLICATE_ORIGINAL_PATH",
                &record.id,
            ));
        }
        if !valid_lower_hex(&record.original_sha256, 64) {
            return Err(ValidationError::record("INVALID_SOURCE_SHA256", &record.id));
        }
        if let Some(proposed_path) = &record.proposed_path {
            validate_relative_path(proposed_path)
                .map_err(|_| ValidationError::at("UNSAFE_MANIFEST_PATH", proposed_path))?;
            if !destinations.insert(proposed_path.to_ascii_lowercase()) {
                return Err(ValidationError::record(
                    "DESTINATION_CASEFOLD_COLLISION",
                    &record.id,
                ));
            }
        }
        if record.scope.is_empty()
            || record.completeness.is_empty()
            || record.representation.is_empty()
        {
            return Err(ValidationError::record(
                "INCOMPLETE_RECORD_SCOPE",
                &record.id,
            ));
        }
        if record.authority_id == MANAGED_AUTHORITY_ID {
            if record.authority_binary_sha256.as_deref() != Some(MANAGED_AUTHORITY_SHA256)
                || record.authority_mvid.as_deref() != Some(MANAGED_MVID)
                || record.authority_sha256_kind != "managed_assembly"
                || record.build_reference.is_some()
            {
                return Err(ValidationError::record(
                    "MANAGED_AUTHORITY_INCOMPLETE",
                    &record.id,
                ));
            }
            if record.type_def.is_none()
                && (!record.methods.historical.is_empty() || !record.methods.current.is_empty())
            {
                return Err(ValidationError::record(
                    "MISSING_MANAGED_TYPEDEF",
                    &record.id,
                ));
            }
        } else if record.authority_id == BUILD_AUTHORITY_ID {
            if record.authority_binary_sha256.is_some()
                || record.authority_mvid.is_some()
                || record.type_def.is_some()
                || record.build_reference.is_none()
                || record.authority_sha256_kind != "BUILD_ASSOCIATION_ONLY"
            {
                return Err(ValidationError::record(
                    "NON_MANAGED_AUTHORITY_INVALID",
                    &record.id,
                ));
            }
            let build = record.build_reference.as_ref().expect("checked above");
            if build.build_id != BUILD_AUTHORITY_ID
                || !valid_lower_hex(&build.historical_client_sha256, 64)
            {
                return Err(ValidationError::record(
                    "BUILD_REFERENCE_INVALID",
                    &record.id,
                ));
            }
        } else {
            return Err(ValidationError::record("UNKNOWN_AUTHORITY", &record.id));
        }
        for reference in &record.evidence_references {
            validate_evidence_reference(reference)
                .map_err(|error| error.with_record(&record.id))?;
        }
        validate_record_methods(record)?;
        if matches!(record.curation_status, CurationStatus::CuratedReady) {
            return Err(ValidationError::record(
                "CURATED_READY_NOT_AUTHORIZED",
                &record.id,
            ));
        }
        if REQUIRED_IMPORT_IDS.contains(&record.id.as_str())
            && (!matches!(record.curation_status, CurationStatus::NeedsEditorialReview)
                || !matches!(
                    record.export_status,
                    ExportStatus::PublicationReviewRequired
                ))
        {
            return Err(ValidationError::record(
                "APPROVED_BATCH_STATUS_INVALID",
                &record.id,
            ));
        }
    }

    validate_supersession(manifest)?;
    Ok(())
}

fn expected_method_for_import(record: &Record) -> Option<&CurrentMethod> {
    if record.methods.current.len() == 1 {
        record.methods.current.first()
    } else {
        None
    }
}

fn receipt_statuses_valid(entry: &ImportEntry) -> bool {
    matches!(entry.exact_byte_comparison, VerificationResult::Pass)
        && matches!(
            entry.editorial_status,
            ReceiptEditorialStatus::NeedsEditorialReview
        )
        && matches!(entry.compilation_status, ReceiptCompilationStatus::NotRun)
        && matches!(
            entry.equivalence_status,
            ReceiptEquivalenceStatus::NotEstablished
        )
        && matches!(entry.publication_status, ReceiptPublicationStatus::Disabled)
        && matches!(entry.runtime_status, ReceiptRuntimeStatus::Parked)
}

pub fn validate_receipt(
    receipt: &ImportReceipt,
    manifest: &Manifest,
) -> Result<(), ValidationError> {
    if receipt.schema != "wuwa-reconstruction-imports-v1"
        || receipt.version != 1
        || receipt.manifest_sha256 != BASELINE_MANIFEST_SHA256
    {
        return Err(ValidationError::new("RECEIPT_ROOT_INVALID"));
    }
    let mut ids = BTreeSet::new();
    let mut imported_paths = BTreeSet::new();
    for entry in &receipt.imports {
        if !ids.insert(entry.id.clone()) {
            return Err(ValidationError::new("DUPLICATE_IMPORT_ID"));
        }
        if !REQUIRED_IMPORT_IDS.contains(&entry.id.as_str()) {
            return Err(ValidationError::record("UNAUTHORIZED_IMPORT_ID", &entry.id));
        }
        let record = record_by_id(manifest, &entry.id)
            .ok_or_else(|| ValidationError::record("UNKNOWN_IMPORT_ID", &entry.id))?;
        let method = expected_method_for_import(record)
            .ok_or_else(|| ValidationError::record("IMPORT_METHOD_SCOPE_INVALID", &entry.id))?;
        if record.proposed_path.is_none()
            || entry.authority_id != record.authority_id
            || entry.method_identity != method.symbol_key
            || entry.original_path != record.original_path
            || entry.original_size != record.original_size
            || entry.original_sha256 != record.original_sha256
            || entry.imported_path != record.proposed_path.clone().unwrap_or_default()
            || entry.implementation_scope != record.scope
            || entry.source_completeness != record.completeness
            || entry.unresolved_dependencies != record.dependencies
            || !receipt_statuses_valid(entry)
        {
            return Err(ValidationError::record(
                "RECEIPT_CORRESPONDENCE_INVALID",
                &entry.id,
            ));
        }
        if !valid_lower_hex(&entry.original_sha256, 64)
            || !valid_lower_hex(&entry.imported_sha256, 64)
        {
            return Err(ValidationError::record("RECEIPT_HASH_INVALID", &entry.id));
        }
        validate_relative_path(&entry.original_path)
            .map_err(|_| ValidationError::record("RECEIPT_PATH_INVALID", &entry.id))?;
        validate_relative_path(&entry.imported_path)
            .map_err(|_| ValidationError::record("RECEIPT_PATH_INVALID", &entry.id))?;
        if !imported_paths.insert(entry.imported_path.to_ascii_lowercase()) {
            return Err(ValidationError::record(
                "RECEIPT_DESTINATION_COLLISION",
                &entry.id,
            ));
        }
    }
    Ok(())
}

fn catalog_invalid(code: &str, id: &str) -> ValidationError {
    ValidationError::record(code, id)
}

fn performance_reference_map<'a>(
    references: &'a [EvidenceReference],
    record_id: &str,
) -> Result<BTreeMap<&'a str, &'a EvidenceReference>, ValidationError> {
    let mut map = BTreeMap::new();
    for reference in references {
        validate_evidence_reference(reference).map_err(|error| error.with_record(record_id))?;
        if map.insert(reference.path.as_str(), reference).is_some() {
            return Err(catalog_invalid(
                "CATALOG_EVIDENCE_REFERENCE_INVALID",
                record_id,
            ));
        }
    }
    Ok(map)
}

fn performance_reference_matches(
    reference: &EvidenceReference,
    expected_hash: Option<&str>,
    record_id: &str,
) -> Result<(), ValidationError> {
    if reference.artifact_sha256.as_deref() != expected_hash {
        return Err(catalog_invalid(
            "CATALOG_EVIDENCE_REFERENCE_INVALID",
            record_id,
        ));
    }
    Ok(())
}

fn validate_catalog_record(
    record: &Record,
    expected: &ExpectedPerformanceRecord,
) -> Result<(), ValidationError> {
    if record.id != "PTIMER" && record.id != "PTIMER_INSTANCE" {
        return Err(catalog_invalid("CATALOG_RECORD_ID_INVALID", &record.id));
    }
    if record.unit_id.is_some() || record.namespace_context.as_deref() != Some("global (observed)")
    {
        return Err(catalog_invalid(
            "CATALOG_RECORD_CORRESPONDENCE_INVALID",
            &record.id,
        ));
    }
    if record.original_path != expected.original_path
        || record.original_size != expected.original_size
        || record.original_sha256 != expected.original_sha256
        || record.hash_basis != "raw_source_file_bytes"
    {
        return Err(catalog_invalid(
            "CATALOG_SOURCE_IDENTITY_INVALID",
            &record.id,
        ));
    }
    if record.authority_id != MANAGED_AUTHORITY_ID
        || record.authority_binary_sha256.as_deref() != Some(MANAGED_AUTHORITY_SHA256)
        || record.authority_sha256_kind != "managed_assembly"
        || record.authority_mvid.as_deref() != Some(MANAGED_MVID)
        || record.build_reference.is_some()
        || record.type_def.as_deref() != Some(expected.type_def)
    {
        return Err(catalog_invalid(
            "CATALOG_AUTHORITY_IDENTITY_INVALID",
            &record.id,
        ));
    }
    if !matches!(record.evidence_status, EvidenceStatus::DirectCilFull)
        || !matches!(
            record.source_status,
            SourceStatus::ReconstructedFromDirectCil
        )
        || !matches!(record.curation_status, CurationStatus::NeedsEditorialReview)
        || !matches!(
            record.export_status,
            ExportStatus::PublicationReviewRequired
        )
    {
        return Err(catalog_invalid("CATALOG_STATUS_INVALID", &record.id));
    }
    if record.verification_basis.trim().is_empty()
        || record.proposed_path.as_deref() != Some(expected.proposed_path)
        || record.proposed_path_reason.trim().is_empty()
    {
        return Err(catalog_invalid("CATALOG_DESTINATION_INVALID", &record.id));
    }
    if record.scope.trim().is_empty()
        || record.completeness.trim().is_empty()
        || record.representation.trim().is_empty()
    {
        return Err(catalog_invalid("CATALOG_INCOMPLETE_SCOPE", &record.id));
    }
    if record.duplicate_group.is_some()
        || !record.supersession.same_authority
        || record.supersession.qualified
        || !record.supersession.scope.is_empty()
        || !record.supersession.supersedes.is_empty()
        || !record.supersession.superseded_by.is_empty()
    {
        return Err(catalog_invalid("CATALOG_SUPERSESSION_INVALID", &record.id));
    }
    if record.relationship.trim().is_empty()
        || record.explanation.trim().is_empty()
        || record.dependencies.is_empty()
        || record.editorial_changes.is_empty()
    {
        return Err(catalog_invalid("CATALOG_INCOMPLETE_RECORD", &record.id));
    }
    if record.validation.source_hash != "RECORDED_SUPPLIED_SOURCE_BYTES"
        || record.validation.authority_identity != "MANAGED_SHA_MVID_TOKEN_SCOPE_RECORDED"
        || record.validation.path_collision != "PRIMARY_FINAL_CHECK_PENDING"
        || record.validation.equivalence != "NOT_ESTABLISHED"
        || record.validation.compilation != "NOT_BUILDABLE_WITH_CURRENT_CONTEXT"
        || record.validation.runtime != "PARKED_NOT_RUN"
        || record.validation.publication != "REVIEW_REQUIRED"
        || record.validation.final_receipt != "PRIMARY_FINAL_RECEIPT_PENDING"
    {
        return Err(catalog_invalid(
            "CATALOG_VALIDATION_STATUS_INVALID",
            &record.id,
        ));
    }

    let record_references = performance_reference_map(&record.evidence_references, &record.id)?;
    let expected_record_paths = BTreeSet::from([
        expected.original_path,
        PERF03_REPORT_PATH,
        PERF03_RAW_PATH,
        PERF03_EVIDENCE_PATH,
    ]);
    if record_references.keys().copied().collect::<BTreeSet<_>>() != expected_record_paths {
        return Err(catalog_invalid(
            "CATALOG_EVIDENCE_REFERENCE_INVALID",
            &record.id,
        ));
    }
    performance_reference_matches(record_references[expected.original_path], None, &record.id)?;
    for (path, hash) in [
        (PERF03_REPORT_PATH, PERF03_REPORT_SHA256),
        (PERF03_RAW_PATH, PERF03_RAW_SHA256),
        (PERF03_EVIDENCE_PATH, PERF03_EVIDENCE_SHA256),
    ] {
        performance_reference_matches(record_references[path], Some(hash), &record.id)?;
    }

    validate_record_methods(record)?;
    if !record.methods.historical.is_empty() {
        return Err(catalog_invalid("CATALOG_HISTORICAL_NOT_EMPTY", &record.id));
    }
    let expected_methods: BTreeMap<&str, &ExpectedPerformanceMethod> = expected
        .methods
        .iter()
        .map(|method| (method.method_def, method))
        .collect();
    if record.methods.current.len() != expected_methods.len() {
        return Err(catalog_invalid("CATALOG_METHOD_SET_INVALID", &record.id));
    }
    let mut seen = BTreeSet::new();
    for method in &record.methods.current {
        let method_expected = expected_methods
            .get(method.method_def.as_str())
            .ok_or_else(|| catalog_invalid("CATALOG_METHOD_SET_INVALID", &record.id))?;
        if !seen.insert(method.method_def.clone()) {
            return Err(catalog_invalid("CATALOG_METHOD_SET_INVALID", &record.id));
        }
        if method.coverage != "DIRECT_CIL_FULL_SELECTED_BODY_PERF03"
            || method.rva.as_deref() != Some(method_expected.rva)
            || method.il_range.as_deref() != Some(method_expected.il_range)
            || !matches!(method.evidence_status, EvidenceStatus::DirectCilFull)
            || !matches!(
                method.source_status,
                SourceStatus::ReconstructedFromDirectCil
            )
            || method.verification_basis.trim().is_empty()
        {
            return Err(catalog_invalid(
                "CATALOG_METHOD_CORRESPONDENCE_INVALID",
                &record.id,
            ));
        }
        let method_references = performance_reference_map(&method.evidence_references, &record.id)?;
        if method_references.keys().copied().collect::<BTreeSet<_>>()
            != BTreeSet::from([PERF03_REPORT_PATH, PERF03_RAW_PATH])
        {
            return Err(catalog_invalid(
                "CATALOG_EVIDENCE_REFERENCE_INVALID",
                &record.id,
            ));
        }
        performance_reference_matches(
            method_references[PERF03_REPORT_PATH],
            Some(PERF03_REPORT_SHA256),
            &record.id,
        )?;
        performance_reference_matches(
            method_references[PERF03_RAW_PATH],
            Some(PERF03_RAW_SHA256),
            &record.id,
        )?;
    }
    if seen.len() != expected_methods.len() {
        return Err(catalog_invalid("CATALOG_METHOD_SET_INVALID", &record.id));
    }
    Ok(())
}

fn validate_performance_source(
    catalog: &PerformanceSourceCatalog,
    baseline: &Manifest,
) -> Result<(), ValidationError> {
    if catalog.schema != "wuwa-reconstruction-performance-source-v1"
        || catalog.version != 1
        || catalog.base_manifest_sha256 != BASELINE_MANIFEST_SHA256
    {
        return Err(ValidationError::new("CATALOG_ROOT_INVALID"));
    }
    if catalog.records.len() != PERFORMANCE_SOURCE_IDS.len() {
        return Err(ValidationError::new("CATALOG_RECORD_SET_INVALID"));
    }

    let mut baseline_original_paths = BTreeSet::new();
    let mut baseline_destinations = BTreeSet::new();
    let mut baseline_methods = BTreeSet::new();
    for record in &baseline.records {
        baseline_original_paths.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            baseline_destinations.insert(path.to_ascii_lowercase());
        }
        baseline_methods.extend(qualified_key_set(record));
    }

    let mut ids = BTreeSet::new();
    let mut original_paths = baseline_original_paths;
    let mut destinations = baseline_destinations;
    let mut catalog_methods = BTreeSet::new();
    for record in &catalog.records {
        if !ids.insert(record.id.clone())
            || baseline
                .records
                .iter()
                .any(|existing| existing.id == record.id)
        {
            return Err(catalog_invalid("CATALOG_RECORD_ID_INVALID", &record.id));
        }
        let expected = expected_performance_record(&record.id)
            .ok_or_else(|| catalog_invalid("CATALOG_RECORD_ID_INVALID", &record.id))?;
        validate_catalog_record(record, &expected)?;
        if !original_paths.insert(record.original_path.to_ascii_lowercase()) {
            return Err(catalog_invalid("CATALOG_SOURCE_PATH_COLLISION", &record.id));
        }
        let proposed = record
            .proposed_path
            .as_deref()
            .ok_or_else(|| catalog_invalid("CATALOG_DESTINATION_INVALID", &record.id))?;
        if !destinations.insert(proposed.to_ascii_lowercase()) {
            return Err(catalog_invalid(
                "CATALOG_DESTINATION_CASEFOLD_COLLISION",
                &record.id,
            ));
        }
        for key in qualified_key_set(record) {
            if baseline_methods.contains(&key) || !catalog_methods.insert(key) {
                return Err(catalog_invalid("CATALOG_METHOD_OVERLAP", &record.id));
            }
        }
    }
    let expected_ids: BTreeSet<&str> = PERFORMANCE_SOURCE_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("CATALOG_RECORD_SET_INVALID"));
    }
    Ok(())
}

fn scheduling_invalid(code: &str, id: &str) -> ValidationError {
    ValidationError::record(code, id)
}

fn scheduling_reference_map<'a>(
    references: &'a [EvidenceReference],
    record_id: &str,
) -> Result<BTreeMap<&'a str, &'a EvidenceReference>, ValidationError> {
    let mut map = BTreeMap::new();
    for reference in references {
        validate_evidence_reference(reference).map_err(|error| error.with_record(record_id))?;
        if map.insert(reference.path.as_str(), reference).is_some() {
            return Err(scheduling_invalid(
                "SCHEDULING_EVIDENCE_REFERENCE_INVALID",
                record_id,
            ));
        }
    }
    Ok(map)
}

fn validate_scheduling_record(
    wrapper: &SchedulingSourceRecord,
    expected: &ExpectedSchedulingRecord,
) -> Result<(), ValidationError> {
    let record = &wrapper.record;
    if record.unit_id.is_some()
        || record.namespace_context.as_deref() != Some("global (observed)")
        || record.original_path != expected.original_path
        || record.original_size != expected.original_size
        || record.original_sha256 != expected.original_sha256
        || record.hash_basis != "raw_source_file_bytes"
    {
        return Err(scheduling_invalid(
            "SCHEDULING_SOURCE_IDENTITY_INVALID",
            &record.id,
        ));
    }
    if record.authority_id != MANAGED_AUTHORITY_ID
        || record.authority_binary_sha256.as_deref() != Some(MANAGED_AUTHORITY_SHA256)
        || record.authority_sha256_kind != "managed_assembly"
        || record.authority_mvid.as_deref() != Some(MANAGED_MVID)
        || record.build_reference.is_some()
        || record.type_def.as_deref() != Some(expected.root_type)
    {
        return Err(scheduling_invalid(
            "SCHEDULING_AUTHORITY_INVALID",
            &record.id,
        ));
    }
    if !matches!(record.evidence_status, EvidenceStatus::Unknown)
        || !matches!(record.source_status, SourceStatus::SourceOnly)
        || !matches!(record.curation_status, CurationStatus::NeedsEditorialReview)
        || !matches!(
            record.export_status,
            ExportStatus::PublicationReviewRequired
        )
    {
        return Err(scheduling_invalid("SCHEDULING_STATUS_INVALID", &record.id));
    }
    if record.verification_basis.trim().is_empty()
        || record.proposed_path.as_deref() != Some(expected.proposed_path)
        || record.proposed_path_reason.trim().is_empty()
    {
        return Err(scheduling_invalid(
            "SCHEDULING_DESTINATION_INVALID",
            &record.id,
        ));
    }
    if record.scope.trim().is_empty()
        || record.completeness.trim().is_empty()
        || record.representation.trim().is_empty()
    {
        return Err(scheduling_invalid(
            "SCHEDULING_INCOMPLETE_SCOPE",
            &record.id,
        ));
    }
    if record.duplicate_group.is_some()
        || !record.supersession.same_authority
        || record.supersession.qualified
        || !record.supersession.scope.is_empty()
        || !record.supersession.supersedes.is_empty()
        || !record.supersession.superseded_by.is_empty()
    {
        return Err(scheduling_invalid(
            "SCHEDULING_SUPERSESSION_INVALID",
            &record.id,
        ));
    }
    if record.relationship.trim().is_empty()
        || record.explanation.trim().is_empty()
        || record.dependencies.is_empty()
        || record.editorial_changes.is_empty()
    {
        return Err(scheduling_invalid(
            "SCHEDULING_INCOMPLETE_RECORD",
            &record.id,
        ));
    }
    if record.validation.source_hash != "RECORDED_SUPPLIED_SOURCE_BYTES"
        || record.validation.authority_identity != "MANAGED_SHA_MVID_TOKEN_SCOPE_RECORDED"
        || record.validation.path_collision != "PRIMARY_FINAL_CHECK_PENDING"
        || record.validation.equivalence != "NOT_ESTABLISHED"
        || record.validation.compilation != "NOT_BUILDABLE_WITH_CURRENT_CONTEXT"
        || record.validation.runtime != "PARKED_NOT_RUN"
        || record.validation.publication != "REVIEW_REQUIRED"
        || record.validation.final_receipt != "PRIMARY_FINAL_RECEIPT_PENDING"
    {
        return Err(scheduling_invalid(
            "SCHEDULING_VALIDATION_STATUS_INVALID",
            &record.id,
        ));
    }
    if !record.methods.historical.is_empty() {
        return Err(scheduling_invalid(
            "SCHEDULING_HISTORICAL_NOT_EMPTY",
            &record.id,
        ));
    }
    let (raw_methods, raw_types) = expected_scheduling_raw_paths(&record.id)
        .ok_or_else(|| scheduling_invalid("SCHEDULING_RECORD_ID_INVALID", &record.id))?;
    let record_references = scheduling_reference_map(&record.evidence_references, &record.id)?;
    for path in [
        expected.original_path,
        "evidence/performance/PERF01_VERIFICATION.md",
        "evidence/performance/bounded_il_critical.cil.txt",
        raw_methods,
        raw_types,
    ] {
        if !record_references.contains_key(path) {
            return Err(scheduling_invalid(
                "SCHEDULING_EVIDENCE_REFERENCE_INVALID",
                &record.id,
            ));
        }
    }

    let expected_methods: BTreeMap<&str, &ExpectedSchedulingMethod> = expected
        .methods
        .iter()
        .map(|method| (method.method_def, method))
        .collect();
    if record.methods.current.len() != expected_methods.len() {
        return Err(scheduling_invalid(
            "SCHEDULING_METHOD_SET_INVALID",
            &record.id,
        ));
    }
    let mut seen = BTreeSet::new();
    for method in &record.methods.current {
        let method_expected = expected_methods
            .get(method.method_def.as_str())
            .ok_or_else(|| scheduling_invalid("SCHEDULING_METHOD_SET_INVALID", &record.id))?;
        if !seen.insert(method.method_def.clone()) {
            return Err(scheduling_invalid(
                "SCHEDULING_METHOD_SET_INVALID",
                &record.id,
            ));
        }
        if method.symbol_key.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
            || method.symbol_key.mvid != MANAGED_MVID
            || method.symbol_key.type_def != method_expected.type_def
            || method.symbol_key.method_def != method_expected.method_def
            || !validate_type_token(&method.symbol_key.type_def)
            || !validate_method_token(&method.symbol_key.method_def)
            || method.method_def != method_expected.method_def
            || method.rva.as_deref() != Some(method_expected.rva)
            || method.il_range.as_deref() != Some(method_expected.il_range)
        {
            return Err(scheduling_invalid(
                "SCHEDULING_METHOD_IDENTITY_INVALID",
                &record.id,
            ));
        }
        let expected_coverage = if method_expected.full {
            "DIRECT_CIL_FULL_SELECTED_BODY_PERF01"
        } else {
            "SOURCE_ONLY_SELECTED_METHOD_PERF01"
        };
        let expected_evidence = if method_expected.full {
            EvidenceStatus::DirectCilFull
        } else {
            EvidenceStatus::SourceOnly
        };
        let expected_source = if method_expected.full {
            SourceStatus::ReconstructedFromDirectCil
        } else {
            SourceStatus::SourceOnly
        };
        if method.coverage != expected_coverage
            || method.evidence_status != expected_evidence
            || method.source_status != expected_source
            || method.verification_basis.trim().is_empty()
        {
            return Err(scheduling_invalid(
                "SCHEDULING_METHOD_PROOF_INVALID",
                &record.id,
            ));
        }
        let method_references = scheduling_reference_map(&method.evidence_references, &record.id)?;
        for path in [expected.original_path, raw_methods] {
            if !method_references.contains_key(path) {
                return Err(scheduling_invalid(
                    "SCHEDULING_EVIDENCE_REFERENCE_INVALID",
                    &record.id,
                ));
            }
        }
        if method_expected.full
            && !method_references.contains_key("evidence/performance/bounded_il_critical.cil.txt")
        {
            return Err(scheduling_invalid(
                "SCHEDULING_FULL_CIL_REFERENCE_MISSING",
                &record.id,
            ));
        }
    }
    if seen.len() != expected_methods.len() {
        return Err(scheduling_invalid(
            "SCHEDULING_METHOD_SET_INVALID",
            &record.id,
        ));
    }

    let expected_associated: BTreeMap<&str, &ExpectedAssociatedType> = expected
        .associated_types
        .iter()
        .map(|associated| (associated.type_def, associated))
        .collect();
    if wrapper.associated_types.len() != expected_associated.len() {
        return Err(scheduling_invalid(
            "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID",
            &record.id,
        ));
    }
    let mut associated_seen = BTreeSet::new();
    for associated in &wrapper.associated_types {
        let expected_type = expected_associated
            .get(associated.type_def.as_str())
            .ok_or_else(|| {
                scheduling_invalid("SCHEDULING_ASSOCIATED_TYPE_SET_INVALID", &record.id)
            })?;
        if associated.type_def == expected.root_type
            || associated.type_def == associated.parent_type_def
            || !validate_type_token(&associated.type_def)
            || !validate_type_token(&associated.parent_type_def)
            || associated.parent_type_def != expected_type.parent_type_def
            || !associated_seen.insert(associated.type_def.clone())
        {
            return Err(scheduling_invalid(
                "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID",
                &record.id,
            ));
        }
        let associated_references =
            scheduling_reference_map(&associated.evidence_references, &record.id)?;
        if !associated_references.contains_key(raw_types) {
            return Err(scheduling_invalid(
                "SCHEDULING_EVIDENCE_REFERENCE_INVALID",
                &record.id,
            ));
        }
    }
    if associated_seen.len() != expected_associated.len() {
        return Err(scheduling_invalid(
            "SCHEDULING_ASSOCIATED_TYPE_SET_INVALID",
            &record.id,
        ));
    }
    Ok(())
}

fn validate_scheduling_source(
    catalog: &SchedulingSourceCatalog,
    baseline: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
) -> Result<(), ValidationError> {
    if catalog.schema != "wuwa-reconstruction-scheduling-source-v1"
        || catalog.version != 1
        || catalog.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || catalog.performance_source_sha256 != PERFORMANCE_SOURCE_V1_SHA256
    {
        return Err(ValidationError::new("SCHEDULING_CATALOG_ROOT_INVALID"));
    }
    if catalog.records.len() != SCHEDULING_SOURCE_IDS.len() {
        return Err(ValidationError::new(
            "SCHEDULING_CATALOG_RECORD_SET_INVALID",
        ));
    }

    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    for record in &baseline.records {
        original_paths.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            destinations.insert(path.to_ascii_lowercase());
        }
        qualified_methods.extend(qualified_key_set(record));
    }
    for record in &performance_catalog.records {
        original_paths.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            destinations.insert(path.to_ascii_lowercase());
        }
        qualified_methods.extend(qualified_key_set(record));
    }

    let mut ids = BTreeSet::new();
    let mut full_count = 0_usize;
    let mut source_only_count = 0_usize;
    for wrapper in &catalog.records {
        let record = &wrapper.record;
        if !ids.insert(record.id.clone())
            || record_by_id(baseline, &record.id).is_some()
            || performance_catalog
                .records
                .iter()
                .any(|existing| existing.id == record.id)
        {
            return Err(scheduling_invalid(
                "SCHEDULING_RECORD_ID_INVALID",
                &record.id,
            ));
        }
        let expected = expected_scheduling_record(&record.id)
            .ok_or_else(|| scheduling_invalid("SCHEDULING_RECORD_ID_INVALID", &record.id))?;
        validate_scheduling_record(wrapper, &expected)?;
        if !original_paths.insert(record.original_path.to_ascii_lowercase()) {
            return Err(scheduling_invalid(
                "SCHEDULING_SOURCE_PATH_COLLISION",
                &record.id,
            ));
        }
        let proposed = record
            .proposed_path
            .as_deref()
            .ok_or_else(|| scheduling_invalid("SCHEDULING_DESTINATION_INVALID", &record.id))?;
        if !destinations.insert(proposed.to_ascii_lowercase()) {
            return Err(scheduling_invalid(
                "SCHEDULING_DESTINATION_CASEFOLD_COLLISION",
                &record.id,
            ));
        }
        for method in &record.methods.current {
            if !qualified_methods.insert(method_key(&method.symbol_key)) {
                return Err(scheduling_invalid("SCHEDULING_METHOD_OVERLAP", &record.id));
            }
            if matches!(method.evidence_status, EvidenceStatus::DirectCilFull) {
                full_count += 1;
            } else if matches!(method.evidence_status, EvidenceStatus::SourceOnly) {
                source_only_count += 1;
            } else {
                return Err(scheduling_invalid(
                    "SCHEDULING_METHOD_PROOF_INVALID",
                    &record.id,
                ));
            }
        }
    }
    let expected_ids: BTreeSet<&str> = SCHEDULING_SOURCE_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new(
            "SCHEDULING_CATALOG_RECORD_SET_INVALID",
        ));
    }
    if full_count != V4_NEW_FULL_METHOD_COUNT
        || source_only_count != V4_NEW_SOURCE_ONLY_METHOD_COUNT
        || full_count + source_only_count != 33
    {
        return Err(ValidationError::new(
            "SCHEDULING_METHOD_PROOF_COUNT_INVALID",
        ));
    }
    Ok(())
}

fn task_invalid(code: &str, id: &str) -> ValidationError {
    ValidationError::record(code, id)
}

fn validate_task_record(
    wrapper: &SchedulingSourceRecord,
    expected: &ExpectedTaskRecord,
) -> Result<(), ValidationError> {
    let record = &wrapper.record;
    if record.unit_id.is_some()
        || record.namespace_context.as_deref() != Some("global (observed)")
        || record.original_path != expected.original_path
        || record.original_size != expected.original_size
        || record.original_sha256 != expected.original_sha256
        || record.hash_basis != "raw_source_file_bytes"
    {
        return Err(task_invalid("TASK_SOURCE_IDENTITY_INVALID", &record.id));
    }
    if record.authority_id != MANAGED_AUTHORITY_ID
        || record.authority_binary_sha256.as_deref() != Some(MANAGED_AUTHORITY_SHA256)
        || record.authority_sha256_kind != "managed_assembly"
        || record.authority_mvid.as_deref() != Some(MANAGED_MVID)
        || record.build_reference.is_some()
        || record.type_def.as_deref() != Some(expected.root_type)
    {
        return Err(task_invalid("TASK_AUTHORITY_INVALID", &record.id));
    }
    if !matches!(record.evidence_status, EvidenceStatus::Unknown)
        || !matches!(record.source_status, SourceStatus::SourceOnly)
        || record.representation != "MIXED"
        || !matches!(record.curation_status, CurationStatus::NeedsEditorialReview)
        || !matches!(
            record.export_status,
            ExportStatus::PublicationReviewRequired
        )
    {
        return Err(task_invalid("TASK_STATUS_INVALID", &record.id));
    }
    if record.verification_basis.trim().is_empty()
        || record.proposed_path.as_deref() != Some(expected.proposed_path)
        || record.proposed_path_reason.trim().is_empty()
    {
        return Err(task_invalid("TASK_DESTINATION_INVALID", &record.id));
    }
    if record.scope.trim().is_empty() || record.completeness.trim().is_empty() {
        return Err(task_invalid("TASK_INCOMPLETE_SCOPE", &record.id));
    }
    if record.duplicate_group.is_some()
        || !record.supersession.same_authority
        || record.supersession.qualified
        || !record.supersession.scope.is_empty()
        || !record.supersession.supersedes.is_empty()
        || !record.supersession.superseded_by.is_empty()
    {
        return Err(task_invalid("TASK_SUPERSESSION_INVALID", &record.id));
    }
    if record.relationship.trim().is_empty()
        || record.explanation.trim().is_empty()
        || record.dependencies.is_empty()
        || record.editorial_changes.is_empty()
    {
        return Err(task_invalid("TASK_INCOMPLETE_RECORD", &record.id));
    }
    if record.validation.source_hash != "RECORDED_SUPPLIED_SOURCE_BYTES"
        || record.validation.authority_identity != "MANAGED_SHA_MVID_TOKEN_SCOPE_RECORDED"
        || record.validation.path_collision != "PRIMARY_FINAL_CHECK_PENDING"
        || record.validation.equivalence != "NOT_ESTABLISHED"
        || record.validation.compilation != "NOT_BUILDABLE_WITH_CURRENT_CONTEXT"
        || record.validation.runtime != "PARKED_NOT_RUN"
        || record.validation.publication != "REVIEW_REQUIRED"
        || record.validation.final_receipt != "PRIMARY_FINAL_RECEIPT_PENDING"
    {
        return Err(task_invalid("TASK_VALIDATION_STATUS_INVALID", &record.id));
    }
    if !record.methods.historical.is_empty() {
        return Err(task_invalid("TASK_HISTORICAL_NOT_EMPTY", &record.id));
    }
    let (raw_methods, raw_types) = expected_task_raw_paths(&record.id)
        .ok_or_else(|| task_invalid("TASK_RECORD_ID_INVALID", &record.id))?;
    let record_references = scheduling_reference_map(&record.evidence_references, &record.id)?;
    for path in [
        expected.original_path,
        PERF02_EVIDENCE_PATH,
        PERF02_RAW_PATH,
        PERF02_MANIFEST_PATH,
        PERF02_REPORT_PATH,
        PERF_TASK01_PATH,
        raw_methods,
        raw_types,
    ] {
        if !record_references.contains_key(path) {
            return Err(task_invalid("TASK_EVIDENCE_REFERENCE_INVALID", &record.id));
        }
    }

    let expected_methods: BTreeMap<&str, &ExpectedTaskMethod> = expected
        .methods
        .iter()
        .map(|method| (method.method_def, method))
        .collect();
    if record.methods.current.len() != expected_methods.len() {
        return Err(task_invalid("TASK_METHOD_SET_INVALID", &record.id));
    }
    let mut seen = BTreeSet::new();
    let mut full = 0_usize;
    let mut source_only = 0_usize;
    let mut metadata = 0_usize;
    for method in &record.methods.current {
        let method_expected = expected_methods
            .get(method.method_def.as_str())
            .ok_or_else(|| task_invalid("TASK_METHOD_SET_INVALID", &record.id))?;
        if !seen.insert(method.method_def.clone()) {
            return Err(task_invalid("TASK_METHOD_SET_INVALID", &record.id));
        }
        if method.symbol_key.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
            || method.symbol_key.mvid != MANAGED_MVID
            || method.symbol_key.type_def != method_expected.type_def
            || method.symbol_key.method_def != method_expected.method_def
            || !validate_type_token(&method.symbol_key.type_def)
            || !validate_method_token(&method.symbol_key.method_def)
            || method.method_def != method_expected.method_def
            || method.rva.as_deref() != Some(method_expected.rva)
            || method.il_range.as_deref() != method_expected.il_range
        {
            return Err(task_invalid("TASK_METHOD_IDENTITY_INVALID", &record.id));
        }
        let (coverage, evidence_status, source_status) = match method_expected.proof {
            TaskProof::Full => (
                "DIRECT_CIL_FULL_SELECTED_BODY_PERF02",
                EvidenceStatus::DirectCilFull,
                SourceStatus::ReconstructedFromDirectCil,
            ),
            TaskProof::Source => (
                "SOURCE_ONLY_SELECTED_METHOD_PERF02",
                EvidenceStatus::SourceOnly,
                SourceStatus::SourceOnly,
            ),
            TaskProof::Metadata => (
                "ABSTRACT_SIGNATURE_METADATA_ONLY_PERF02",
                EvidenceStatus::MetadataOnly,
                SourceStatus::MetadataOnly,
            ),
        };
        if method.coverage != coverage
            || method.evidence_status != evidence_status
            || method.source_status != source_status
            || method.verification_basis.trim().is_empty()
        {
            return Err(task_invalid("TASK_METHOD_PROOF_INVALID", &record.id));
        }
        match method_expected.proof {
            TaskProof::Full => full += 1,
            TaskProof::Source => source_only += 1,
            TaskProof::Metadata => {
                metadata += 1;
                if method.rva.as_deref() != Some("0x0")
                    || method.il_range.is_some()
                    || !method
                        .verification_basis
                        .to_ascii_lowercase()
                        .contains("abstract")
                    || !method
                        .verification_basis
                        .to_ascii_lowercase()
                        .contains("no body")
                    || !method
                        .verification_basis
                        .to_ascii_lowercase()
                        .contains("code size 0")
                {
                    return Err(task_invalid("TASK_METADATA_BODY_INVENTED", &record.id));
                }
            }
        }
        let method_references = scheduling_reference_map(&method.evidence_references, &record.id)?;
        if !method_references.contains_key(expected.original_path) {
            return Err(task_invalid("TASK_EVIDENCE_REFERENCE_INVALID", &record.id));
        }
        if method_expected.proof == TaskProof::Metadata {
            if !method_references.contains_key(raw_types) {
                return Err(task_invalid("TASK_EVIDENCE_REFERENCE_INVALID", &record.id));
            }
        } else if !method_references.contains_key(raw_methods) {
            return Err(task_invalid("TASK_EVIDENCE_REFERENCE_INVALID", &record.id));
        }
        if method_expected.proof == TaskProof::Full
            && !method_references.contains_key(PERF02_RAW_PATH)
        {
            return Err(task_invalid("TASK_FULL_CIL_REFERENCE_MISSING", &record.id));
        }
    }
    if full + source_only + metadata != expected_methods.len() {
        return Err(task_invalid("TASK_METHOD_SET_INVALID", &record.id));
    }

    let expected_associated: BTreeMap<&str, &ExpectedAssociatedType> = expected
        .associated_types
        .iter()
        .map(|associated| (associated.type_def, associated))
        .collect();
    if wrapper.associated_types.len() != expected_associated.len() {
        return Err(task_invalid("TASK_ASSOCIATED_TYPE_SET_INVALID", &record.id));
    }
    let mut associated_seen = BTreeSet::new();
    for associated in &wrapper.associated_types {
        let expected_type = expected_associated
            .get(associated.type_def.as_str())
            .ok_or_else(|| task_invalid("TASK_ASSOCIATED_TYPE_SET_INVALID", &record.id))?;
        if associated.type_def == expected.root_type
            || associated.type_def == associated.parent_type_def
            || !validate_type_token(&associated.type_def)
            || !validate_type_token(&associated.parent_type_def)
            || associated.parent_type_def != expected_type.parent_type_def
            || !associated_seen.insert(associated.type_def.clone())
        {
            return Err(task_invalid("TASK_ASSOCIATED_TYPE_SET_INVALID", &record.id));
        }
        let associated_references =
            scheduling_reference_map(&associated.evidence_references, &record.id)?;
        if !associated_references.contains_key(raw_types) {
            return Err(task_invalid("TASK_EVIDENCE_REFERENCE_INVALID", &record.id));
        }
    }
    if associated_seen.len() != expected_associated.len() {
        return Err(task_invalid("TASK_ASSOCIATED_TYPE_SET_INVALID", &record.id));
    }
    Ok(())
}

fn validate_task_execution_source(
    catalog: &TaskExecutionSourceCatalog,
    baseline: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
) -> Result<(), ValidationError> {
    if catalog.schema != "wuwa-reconstruction-task-execution-source-v1"
        || catalog.version != 1
        || catalog.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || catalog.performance_source_sha256 != PERFORMANCE_SOURCE_V1_SHA256
        || catalog.scheduling_source_sha256 != SCHEDULING_SOURCE_V1_SHA256
    {
        return Err(ValidationError::new("TASK_CATALOG_ROOT_INVALID"));
    }
    if catalog.records.len() != TASK_EXECUTION_SOURCE_IDS.len() {
        return Err(ValidationError::new("TASK_CATALOG_RECORD_SET_INVALID"));
    }

    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    for record in &baseline.records {
        original_paths.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            destinations.insert(path.to_ascii_lowercase());
        }
        qualified_methods.extend(qualified_key_set(record));
    }
    for record in performance_catalog.records.iter().chain(
        scheduling_catalog
            .records
            .iter()
            .map(|wrapper| &wrapper.record),
    ) {
        original_paths.insert(record.original_path.to_ascii_lowercase());
        if let Some(path) = &record.proposed_path {
            destinations.insert(path.to_ascii_lowercase());
        }
        qualified_methods.extend(qualified_key_set(record));
    }

    let mut ids = BTreeSet::new();
    let mut full_count = 0_usize;
    let mut source_only_count = 0_usize;
    let mut metadata_count = 0_usize;
    let mut method_count = 0_usize;
    for wrapper in &catalog.records {
        let record = &wrapper.record;
        if !ids.insert(record.id.clone())
            || record_by_id(baseline, &record.id).is_some()
            || performance_catalog
                .records
                .iter()
                .any(|existing| existing.id == record.id)
            || scheduling_catalog
                .records
                .iter()
                .any(|existing| existing.record.id == record.id)
        {
            return Err(task_invalid("TASK_RECORD_ID_INVALID", &record.id));
        }
        let expected = expected_task_record(&record.id)
            .ok_or_else(|| task_invalid("TASK_RECORD_ID_INVALID", &record.id))?;
        validate_task_record(wrapper, &expected)?;
        if !original_paths.insert(record.original_path.to_ascii_lowercase()) {
            return Err(task_invalid("TASK_SOURCE_PATH_COLLISION", &record.id));
        }
        let proposed = record
            .proposed_path
            .as_deref()
            .ok_or_else(|| task_invalid("TASK_DESTINATION_INVALID", &record.id))?;
        if !destinations.insert(proposed.to_ascii_lowercase()) {
            return Err(task_invalid(
                "TASK_DESTINATION_CASEFOLD_COLLISION",
                &record.id,
            ));
        }
        for method in &record.methods.current {
            method_count += 1;
            if !qualified_methods.insert(method_key(&method.symbol_key)) {
                return Err(task_invalid("TASK_METHOD_OVERLAP", &record.id));
            }
            match method.evidence_status {
                EvidenceStatus::DirectCilFull => full_count += 1,
                EvidenceStatus::SourceOnly => source_only_count += 1,
                EvidenceStatus::MetadataOnly => metadata_count += 1,
                _ => return Err(task_invalid("TASK_METHOD_PROOF_INVALID", &record.id)),
            }
        }
    }
    let expected_ids: BTreeSet<&str> = TASK_EXECUTION_SOURCE_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("TASK_CATALOG_RECORD_SET_INVALID"));
    }
    if method_count != 51
        || full_count != V5_NEW_FULL_METHOD_COUNT
        || source_only_count != V5_NEW_SOURCE_ONLY_METHOD_COUNT
        || metadata_count != V5_NEW_METADATA_ONLY_METHOD_COUNT
    {
        return Err(ValidationError::new("TASK_METHOD_PROOF_COUNT_INVALID"));
    }
    Ok(())
}

fn v2_entry_id(entry: &ImportEntryV2) -> &str {
    match entry {
        ImportEntryV2::Event(entry) => &entry.id,
        ImportEntryV2::Mapped(entry) => &entry.id,
    }
}

fn v2_entry_methods(entry: &ImportEntryV2) -> Vec<&MethodIdentity> {
    match entry {
        ImportEntryV2::Event(entry) => vec![&entry.method_identity],
        ImportEntryV2::Mapped(entry) => entry
            .methods
            .iter()
            .map(|method| &method.method_identity)
            .collect(),
    }
}

fn v2_receipt_statuses_valid(entry: &MappedImportEntry) -> bool {
    matches!(entry.exact_byte_comparison, VerificationResult::Pass)
        && matches!(
            entry.editorial_status,
            ReceiptEditorialStatus::NeedsEditorialReview
        )
        && matches!(entry.compilation_status, ReceiptCompilationStatus::NotRun)
        && matches!(
            entry.equivalence_status,
            ReceiptEquivalenceStatus::NotEstablished
        )
        && matches!(entry.publication_status, ReceiptPublicationStatus::Disabled)
        && matches!(entry.runtime_status, ReceiptRuntimeStatus::Parked)
}

fn v2_invalid(code: &str, id: &str) -> ValidationError {
    ValidationError::record(code, id)
}

fn validate_v2_method_shape(identity: &MethodIdentity, id: &str) -> Result<(), ValidationError> {
    if !valid_lower_hex(&identity.full_authority_sha256, 64)
        || !valid_uuid(&identity.mvid)
        || !validate_type_token(&identity.type_def)
        || !validate_method_token(&identity.method_def)
        || identity.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
        || identity.mvid != MANAGED_MVID
    {
        return Err(v2_invalid("V2_METHOD_IDENTITY_INVALID", id));
    }
    Ok(())
}

pub fn validate_receipt_v2(
    receipt: &ImportReceiptV2,
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
) -> Result<(), ValidationError> {
    if receipt.schema != "wuwa-reconstruction-imports-v2"
        || receipt.version != 2
        || receipt.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || receipt.previous_receipt_sha256 != BASELINE_RECEIPT_V1_SHA256
    {
        return Err(ValidationError::new("V2_ROOT_INVALID"));
    }

    let mut ids = BTreeSet::new();
    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        if !ids.insert(id.to_owned()) {
            return Err(v2_invalid("V2_DUPLICATE_IMPORT_ID", id));
        }
        if !REQUIRED_V2_IMPORT_IDS.contains(&id) {
            return Err(v2_invalid("V2_UNAUTHORIZED_IMPORT_ID", id));
        }

        let (authority_id, original_path, imported_path, original_sha256, imported_sha256) =
            match entry {
                ImportEntryV2::Event(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
                ImportEntryV2::Mapped(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
            };
        for path in [original_path, imported_path] {
            validate_relative_path(path)
                .map_err(|_| ValidationError::at("V2_PATH_INVALID", path).with_record(id))?;
        }
        if !original_paths.insert(original_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V2_SOURCE_PATH_COLLISION", id));
        }
        if !destinations.insert(imported_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V2_DESTINATION_CASEFOLD_COLLISION", id));
        }
        if !valid_lower_hex(original_sha256, 64) || !valid_lower_hex(imported_sha256, 64) {
            return Err(v2_invalid("V2_HASH_INVALID", id));
        }
        if !valid_authority_syntax(authority_id) {
            return Err(v2_invalid("V2_AUTHORITY_INVALID", id));
        }
        for identity in v2_entry_methods(entry) {
            validate_v2_method_shape(identity, id)?;
            if !qualified_methods.insert(method_key(identity)) {
                return Err(v2_invalid("V2_DUPLICATE_QUALIFIED_METHOD", id));
            }
        }
    }
    let expected_ids: BTreeSet<&str> = REQUIRED_V2_IMPORT_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("V2_IMPORT_ID_SET_INVALID"));
    }

    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        match entry {
            ImportEntryV2::Event(_) if !REQUIRED_IMPORT_IDS.contains(&id) => {
                return Err(v2_invalid("V2_ENTRY_SHAPE_INVALID", id));
            }
            ImportEntryV2::Mapped(_) if !matches!(id, "D" | "T") => {
                return Err(v2_invalid("V2_ENTRY_SHAPE_INVALID", id));
            }
            _ => {}
        }
    }

    validate_receipt(historical_v1, manifest)
        .map_err(|_| ValidationError::new("V2_HISTORICAL_RECEIPT_INVALID"))?;
    let historical_by_id: BTreeMap<&str, &ImportEntry> = historical_v1
        .imports
        .iter()
        .map(|entry| (entry.id.as_str(), entry))
        .collect();
    let event_ids: BTreeSet<&str> = receipt
        .imports
        .iter()
        .filter_map(|entry| match entry {
            ImportEntryV2::Event(entry) => Some(entry.id.as_str()),
            ImportEntryV2::Mapped(_) => None,
        })
        .collect();
    let historical_event_ids: BTreeSet<&str> = historical_v1
        .imports
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    if event_ids != historical_event_ids {
        return Err(ValidationError::new("V2_HISTORICAL_EVENT_SET_INVALID"));
    }

    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        match entry {
            ImportEntryV2::Event(entry) => {
                if !REQUIRED_IMPORT_IDS.contains(&id) {
                    return Err(v2_invalid("V2_ENTRY_SHAPE_INVALID", id));
                }
                let historical = historical_by_id
                    .get(id)
                    .ok_or_else(|| v2_invalid("V2_HISTORICAL_EVENT_MISSING", id))?;
                if entry != *historical {
                    return Err(v2_invalid("V2_EVENT_ENTRY_MISMATCH", id));
                }
            }
            ImportEntryV2::Mapped(entry) => {
                if !matches!(id, "D" | "T") {
                    return Err(v2_invalid("V2_ENTRY_SHAPE_INVALID", id));
                }
                let record = record_by_id(manifest, id)
                    .ok_or_else(|| v2_invalid("V2_MANIFEST_RECORD_MISSING", id))?;
                if record.proposed_path.is_none()
                    || entry.authority_id != record.authority_id
                    || entry.original_path != record.original_path
                    || entry.imported_path != record.proposed_path.clone().unwrap_or_default()
                    || entry.original_size != record.original_size
                    || entry.original_sha256 != record.original_sha256
                    || entry.imported_size != record.original_size
                    || entry.imported_sha256 != record.original_sha256
                    || entry.implementation_scope != record.scope
                    || entry.source_completeness != record.completeness
                    || entry.unresolved_dependencies != record.dependencies
                    || !v2_receipt_statuses_valid(entry)
                {
                    return Err(v2_invalid("V2_ENTRY_CORRESPONDENCE_INVALID", id));
                }
                if entry.evidence_status != record.evidence_status
                    || entry.source_status != record.source_status
                {
                    return Err(v2_invalid("V2_ARTIFACT_STATUS_MISMATCH", id));
                }
                if !matches!(
                    (id, &record.curation_status, &record.export_status),
                    (
                        "D",
                        CurationStatus::NeedsProvenanceReview,
                        ExportStatus::PublicationReviewRequired
                    ) | (
                        "T",
                        CurationStatus::PartialOrSourceOnly,
                        ExportStatus::PublicationReviewRequired
                    )
                ) {
                    return Err(v2_invalid("V2_MANIFEST_STATUS_INVALID", id));
                }

                let expected_methods: BTreeMap<String, &CurrentMethod> = record
                    .methods
                    .current
                    .iter()
                    .map(|method| (method_key(&method.symbol_key), method))
                    .collect();
                let actual_method_keys: BTreeSet<String> = entry
                    .methods
                    .iter()
                    .map(|method| method_key(&method.method_identity))
                    .collect();
                let expected_method_keys: BTreeSet<String> =
                    expected_methods.keys().cloned().collect();
                let expected_method_defs: BTreeSet<&str> = match id {
                    "D" => V2_D_METHOD_DEFS.into_iter().collect(),
                    "T" => V2_T_METHOD_DEFS.into_iter().collect(),
                    _ => unreachable!("mapped v2 entry shape checked above"),
                };
                let manifest_method_defs: BTreeSet<&str> = record
                    .methods
                    .current
                    .iter()
                    .map(|method| method.method_def.as_str())
                    .collect();
                let actual_method_defs: BTreeSet<&str> = entry
                    .methods
                    .iter()
                    .map(|method| method.method_identity.method_def.as_str())
                    .collect();
                if manifest_method_defs != expected_method_defs {
                    return Err(v2_invalid("V2_MANIFEST_METHOD_SET_INVALID", id));
                }
                if actual_method_defs != expected_method_defs {
                    return Err(v2_invalid("V2_METHOD_SET_MISMATCH", id));
                }
                if actual_method_keys != expected_method_keys {
                    return Err(v2_invalid("V2_METHOD_SET_MISMATCH", id));
                }
                for method in &entry.methods {
                    let expected = expected_methods
                        .get(&method_key(&method.method_identity))
                        .expect("method set checked above");
                    if method.evidence_status != expected.evidence_status
                        || method.source_status != expected.source_status
                    {
                        return Err(v2_invalid("V2_METHOD_STATUS_MISMATCH", id));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_v3_method_shape(identity: &MethodIdentity, id: &str) -> Result<(), ValidationError> {
    if !valid_lower_hex(&identity.full_authority_sha256, 64)
        || !valid_uuid(&identity.mvid)
        || !validate_type_token(&identity.type_def)
        || !validate_method_token(&identity.method_def)
        || identity.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
        || identity.mvid != MANAGED_MVID
    {
        return Err(v2_invalid("V3_METHOD_IDENTITY_INVALID", id));
    }
    Ok(())
}

fn mapped_entry_matches_new_record(entry: &MappedImportEntry, record: &Record) -> bool {
    let Some(proposed_path) = record.proposed_path.as_deref() else {
        return false;
    };
    if entry.id != record.id
        || entry.authority_id != record.authority_id
        || entry.original_path != record.original_path
        || entry.imported_path != proposed_path
        || entry.original_size != record.original_size
        || entry.original_sha256 != record.original_sha256
        || entry.imported_size != record.original_size
        || entry.imported_sha256 != record.original_sha256
        || entry.implementation_scope != record.scope
        || entry.source_completeness != record.completeness
        || entry.unresolved_dependencies != record.dependencies
        || entry.evidence_status != record.evidence_status
        || entry.source_status != record.source_status
        || !matches!(entry.exact_byte_comparison, VerificationResult::Pass)
        || !matches!(
            entry.editorial_status,
            ReceiptEditorialStatus::NeedsEditorialReview
        )
        || !matches!(entry.compilation_status, ReceiptCompilationStatus::NotRun)
        || !matches!(
            entry.equivalence_status,
            ReceiptEquivalenceStatus::NotEstablished
        )
        || !matches!(entry.publication_status, ReceiptPublicationStatus::Disabled)
        || !matches!(entry.runtime_status, ReceiptRuntimeStatus::Parked)
        || !matches!(record.curation_status, CurationStatus::NeedsEditorialReview)
        || !matches!(
            record.export_status,
            ExportStatus::PublicationReviewRequired
        )
    {
        return false;
    }
    if entry.methods.len() != record.methods.current.len() {
        return false;
    }
    let expected: BTreeMap<String, &CurrentMethod> = record
        .methods
        .current
        .iter()
        .map(|method| (method_key(&method.symbol_key), method))
        .collect();
    let mut actual = BTreeSet::new();
    for method in &entry.methods {
        let key = method_key(&method.method_identity);
        let Some(expected_method) = expected.get(&key) else {
            return false;
        };
        if !actual.insert(key)
            || method.evidence_status != expected_method.evidence_status
            || method.source_status != expected_method.source_status
        {
            return false;
        }
    }
    actual.len() == expected.len() && actual == expected.keys().cloned().collect()
}

pub fn validate_receipt_v3(
    receipt: &ImportReceiptV3,
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    catalog: &PerformanceSourceCatalog,
) -> Result<(), ValidationError> {
    if receipt.schema != "wuwa-reconstruction-imports-v3"
        || receipt.version != 3
        || receipt.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || receipt.previous_receipt_sha256 != BASELINE_RECEIPT_V2_SHA256
        || receipt.performance_source_sha256 != PERFORMANCE_SOURCE_V1_SHA256
    {
        return Err(ValidationError::new("V3_ROOT_INVALID"));
    }
    validate_receipt(historical_v1, manifest)
        .map_err(|_| ValidationError::new("V3_HISTORICAL_V1_INVALID"))?;
    validate_receipt_v2(historical_v2, manifest, historical_v1)
        .map_err(|_| ValidationError::new("V3_HISTORICAL_V2_INVALID"))?;
    validate_performance_source(catalog, manifest)
        .map_err(|_| ValidationError::new("V3_PERFORMANCE_SOURCE_INVALID"))?;

    let historical_by_id: BTreeMap<&str, &ImportEntryV2> = historical_v2
        .imports
        .iter()
        .map(|entry| (v2_entry_id(entry), entry))
        .collect();
    let catalog_by_id: BTreeMap<&str, &Record> = catalog
        .records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect();

    let mut ids = BTreeSet::new();
    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    let mut method_total = 0_usize;
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        if !ids.insert(id.to_owned()) {
            return Err(v2_invalid("V3_DUPLICATE_IMPORT_ID", id));
        }
        if !REQUIRED_V3_IMPORT_IDS.contains(&id) {
            return Err(v2_invalid("V3_UNAUTHORIZED_IMPORT_ID", id));
        }
        let (authority_id, original_path, imported_path, original_sha256, imported_sha256) =
            match entry {
                ImportEntryV2::Event(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
                ImportEntryV2::Mapped(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
            };
        for path in [original_path, imported_path] {
            validate_relative_path(path)
                .map_err(|_| ValidationError::at("V3_PATH_INVALID", path).with_record(id))?;
        }
        if !original_paths.insert(original_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V3_SOURCE_PATH_COLLISION", id));
        }
        if !destinations.insert(imported_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V3_DESTINATION_CASEFOLD_COLLISION", id));
        }
        if !valid_lower_hex(original_sha256, 64) || !valid_lower_hex(imported_sha256, 64) {
            return Err(v2_invalid("V3_HASH_INVALID", id));
        }
        if !valid_authority_syntax(authority_id) {
            return Err(v2_invalid("V3_AUTHORITY_INVALID", id));
        }
        for identity in v2_entry_methods(entry) {
            validate_v3_method_shape(identity, id)?;
            if !qualified_methods.insert(method_key(identity)) {
                return Err(v2_invalid("V3_DUPLICATE_QUALIFIED_METHOD", id));
            }
            method_total += 1;
        }
    }
    let expected_ids: BTreeSet<&str> = REQUIRED_V3_IMPORT_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("V3_IMPORT_ID_SET_INVALID"));
    }

    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        match entry {
            ImportEntryV2::Event(entry) => {
                if !REQUIRED_IMPORT_IDS.contains(&id) {
                    return Err(v2_invalid("V3_ENTRY_SHAPE_INVALID", id));
                }
                let Some(ImportEntryV2::Event(historical)) = historical_by_id.get(id) else {
                    return Err(v2_invalid("V3_HISTORICAL_ENTRY_MISSING", id));
                };
                if entry != historical {
                    return Err(v2_invalid("V3_HISTORICAL_ENTRY_MISMATCH", id));
                }
            }
            ImportEntryV2::Mapped(entry) => {
                if matches!(id, "D" | "T") {
                    let Some(ImportEntryV2::Mapped(historical)) = historical_by_id.get(id) else {
                        return Err(v2_invalid("V3_HISTORICAL_ENTRY_MISSING", id));
                    };
                    if entry != historical {
                        return Err(v2_invalid("V3_HISTORICAL_ENTRY_MISMATCH", id));
                    }
                } else if matches!(id, "PTIMER" | "PTIMER_INSTANCE") {
                    let Some(record) = catalog_by_id.get(id) else {
                        return Err(v2_invalid("V3_PERFORMANCE_SOURCE_RECORD_MISSING", id));
                    };
                    if !mapped_entry_matches_new_record(entry, record) {
                        return Err(v2_invalid("V3_ENTRY_CORRESPONDENCE_INVALID", id));
                    }
                } else {
                    return Err(v2_invalid("V3_ENTRY_SHAPE_INVALID", id));
                }
            }
        }
    }
    if method_total != V3_QUALIFIED_METHOD_COUNT
        || qualified_methods.len() != V3_QUALIFIED_METHOD_COUNT
    {
        return Err(ValidationError::new("V3_METHOD_COUNT_INVALID"));
    }
    Ok(())
}

fn validate_v4_method_shape(identity: &MethodIdentity, id: &str) -> Result<(), ValidationError> {
    if !valid_lower_hex(&identity.full_authority_sha256, 64)
        || !valid_uuid(&identity.mvid)
        || !validate_type_token(&identity.type_def)
        || !validate_method_token(&identity.method_def)
        || identity.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
        || identity.mvid != MANAGED_MVID
    {
        return Err(v2_invalid("V4_METHOD_IDENTITY_INVALID", id));
    }
    Ok(())
}

pub fn validate_receipt_v4(
    receipt: &ImportReceiptV4,
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    historical_v3: &ImportReceiptV3,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
) -> Result<(), ValidationError> {
    if receipt.schema != "wuwa-reconstruction-imports-v4"
        || receipt.version != 4
        || receipt.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || receipt.previous_receipt_sha256 != BASELINE_RECEIPT_V3_SHA256
        || receipt.performance_source_sha256 != PERFORMANCE_SOURCE_V1_SHA256
        || receipt.scheduling_source_sha256 != SCHEDULING_SOURCE_V1_SHA256
    {
        return Err(ValidationError::new("V4_ROOT_INVALID"));
    }
    validate_receipt(historical_v1, manifest)
        .map_err(|_| ValidationError::new("V4_HISTORICAL_V1_INVALID"))?;
    validate_receipt_v2(historical_v2, manifest, historical_v1)
        .map_err(|_| ValidationError::new("V4_HISTORICAL_V2_INVALID"))?;
    validate_receipt_v3(
        historical_v3,
        manifest,
        historical_v1,
        historical_v2,
        performance_catalog,
    )
    .map_err(|_| ValidationError::new("V4_HISTORICAL_V3_INVALID"))?;
    validate_scheduling_source(scheduling_catalog, manifest, performance_catalog)
        .map_err(|_| ValidationError::new("V4_SCHEDULING_SOURCE_INVALID"))?;

    let historical_by_id: BTreeMap<&str, &ImportEntryV2> = historical_v3
        .imports
        .iter()
        .map(|entry| (v2_entry_id(entry), entry))
        .collect();
    let scheduling_by_id: BTreeMap<&str, &Record> = scheduling_catalog
        .records
        .iter()
        .map(|wrapper| (wrapper.record.id.as_str(), &wrapper.record))
        .collect();

    let mut ids = BTreeSet::new();
    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    let mut method_total = 0_usize;
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        if !ids.insert(id.to_owned()) {
            return Err(v2_invalid("V4_DUPLICATE_IMPORT_ID", id));
        }
        if !REQUIRED_V4_IMPORT_IDS.contains(&id) {
            return Err(v2_invalid("V4_UNAUTHORIZED_IMPORT_ID", id));
        }
        let (authority_id, original_path, imported_path, original_sha256, imported_sha256) =
            match entry {
                ImportEntryV2::Event(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
                ImportEntryV2::Mapped(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
            };
        for path in [original_path, imported_path] {
            validate_relative_path(path)
                .map_err(|_| ValidationError::at("V4_PATH_INVALID", path).with_record(id))?;
        }
        if !original_paths.insert(original_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V4_SOURCE_PATH_COLLISION", id));
        }
        if !destinations.insert(imported_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V4_DESTINATION_CASEFOLD_COLLISION", id));
        }
        if !valid_lower_hex(original_sha256, 64) || !valid_lower_hex(imported_sha256, 64) {
            return Err(v2_invalid("V4_HASH_INVALID", id));
        }
        if !valid_authority_syntax(authority_id) {
            return Err(v2_invalid("V4_AUTHORITY_INVALID", id));
        }
        for identity in v2_entry_methods(entry) {
            validate_v4_method_shape(identity, id)?;
            if !qualified_methods.insert(method_key(identity)) {
                return Err(v2_invalid("V4_DUPLICATE_QUALIFIED_METHOD", id));
            }
            method_total += 1;
        }
    }
    let expected_ids: BTreeSet<&str> = REQUIRED_V4_IMPORT_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("V4_IMPORT_ID_SET_INVALID"));
    }

    let mut new_full_count = 0_usize;
    let mut new_source_only_count = 0_usize;
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        match entry {
            ImportEntryV2::Event(entry) => {
                if !REQUIRED_IMPORT_IDS.contains(&id) {
                    return Err(v2_invalid("V4_ENTRY_SHAPE_INVALID", id));
                }
                let Some(ImportEntryV2::Event(historical)) = historical_by_id.get(id) else {
                    return Err(v2_invalid("V4_HISTORICAL_ENTRY_MISSING", id));
                };
                if entry != historical {
                    return Err(v2_invalid("V4_HISTORICAL_ENTRY_MISMATCH", id));
                }
            }
            ImportEntryV2::Mapped(entry) => {
                if matches!(id, "D" | "T" | "PTIMER" | "PTIMER_INSTANCE") {
                    let Some(ImportEntryV2::Mapped(historical)) = historical_by_id.get(id) else {
                        return Err(v2_invalid("V4_HISTORICAL_ENTRY_MISSING", id));
                    };
                    if entry != historical {
                        return Err(v2_invalid("V4_HISTORICAL_ENTRY_MISMATCH", id));
                    }
                } else if matches!(id, "PSCHED_GRAPH" | "PSCHED_UTILS" | "PSCHED_HEAP") {
                    let Some(record) = scheduling_by_id.get(id) else {
                        return Err(v2_invalid("V4_SCHEDULING_RECORD_MISSING", id));
                    };
                    if !mapped_entry_matches_new_record(entry, record) {
                        return Err(v2_invalid("V4_ENTRY_CORRESPONDENCE_INVALID", id));
                    }
                    for method in &entry.methods {
                        if matches!(method.evidence_status, EvidenceStatus::DirectCilFull) {
                            new_full_count += 1;
                        } else if matches!(method.evidence_status, EvidenceStatus::SourceOnly) {
                            new_source_only_count += 1;
                        } else {
                            return Err(v2_invalid("V4_METHOD_STATUS_MISMATCH", id));
                        }
                    }
                } else {
                    return Err(v2_invalid("V4_ENTRY_SHAPE_INVALID", id));
                }
            }
        }
    }
    if method_total != V4_QUALIFIED_METHOD_COUNT
        || qualified_methods.len() != V4_QUALIFIED_METHOD_COUNT
        || new_full_count != V4_NEW_FULL_METHOD_COUNT
        || new_source_only_count != V4_NEW_SOURCE_ONLY_METHOD_COUNT
    {
        return Err(ValidationError::new("V4_METHOD_COUNT_INVALID"));
    }
    Ok(())
}

fn validate_v5_method_shape(identity: &MethodIdentity, id: &str) -> Result<(), ValidationError> {
    if !valid_lower_hex(&identity.full_authority_sha256, 64)
        || !valid_uuid(&identity.mvid)
        || !validate_type_token(&identity.type_def)
        || !validate_method_token(&identity.method_def)
        || identity.full_authority_sha256 != MANAGED_AUTHORITY_SHA256
        || identity.mvid != MANAGED_MVID
    {
        return Err(v2_invalid("V5_METHOD_IDENTITY_INVALID", id));
    }
    Ok(())
}

pub fn validate_receipt_v5(
    receipt: &ImportReceiptV5,
    manifest: &Manifest,
    historical_v1: &ImportReceipt,
    historical_v2: &ImportReceiptV2,
    historical_v3: &ImportReceiptV3,
    historical_v4: &ImportReceiptV4,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
    task_catalog: &TaskExecutionSourceCatalog,
) -> Result<(), ValidationError> {
    if receipt.schema != "wuwa-reconstruction-imports-v5"
        || receipt.version != 5
        || receipt.manifest_sha256 != BASELINE_MANIFEST_SHA256
        || receipt.previous_receipt_sha256 != BASELINE_RECEIPT_V4_SHA256
        || receipt.performance_source_sha256 != PERFORMANCE_SOURCE_V1_SHA256
        || receipt.scheduling_source_sha256 != SCHEDULING_SOURCE_V1_SHA256
        || receipt.task_execution_source_sha256 != TASK_EXECUTION_SOURCE_V1_SHA256
    {
        return Err(ValidationError::new("V5_ROOT_INVALID"));
    }
    validate_receipt(historical_v1, manifest)
        .map_err(|_| ValidationError::new("V5_HISTORICAL_V1_INVALID"))?;
    validate_receipt_v2(historical_v2, manifest, historical_v1)
        .map_err(|_| ValidationError::new("V5_HISTORICAL_V2_INVALID"))?;
    validate_receipt_v3(
        historical_v3,
        manifest,
        historical_v1,
        historical_v2,
        performance_catalog,
    )
    .map_err(|_| ValidationError::new("V5_HISTORICAL_V3_INVALID"))?;
    validate_receipt_v4(
        historical_v4,
        manifest,
        historical_v1,
        historical_v2,
        historical_v3,
        performance_catalog,
        scheduling_catalog,
    )
    .map_err(|_| ValidationError::new("V5_HISTORICAL_V4_INVALID"))?;
    validate_task_execution_source(
        task_catalog,
        manifest,
        performance_catalog,
        scheduling_catalog,
    )
    .map_err(|_| ValidationError::new("V5_TASK_SOURCE_INVALID"))?;

    let historical_by_id: BTreeMap<&str, &ImportEntryV2> = historical_v4
        .imports
        .iter()
        .map(|entry| (v2_entry_id(entry), entry))
        .collect();
    let task_by_id: BTreeMap<&str, &Record> = task_catalog
        .records
        .iter()
        .map(|wrapper| (wrapper.record.id.as_str(), &wrapper.record))
        .collect();

    let mut ids = BTreeSet::new();
    let mut original_paths = BTreeSet::new();
    let mut destinations = BTreeSet::new();
    let mut qualified_methods = BTreeSet::new();
    let mut method_total = 0_usize;
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        if !ids.insert(id.to_owned()) {
            return Err(v2_invalid("V5_DUPLICATE_IMPORT_ID", id));
        }
        if !REQUIRED_V5_IMPORT_IDS.contains(&id) {
            return Err(v2_invalid("V5_UNAUTHORIZED_IMPORT_ID", id));
        }
        let (authority_id, original_path, imported_path, original_sha256, imported_sha256) =
            match entry {
                ImportEntryV2::Event(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
                ImportEntryV2::Mapped(entry) => (
                    entry.authority_id.as_str(),
                    entry.original_path.as_str(),
                    entry.imported_path.as_str(),
                    entry.original_sha256.as_str(),
                    entry.imported_sha256.as_str(),
                ),
            };
        for path in [original_path, imported_path] {
            validate_relative_path(path)
                .map_err(|_| ValidationError::at("V5_PATH_INVALID", path).with_record(id))?;
        }
        if !original_paths.insert(original_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V5_SOURCE_PATH_COLLISION", id));
        }
        if !destinations.insert(imported_path.to_ascii_lowercase()) {
            return Err(v2_invalid("V5_DESTINATION_CASEFOLD_COLLISION", id));
        }
        if !valid_lower_hex(original_sha256, 64) || !valid_lower_hex(imported_sha256, 64) {
            return Err(v2_invalid("V5_HASH_INVALID", id));
        }
        if !valid_authority_syntax(authority_id) {
            return Err(v2_invalid("V5_AUTHORITY_INVALID", id));
        }
        for identity in v2_entry_methods(entry) {
            validate_v5_method_shape(identity, id)?;
            if !qualified_methods.insert(method_key(identity)) {
                return Err(v2_invalid("V5_DUPLICATE_QUALIFIED_METHOD", id));
            }
            method_total += 1;
        }
    }
    let expected_ids: BTreeSet<&str> = REQUIRED_V5_IMPORT_IDS.into_iter().collect();
    let actual_ids: BTreeSet<&str> = ids.iter().map(String::as_str).collect();
    if actual_ids != expected_ids {
        return Err(ValidationError::new("V5_IMPORT_ID_SET_INVALID"));
    }

    let mut new_full_count = 0_usize;
    let mut new_source_only_count = 0_usize;
    let mut new_metadata_only_count = 0_usize;
    for entry in &receipt.imports {
        let id = v2_entry_id(entry);
        match entry {
            ImportEntryV2::Event(entry) => {
                if !REQUIRED_IMPORT_IDS.contains(&id) {
                    return Err(v2_invalid("V5_ENTRY_SHAPE_INVALID", id));
                }
                let Some(ImportEntryV2::Event(historical)) = historical_by_id.get(id) else {
                    return Err(v2_invalid("V5_HISTORICAL_ENTRY_MISSING", id));
                };
                if entry != historical {
                    return Err(v2_invalid("V5_HISTORICAL_ENTRY_MISMATCH", id));
                }
            }
            ImportEntryV2::Mapped(entry) => {
                if matches!(
                    id,
                    "D" | "T"
                        | "PTIMER"
                        | "PTIMER_INSTANCE"
                        | "PSCHED_GRAPH"
                        | "PSCHED_UTILS"
                        | "PSCHED_HEAP"
                ) {
                    let Some(ImportEntryV2::Mapped(historical)) = historical_by_id.get(id) else {
                        return Err(v2_invalid("V5_HISTORICAL_ENTRY_MISSING", id));
                    };
                    if entry != historical {
                        return Err(v2_invalid("V5_HISTORICAL_ENTRY_MISMATCH", id));
                    }
                } else if matches!(id, "PEXEC_SYSTEM" | "PEXEC_BASE" | "PEXEC_QUEUE") {
                    let Some(record) = task_by_id.get(id) else {
                        return Err(v2_invalid("V5_TASK_RECORD_MISSING", id));
                    };
                    if !mapped_entry_matches_new_record(entry, record) {
                        return Err(v2_invalid("V5_ENTRY_CORRESPONDENCE_INVALID", id));
                    }
                    for method in &entry.methods {
                        match method.evidence_status {
                            EvidenceStatus::DirectCilFull => new_full_count += 1,
                            EvidenceStatus::SourceOnly => new_source_only_count += 1,
                            EvidenceStatus::MetadataOnly => new_metadata_only_count += 1,
                            _ => return Err(v2_invalid("V5_METHOD_STATUS_MISMATCH", id)),
                        }
                    }
                } else {
                    return Err(v2_invalid("V5_ENTRY_SHAPE_INVALID", id));
                }
            }
        }
    }
    if method_total != V5_QUALIFIED_METHOD_COUNT
        || qualified_methods.len() != V5_QUALIFIED_METHOD_COUNT
        || new_full_count != V5_NEW_FULL_METHOD_COUNT
        || new_source_only_count != V5_NEW_SOURCE_ONLY_METHOD_COUNT
        || new_metadata_only_count != V5_NEW_METADATA_ONLY_METHOD_COUNT
    {
        return Err(ValidationError::new("V5_METHOD_COUNT_INVALID"));
    }
    Ok(())
}

fn trusted_manifest_from_bytes(bytes: &[u8]) -> Result<Manifest, AppError> {
    let manifest = parse_manifest_bytes(bytes).map_err(AppError::from_validation)?;
    if sha256_hex(bytes) != BASELINE_MANIFEST_SHA256 {
        return Err(AppError::schema("MANIFEST_ANCHOR_MISMATCH"));
    }
    Ok(manifest)
}

fn trusted_performance_source_from_bytes(
    bytes: &[u8],
    baseline: &Manifest,
) -> Result<PerformanceSourceCatalog, AppError> {
    let catalog =
        parse_performance_source_bytes(bytes, baseline).map_err(AppError::from_validation)?;
    if sha256_hex(bytes) != PERFORMANCE_SOURCE_V1_SHA256 {
        return Err(AppError::schema("CATALOG_ANCHOR_MISMATCH"));
    }
    Ok(catalog)
}

fn trusted_scheduling_source_from_bytes(
    bytes: &[u8],
    baseline: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
) -> Result<SchedulingSourceCatalog, AppError> {
    let catalog = parse_scheduling_source_bytes(bytes, baseline, performance_catalog)
        .map_err(AppError::from_validation)?;
    if sha256_hex(bytes) != SCHEDULING_SOURCE_V1_SHA256 {
        return Err(AppError::schema("SCHEDULING_ANCHOR_MISMATCH"));
    }
    Ok(catalog)
}

fn trusted_task_execution_source_from_bytes(
    bytes: &[u8],
    baseline: &Manifest,
    performance_catalog: &PerformanceSourceCatalog,
    scheduling_catalog: &SchedulingSourceCatalog,
) -> Result<TaskExecutionSourceCatalog, AppError> {
    let catalog =
        parse_task_execution_source_bytes(bytes, baseline, performance_catalog, scheduling_catalog)
            .map_err(AppError::from_validation)?;
    if sha256_hex(bytes) != TASK_EXECUTION_SOURCE_V1_SHA256 {
        return Err(AppError::schema("TASK_ANCHOR_MISMATCH"));
    }
    Ok(catalog)
}

fn read_required_relative(
    root: &Path,
    relative: &str,
    missing_code: &'static str,
    not_regular_code: &'static str,
) -> Result<Vec<u8>, AppError> {
    match inspect_relative_inner(root, relative)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, relative))?
    {
        SafePathState::Missing => Err(AppError::execution_at(missing_code, relative)),
        SafePathState::File => read_relative(root, relative)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, relative)),
        SafePathState::Directory => Err(AppError::execution_at(not_regular_code, relative)),
    }
}

impl ActiveReceipt {
    fn from_v1(receipt: &ImportReceipt) -> Self {
        Self {
            ledger_path: RECEIPT_REL.to_owned(),
            version: 1,
            files: receipt
                .imports
                .iter()
                .map(|entry| ImportedFileView {
                    id: entry.id.clone(),
                    original_path: entry.original_path.clone(),
                    imported: SourceArtifact {
                        path: entry.imported_path.clone(),
                        size: entry.imported_size,
                        sha256: entry.imported_sha256.clone(),
                    },
                    methods: vec![entry.method_identity.clone()],
                })
                .collect(),
        }
    }

    fn from_v2(receipt: &ImportReceiptV2) -> Self {
        Self {
            ledger_path: RECEIPT_V2_REL.to_owned(),
            version: 2,
            files: imported_files_from_v2_entries(&receipt.imports),
        }
    }

    fn from_v3(receipt: &ImportReceiptV3) -> Self {
        Self {
            ledger_path: RECEIPT_V3_REL.to_owned(),
            version: 3,
            files: imported_files_from_v2_entries(&receipt.imports),
        }
    }

    fn from_v4(receipt: &ImportReceiptV4) -> Self {
        Self {
            ledger_path: RECEIPT_V4_REL.to_owned(),
            version: 4,
            files: imported_files_from_v2_entries(&receipt.imports),
        }
    }

    fn from_v5(receipt: &ImportReceiptV5) -> Self {
        Self {
            ledger_path: RECEIPT_V5_REL.to_owned(),
            version: 5,
            files: imported_files_from_v2_entries(&receipt.imports),
        }
    }
}

fn imported_files_from_v2_entries(entries: &[ImportEntryV2]) -> Vec<ImportedFileView> {
    entries
        .iter()
        .map(|entry| match entry {
            ImportEntryV2::Event(entry) => ImportedFileView {
                id: entry.id.clone(),
                original_path: entry.original_path.clone(),
                imported: SourceArtifact {
                    path: entry.imported_path.clone(),
                    size: entry.imported_size,
                    sha256: entry.imported_sha256.clone(),
                },
                methods: vec![entry.method_identity.clone()],
            },
            ImportEntryV2::Mapped(entry) => ImportedFileView {
                id: entry.id.clone(),
                original_path: entry.original_path.clone(),
                imported: SourceArtifact {
                    path: entry.imported_path.clone(),
                    size: entry.imported_size,
                    sha256: entry.imported_sha256.clone(),
                },
                methods: entry
                    .methods
                    .iter()
                    .map(|method| method.method_identity.clone())
                    .collect(),
            },
        })
        .collect()
}

fn load_context_baseline(
    root: &Path,
) -> Result<(PathBuf, Manifest, Option<ActiveReceipt>), AppError> {
    let bytes = read_relative(&root, MANIFEST_REL)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, MANIFEST_REL))?;
    let manifest = trusted_manifest_from_bytes(&bytes)?;

    let v5_state = inspect_relative_inner(root, RECEIPT_V5_REL)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V5_REL))?;
    if v5_state != SafePathState::Missing {
        if v5_state != SafePathState::File {
            return Err(AppError::execution_at(
                "V5_RECEIPT_NOT_REGULAR",
                RECEIPT_V5_REL,
            ));
        }
        let historical_bytes = read_required_relative(
            root,
            RECEIPT_REL,
            "V5_HISTORICAL_RECEIPT_MISSING",
            "V5_HISTORICAL_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&historical_bytes) != BASELINE_RECEIPT_V1_SHA256 {
            return Err(AppError::schema("V5_HISTORICAL_RECEIPT_HASH_MISMATCH"));
        }
        let historical =
            parse_receipt_bytes(&historical_bytes, &manifest).map_err(AppError::from_validation)?;

        let previous_bytes = read_required_relative(
            root,
            RECEIPT_V2_REL,
            "V5_PREVIOUS_RECEIPT_MISSING",
            "V5_PREVIOUS_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&previous_bytes) != BASELINE_RECEIPT_V2_SHA256 {
            return Err(AppError::schema("V5_PREVIOUS_RECEIPT_HASH_MISMATCH"));
        }
        let previous = parse_receipt_v2_bytes(&previous_bytes, &manifest, &historical)
            .map_err(AppError::from_validation)?;

        let v3_bytes = read_required_relative(
            root,
            RECEIPT_V3_REL,
            "V5_V3_RECEIPT_MISSING",
            "V5_V3_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&v3_bytes) != BASELINE_RECEIPT_V3_SHA256 {
            return Err(AppError::schema("V5_V3_RECEIPT_HASH_MISMATCH"));
        }
        let performance_bytes = read_required_relative(
            root,
            PERFORMANCE_SOURCE_REL,
            "V5_PERFORMANCE_SOURCE_MISSING",
            "V5_PERFORMANCE_SOURCE_NOT_REGULAR",
        )?;
        let performance_catalog =
            trusted_performance_source_from_bytes(&performance_bytes, &manifest)?;
        let v3 = parse_receipt_v3_bytes(
            &v3_bytes,
            &manifest,
            &historical,
            &previous,
            &performance_catalog,
        )
        .map_err(AppError::from_validation)?;

        let scheduling_bytes = read_required_relative(
            root,
            SCHEDULING_SOURCE_REL,
            "V5_SCHEDULING_SOURCE_MISSING",
            "V5_SCHEDULING_SOURCE_NOT_REGULAR",
        )?;
        let scheduling_catalog = trusted_scheduling_source_from_bytes(
            &scheduling_bytes,
            &manifest,
            &performance_catalog,
        )?;

        let v4_bytes = read_required_relative(
            root,
            RECEIPT_V4_REL,
            "V5_V4_RECEIPT_MISSING",
            "V5_V4_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&v4_bytes) != BASELINE_RECEIPT_V4_SHA256 {
            return Err(AppError::schema("V5_V4_RECEIPT_HASH_MISMATCH"));
        }
        let v4 = parse_receipt_v4_bytes(
            &v4_bytes,
            &manifest,
            &historical,
            &previous,
            &v3,
            &performance_catalog,
            &scheduling_catalog,
        )
        .map_err(AppError::from_validation)?;

        let task_bytes = read_required_relative(
            root,
            TASK_EXECUTION_SOURCE_REL,
            "V5_TASK_SOURCE_MISSING",
            "V5_TASK_SOURCE_NOT_REGULAR",
        )?;
        let task_catalog = trusted_task_execution_source_from_bytes(
            &task_bytes,
            &manifest,
            &performance_catalog,
            &scheduling_catalog,
        )?;

        let bytes = read_relative(root, RECEIPT_V5_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V5_REL))?;
        let receipt = parse_receipt_v5_bytes(
            &bytes,
            &manifest,
            &historical,
            &previous,
            &v3,
            &v4,
            &performance_catalog,
            &scheduling_catalog,
            &task_catalog,
        )
        .map_err(AppError::from_validation)?;
        let mut combined = manifest;
        combined.records.extend(performance_catalog.records);
        combined.records.extend(
            scheduling_catalog
                .records
                .into_iter()
                .map(|wrapper| wrapper.record),
        );
        combined.records.extend(
            task_catalog
                .records
                .into_iter()
                .map(|wrapper| wrapper.record),
        );
        return Ok((
            root.to_path_buf(),
            combined,
            Some(ActiveReceipt::from_v5(&receipt)),
        ));
    }

    let v4_state = inspect_relative_inner(root, RECEIPT_V4_REL)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V4_REL))?;
    if v4_state != SafePathState::Missing {
        if v4_state != SafePathState::File {
            return Err(AppError::execution_at(
                "V4_RECEIPT_NOT_REGULAR",
                RECEIPT_V4_REL,
            ));
        }
        let historical_bytes = read_required_relative(
            root,
            RECEIPT_REL,
            "V4_HISTORICAL_RECEIPT_MISSING",
            "V4_HISTORICAL_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&historical_bytes) != BASELINE_RECEIPT_V1_SHA256 {
            return Err(AppError::schema("V4_HISTORICAL_RECEIPT_HASH_MISMATCH"));
        }
        let historical =
            parse_receipt_bytes(&historical_bytes, &manifest).map_err(AppError::from_validation)?;

        let previous_bytes = read_required_relative(
            root,
            RECEIPT_V2_REL,
            "V4_PREVIOUS_RECEIPT_MISSING",
            "V4_PREVIOUS_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&previous_bytes) != BASELINE_RECEIPT_V2_SHA256 {
            return Err(AppError::schema("V4_PREVIOUS_RECEIPT_HASH_MISMATCH"));
        }
        let previous = parse_receipt_v2_bytes(&previous_bytes, &manifest, &historical)
            .map_err(AppError::from_validation)?;

        let v3_bytes = read_required_relative(
            root,
            RECEIPT_V3_REL,
            "V4_V3_RECEIPT_MISSING",
            "V4_V3_RECEIPT_NOT_REGULAR",
        )?;
        if sha256_hex(&v3_bytes) != BASELINE_RECEIPT_V3_SHA256 {
            return Err(AppError::schema("V4_V3_RECEIPT_HASH_MISMATCH"));
        }
        let performance_bytes = read_required_relative(
            root,
            PERFORMANCE_SOURCE_REL,
            "V4_PERFORMANCE_SOURCE_MISSING",
            "V4_PERFORMANCE_SOURCE_NOT_REGULAR",
        )?;
        let performance_catalog =
            trusted_performance_source_from_bytes(&performance_bytes, &manifest)?;
        let v3 = parse_receipt_v3_bytes(
            &v3_bytes,
            &manifest,
            &historical,
            &previous,
            &performance_catalog,
        )
        .map_err(AppError::from_validation)?;

        let scheduling_bytes = read_required_relative(
            root,
            SCHEDULING_SOURCE_REL,
            "V4_SCHEDULING_SOURCE_MISSING",
            "V4_SCHEDULING_SOURCE_NOT_REGULAR",
        )?;
        let scheduling_catalog = trusted_scheduling_source_from_bytes(
            &scheduling_bytes,
            &manifest,
            &performance_catalog,
        )?;

        let bytes = read_relative(root, RECEIPT_V4_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V4_REL))?;
        let receipt = parse_receipt_v4_bytes(
            &bytes,
            &manifest,
            &historical,
            &previous,
            &v3,
            &performance_catalog,
            &scheduling_catalog,
        )
        .map_err(AppError::from_validation)?;
        let mut combined = manifest;
        combined.records.extend(performance_catalog.records);
        combined.records.extend(
            scheduling_catalog
                .records
                .into_iter()
                .map(|wrapper| wrapper.record),
        );
        return Ok((
            root.to_path_buf(),
            combined,
            Some(ActiveReceipt::from_v4(&receipt)),
        ));
    }

    let v3_state = inspect_relative_inner(root, RECEIPT_V3_REL)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V3_REL))?;
    if v3_state != SafePathState::Missing {
        if v3_state != SafePathState::File {
            return Err(AppError::execution_at(
                "V3_RECEIPT_NOT_REGULAR",
                RECEIPT_V3_REL,
            ));
        }
        let historical_state = inspect_relative_inner(root, RECEIPT_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?;
        let historical_bytes = match historical_state {
            SafePathState::Missing => {
                return Err(AppError::execution_at(
                    "V3_HISTORICAL_RECEIPT_MISSING",
                    RECEIPT_REL,
                ));
            }
            SafePathState::File => read_relative(root, RECEIPT_REL)
                .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?,
            SafePathState::Directory => {
                return Err(AppError::execution_at(
                    "V3_HISTORICAL_RECEIPT_NOT_REGULAR",
                    RECEIPT_REL,
                ));
            }
        };
        if sha256_hex(&historical_bytes) != BASELINE_RECEIPT_V1_SHA256 {
            return Err(AppError::schema("V3_HISTORICAL_RECEIPT_HASH_MISMATCH"));
        }
        let historical =
            parse_receipt_bytes(&historical_bytes, &manifest).map_err(AppError::from_validation)?;

        let previous_state = inspect_relative_inner(root, RECEIPT_V2_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V2_REL))?;
        let previous_bytes = match previous_state {
            SafePathState::Missing => {
                return Err(AppError::execution_at(
                    "V3_PREVIOUS_RECEIPT_MISSING",
                    RECEIPT_V2_REL,
                ));
            }
            SafePathState::File => read_relative(root, RECEIPT_V2_REL)
                .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V2_REL))?,
            SafePathState::Directory => {
                return Err(AppError::execution_at(
                    "V3_PREVIOUS_RECEIPT_NOT_REGULAR",
                    RECEIPT_V2_REL,
                ));
            }
        };
        if sha256_hex(&previous_bytes) != BASELINE_RECEIPT_V2_SHA256 {
            return Err(AppError::schema("V3_PREVIOUS_RECEIPT_HASH_MISMATCH"));
        }
        let previous = parse_receipt_v2_bytes(&previous_bytes, &manifest, &historical)
            .map_err(AppError::from_validation)?;

        let catalog_state =
            inspect_relative_inner(root, PERFORMANCE_SOURCE_REL).map_err(|failure| {
                fs_error(ErrorKind::Execution, failure.code, PERFORMANCE_SOURCE_REL)
            })?;
        let catalog_bytes = match catalog_state {
            SafePathState::Missing => {
                return Err(AppError::execution_at(
                    "V3_PERFORMANCE_SOURCE_MISSING",
                    PERFORMANCE_SOURCE_REL,
                ));
            }
            SafePathState::File => {
                read_relative(root, PERFORMANCE_SOURCE_REL).map_err(|failure| {
                    fs_error(ErrorKind::Execution, failure.code, PERFORMANCE_SOURCE_REL)
                })?
            }
            SafePathState::Directory => {
                return Err(AppError::execution_at(
                    "V3_PERFORMANCE_SOURCE_NOT_REGULAR",
                    PERFORMANCE_SOURCE_REL,
                ));
            }
        };
        let catalog = trusted_performance_source_from_bytes(&catalog_bytes, &manifest)?;

        let bytes = read_relative(root, RECEIPT_V3_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V3_REL))?;
        let receipt = parse_receipt_v3_bytes(&bytes, &manifest, &historical, &previous, &catalog)
            .map_err(AppError::from_validation)?;
        let mut combined = manifest;
        combined.records.extend(catalog.records);
        return Ok((
            root.to_path_buf(),
            combined,
            Some(ActiveReceipt::from_v3(&receipt)),
        ));
    }

    let v2_state = inspect_relative_inner(root, RECEIPT_V2_REL)
        .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V2_REL))?;
    let active = match v2_state {
        SafePathState::Missing => match inspect_relative_inner(root, RECEIPT_REL)
            .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?
        {
            SafePathState::Missing => None,
            SafePathState::File => {
                let bytes = read_relative(root, RECEIPT_REL)
                    .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?;
                let receipt =
                    parse_receipt_bytes(&bytes, &manifest).map_err(AppError::from_validation)?;
                Some(ActiveReceipt::from_v1(&receipt))
            }
            SafePathState::Directory => {
                return Err(AppError::execution_at("RECEIPT_NOT_REGULAR", RECEIPT_REL));
            }
        },
        SafePathState::File => {
            let historical_state = inspect_relative_inner(root, RECEIPT_REL)
                .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?;
            let historical_bytes = match historical_state {
                SafePathState::Missing => {
                    return Err(AppError::execution_at(
                        "V2_HISTORICAL_RECEIPT_MISSING",
                        RECEIPT_REL,
                    ));
                }
                SafePathState::File => read_relative(root, RECEIPT_REL)
                    .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_REL))?,
                SafePathState::Directory => {
                    return Err(AppError::execution_at(
                        "V2_HISTORICAL_RECEIPT_NOT_REGULAR",
                        RECEIPT_REL,
                    ));
                }
            };
            if sha256_hex(&historical_bytes) != BASELINE_RECEIPT_V1_SHA256 {
                return Err(AppError::schema("V2_HISTORICAL_RECEIPT_HASH_MISMATCH"));
            }
            let historical = parse_receipt_bytes(&historical_bytes, &manifest)
                .map_err(AppError::from_validation)?;
            let bytes = read_relative(root, RECEIPT_V2_REL)
                .map_err(|failure| fs_error(ErrorKind::Execution, failure.code, RECEIPT_V2_REL))?;
            let receipt = parse_receipt_v2_bytes(&bytes, &manifest, &historical)
                .map_err(AppError::from_validation)?;
            Some(ActiveReceipt::from_v2(&receipt))
        }
        SafePathState::Directory => {
            return Err(AppError::execution_at(
                "V2_RECEIPT_NOT_REGULAR",
                RECEIPT_V2_REL,
            ));
        }
    };
    Ok((root.to_path_buf(), manifest, active))
}

struct LoadContext {
    root: PathBuf,
    manifest: Manifest,
    active: Option<ActiveReceipt>,
    provenance: provenance::ProvenanceView,
}

fn provenance_app_error(error: provenance::ProvenanceError) -> AppError {
    AppError::from_validation(ValidationError {
        code: error.code,
        path: error.path.as_deref().map(safe_path_label),
        record_id: error.record_id,
    })
}

fn load_context_at(root: &Path) -> Result<LoadContext, AppError> {
    load_context_at_with_trust(root, &provenance::ProvenanceTrust::production())
}

fn load_context_at_with_trust(
    root: &Path,
    trust: &provenance::ProvenanceTrust,
) -> Result<LoadContext, AppError> {
    let (root, mut manifest, mut active) = load_context_baseline(root)?;
    bulk::apply_index(&root, &mut manifest, &mut active).map_err(AppError::from_validation)?;
    let provenance = provenance::apply_effective_view(&root, &mut manifest, active.as_ref(), trust)
        .map_err(provenance_app_error)?;
    Ok(LoadContext {
        root,
        manifest,
        active,
        provenance,
    })
}

pub fn load_active_receipt(root: &Path) -> Result<Option<ActiveReceipt>, ValidationError> {
    load_context_at(root)
        .map(|context| context.active)
        .map_err(|error| error.issue)
}

/// Minimal read-only access to the current accepted imported method identities.
/// Used by the supplementary source-closure commands; it does not alter the
/// baseline context, receipts, or graph.
pub(crate) fn current_imported_method_identities(
    root: &Path,
) -> Result<Vec<MethodIdentity>, String> {
    let context = load_context_at(root).map_err(|error| error.issue.code)?;
    let active = context
        .active
        .ok_or_else(|| "NO_ACTIVE_RECEIPT".to_string())?;
    Ok(active
        .files
        .iter()
        .flat_map(|file| file.methods.iter().cloned())
        .collect())
}

pub(crate) struct CurrentMethodClass {
    pub identity: MethodIdentity,
    pub evidence_status: String,
}

pub(crate) fn current_imported_method_classes(
    root: &Path,
) -> Result<Vec<CurrentMethodClass>, String> {
    let context = load_context_at(root).map_err(|error| error.issue.code)?;
    let mut output = Vec::new();
    for record in &context.manifest.records {
        for method in &record.methods.current {
            output.push(CurrentMethodClass {
                identity: method.symbol_key.clone(),
                evidence_status: method.evidence_status.as_str().to_string(),
            });
        }
    }
    Ok(output)
}

/// Read-only, typed view of one current imported method. This is deliberately a
/// narrow projection of the already-validated effective context: it does not
/// clone the catalog, invent a second authority, or serialize arbitrary
/// manifest data.
pub(crate) struct CurrentMethodMatch {
    pub identity: MethodIdentity,
    pub record_id: String,
    pub record_original_path: String,
    pub record_original_size: u64,
    pub record_original_sha256: String,
    pub record_proposed_path: Option<String>,
    pub record_authority_id: String,
    pub record_authority_sha256: Option<String>,
    pub record_authority_mvid: Option<String>,
    pub record_type_def: Option<String>,
    pub record_scope: String,
    pub record_completeness: String,
    pub record_representation: String,
    pub record_curation_status: String,
    pub record_export_status: String,
    pub record_evidence_status: String,
    pub record_source_status: String,
    pub record_supersedes: Vec<String>,
    pub record_superseded_by: Vec<String>,
    pub method_coverage: String,
    pub method_evidence_status: String,
    pub method_source_status: String,
    pub method_verification_basis: String,
    pub method_rva: Option<String>,
    pub method_il_range: Option<String>,
    pub method_evidence_references: Vec<EvidenceReference>,
    pub source_provenance: Option<provenance::SourceProvenance>,
    pub imported_file: Option<ImportedFileView>,
    pub is_active_primary: bool,
    pub active_scope: bool,
}

pub(crate) struct ExactHistoricalMatch {
    pub record_id: String,
    pub record_original_path: String,
    pub record_original_size: u64,
    pub record_original_sha256: String,
    pub record_curation_status: String,
    pub record_export_status: String,
    pub record_evidence_status: String,
    pub record_source_status: String,
    pub record_supersedes: Vec<String>,
    pub record_superseded_by: Vec<String>,
    pub record_type_def: Option<String>,
    pub symbol_key: MethodIdentity,
    pub name: Option<String>,
    pub coverage: String,
    pub imported_file: Option<ImportedFileView>,
    pub has_current_exact_match: bool,
    pub same_record_as_primary: bool,
}

pub(crate) struct CurrentMethodAccess {
    pub identity: MethodIdentity,
    pub record_id: String,
    pub record_original_path: String,
    pub record_original_size: u64,
    pub record_original_sha256: String,
    pub record_proposed_path: Option<String>,
    pub record_authority_id: String,
    pub record_authority_sha256: Option<String>,
    pub record_authority_mvid: Option<String>,
    pub record_type_def: Option<String>,
    pub record_scope: String,
    pub record_completeness: String,
    pub record_representation: String,
    pub record_curation_status: String,
    pub record_export_status: String,
    pub record_evidence_status: String,
    pub record_source_status: String,
    pub record_supersedes: Vec<String>,
    pub record_superseded_by: Vec<String>,
    pub method_coverage: String,
    pub method_evidence_status: String,
    pub method_source_status: String,
    pub method_verification_basis: String,
    pub method_rva: Option<String>,
    pub method_il_range: Option<String>,
    pub method_evidence_references: Vec<EvidenceReference>,
    pub source_provenance: Option<provenance::SourceProvenance>,
    pub imported_file: Option<ImportedFileView>,
    pub historical: Vec<HistoricalMethod>,
    pub alternative_record_ids: Vec<String>,
    pub current_matches: Vec<CurrentMethodMatch>,
    pub exact_historical_matches: Vec<ExactHistoricalMatch>,
}

fn identity_matches(left: &MethodIdentity, right: &MethodIdentity) -> bool {
    left.full_authority_sha256
        .eq_ignore_ascii_case(&right.full_authority_sha256)
        && left.mvid.eq_ignore_ascii_case(&right.mvid)
        && left.type_def.eq_ignore_ascii_case(&right.type_def)
        && left.method_def.eq_ignore_ascii_case(&right.method_def)
}

fn active_file_for(
    active: Option<&ActiveReceipt>,
    record_id: &str,
    identity: &MethodIdentity,
) -> Option<ImportedFileView> {
    active.and_then(|active| {
        active
            .files
            .iter()
            .find(|file| {
                file.id == record_id
                    && file
                        .methods
                        .iter()
                        .any(|candidate| identity_matches(candidate, identity))
            })
            .cloned()
    })
}

fn current_match_from(
    record: &Record,
    method: &CurrentMethod,
    active: Option<&ActiveReceipt>,
    identity: &MethodIdentity,
    is_active_primary: bool,
) -> CurrentMethodMatch {
    let imported_file = active_file_for(active, &record.id, identity);
    CurrentMethodMatch {
        identity: method.symbol_key.clone(),
        record_id: record.id.clone(),
        record_original_path: record.original_path.clone(),
        record_original_size: record.original_size,
        record_original_sha256: record.original_sha256.clone(),
        record_proposed_path: record.proposed_path.clone(),
        record_authority_id: record.authority_id.clone(),
        record_authority_sha256: record.authority_binary_sha256.clone(),
        record_authority_mvid: record.authority_mvid.clone(),
        record_type_def: record.type_def.clone(),
        record_scope: record.scope.clone(),
        record_completeness: record.completeness.clone(),
        record_representation: record.representation.clone(),
        record_curation_status: record.curation_status.as_str().to_string(),
        record_export_status: record.export_status.as_str().to_string(),
        record_evidence_status: record.evidence_status.as_str().to_string(),
        record_source_status: record.source_status.as_str().to_string(),
        record_supersedes: record.supersession.supersedes.clone(),
        record_superseded_by: record.supersession.superseded_by.clone(),
        method_coverage: method.coverage.clone(),
        method_evidence_status: method.evidence_status.as_str().to_string(),
        method_source_status: method.source_status.as_str().to_string(),
        method_verification_basis: method.verification_basis.clone(),
        method_rva: method.rva.clone(),
        method_il_range: method.il_range.clone(),
        method_evidence_references: method.evidence_references.clone(),
        source_provenance: method.source_provenance.clone(),
        imported_file: imported_file.clone(),
        is_active_primary,
        active_scope: imported_file.is_some(),
    }
}

pub(crate) fn current_imported_method_access(
    root: &Path,
    identity: &MethodIdentity,
) -> Result<Option<CurrentMethodAccess>, String> {
    let context = load_context_at(root).map_err(|error| error.issue.code)?;
    let authority = &context.manifest.authority_policy.managed;
    if !authority
        .authority_binary_sha256
        .eq_ignore_ascii_case(&identity.full_authority_sha256)
        || !authority.mvid.eq_ignore_ascii_case(&identity.mvid)
    {
        return Err("CURRENT_AUTHORITY_MISMATCH".to_string());
    }
    let active = context.active.as_ref();
    let mut matches = Vec::new();
    for record in &context.manifest.records {
        for method in &record.methods.current {
            if identity_matches(&method.symbol_key, identity) {
                matches.push((record, method));
            }
        }
    }
    if matches.is_empty() {
        return Ok(None);
    }
    let active_files = active
        .map(|active| {
            active
                .files
                .iter()
                .filter(|file| {
                    file.methods
                        .iter()
                        .any(|candidate| identity_matches(candidate, identity))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if active_files.len() > 1 {
        return Err("CURRENT_AUTHORITY_AMBIGUOUS_ACTIVE_FILES".to_string());
    }
    let selected = if let Some(file) = active_files.first() {
        matches
            .iter()
            .find(|(record, _)| record.id == file.id)
            .copied()
            .ok_or_else(|| "CURRENT_AUTHORITY_ACTIVE_RECORD_MISSING".to_string())?
    } else {
        let rank = |status: &EvidenceStatus| match status {
            EvidenceStatus::DirectCilFull => 4_u8,
            EvidenceStatus::DirectCilPartial => 3,
            EvidenceStatus::SourceOnly => 2,
            EvidenceStatus::MetadataOnly => 1,
            EvidenceStatus::Unknown => 1,
            EvidenceStatus::Blocked => 0,
        };
        matches
            .iter()
            .copied()
            .max_by(|(left_record, left_method), (right_record, right_method)| {
                rank(&left_method.evidence_status)
                    .cmp(&rank(&right_method.evidence_status))
                    .then_with(|| left_record.id.cmp(&right_record.id))
            })
            .expect("matches is non-empty")
    };
    let (record, method) = selected;
    let current_matches = matches
        .iter()
        .map(|(candidate_record, candidate_method)| {
            current_match_from(
                candidate_record,
                candidate_method,
                active,
                identity,
                std::ptr::eq(*candidate_record, record) && std::ptr::eq(*candidate_method, method),
            )
        })
        .collect::<Vec<_>>();
    let mut historical_seen = BTreeSet::new();
    let mut exact_historical_matches = Vec::new();
    for candidate_record in &context.manifest.records {
        for historical in &candidate_record.methods.historical {
            if !identity_matches(&historical.symbol_key, identity) {
                continue;
            }
            let key = format!(
                "{}\0{}\0{}",
                candidate_record.id,
                historical.symbol_key.type_def,
                historical.symbol_key.method_def
            );
            if !historical_seen.insert(key) {
                continue;
            }
            let has_current_exact_match = matches
                .iter()
                .any(|(current_record, _)| current_record.id == candidate_record.id);
            exact_historical_matches.push(ExactHistoricalMatch {
                record_id: candidate_record.id.clone(),
                record_original_path: candidate_record.original_path.clone(),
                record_original_size: candidate_record.original_size,
                record_original_sha256: candidate_record.original_sha256.clone(),
                record_curation_status: candidate_record.curation_status.as_str().to_string(),
                record_export_status: candidate_record.export_status.as_str().to_string(),
                record_evidence_status: candidate_record.evidence_status.as_str().to_string(),
                record_source_status: candidate_record.source_status.as_str().to_string(),
                record_supersedes: candidate_record.supersession.supersedes.clone(),
                record_superseded_by: candidate_record.supersession.superseded_by.clone(),
                record_type_def: candidate_record.type_def.clone(),
                symbol_key: historical.symbol_key.clone(),
                name: historical.name.clone(),
                coverage: historical.coverage.clone(),
                imported_file: active_file_for(active, &candidate_record.id, identity),
                has_current_exact_match,
                same_record_as_primary: candidate_record.id == record.id,
            });
        }
    }
    let alternative_record_ids = current_matches
        .iter()
        .filter(|candidate| !candidate.is_active_primary)
        .map(|candidate| candidate.record_id.clone())
        .chain(
            exact_historical_matches
                .iter()
                .filter(|candidate| candidate.record_id != record.id)
                .map(|candidate| candidate.record_id.clone()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let imported_file = active_file_for(active, &record.id, identity);
    Ok(Some(CurrentMethodAccess {
        identity: method.symbol_key.clone(),
        record_id: record.id.clone(),
        record_original_path: record.original_path.clone(),
        record_original_size: record.original_size,
        record_original_sha256: record.original_sha256.clone(),
        record_proposed_path: record.proposed_path.clone(),
        record_authority_id: record.authority_id.clone(),
        record_authority_sha256: record.authority_binary_sha256.clone(),
        record_authority_mvid: record.authority_mvid.clone(),
        record_type_def: record.type_def.clone(),
        record_scope: record.scope.clone(),
        record_completeness: record.completeness.clone(),
        record_representation: record.representation.clone(),
        record_curation_status: record.curation_status.as_str().to_string(),
        record_export_status: record.export_status.as_str().to_string(),
        record_evidence_status: record.evidence_status.as_str().to_string(),
        record_source_status: record.source_status.as_str().to_string(),
        record_supersedes: record.supersession.supersedes.clone(),
        record_superseded_by: record.supersession.superseded_by.clone(),
        method_coverage: method.coverage.clone(),
        method_evidence_status: method.evidence_status.as_str().to_string(),
        method_source_status: method.source_status.as_str().to_string(),
        method_verification_basis: method.verification_basis.clone(),
        method_rva: method.rva.clone(),
        method_il_range: method.il_range.clone(),
        method_evidence_references: method.evidence_references.clone(),
        source_provenance: method.source_provenance.clone(),
        imported_file,
        historical: record.methods.historical.clone(),
        alternative_record_ids,
        current_matches,
        exact_historical_matches,
    }))
}

fn issue_value(issue: &ValidationError) -> Value {
    let mut entries = BTreeMap::new();
    entries.insert("code".to_owned(), Value::String(issue.code.clone()));
    if let Some(path) = &issue.path {
        entries.insert("path".to_owned(), Value::String(path.clone()));
    }
    if let Some(record_id) = &issue.record_id {
        entries.insert("record_id".to_owned(), Value::String(record_id.clone()));
    }
    Value::Object(entries.into_iter().collect())
}

fn finding(code: &str, path: Option<&str>, record_id: Option<&str>) -> Value {
    let issue = ValidationError {
        code: code.to_owned(),
        path: path.map(safe_path_label),
        record_id: record_id.map(str::to_owned),
    };
    issue_value(&issue)
}

fn envelope(
    command: &str,
    exit: u8,
    counts: Value,
    records: Vec<Value>,
    findings: Vec<Value>,
    mut extras: BTreeMap<String, Value>,
) -> Value {
    let mut object = BTreeMap::new();
    object.insert("command".to_owned(), Value::String(command.to_owned()));
    object.insert("ok".to_owned(), Value::Bool(exit == 0));
    object.insert("exit".to_owned(), Value::from(exit));
    object.insert("counts".to_owned(), counts);
    object.insert("records".to_owned(), Value::Array(records));
    object.insert("findings".to_owned(), Value::Array(findings));
    extras
        .entry("active_ledger".to_owned())
        .or_insert(Value::Null);
    object.extend(extras);
    Value::Object(object.into_iter().collect())
}

fn error_envelope(command: &str, error: AppError) -> (u8, Value) {
    let exit = match error.kind {
        ErrorKind::Schema | ErrorKind::Execution => 2,
    };
    (
        exit,
        envelope(
            command,
            exit,
            json!({}),
            Vec::new(),
            vec![issue_value(&error.issue)],
            BTreeMap::new(),
        ),
    )
}

fn manifest_counts(manifest: &Manifest, active: Option<&ActiveReceipt>) -> Value {
    let mut curation = BTreeMap::<String, u64>::new();
    let mut exports = BTreeMap::<String, u64>::new();
    let mut qualified = BTreeSet::new();
    let mut planned_ids = BTreeSet::new();
    let mut historical = 0_u64;
    let mut current = 0_u64;
    for record in &manifest.records {
        *curation
            .entry(record.curation_status.as_str().to_owned())
            .or_default() += 1;
        *exports
            .entry(record.export_status.as_str().to_owned())
            .or_default() += 1;
        historical += record.methods.historical.len() as u64;
        current += record.methods.current.len() as u64;
        qualified.extend(qualified_key_set(record));
        if record.proposed_path.is_some() {
            planned_ids.insert(record.id.clone());
        }
    }
    let imported_ids = import_ids(active);
    let imported_methods = active
        .map(|receipt| {
            receipt
                .files
                .iter()
                .flat_map(|file| file.methods.iter().map(method_key))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let imported = imported_ids.len() as u64;
    let missing = planned_ids.difference(&imported_ids).count() as u64;
    json!({
        "curation": curation,
        "export": exports,
        "historical_entries": historical,
        "current_entries": current,
        "unique_qualified_symbols": qualified.len(),
        "planned": planned_ids.len(),
        "imported": imported,
        "imported_file_count": imported,
        "imported_qualified_methods": imported_methods.len(),
        "missing_imports": missing
    })
}

fn import_ids(active: Option<&ActiveReceipt>) -> BTreeSet<String> {
    active
        .map(|receipt| receipt.files.iter().map(|file| file.id.clone()).collect())
        .unwrap_or_default()
}

fn active_ledger_value(active: Option<&ActiveReceipt>) -> Value {
    active.map_or(Value::Null, |receipt| {
        json!({
            "path": receipt.ledger_path,
            "version": receipt.version
        })
    })
}

fn active_extras(active: Option<&ActiveReceipt>) -> BTreeMap<String, Value> {
    BTreeMap::from([("active_ledger".to_owned(), active_ledger_value(active))])
}

fn evidence_anchor_value(anchor: &provenance::EvidenceAnchor) -> Value {
    json!({
        "root": anchor.root.as_str(),
        "path": anchor.path,
        "artifact_sha256": anchor.artifact_sha256,
        "method_identity": method_identity_value(&anchor.method_identity),
        "evidence_class": anchor.evidence_class.as_str(),
        "precision": anchor.precision.as_str(),
        "start_line": anchor.start_line,
        "end_line": anchor.end_line,
        "accepted_map_row": anchor.accepted_map_row,
        "accepted_map_line": anchor.accepted_map_line,
        "basis": anchor.basis
    })
}

fn imported_anchor_value(anchor: &provenance::ImportedAnchor) -> Value {
    json!({
        "root": anchor.root.as_str(),
        "path": anchor.path,
        "whole_file_sha256": anchor.whole_file_sha256,
        "method_identity": method_identity_value(&anchor.method_identity),
        "precision": anchor.precision.as_str(),
        "start_line": anchor.start_line,
        "end_line": anchor.end_line
    })
}

fn method_identity_value(identity: &MethodIdentity) -> Value {
    json!({
        "full_authority_sha256": identity.full_authority_sha256,
        "mvid": identity.mvid,
        "typeDef": identity.type_def,
        "methodDef": identity.method_def
    })
}

fn source_provenance_value(view: &provenance::SourceProvenance) -> Value {
    match view {
        provenance::SourceProvenance::ProfileEntry {
            status,
            action,
            imported,
            evidence_anchors,
            legacy_original_references,
        } => json!({
            "status": status.as_str(),
            "verification": "PROVENANCE_NOT_VERIFIED",
            "action": action.as_str(),
            "imported_source": imported_anchor_value(imported),
            "supporting_anchors": evidence_anchors
                .iter()
                .map(evidence_anchor_value)
                .collect::<Vec<_>>(),
            "legacy_original_references": legacy_original_references
                .iter()
                .map(|reference| json!({
                    "path": reference.path,
                    "locator": reference.locator,
                    "artifact_sha256": reference.artifact_sha256,
                    "status": "LEGACY_ORIGINAL_UNCORRECTED"
                }))
                .collect::<Vec<_>>()
        }),
        provenance::SourceProvenance::NotValidated => json!({
            "status": "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE",
            "verification": "PROVENANCE_NOT_VERIFIED"
        }),
    }
}

fn provenance_summary_value(
    view: &provenance::ProvenanceView,
    active: Option<&ActiveReceipt>,
) -> Value {
    let global_required = active
        .map(|active| {
            active
                .files
                .iter()
                .map(|file| file.methods.len() as u64)
                .sum::<u64>()
        })
        .unwrap_or(0);
    let covered = view.covered();
    let status = match view.state {
        provenance::ProvenanceState::NotApplicableBulkCorrection => {
            "NOT_APPLICABLE_BULK_CORRECTION"
        }
        provenance::ProvenanceState::Absent => "ABSENT",
        provenance::ProvenanceState::PendingReview => "PENDING_PRIMARY_REVIEW",
        provenance::ProvenanceState::StructurallyApplied => "PARTIAL",
    };
    let verification = match view.state {
        provenance::ProvenanceState::NotApplicableBulkCorrection => "NOT_APPLICABLE",
        provenance::ProvenanceState::Absent | provenance::ProvenanceState::PendingReview => {
            "BLOCKED"
        }
        provenance::ProvenanceState::StructurallyApplied => "NOT_CHECKED",
    };
    json!({
        "status": status,
        "verification": verification,
        "verify_scope": "BYTE_INTEGRITY_ONLY",
        "profile": provenance::PROVENANCE_PROFILE,
        "errata_path": provenance::ERRATA_REL,
        "errata_sha256": view.errata_sha256,
        "entries": view.entries,
        "resolved_entries": view.resolved_entries,
        "activation_block": view.activation_block,
        "actions": {
            "CORRECT": view.corrected,
            "CONFIRM": view.confirmed,
            "REFINE": view.refined
        },
        "coverage": {
            "global_required_keys": global_required,
            "covered": covered,
            "not_validated": global_required.saturating_sub(covered)
        },
        "overall_acceptance": "BLOCKED"
    })
}

fn lookup_counts(
    matched_records: usize,
    qualified_symbols: usize,
    active: Option<&ActiveReceipt>,
) -> Value {
    let imported_ids = import_ids(active);
    let imported_methods = active
        .map(|receipt| {
            receipt
                .files
                .iter()
                .flat_map(|file| file.methods.iter().map(method_key))
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    json!({
        "matched_records": matched_records,
        "qualified_symbols": qualified_symbols,
        "imported": imported_ids.len(),
        "imported_file_count": imported_ids.len(),
        "imported_qualified_methods": imported_methods.len()
    })
}

fn record_import_status(record: &Record, imported: &BTreeSet<String>) -> ImportStatus {
    if record.proposed_path.is_none() {
        ImportStatus::NotApplicable
    } else if imported.contains(&record.id) {
        ImportStatus::Imported
    } else {
        ImportStatus::NotImported
    }
}

fn record_status_value(record: &Record, imported: &BTreeSet<String>) -> Value {
    json!({
        "id": record.id,
        "authority_id": record.authority_id,
        "original_path": record.original_path,
        "proposed_path": record.proposed_path,
        "curation_status": record.curation_status.as_str(),
        "export_status": record.export_status.as_str(),
        "evidence_status": record.evidence_status.as_str(),
        "source_status": record.source_status.as_str(),
        "historical_entries": record.methods.historical.len(),
        "current_entries": record.methods.current.len(),
        "import_status": record_import_status(record, imported).as_str()
    })
}

fn sorted_record_values(manifest: &Manifest, imported: &BTreeSet<String>) -> Vec<Value> {
    let mut records: Vec<&Record> = manifest.records.iter().collect();
    records.sort_by(|left, right| left.id.cmp(&right.id));
    records
        .into_iter()
        .map(|record| record_status_value(record, imported))
        .collect()
}

fn check_unrecorded_destinations(
    root: &Path,
    manifest: &Manifest,
    imported: &BTreeSet<String>,
) -> Result<Vec<Value>, AppError> {
    let mut findings = Vec::new();
    let mut records: Vec<&Record> = manifest
        .records
        .iter()
        .filter(|record| record.proposed_path.is_some() && !imported.contains(&record.id))
        .collect();
    records.sort_by(|left, right| left.id.cmp(&right.id));
    for record in records {
        let path = record.proposed_path.as_deref().unwrap_or_default();
        match inspect_relative_inner(root, path) {
            Ok(SafePathState::Missing) => {}
            Ok(SafePathState::File | SafePathState::Directory) => {
                findings.push(finding("UNRECORDED_FILE", Some(path), Some(&record.id)));
            }
            Err(failure) => {
                return Err(fs_error(ErrorKind::Execution, failure.code, path));
            }
        }
    }
    Ok(findings)
}

pub fn run_status() -> (u8, Value) {
    run_status_at(&curated_root())
}

pub fn run_status_at(root_path: &Path) -> (u8, Value) {
    run_status_at_with_trust(root_path, &provenance::ProvenanceTrust::production())
}

pub fn run_status_at_with_trust(
    root_path: &Path,
    trust: &provenance::ProvenanceTrust,
) -> (u8, Value) {
    let LoadContext {
        root,
        manifest,
        active,
        provenance,
    } = match load_context_at_with_trust(root_path, trust) {
        Ok(context) => context,
        Err(error) => return error_envelope("status", error),
    };
    let imported = import_ids(active.as_ref());
    let mut findings = match check_unrecorded_destinations(&root, &manifest, &imported) {
        Ok(findings) => findings,
        Err(error) => return error_envelope("status", error),
    };
    if let Some(active) = &active {
        for file in &active.files {
            match inspect_relative_inner(&root, &file.imported.path) {
                Ok(SafePathState::File) => {}
                Ok(SafePathState::Missing) => findings.push(finding(
                    "MISSING_IMPORTED_FILE",
                    Some(&file.imported.path),
                    Some(&file.id),
                )),
                Ok(SafePathState::Directory) => findings.push(finding(
                    "IMPORTED_PATH_NOT_FILE",
                    Some(&file.imported.path),
                    Some(&file.id),
                )),
                Err(failure) => {
                    return error_envelope(
                        "status",
                        fs_error(ErrorKind::Execution, failure.code, &file.imported.path),
                    );
                }
            }
        }
    }
    findings.sort_by_key(|value| value.to_string());
    let exit = if findings.is_empty() { 0 } else { 1 };
    let mut extras = active_extras(active.as_ref());
    extras.insert(
        "provenance".to_owned(),
        provenance_summary_value(&provenance, active.as_ref()),
    );
    extras.insert(
        "verify_scope".to_owned(),
        Value::String("BYTE_INTEGRITY_ONLY".to_owned()),
    );
    (
        exit,
        envelope(
            "status",
            exit,
            manifest_counts(&manifest, active.as_ref()),
            sorted_record_values(&manifest, &imported),
            findings,
            extras,
        ),
    )
}

fn evidence_reference_value(reference: &EvidenceReference) -> Value {
    json!({
        "path": reference.path,
        "locator": reference.locator,
        "artifact_sha256": reference.artifact_sha256,
        "verification": VerificationResult::NotChecked.as_str()
    })
}

fn method_lookup_value(
    era: &str,
    method_def: &str,
    coverage: &str,
    evidence_status: &str,
    source_status: &str,
    verification_basis: &str,
    symbol_key: &MethodIdentity,
    rva: Option<&str>,
    il_range: Option<&str>,
    references: &[EvidenceReference],
    source_provenance: Option<&provenance::SourceProvenance>,
    name: Option<&str>,
) -> Value {
    let mut object = BTreeMap::new();
    object.insert("era".to_owned(), Value::String(era.to_owned()));
    object.insert("methodDef".to_owned(), Value::String(method_def.to_owned()));
    object.insert("coverage".to_owned(), Value::String(coverage.to_owned()));
    object.insert(
        "evidence_status".to_owned(),
        Value::String(evidence_status.to_owned()),
    );
    object.insert(
        "source_status".to_owned(),
        Value::String(source_status.to_owned()),
    );
    object.insert(
        "verification_basis".to_owned(),
        Value::String(verification_basis.to_owned()),
    );
    object.insert(
        "qualified_key".to_owned(),
        json!({
            "full_authority_sha256": symbol_key.full_authority_sha256,
            "mvid": symbol_key.mvid,
            "typeDef": symbol_key.type_def,
            "methodDef": symbol_key.method_def
        }),
    );
    object.insert(
        "rva".to_owned(),
        rva.map_or(Value::Null, |value| Value::String(value.to_owned())),
    );
    object.insert(
        "il_range".to_owned(),
        il_range.map_or(Value::Null, |value| Value::String(value.to_owned())),
    );
    object.insert(
        "evidence_references".to_owned(),
        Value::Array(references.iter().map(evidence_reference_value).collect()),
    );
    object.insert(
        "source_provenance".to_owned(),
        source_provenance.map_or(Value::Null, source_provenance_value),
    );
    if let Some(name) = name {
        object.insert("name".to_owned(), Value::String(name.to_owned()));
    }
    Value::Object(object.into_iter().collect())
}

fn lookup_record_value(
    record: &Record,
    current: &[&CurrentMethod],
    historical: &[&HistoricalMethod],
    imported: &BTreeSet<String>,
) -> Value {
    let current_values = current
        .iter()
        .map(|method| {
            method_lookup_value(
                "CURRENT",
                &method.method_def,
                &method.coverage,
                method.evidence_status.as_str(),
                method.source_status.as_str(),
                &method.verification_basis,
                &method.symbol_key,
                method.rva.as_deref(),
                method.il_range.as_deref(),
                &method.evidence_references,
                method.source_provenance.as_ref(),
                None,
            )
        })
        .collect::<Vec<_>>();
    let historical_values = historical
        .iter()
        .map(|method| {
            method_lookup_value(
                "HISTORICAL",
                &method.symbol_key.method_def,
                &method.coverage,
                record.evidence_status.as_str(),
                record.source_status.as_str(),
                &record.verification_basis,
                &method.symbol_key,
                None,
                None,
                &record.evidence_references,
                None,
                method.name.as_deref(),
            )
        })
        .collect::<Vec<_>>();
    json!({
        "id": record.id,
        "authority_id": record.authority_id,
        "typeDef": record.type_def,
        "source": {
            "original_path": record.original_path,
            "proposed_path": record.proposed_path,
            "import_status": record_import_status(record, imported).as_str()
        },
        "current": current_values,
        "historical": historical_values,
        "evidence_references": record.evidence_references.iter().map(evidence_reference_value).collect::<Vec<_>>(),
        "dependencies": record.dependencies,
        "scope": record.scope,
        "completeness": record.completeness,
        "curation_status": record.curation_status.as_str(),
        "export_status": record.export_status.as_str(),
        "compilation": record.validation.compilation,
        "runtime": record.validation.runtime,
        "equivalence": record.validation.equivalence,
        "publication_limits": {
            "curation_status": record.curation_status.as_str(),
            "export_status": record.export_status.as_str(),
            "publication": record.validation.publication
        }
    })
}

pub fn run_lookup(authority: &str, token: &str) -> (u8, Value) {
    run_lookup_at(&curated_root(), authority, token)
}

pub fn run_lookup_at(root_path: &Path, authority: &str, token: &str) -> (u8, Value) {
    run_lookup_at_with_trust(
        root_path,
        authority,
        token,
        &provenance::ProvenanceTrust::production(),
    )
}

pub fn run_lookup_at_with_trust(
    root_path: &Path,
    authority: &str,
    token: &str,
    trust: &provenance::ProvenanceTrust,
) -> (u8, Value) {
    if !valid_authority_syntax(authority) {
        return error_envelope("lookup", AppError::schema("MALFORMED_AUTHORITY"));
    }
    if !validate_method_token(token) {
        return error_envelope("lookup", AppError::schema("MALFORMED_METHODDEF"));
    }
    let LoadContext {
        root: _root,
        manifest,
        active,
        provenance,
    } = match load_context_at_with_trust(root_path, trust) {
        Ok(context) => context,
        Err(error) => return error_envelope("lookup", error),
    };
    let imported = import_ids(active.as_ref());
    let mut matched_records: Vec<(&Record, Vec<&CurrentMethod>, Vec<&HistoricalMethod>)> =
        Vec::new();
    let mut qualified = BTreeSet::new();
    for record in &manifest.records {
        if record.authority_id != authority {
            continue;
        }
        let current: Vec<&CurrentMethod> = record
            .methods
            .current
            .iter()
            .filter(|method| method.method_def == token)
            .collect();
        let historical: Vec<&HistoricalMethod> = record
            .methods
            .historical
            .iter()
            .filter(|method| method.symbol_key.method_def == token)
            .collect();
        if current.is_empty() && historical.is_empty() {
            continue;
        }
        qualified.extend(current.iter().map(|method| method_key(&method.symbol_key)));
        qualified.extend(
            historical
                .iter()
                .map(|method| method_key(&method.symbol_key)),
        );
        matched_records.push((record, current, historical));
    }
    matched_records.sort_by(|left, right| left.0.id.cmp(&right.0.id));
    let matches: Vec<Value> = matched_records
        .iter()
        .map(|(record, current, historical)| {
            lookup_record_value(record, current, historical, &imported)
        })
        .collect();
    let mut extras = BTreeMap::new();
    extras.insert("matches".to_owned(), Value::Array(matches.clone()));
    extras.insert(
        "ambiguous".to_owned(),
        Value::Array(
            qualified
                .iter()
                .map(|key| Value::String(key.clone()))
                .collect(),
        ),
    );
    extras.insert(
        "active_ledger".to_owned(),
        active_ledger_value(active.as_ref()),
    );
    extras.insert(
        "provenance".to_owned(),
        provenance_summary_value(&provenance, active.as_ref()),
    );
    extras.insert(
        "verify_scope".to_owned(),
        Value::String("BYTE_INTEGRITY_ONLY".to_owned()),
    );
    if matched_records.is_empty() {
        return (
            1,
            envelope(
                "lookup",
                1,
                lookup_counts(0, 0, active.as_ref()),
                Vec::new(),
                vec![finding("NOT_FOUND", None, None)],
                extras,
            ),
        );
    }
    if qualified.len() > 1 {
        return (
            2,
            envelope(
                "lookup",
                2,
                lookup_counts(matched_records.len(), qualified.len(), active.as_ref()),
                matches.clone(),
                vec![finding("AMBIGUOUS_QUALIFIED_SYMBOL", None, None)],
                extras,
            ),
        );
    }
    (
        0,
        envelope(
            "lookup",
            0,
            lookup_counts(matched_records.len(), qualified.len(), active.as_ref()),
            matches,
            Vec::new(),
            extras,
        ),
    )
}

fn check_imported_artifact(
    root: &Path,
    id: &str,
    expected: &SourceArtifact,
) -> Result<SourceArtifact, ValidationError> {
    let bytes = read_relative(root, &expected.path)
        .map_err(|failure| ValidationError::at(failure.code, &expected.path))?;
    let artifact = SourceArtifact {
        path: expected.path.clone(),
        size: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    };
    if artifact.size != expected.size {
        return Err(ValidationError::record("IMPORTED_SIZE_MISMATCH", id));
    }
    if artifact.sha256 != expected.sha256 {
        return Err(ValidationError::record("IMPORTED_HASH_MISMATCH", id));
    }
    Ok(artifact)
}

fn check_import_entry_file(
    root: &Path,
    entry: &ImportEntry,
) -> Result<SourceArtifact, ValidationError> {
    check_imported_artifact(
        root,
        &entry.id,
        &SourceArtifact {
            path: entry.imported_path.clone(),
            size: entry.imported_size,
            sha256: entry.imported_sha256.clone(),
        },
    )
}

pub fn check_import_file(
    root: &Path,
    entry: &ImportEntry,
) -> Result<VerificationResult, ValidationError> {
    check_import_entry_file(root, entry).map(|_| VerificationResult::Pass)
}

pub fn check_imported_file_view(
    root: &Path,
    file: &ImportedFileView,
) -> Result<VerificationResult, ValidationError> {
    check_imported_file_view_artifact(root, file).map(|_| VerificationResult::Pass)
}

fn check_imported_file_view_artifact(
    root: &Path,
    file: &ImportedFileView,
) -> Result<SourceArtifact, ValidationError> {
    check_imported_artifact(root, &file.id, &file.imported)
}

fn check_root_directory(root: &Path) -> Result<SafePathState, FsFailure> {
    let absolute = absolute_path(root)?;
    check_chain(&absolute)
}

fn check_source_artifact(root: &Path, record: &Record) -> Result<SourceArtifact, ValidationError> {
    let bytes = read_relative(root, &record.original_path)
        .map_err(|failure| ValidationError::at(failure.code, &record.original_path))?;
    let artifact = SourceArtifact {
        path: record.original_path.clone(),
        size: bytes.len() as u64,
        sha256: sha256_hex(&bytes),
    };
    if artifact.size != record.original_size {
        return Err(ValidationError::record("SOURCE_SIZE_MISMATCH", &record.id));
    }
    if artifact.sha256 != record.original_sha256 {
        return Err(ValidationError::record("SOURCE_HASH_MISMATCH", &record.id));
    }
    Ok(artifact)
}

fn add_not_checked(list: &mut Vec<Value>, kind: &str, id: Option<&str>, path: &str, reason: &str) {
    list.push(json!({
        "kind": kind,
        "id": id,
        "path": path,
        "status": VerificationResult::NotChecked.as_str(),
        "reason": reason
    }));
}

fn add_missing(list: &mut Vec<Value>, kind: &str, id: Option<&str>, path: &str, code: &str) {
    list.push(json!({
        "kind": kind,
        "id": id,
        "path": path,
        "status": VerificationResult::Fail.as_str(),
        "code": code
    }));
}

fn all_evidence_references(manifest: &Manifest) -> Vec<(String, String, String)> {
    let mut references = BTreeSet::new();
    for record in &manifest.records {
        for reference in &record.evidence_references {
            references.insert((
                record.id.clone(),
                reference.path.clone(),
                reference.locator.clone(),
            ));
        }
        for method in &record.methods.current {
            for reference in &method.evidence_references {
                references.insert((
                    record.id.clone(),
                    reference.path.clone(),
                    reference.locator.clone(),
                ));
            }
            if let Some(provenance::SourceProvenance::ProfileEntry {
                legacy_original_references,
                ..
            }) = &method.source_provenance
            {
                for reference in legacy_original_references {
                    references.insert((
                        record.id.clone(),
                        reference.path.clone(),
                        reference.locator.clone(),
                    ));
                }
            }
        }
    }
    references.into_iter().collect()
}

pub fn run_verify(research_root: Option<PathBuf>) -> (u8, Value) {
    run_verify_at(&curated_root(), research_root)
}

pub fn run_verify_at(curated_path: &Path, research_root: Option<PathBuf>) -> (u8, Value) {
    run_verify_at_with_trust(
        curated_path,
        research_root,
        &provenance::ProvenanceTrust::production(),
    )
}

pub fn run_verify_at_with_trust(
    curated_path: &Path,
    research_root: Option<PathBuf>,
    trust: &provenance::ProvenanceTrust,
) -> (u8, Value) {
    let LoadContext {
        root: curated,
        manifest,
        active,
        provenance,
    } = match load_context_at_with_trust(curated_path, trust) {
        Ok(context) => context,
        Err(error) => return error_envelope("verify", error),
    };
    let imported = import_ids(active.as_ref());
    let mut checks = Vec::new();
    let mut missing = Vec::new();
    let mut not_checked = Vec::new();
    let mut findings = Vec::new();
    let mut source_bytes = BTreeMap::<String, Vec<u8>>::new();
    let root_available = if let Some(root) = &research_root {
        match check_root_directory(root) {
            Ok(SafePathState::Directory) => true,
            Ok(SafePathState::Missing) => {
                findings.push(finding(
                    "RESEARCH_ROOT_MISSING",
                    Some("research-root"),
                    None,
                ));
                false
            }
            Ok(SafePathState::File) => {
                return error_envelope(
                    "verify",
                    AppError::execution("RESEARCH_ROOT_NOT_DIRECTORY"),
                );
            }
            Err(failure) => {
                return error_envelope(
                    "verify",
                    fs_error(ErrorKind::Execution, failure.code, "research-root"),
                );
            }
        }
    } else {
        false
    };

    let required_ids: BTreeSet<&str> = match active.as_ref().map(|receipt| receipt.version) {
        Some(6) => active
            .as_ref()
            .expect("active receipt")
            .files
            .iter()
            .map(|file| file.id.as_str())
            .collect(),
        Some(5) => REQUIRED_V5_IMPORT_IDS.into_iter().collect(),
        Some(4) => REQUIRED_V4_IMPORT_IDS.into_iter().collect(),
        Some(3) => REQUIRED_V3_IMPORT_IDS.into_iter().collect(),
        Some(2) => REQUIRED_V2_IMPORT_IDS.into_iter().collect(),
        _ => REQUIRED_IMPORT_IDS.into_iter().collect(),
    };
    let mut required_records: Vec<&Record> = manifest
        .records
        .iter()
        .filter(|record| required_ids.contains(record.id.as_str()))
        .collect();
    required_records.sort_by(|left, right| left.id.cmp(&right.id));
    if root_available {
        let root = research_root.as_deref().expect("root availability checked");
        for record in &required_records {
            match check_source_artifact(root, record) {
                Ok(artifact) => {
                    if let Ok(bytes) = read_relative(root, &record.original_path) {
                        source_bytes.insert(record.id.clone(), bytes);
                    }
                    checks.push(json!({
                        "kind": "ORIGINAL_SOURCE",
                        "id": record.id,
                        "path": record.original_path,
                        "status": VerificationResult::Pass.as_str(),
                        "size": artifact.size,
                        "sha256": artifact.sha256
                    }));
                }
                Err(error) => {
                    let code = error.code.clone();
                    if code == "MISSING_PATH" {
                        add_missing(
                            &mut missing,
                            "ORIGINAL_SOURCE",
                            Some(&record.id),
                            &record.original_path,
                            "REQUIRED_SOURCE_MISSING",
                        );
                    } else {
                        checks.push(json!({
                            "kind": "ORIGINAL_SOURCE",
                            "id": record.id,
                            "path": record.original_path,
                            "status": VerificationResult::Fail.as_str(),
                            "code": code
                        }));
                    }
                    findings.push(finding(
                        &code,
                        Some(&record.original_path),
                        Some(&record.id),
                    ));
                }
            }
        }
    } else {
        for record in &required_records {
            add_not_checked(
                &mut not_checked,
                "ORIGINAL_SOURCE",
                Some(&record.id),
                &record.original_path,
                "RESEARCH_ROOT_NOT_SUPPLIED",
            );
        }
        findings.push(finding("REQUIRED_ORIGINALS_NOT_CHECKED", None, None));
    }

    for record in manifest
        .records
        .iter()
        .filter(|record| !required_ids.contains(record.id.as_str()))
    {
        add_not_checked(
            &mut not_checked,
            "OPTIONAL_ORIGINAL",
            Some(&record.id),
            &record.original_path,
            "OUTSIDE_PHASE1_SCOPE",
        );
    }
    for (id, path, locator) in all_evidence_references(&manifest) {
        not_checked.push(json!({
            "kind": "EVIDENCE_REFERENCE",
            "id": id,
            "path": path,
            "locator": locator,
            "status": VerificationResult::NotChecked.as_str(),
            "reason": "REFERENCE_CONTENT_NOT_READ"
        }));
    }

    if let Some(active) = &active {
        let mut files: Vec<&ImportedFileView> = active.files.iter().collect();
        files.sort_by(|left, right| left.id.cmp(&right.id));
        for file in files {
            match check_imported_file_view_artifact(&curated, file) {
                Ok(artifact) => {
                    checks.push(json!({
                        "kind": "IMPORTED_FILE",
                        "id": file.id,
                        "path": file.imported.path,
                        "status": VerificationResult::Pass.as_str(),
                        "size": artifact.size,
                        "sha256": artifact.sha256
                    }));
                    if root_available {
                        if let Some(bytes) = source_bytes.get(&file.id) {
                            if bytes
                                != &read_relative(&curated, &file.imported.path).unwrap_or_default()
                            {
                                findings.push(finding(
                                    "EXACT_BYTE_MISMATCH",
                                    Some(&file.imported.path),
                                    Some(&file.id),
                                ));
                                checks.push(json!({
                                    "kind": "EXACT_BYTE_COMPARISON",
                                    "id": file.id,
                                    "path": file.imported.path,
                                    "status": VerificationResult::Fail.as_str(),
                                    "code": "EXACT_BYTE_MISMATCH"
                                }));
                            } else {
                                checks.push(json!({
                                    "kind": "EXACT_BYTE_COMPARISON",
                                    "id": file.id,
                                    "path": file.imported.path,
                                    "status": VerificationResult::Pass.as_str()
                                }));
                            }
                        }
                    } else {
                        add_not_checked(
                            &mut not_checked,
                            "EXACT_BYTE_COMPARISON",
                            Some(&file.id),
                            &file.original_path,
                            "CURRENT_ORIGINAL_NOT_READ",
                        );
                    }
                }
                Err(error) => {
                    let code = error.code.clone();
                    if code == "MISSING_PATH" {
                        add_missing(
                            &mut missing,
                            "IMPORTED_FILE",
                            Some(&file.id),
                            &file.imported.path,
                            "IMPORTED_FILE_MISSING",
                        );
                    } else {
                        checks.push(json!({
                            "kind": "IMPORTED_FILE",
                            "id": file.id,
                            "path": file.imported.path,
                            "status": VerificationResult::Fail.as_str(),
                            "code": code
                        }));
                    }
                    findings.push(finding(&code, Some(&file.imported.path), Some(&file.id)));
                }
            }
        }
    }

    checks.sort_by_key(|value| value.to_string());
    missing.sort_by_key(|value| value.to_string());
    not_checked.sort_by_key(|value| value.to_string());
    findings.sort_by_key(|value| value.to_string());
    let exit = if findings.is_empty() { 0 } else { 1 };
    let mut extras = active_extras(active.as_ref());
    extras.insert(
        "provenance".to_owned(),
        provenance_summary_value(&provenance, active.as_ref()),
    );
    extras.insert(
        "verify_scope".to_owned(),
        Value::String("BYTE_INTEGRITY_ONLY".to_owned()),
    );
    extras.insert("checks".to_owned(), Value::Array(checks));
    extras.insert("missing".to_owned(), Value::Array(missing));
    extras.insert("not_checked".to_owned(), Value::Array(not_checked));
    (
        exit,
        envelope(
            "verify",
            exit,
            manifest_counts(&manifest, active.as_ref()),
            sorted_record_values(&manifest, &imported),
            findings,
            extras,
        ),
    )
}

#[derive(Debug, Clone, Default)]
struct ProvenanceCoverage {
    total: u64,
    covered: u64,
    ordinary_gameplay: u64,
    gameplay_associated: u64,
    performance: u64,
    legacy_v5: u64,
}

fn coverage_key(file_id: &str, type_def: &str, method_def: &str) -> String {
    format!(
        "{}|{}|{}",
        file_id.to_ascii_uppercase(),
        type_def.to_ascii_uppercase(),
        method_def.to_ascii_uppercase()
    )
}

fn errata_entry_key(entry: &provenance::ErrataEntry) -> String {
    coverage_key(
        &entry.target_catalog_record_id,
        &entry.method_identity.type_def,
        &entry.method_identity.method_def,
    )
}

fn provenance_catalog_categories(
    curated_path: &Path,
) -> Result<BTreeMap<String, &'static str>, AppError> {
    let bytes = provenance::checked_file_bytes(curated_path, bulk::CATALOG_REL)
        .map_err(provenance_app_error)?;
    let catalog = parse_strict_value(&bytes).map_err(AppError::from_validation)?;
    let records = catalog
        .get("records")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::schema("CATALOG_RECORDS_INVALID"))?;
    let mut categories = BTreeMap::new();
    for record in records {
        let id = record.get("id").and_then(Value::as_str).unwrap_or_default();
        let cohort = record
            .get("cohort")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let Some(methods) = record.get("methods").and_then(Value::as_array) else {
            continue;
        };
        for method in methods {
            let type_def = method
                .get("type_def")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let method_def = method
                .get("method_def")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let role = method
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let category = if cohort == bulk::GAMEPLAY_COHORT {
                if role == "ORDINARY" {
                    "ORDINARY"
                } else {
                    "GAMEPLAY_ASSOCIATED"
                }
            } else if cohort == bulk::PERFORMANCE_COHORT {
                "PERFORMANCE"
            } else {
                "OTHER"
            };
            categories.insert(coverage_key(id, type_def, method_def), category);
        }
    }
    Ok(categories)
}

fn provenance_coverage(
    view: &provenance::ProvenanceView,
    active: Option<&ActiveReceipt>,
    categories: &BTreeMap<String, &'static str>,
) -> ProvenanceCoverage {
    let mut covered_keys = BTreeSet::new();
    if view.state == provenance::ProvenanceState::StructurallyApplied {
        if let Some(document) = &view.document {
            for entry in &document.entries {
                if matches!(
                    entry.outcome,
                    provenance::Outcome::ResolvedExact | provenance::Outcome::ResolvedConservative
                ) {
                    covered_keys.insert(errata_entry_key(entry));
                }
            }
        }
    }
    let mut coverage = ProvenanceCoverage::default();
    let Some(active) = active else {
        return coverage;
    };
    for file in &active.files {
        for method in &file.methods {
            coverage.total += 1;
            let key = coverage_key(&file.id, &method.type_def, &method.method_def);
            if covered_keys.contains(&key) {
                coverage.covered += 1;
                continue;
            }
            match categories.get(&key).copied() {
                Some("ORDINARY") => coverage.ordinary_gameplay += 1,
                Some("GAMEPLAY_ASSOCIATED") => coverage.gameplay_associated += 1,
                Some("PERFORMANCE") => coverage.performance += 1,
                _ if file.id == "P" => coverage.performance += 1,
                _ => coverage.legacy_v5 += 1,
            }
        }
    }
    coverage
}

pub fn run_verify_provenance(research_root: Option<PathBuf>) -> (u8, Value) {
    run_verify_provenance_at(&curated_root(), research_root)
}

pub fn run_verify_provenance_at(
    curated_path: &Path,
    research_root: Option<PathBuf>,
) -> (u8, Value) {
    run_verify_provenance_at_with_trust(
        curated_path,
        research_root,
        &provenance::ProvenanceTrust::production(),
    )
}

pub fn run_verify_provenance_at_with_trust(
    curated_path: &Path,
    research_root: Option<PathBuf>,
    trust: &provenance::ProvenanceTrust,
) -> (u8, Value) {
    let LoadContext {
        manifest: _manifest,
        active,
        provenance: view,
        ..
    } = match load_context_at_with_trust(curated_path, trust) {
        Ok(context) => context,
        Err(error) => return error_envelope("verify-provenance", error),
    };
    let (byte_exit, byte_value) =
        run_verify_at_with_trust(curated_path, research_root.clone(), trust);
    let mut findings = Vec::new();
    if byte_exit != 0 {
        findings.push(finding("BYTE_VERIFY_FAILED", None, None));
    }
    let categories = if active.as_ref().is_some_and(|receipt| receipt.version == 6) {
        match provenance_catalog_categories(curated_path) {
            Ok(categories) => categories,
            Err(error) => return error_envelope("verify-provenance", error),
        }
    } else {
        BTreeMap::new()
    };
    let coverage = provenance_coverage(&view, active.as_ref(), &categories);
    let mut activation_status = "NOT_RUN";
    let mut activation_code = Value::Null;
    match view.state {
        provenance::ProvenanceState::NotApplicableBulkCorrection => {
            findings.push(finding(
                "PROVENANCE_NOT_APPLICABLE_BULK_CORRECTION",
                None,
                None,
            ));
        }
        provenance::ProvenanceState::Absent => {
            findings.push(finding("PROVENANCE_ERRATA_ABSENT", None, None));
        }
        provenance::ProvenanceState::PendingReview => {
            let code = view
                .activation_block
                .clone()
                .unwrap_or_else(|| "ACTIVATION_REVIEW_NOT_APPROVED".to_owned());
            activation_status = "FAIL";
            activation_code = Value::String(code.clone());
            findings.push(finding(&code, None, None));
        }
        provenance::ProvenanceState::StructurallyApplied => {
            match (research_root.as_deref(), view.document.as_ref()) {
                (Some(root), Some(document)) => {
                    match document.validate_for_activation(curated_path, root) {
                        Ok(()) => activation_status = "PASS",
                        Err(error) => {
                            activation_status = "FAIL";
                            activation_code = Value::String(error.code.clone());
                            findings.push(finding(
                                &error.code,
                                error.path.as_deref(),
                                error.record_id.as_deref(),
                            ));
                        }
                    }
                }
                _ => findings.push(finding("PROVENANCE_RESEARCH_ROOT_REQUIRED", None, None)),
            }
        }
    }
    let classified = coverage.covered
        + coverage.ordinary_gameplay
        + coverage.gameplay_associated
        + coverage.performance
        + coverage.legacy_v5;
    let uncovered = coverage.total.saturating_sub(coverage.covered);
    if uncovered > 0 {
        findings.push(finding("PROVENANCE_COVERAGE_INCOMPLETE", None, None));
    }
    if classified != coverage.total {
        findings.push(finding(
            "PROVENANCE_COVERAGE_CLASSIFICATION_INCOMPLETE",
            None,
            None,
        ));
    }
    let overall_pass = byte_exit == 0
        && view.state == provenance::ProvenanceState::StructurallyApplied
        && activation_status == "PASS"
        && uncovered == 0
        && classified == coverage.total;
    let overall = if overall_pass { "PASS" } else { "BLOCKED" };
    findings.sort_by_key(|value| value.to_string());
    findings.dedup();

    let mut summary = provenance_summary_value(&view, active.as_ref());
    if let Some(object) = summary.as_object_mut() {
        object.insert(
            "coverage".to_owned(),
            json!({
                "profile": provenance::PROVENANCE_PROFILE,
                "global_required_keys": coverage.total,
                "covered": coverage.covered,
                "not_validated": {
                    "ordinary_gameplay": coverage.ordinary_gameplay,
                    "gameplay_associated": coverage.gameplay_associated,
                    "performance": coverage.performance,
                    "legacy_v5": coverage.legacy_v5,
                    "total": uncovered
                }
            }),
        );
        object.insert(
            "overall_acceptance".to_owned(),
            Value::String(overall.to_owned()),
        );
    }
    let mut extras = BTreeMap::new();
    extras.insert(
        "byte_verify".to_owned(),
        json!({
            "exit": byte_exit,
            "ok": byte_exit == 0,
            "counts": byte_value.get("counts").cloned().unwrap_or(Value::Null),
            "findings": byte_value.get("findings").cloned().unwrap_or(Value::Null)
        }),
    );
    extras.insert("provenance".to_owned(), summary);
    extras.insert(
        "activation_verification".to_owned(),
        json!({
            "status": activation_status,
            "code": activation_code
        }),
    );
    extras.insert(
        "overall_acceptance".to_owned(),
        Value::String(overall.to_owned()),
    );
    extras.insert(
        "verify_scope".to_owned(),
        Value::String("BYTE_INTEGRITY_PLUS_PROVENANCE_ACTIVATION".to_owned()),
    );
    let exit = if byte_exit == 2 {
        2
    } else if overall_pass {
        0
    } else {
        1
    };
    let mut profile_counts = BTreeMap::<String, u64>::new();
    if let Some(document) = &view.document {
        for entry in &document.entries {
            *profile_counts
                .entry(entry.profile.as_str().to_owned())
                .or_default() += 1;
        }
    }
    let counts = json!({
        "active_files": active.as_ref().map(|receipt| receipt.files.len()).unwrap_or(0),
        "active_methods": coverage.total,
        "provenance_entries": view.entries,
        "resolved_entries": view.resolved_entries,
        "covered": coverage.covered,
        "not_validated": uncovered,
        "corrected": view.corrected,
        "confirmed": view.confirmed,
        "refined": view.refined,
        "profiles": profile_counts,
        "byte_verify_exit": byte_exit
    });
    (
        exit,
        envelope(
            "verify-provenance",
            exit,
            counts,
            Vec::new(),
            findings,
            extras,
        ),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliCommand {
    Status,
    Lookup { authority: String, token: String },
    Verify { research_root: Option<PathBuf> },
    VerifyProvenance { research_root: Option<PathBuf> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCli {
    pub command_name: String,
    pub curated_root: Option<PathBuf>,
    pub command: CliCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub command: String,
    pub code: String,
}

fn cli_error(command: &str, code: &str) -> CliError {
    CliError {
        command: command.to_owned(),
        code: code.to_owned(),
    }
}

pub fn parse_cli<I>(arguments: I) -> Result<ParsedCli, CliError>
where
    I: IntoIterator<Item = OsString>,
{
    let arguments: Vec<OsString> = arguments.into_iter().collect();
    let command = match arguments.first().and_then(|value| value.to_str()) {
        Some(value) => value,
        None => return Err(cli_error("unknown", "MISSING_COMMAND")),
    };
    let command_name = command.to_owned();
    let values: Vec<String> = arguments
        .iter()
        .skip(1)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| cli_error(command, "NON_UTF8_ARGUMENT"))
        })
        .collect::<Result<_, _>>()?;
    let mut json_count = 0_u8;
    let mut positional = Vec::new();
    let mut research_root = None;
    let mut curated_root = None;
    let mut index = 0;
    while index < values.len() {
        match values[index].as_str() {
            "--json" => {
                json_count = json_count.saturating_add(1);
                if json_count > 1 {
                    return Err(cli_error(command, "DUPLICATE_JSON_OPTION"));
                }
            }
            "--research-root" if command == "verify" || command == "verify-provenance" => {
                if research_root.is_some()
                    || index + 1 >= values.len()
                    || values[index + 1].starts_with('-')
                {
                    return Err(cli_error(command, "MALFORMED_RESEARCH_ROOT_OPTION"));
                }
                research_root = Some(PathBuf::from(&values[index + 1]));
                index += 1;
            }
            "--curated-root"
                if matches!(
                    command,
                    "status" | "lookup" | "verify" | "verify-provenance"
                ) =>
            {
                if curated_root.is_some() {
                    return Err(cli_error(command, "DUPLICATE_CURATED_ROOT_OPTION"));
                }
                if index + 1 >= values.len()
                    || values[index + 1].is_empty()
                    || values[index + 1].starts_with('-')
                {
                    return Err(cli_error(command, "MALFORMED_CURATED_ROOT_OPTION"));
                }
                curated_root = Some(PathBuf::from(&values[index + 1]));
                index += 1;
            }
            value if value.starts_with('-') => return Err(cli_error(command, "UNKNOWN_OPTION")),
            value => positional.push(value.to_owned()),
        }
        index += 1;
    }
    let parsed = match command {
        "status" if positional.is_empty() && research_root.is_none() => CliCommand::Status,
        "lookup" if positional.len() == 2 && research_root.is_none() => CliCommand::Lookup {
            authority: positional[0].clone(),
            token: positional[1].clone(),
        },
        "verify" if positional.is_empty() => CliCommand::Verify { research_root },
        "verify-provenance" if positional.is_empty() => {
            CliCommand::VerifyProvenance { research_root }
        }
        "status" | "lookup" | "verify" | "verify-provenance" => {
            return Err(cli_error(command, "INVALID_ARGUMENTS"));
        }
        _ => return Err(cli_error(command, "UNKNOWN_COMMAND")),
    };
    Ok(ParsedCli {
        command_name,
        curated_root,
        command: parsed,
    })
}

fn cli_error_envelope(error: CliError) -> (u8, Value) {
    (
        2,
        envelope(
            &error.command,
            2,
            json!({}),
            Vec::new(),
            vec![finding(&error.code, None, None)],
            BTreeMap::new(),
        ),
    )
}

pub fn run_with_args<I>(arguments: I) -> (u8, Value)
where
    I: IntoIterator<Item = OsString>,
{
    let parsed = match parse_cli(arguments) {
        Ok(parsed) => parsed,
        Err(error) => return cli_error_envelope(error),
    };
    let curated_root = cli_curated_root(parsed.curated_root);
    match parsed.command {
        CliCommand::Status => run_status_at(&curated_root),
        CliCommand::Lookup { authority, token } => run_lookup_at(&curated_root, &authority, &token),
        CliCommand::Verify { research_root } => run_verify_at(&curated_root, research_root),
        CliCommand::VerifyProvenance { research_root } => {
            run_verify_provenance_at(&curated_root, research_root)
        }
    }
}

#[cfg(test)]
mod cli_root_tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT_FIXTURE: AtomicU64 = AtomicU64::new(0);

    fn parsed(values: &[&str]) -> Result<ParsedCli, CliError> {
        parse_cli(values.iter().map(OsString::from))
    }

    fn fixture_root(label: &str) -> PathBuf {
        let number = NEXT_ROOT_FIXTURE.fetch_add(1, Ordering::Relaxed);
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-fixtures")
            .join(format!("cli-root-{label}-{}-{number}", std::process::id()));
        fs::create_dir_all(&path).expect("create owned CLI root fixture");
        path
    }

    #[test]
    fn curated_root_option_parses_for_every_ported_command() {
        for values in [
            vec!["status", "--curated-root", "root with spaces"],
            vec![
                "lookup",
                "MANAGED-CS-E3FC67839851",
                "0x0600042E",
                "--curated-root",
                "root with spaces",
            ],
            vec!["verify", "--curated-root", "root with spaces"],
            vec!["verify-provenance", "--curated-root", "root with spaces"],
        ] {
            let parsed = parsed(&values).expect("curated root parses");
            assert_eq!(parsed.curated_root, Some(PathBuf::from("root with spaces")));
        }
    }

    #[test]
    fn curated_root_option_rejects_duplicate_missing_and_malformed_values() {
        let error = parsed(&["status", "--curated-root", "one", "--curated-root", "two"])
            .expect_err("duplicate curated root is rejected");
        assert_eq!(error.code, "DUPLICATE_CURATED_ROOT_OPTION");

        let error = parsed(&["status", "--curated-root"]).expect_err("missing value is rejected");
        assert_eq!(error.code, "MALFORMED_CURATED_ROOT_OPTION");

        let error = parsed(&["status", "--curated-root", "--json"])
            .expect_err("option-looking value is rejected");
        assert_eq!(error.code, "MALFORMED_CURATED_ROOT_OPTION");

        let error = parse_cli(vec![
            OsString::from("status"),
            OsString::from("--curated-root"),
            OsString::from(""),
        ])
        .expect_err("empty value is rejected");
        assert_eq!(error.code, "MALFORMED_CURATED_ROOT_OPTION");

        let error = parsed(&["unknown-command", "--curated-root", "root"])
            .expect_err("unsupported command does not accept curated root");
        assert_eq!(error.code, "UNKNOWN_OPTION");
    }

    #[test]
    fn runtime_root_uses_nearest_metadata_ancestor_and_explicit_root_wins() {
        let fixture = fixture_root("ancestor");
        let root = fixture.join("repo with spaces");
        fs::create_dir_all(root.join("metadata")).expect("metadata dir");
        fs::write(root.join("metadata/migration.json"), b"{}").expect("migration marker");
        let nested = root.join("tools/deep/subdir");
        fs::create_dir_all(&nested).expect("nested dir");
        assert_eq!(nearest_metadata_root(&nested), Some(root.clone()));

        let empty = fixture.join("empty explicit root");
        fs::create_dir_all(&empty).expect("empty explicit root");
        assert_eq!(cli_curated_root(Some(empty.clone())), empty);

        fs::remove_dir_all(&fixture).expect("cleanup owned CLI root fixture");
    }

    #[cfg(windows)]
    #[test]
    fn runtime_root_accepts_spaced_ancestor_junction_without_skipping_creation() {
        use std::os::windows::fs::MetadataExt;

        struct JunctionFixture {
            parent: PathBuf,
            link: PathBuf,
        }
        impl Drop for JunctionFixture {
            fn drop(&mut self) {
                if self.link.exists() {
                    let _ = fs::remove_dir(&self.link);
                }
                let _ = fs::remove_dir_all(&self.parent);
            }
        }

        let parent = fixture_root("ancestor-junction");
        let target = parent.join("junction target with spaces");
        let link = parent.join("junction link with spaces");
        fs::create_dir_all(target.join("metadata")).expect("target metadata dir");
        fs::write(target.join("metadata/migration.json"), b"{}").expect("target marker");
        fs::create_dir_all(target.join("nested/deeper")).expect("target nested dir");
        let output = std::process::Command::new("cmd.exe")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .output()
            .expect("run native mklink /J");
        assert_eq!(
            output.status.code(),
            Some(0),
            "junction creation failed: stdout={:?} stderr={:?}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let _fixture = JunctionFixture {
            parent: parent.clone(),
            link: link.clone(),
        };
        let attributes = fs::symlink_metadata(&link)
            .expect("junction metadata")
            .file_attributes();
        assert_ne!(
            attributes & 0x400,
            0,
            "ancestor link is not a reparse point"
        );
        let nested = link.join("nested/deeper");
        assert_eq!(nearest_metadata_root(&nested), Some(link.clone()));
        assert!(link.join("metadata").join("migration.json").is_file());

        let (handler_exit, handler_value) = run_status_at(&link);
        assert_eq!(
            handler_exit, 2,
            "junction root must fail closed in the handler: {handler_value}"
        );
        assert_eq!(
            handler_value["findings"][0]["code"],
            "UNSAFE_REPARSE_OR_SYMLINK"
        );

        let (cli_exit, cli_value) = run_with_args(vec![
            OsString::from("status"),
            OsString::from("--json"),
            OsString::from("--curated-root"),
            link.clone().into_os_string(),
        ]);
        assert_eq!(
            cli_exit, 2,
            "junction root must fail closed through the CLI: {cli_value}"
        );
        assert_eq!(
            cli_value["findings"][0]["code"],
            "UNSAFE_REPARSE_OR_SYMLINK"
        );
    }
}
