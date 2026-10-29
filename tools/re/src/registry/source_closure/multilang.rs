use super::*;

const SUCCESSOR_SCHEMA: &str = "wuwa-reconstruction-source-closure-multilang-v1";
const SUCCESSOR_STATUS: &str = "PRIMARY_APPROVED_SCOPED_MULTILANG_CANDIDATE";
const EXPECTED_PREDECESSOR_SHA256: &str =
    "42f70e404a13bfbe5545ba81b946630d60a5ec15f2218d004abedff6a95e1b1d";
const EXPECTED_PREDECESSOR_SIZE: u64 = 198579;
const EXPECTED_ORIGINS: usize = 34;
const EXPECTED_EXISTING: usize = 7;
const EXPECTED_NEW: usize = 19;
const EXPECTED_PHYSICAL: usize = 26;
const EXPECTED_NOT_IMPORTED: usize = 8;
const EXPECTED_PRIMARY: usize = 1;
const EXPECTED_PROJECTIONS: usize = 22;
const EXPECTED_HISTORICAL: usize = 3;
const EXPECTED_PREDECESSOR_CURATED: usize = 112;
const EXPECTED_CURATED: usize = 131;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Predecessor {
    path: String,
    size: u64,
    sha256: String,
    origin_set_size: u64,
    origin_ids: Vec<String>,
    baseline_curated_files: u64,
    imported_files: u64,
    expected_curated_files: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LanguageCount {
    total: u64,
    #[serde(rename = "new")]
    new_count: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Summary {
    origins: u64,
    existing_imports: u64,
    new_imports: u64,
    physical_files: u64,
    not_imported: u64,
    primary: u64,
    projections: u64,
    historical: u64,
    origin_language_counts: BTreeMap<String, u64>,
    physical_language_counts: BTreeMap<String, LanguageCount>,
    predecessor_expected_curated_files: u64,
    new_expected_curated_files: u64,
    expected_curated_files: u64,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum SuccessorDisposition {
    ImportedSourceProjection,
    ImportedPrimarySource,
    DeclarationOnlyNotImported,
    ReferenceOnly,
    PreservedHistoricalAlternative,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum NormalizedOriginClass {
    RecoveredImplementation,
    RecoveredSourceProjection,
    ShippedScript,
    IndependentResearchModel,
    DeclarationOrInterface,
    HistoricalAlternative,
    MetadataOnly,
    Unresolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RouteTier {
    PrimaryTooling,
    Projection,
    Declaration,
    Reference,
    Historical,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedRoute {
    origin_id: &'static str,
    disposition: SuccessorDisposition,
    category: &'static str,
    origin_class: NormalizedOriginClass,
    representation_class: &'static str,
    tier: RouteTier,
    path: Option<&'static str>,
    mode: Option<&'static str>,
    already_imported: bool,
}

const EXPECTED_ROUTES: [ExpectedRoute; 34] = [
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-BOSSPILINGBUFF",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/BossPilingBuff.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-BOSSRUSHBUFF",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/BossRushBuff.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-COMBOTEACHING",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/ComboTeaching.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-FLAGCHALLENGEROLEBUFF",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/FlagChallengeRoleBuff.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-FLOWTEXT",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/FlowText.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-INTERACTDATA",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/InteractData.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-KUROTATOGENERALGROWTH",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/KurotatoGeneralGrowth.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-PAYSHOPTAB",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/PayShopTab.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-ROVERROGUEROOM",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/RoverRogueRoom.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-SKILLTREE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/SkillTree.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-SORTRULE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/SortRule.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-CONFIG-TETRISMAP",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/scripts/typescript/declarations/config/TetrisMap.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-LOADING-PIPELINE",
        disposition: SuccessorDisposition::ReferenceOnly,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::Unresolved,
        representation_class: "EMPTY_REFERENCE_SKELETON",
        tier: RouteTier::Reference,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-STREAMING-COMPLETION-TRANSITION",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "DECLARATIVE_PROJECTION_WITH_CONST_TABLES",
        tier: RouteTier::Projection,
        path: Some("src/projections/typescript/StreamingCompletionTransition.ts"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-TS-DATALAYER-TRANSITION",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/scripts/reconstructed/transition/DataLayerTransition.ts"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-EVENT-CORE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/scripts/reconstructed/event/Event.js"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-EVENT-CSHARP-BRIDGE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/scripts/reconstructed/event/EventCSharpBridge.js"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-EVENT-CSHARP-BRIDGE-R15",
        disposition: SuccessorDisposition::PreservedHistoricalAlternative,
        category: "HISTORICAL",
        origin_class: NormalizedOriginClass::HistoricalAlternative,
        representation_class: "HISTORICAL_ALTERNATIVE_SOURCE_VIEW",
        tier: RouteTier::Historical,
        path: Some("src/scripts/javascript/historical/bridge-r15/EventCSharpBridge.js"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-CHARACTER-HIT-R27",
        disposition: SuccessorDisposition::PreservedHistoricalAlternative,
        category: "HISTORICAL",
        origin_class: NormalizedOriginClass::HistoricalAlternative,
        representation_class: "HISTORICAL_ALTERNATIVE_SOURCE_VIEW",
        tier: RouteTier::Historical,
        path: Some(
            "src/scripts/javascript/historical/gameplay-r27/CharacterHitComponent.historical.reconstructed.js",
        ),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-ROLE-ELEMENT-R28",
        disposition: SuccessorDisposition::PreservedHistoricalAlternative,
        category: "HISTORICAL",
        origin_class: NormalizedOriginClass::HistoricalAlternative,
        representation_class: "HISTORICAL_ALTERNATIVE_SOURCE_VIEW",
        tier: RouteTier::Historical,
        path: Some(
            "src/scripts/javascript/historical/gameplay-r28/RoleElementComponent.historical.reconstructed.js",
        ),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-JS-CHECK-PERF03B-SEMANTICS",
        disposition: SuccessorDisposition::ReferenceOnly,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::IndependentResearchModel,
        representation_class: "INTERNAL_CHECKER",
        tier: RouteTier::Reference,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-CPP-CORECLR-NATIVE-THREAD",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/models/native/CoreclrNativeThread.cpp"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-CPP-CORECLR-THREAD-ORIGIN",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/models/native/CoreclrThreadOrigin.cpp"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-CPP-LIBXELL-HELPER-TARGET",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/models/native/LibxellHelperTarget.cpp"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-CPP-LIBXELL-SLEEP-IMPLEMENTATION",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::RecoveredSourceProjection,
        representation_class: "SOURCE_LIKE_PROJECTION",
        tier: RouteTier::Projection,
        path: Some("src/models/native/LibxellSleepImplementation.cpp"),
        mode: Some("EXISTING_IMMUTABLE"),
        already_imported: true,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-FEVENTSYSTEM",
        disposition: SuccessorDisposition::ReferenceOnly,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::MetadataOnly,
        representation_class: "METADATA_ONLY_COMMENT_SHELL",
        tier: RouteTier::Reference,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-KUROKVSTORE",
        disposition: SuccessorDisposition::ReferenceOnly,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::MetadataOnly,
        representation_class: "METADATA_ONLY_COMMENT_SHELL",
        tier: RouteTier::Reference,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-STREAMING-COMPLETION-BRIDGE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/projections/native/include/StreamingCompletionBridge.hpp"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-WORLD-PARTITION-BRIDGE",
        disposition: SuccessorDisposition::ImportedSourceProjection,
        category: "PROJECTION",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_OR_INTERFACE",
        tier: RouteTier::Declaration,
        path: Some("src/projections/native/include/WorldPartitionBridge.hpp"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-EVIDENCE-LABELS",
        disposition: SuccessorDisposition::ImportedPrimarySource,
        category: "PRIMARY",
        origin_class: NormalizedOriginClass::IndependentResearchModel,
        representation_class: "PRIMARY_TOOLING_DECLARATION",
        tier: RouteTier::PrimaryTooling,
        path: Some("src/native/include/research/evidence_labels.hpp"),
        mode: Some("EXACT_BYTE_COPY"),
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-RESOURCE-INTERFACES",
        disposition: SuccessorDisposition::DeclarationOnlyNotImported,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_ONLY_PLACEHOLDER",
        tier: RouteTier::Declaration,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-STREAMING-INTERFACES",
        disposition: SuccessorDisposition::DeclarationOnlyNotImported,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_ONLY_PLACEHOLDER",
        tier: RouteTier::Declaration,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-TRANSITION-INTERFACES",
        disposition: SuccessorDisposition::DeclarationOnlyNotImported,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_ONLY_PLACEHOLDER",
        tier: RouteTier::Declaration,
        path: None,
        mode: None,
        already_imported: false,
    },
    ExpectedRoute {
        origin_id: "SRC-HPP-WORLD-PARTITION-INTERFACES",
        disposition: SuccessorDisposition::DeclarationOnlyNotImported,
        category: "NOT_IMPORTED",
        origin_class: NormalizedOriginClass::DeclarationOrInterface,
        representation_class: "DECLARATION_ONLY_PLACEHOLDER",
        tier: RouteTier::Declaration,
        path: None,
        mode: None,
        already_imported: false,
    },
];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PhysicalImport {
    path: String,
    size: u64,
    sha256: String,
    mode: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SuccessorEntry {
    origin_id: String,
    original: ArtifactRef,
    language: String,
    subsystem: String,
    predecessor_origin_class: String,
    origin_class: NormalizedOriginClass,
    representation_class: String,
    category: String,
    implementation_content: String,
    declaration_content: String,
    provenance_confidence: String,
    authority: Authority,
    identity_scheme: String,
    evidence_references: Vec<ProofAnchor>,
    dependencies: Vec<String>,
    overlap: String,
    distribution: String,
    disposition: SuccessorDisposition,
    already_imported: bool,
    historical_active: bool,
    reason: String,
    function_count: u64,
    body_count: u64,
    declaration_count: u64,
    field_count: u64,
    partial_count: u64,
    syntax_status: String,
    physical_import: Option<PhysicalImport>,
    line_evidence: Vec<String>,
    limitations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SuccessorManifest {
    schema: String,
    manifest_version: u64,
    campaign: String,
    status: String,
    not_final: bool,
    publication: String,
    predecessor: Predecessor,
    summary: Summary,
    new_paths: Vec<String>,
    existing_import_paths: Vec<String>,
    not_imported_origin_ids: Vec<String>,
    entries: Vec<SuccessorEntry>,
    limits: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct Successor {
    manifest: SuccessorManifest,
    sha256: String,
}

impl Successor {
    pub(super) fn sha256(&self) -> &str {
        &self.sha256
    }

    pub(super) fn summary_value(&self) -> Value {
        json!({
            "schema": self.manifest.schema,
            "campaign": self.manifest.campaign,
            "status": self.manifest.status,
            "not_final": self.manifest.not_final,
            "publication": self.manifest.publication,
            "manifest_sha256": self.sha256,
            "summary": self.manifest.summary,
            "new_paths": self.manifest.new_paths,
            "existing_import_paths": self.manifest.existing_import_paths,
            "not_imported_origin_ids": self.manifest.not_imported_origin_ids,
            "limits": self.manifest.limits,
        })
    }

    pub(super) fn expected_paths(&self) -> BTreeSet<String> {
        self.manifest.new_paths.iter().cloned().collect()
    }

    pub(super) fn extra_survey_roots(&self) -> &'static [&'static str] {
        &MULTILANG_CURATED_SURVEY_ROOTS
    }

    pub(super) fn lookup_value(&self, origin_id: &str) -> Option<Value> {
        let entry = self
            .manifest
            .entries
            .iter()
            .find(|entry| entry.origin_id == origin_id)?;
        serde_json::to_value(entry).ok()
    }

    pub(super) fn artifact_lookup_value(&self, root_id: &str, path: &str) -> Option<Value> {
        let entry = self
            .manifest
            .entries
            .iter()
            .find(|entry| entry.original.root_id == root_id && entry.original.path == path)?;
        serde_json::to_value(entry).ok()
    }

    /// Exact successor physical-import lookup. This is a closed join over the
    /// reviewed 26 physical routes, not arbitrary path discovery.
    pub(super) fn physical_lookup_value(&self, path: &str) -> Option<Value> {
        let entry = self.manifest.entries.iter().find(|entry| {
            entry
                .physical_import
                .as_ref()
                .map(|import| import.path.as_str())
                == Some(path)
        })?;
        let physical = entry.physical_import.as_ref()?;
        Some(json!({
            "origin_id": entry.origin_id,
            "language": entry.language,
            "subsystem": entry.subsystem,
            "category": entry.category,
            "disposition": entry.disposition,
            "representation_class": entry.representation_class,
            "already_imported": entry.already_imported,
            "historical_active": entry.historical_active,
            "reason": entry.reason,
            "original": entry.original,
            "physical_import": physical,
            "authority": entry.authority,
            "identity_scheme": entry.identity_scheme,
            "limitations": entry.limitations,
        }))
    }

    pub(super) fn physical_paths(&self) -> BTreeSet<String> {
        self.manifest
            .entries
            .iter()
            .filter_map(|entry| {
                entry
                    .physical_import
                    .as_ref()
                    .map(|import| import.path.clone())
            })
            .collect()
    }
}

fn successor_pin(trust: &Trust) -> RegistryResult<&str> {
    trust.multilang_manifest_sha256.as_deref().ok_or_else(|| {
        reg(
            "TRUST_PIN_UNAVAILABLE",
            "successor manifest pin is not authorized; candidate CLI fails closed",
        )
    })
}

fn decode_candidate(
    bytes: &[u8],
    expected: &str,
    roots: &Roots,
    parent: &SourceManifest,
    parent_bytes: &[u8],
    verify_bytes: bool,
) -> RegistryResult<Successor> {
    let hash = crate::sha256_hex(bytes);
    if hash != expected {
        return Err(reg(
            "TRUST_MULTILANG_HASH_MISMATCH",
            "successor manifest bytes do not match the explicit trust pin",
        ));
    }
    let value = crate::parse_strict_value(bytes).map_err(|_| {
        reg(
            "MULTILANG_MANIFEST_INVALID_JSON",
            "successor manifest is not strict JSON",
        )
    })?;
    let manifest: SuccessorManifest = serde_json::from_value(value).map_err(|_| {
        reg(
            "MULTILANG_MANIFEST_INVALID_SHAPE",
            "successor manifest shape is invalid",
        )
    })?;
    validate_successor(&manifest, parent, parent_bytes, roots, verify_bytes)?;
    Ok(Successor {
        manifest,
        sha256: hash,
    })
}

pub(super) fn load_candidate(
    roots: &Roots,
    parent: &SourceManifest,
    parent_bytes: &[u8],
    trust: &Trust,
    verify_bytes: bool,
) -> RegistryResult<Successor> {
    let expected = successor_pin(trust)?;
    let path = roots.curated.join(MULTILANG_MANIFEST_REL);
    inspect_existing_file(&path)?;
    let bytes = fs::read(&path).map_err(|_| {
        reg(
            "MULTILANG_MANIFEST_MISSING",
            "successor manifest read failed",
        )
    })?;
    decode_candidate(&bytes, expected, roots, parent, parent_bytes, verify_bytes)
}

#[cfg(test)]
fn load_candidate_bytes(
    bytes: &[u8],
    roots: &Roots,
    parent: &SourceManifest,
    parent_bytes: &[u8],
    trust: &Trust,
    verify_bytes: bool,
) -> RegistryResult<Successor> {
    let expected = successor_pin(trust)?;
    decode_candidate(bytes, expected, roots, parent, parent_bytes, verify_bytes)
}

fn same_artifact(left: &ArtifactRef, right: &ArtifactRef) -> bool {
    left.root_id == right.root_id
        && left.path == right.path
        && left.size == right.size
        && left.sha256 == right.sha256
}

fn layer_language(layer: &str) -> &'static str {
    match layer {
        "TS" => "TS",
        "JS" => "JS",
        "CPP" => "CPP",
        "CPP_HEADER" => "CPP_HEADER",
        _ => "UNKNOWN",
    }
}

fn expected_extension(language: &str) -> &'static str {
    match language {
        "TS" => "ts",
        "JS" => "js",
        "CPP" => "cpp",
        "CPP_HEADER" => "hpp",
        _ => "",
    }
}

fn duplicate_set(values: &[String]) -> bool {
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    set.len() != values.len()
}

fn typed_equal<T: Serialize>(left: &T, right: &T) -> bool {
    serde_json::to_value(left).ok() == serde_json::to_value(right).ok()
}

fn validate_successor(
    manifest: &SuccessorManifest,
    parent: &SourceManifest,
    parent_bytes: &[u8],
    roots: &Roots,
    verify_bytes: bool,
) -> RegistryResult<()> {
    if manifest.schema != SUCCESSOR_SCHEMA
        || manifest.manifest_version != 1
        || manifest.campaign != "RE-MULTILANG-SRC01"
        || manifest.status != SUCCESSOR_STATUS
        || !manifest.not_final
        || manifest.publication != "DISABLED_INTERNAL_ONLY"
    {
        return Err(reg(
            "MULTILANG_MANIFEST_HEADER_INVALID",
            "successor schema/status/finality/publication header is invalid",
        ));
    }
    let parent_sha = crate::sha256_hex(parent_bytes);
    if parent_sha != EXPECTED_PREDECESSOR_SHA256
        || parent_bytes.len() as u64 != EXPECTED_PREDECESSOR_SIZE
    {
        return Err(reg(
            "MULTILANG_PREDECESSOR_ACTUAL_MISMATCH",
            "actual predecessor bytes are not the accepted 42f70e40 pin",
        ));
    }
    if manifest.predecessor.path != MANIFEST_REL
        || manifest.predecessor.size != parent_bytes.len() as u64
        || manifest.predecessor.sha256 != parent_sha
        || manifest.predecessor.origin_set_size != EXPECTED_ORIGINS as u64
        || manifest.predecessor.baseline_curated_files != 104
        || manifest.predecessor.imported_files != parent.imports.len() as u64
        || manifest.predecessor.expected_curated_files != EXPECTED_PREDECESSOR_CURATED as u64
    {
        return Err(reg(
            "MULTILANG_PREDECESSOR_BINDING_INVALID",
            "successor predecessor size/hash/count binding is invalid",
        ));
    }
    let parent_ids = parent
        .non_managed_artifacts
        .iter()
        .map(|row| row.origin_id.clone())
        .collect::<BTreeSet<_>>();
    let predecessor_ids = manifest
        .predecessor
        .origin_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if parent_ids.len() != EXPECTED_ORIGINS
        || predecessor_ids.len() != EXPECTED_ORIGINS
        || predecessor_ids != parent_ids
        || duplicate_set(&manifest.predecessor.origin_ids)
    {
        return Err(reg(
            "MULTILANG_PREDECESSOR_ORIGIN_SET_MISMATCH",
            "predecessor origin set is not the exact accepted 34",
        ));
    }
    if manifest.entries.len() != EXPECTED_ORIGINS
        || duplicate_set(
            &manifest
                .entries
                .iter()
                .map(|entry| entry.origin_id.clone())
                .collect::<Vec<_>>(),
        )
    {
        return Err(reg(
            "MULTILANG_ORIGIN_SET_INVALID",
            "successor entries must contain each origin exactly once",
        ));
    }
    let entry_ids = manifest
        .entries
        .iter()
        .map(|entry| entry.origin_id.clone())
        .collect::<BTreeSet<_>>();
    if entry_ids != parent_ids {
        return Err(reg(
            "MULTILANG_ORIGIN_SET_MISMATCH",
            "successor origin set differs from the predecessor 34",
        ));
    }

    let parent_by_id = parent
        .non_managed_artifacts
        .iter()
        .map(|row| (row.origin_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let import_by_origin = parent
        .imports
        .iter()
        .filter_map(|import| import.origin_id.as_ref().map(|id| (id.clone(), import)))
        .collect::<BTreeMap<_, _>>();
    let old_curated = parent
        .baseline
        .accepted_composition
        .baseline_curated_paths
        .iter()
        .cloned()
        .chain(
            parent
                .imports
                .iter()
                .map(|import| import.destination.path.clone()),
        )
        .collect::<BTreeSet<_>>();

    let route_by_id = EXPECTED_ROUTES
        .iter()
        .map(|route| (route.origin_id, route))
        .collect::<BTreeMap<_, _>>();
    if route_by_id.len() != EXPECTED_ORIGINS {
        return Err(reg(
            "MULTILANG_ROUTE_TABLE_INVALID",
            "compiled exact route table must contain 34 unique origins",
        ));
    }
    let route_paths = EXPECTED_ROUTES
        .iter()
        .filter_map(|route| route.path)
        .collect::<BTreeSet<_>>();
    if route_paths.len() != EXPECTED_PHYSICAL {
        return Err(reg(
            "MULTILANG_ROUTE_TABLE_INVALID",
            "compiled exact route table must contain 26 unique physical paths",
        ));
    }
    let malformed_configs = [
        "SRC-TS-CONFIG-KUROTATOGENERALGROWTH",
        "SRC-TS-CONFIG-SKILLTREE",
    ];

    let mut physical_paths = BTreeSet::new();
    let mut new_paths = BTreeSet::new();
    let mut existing_paths = BTreeSet::new();
    let mut not_imported_ids = BTreeSet::new();
    let mut primary = 0_u64;
    let mut projections = 0_u64;
    let mut historical_count = 0_u64;
    let mut existing_count = 0_u64;
    let mut new_count = 0_u64;
    let mut origin_language_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut physical_language_counts: BTreeMap<String, LanguageCount> = BTreeMap::new();

    for entry in &manifest.entries {
        let parent_row = parent_by_id.get(&entry.origin_id).ok_or_else(|| {
            reg(
                "MULTILANG_ORIGIN_NOT_IN_PREDECESSOR",
                format!("origin is not in predecessor: {}", entry.origin_id),
            )
        })?;
        let route = route_by_id.get(entry.origin_id.as_str()).ok_or_else(|| {
            reg(
                "MULTILANG_ROUTE_NOT_FOUND",
                format!("origin has no exact reviewed route: {}", entry.origin_id),
            )
        })?;
        if !same_artifact(&entry.original, &parent_row.artifact) {
            return Err(reg(
                "MULTILANG_ORIGIN_BINDING_MISMATCH",
                format!("origin ref differs from predecessor: {}", entry.origin_id),
            ));
        }
        if entry.predecessor_origin_class != parent_row.origin_class {
            return Err(reg(
                "MULTILANG_PREDECESSOR_ORIGIN_CLASS_MISMATCH",
                format!("predecessor origin label differs: {}", entry.origin_id),
            ));
        }
        if !typed_equal(&entry.authority, &parent_row.authority) {
            return Err(reg(
                "MULTILANG_AUTHORITY_MISMATCH",
                format!(
                    "authority is not the exact parent authority: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.identity_scheme != parent_row.identity_scheme {
            return Err(reg(
                "MULTILANG_IDENTITY_SCHEME_MISMATCH",
                format!(
                    "identity scheme is not the exact parent identity scheme: {}",
                    entry.origin_id
                ),
            ));
        }
        if !typed_equal(&entry.evidence_references, &parent_row.proof_anchors) {
            return Err(reg(
                "MULTILANG_EVIDENCE_REFERENCE_MISMATCH",
                format!(
                    "evidence references are not the exact parent proof anchors: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.dependencies != parent_row.dependencies
            || entry.distribution != parent_row.public_distribution
        {
            return Err(reg(
                "MULTILANG_ORIGIN_COLUMN_MISMATCH",
                format!("predecessor-derived columns differ: {}", entry.origin_id),
            ));
        }
        if entry.disposition != route.disposition {
            return Err(reg(
                "MULTILANG_ROUTE_DISPOSITION_MISMATCH",
                format!(
                    "disposition does not match exact reviewed route: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.category != route.category {
            return Err(reg(
                "MULTILANG_ROUTE_CATEGORY_MISMATCH",
                format!(
                    "category does not match exact reviewed route: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.origin_class != route.origin_class {
            return Err(reg(
                "MULTILANG_ROUTE_ORIGIN_CLASS_MISMATCH",
                format!(
                    "normalized origin class does not match exact reviewed route: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.representation_class != route.representation_class {
            return Err(reg(
                "MULTILANG_ROUTE_REPRESENTATION_MISMATCH",
                format!(
                    "representation class does not match exact reviewed route: {}",
                    entry.origin_id
                ),
            ));
        }
        if entry.already_imported != route.already_imported {
            return Err(reg(
                "MULTILANG_ROUTE_IMPORT_MODE_MISMATCH",
                format!(
                    "already_imported does not match exact reviewed route: {}",
                    entry.origin_id
                ),
            ));
        }
        match (route.path, &entry.physical_import) {
            (Some(expected_path), Some(physical)) => {
                if physical.path != expected_path {
                    return Err(reg(
                        "MULTILANG_ROUTE_DESTINATION_MISMATCH",
                        format!(
                            "destination is not the exact reviewed path: {}",
                            entry.origin_id
                        ),
                    ));
                }
                if Some(physical.mode.as_str()) != route.mode {
                    return Err(reg(
                        "MULTILANG_ROUTE_MODE_MISMATCH",
                        format!(
                            "destination mode is not the exact reviewed mode: {}",
                            entry.origin_id
                        ),
                    ));
                }
                if ext_of(&physical.path) != expected_extension(&entry.language) {
                    return Err(reg(
                        "MULTILANG_ROUTE_EXTENSION_MISMATCH",
                        format!(
                            "destination extension does not match language: {}",
                            entry.origin_id
                        ),
                    ));
                }
            }
            (None, None) => {}
            (Some(_), None) => {
                return Err(reg(
                    "MULTILANG_ROUTE_PHYSICAL_MISSING",
                    format!("reviewed physical route has no import: {}", entry.origin_id),
                ));
            }
            (None, Some(_)) => {
                return Err(reg(
                    "MULTILANG_ROUTE_PHYSICAL_FORBIDDEN",
                    format!(
                        "non-import reviewed route must not have a physical import: {}",
                        entry.origin_id
                    ),
                ));
            }
        }
        let expected_language = layer_language(&parent_row.layer);
        if entry.language != expected_language {
            return Err(reg(
                "MULTILANG_LANGUAGE_MISMATCH",
                format!(
                    "language does not match predecessor layer: {}",
                    entry.origin_id
                ),
            ));
        }
        *origin_language_counts
            .entry(entry.language.clone())
            .or_insert(0) += 1;
        if entry.line_evidence.is_empty()
            || entry
                .line_evidence
                .iter()
                .any(|line| !line.starts_with("line "))
            || entry.reason.trim().is_empty()
            || entry.subsystem.trim().is_empty()
            || entry.provenance_confidence.trim().is_empty()
            || entry.declaration_content.trim().is_empty()
            || entry.limitations.is_empty()
        {
            return Err(reg(
                "MULTILANG_REVIEW_EVIDENCE_MISSING",
                format!("review evidence is incomplete: {}", entry.origin_id),
            ));
        }

        let is_config = entry.origin_id.starts_with("SRC-TS-CONFIG-");
        let is_malformed = malformed_configs.contains(&entry.origin_id.as_str());
        let is_historical = route.tier == RouteTier::Historical;
        if entry.language == "TS" {
            let expected_syntax = if is_malformed {
                "DECLARED_MALFORMED_BY_INSPECTION"
            } else {
                "UNVERIFIED_TS_SYNTAX_TSC_UNAVAILABLE"
            };
            if entry.syntax_status != expected_syntax {
                return Err(reg(
                    "MULTILANG_SYNTAX_STATUS_MISMATCH",
                    format!("TS syntax status is not honest: {}", entry.origin_id),
                ));
            }
        } else if entry.language == "JS" {
            if entry.syntax_status != "SYNTAX_OK_NODE_CHECK" {
                return Err(reg(
                    "MULTILANG_SYNTAX_STATUS_MISMATCH",
                    format!("JS syntax status is not node --check: {}", entry.origin_id),
                ));
            }
        } else if entry.language == "CPP" {
            if entry.syntax_status != "UNVERIFIED_CPP_OPAQUE_MODEL" {
                return Err(reg(
                    "MULTILANG_SYNTAX_STATUS_MISMATCH",
                    format!(
                        "CPP opaque model must remain unverified: {}",
                        entry.origin_id
                    ),
                ));
            }
        } else if entry.language == "CPP_HEADER" {
            if entry.syntax_status != "SYNTAX_OK_GXX_SYNTAX_ONLY" {
                return Err(reg(
                    "MULTILANG_SYNTAX_STATUS_MISMATCH",
                    format!(
                        "header syntax status is not g++ syntax-only: {}",
                        entry.origin_id
                    ),
                ));
            }
            if entry.function_count != 0 || entry.body_count != 0 {
                return Err(reg(
                    "MULTILANG_HEADER_BODY_FORBIDDEN",
                    format!("header must not claim bodies: {}", entry.origin_id),
                ));
            }
        }

        match entry.disposition {
            SuccessorDisposition::ImportedPrimarySource => {
                if route.tier != RouteTier::PrimaryTooling
                    || entry.origin_id != "SRC-HPP-EVIDENCE-LABELS"
                    || entry.language != "CPP_HEADER"
                    || entry.representation_class != "PRIMARY_TOOLING_DECLARATION"
                    || entry.function_count != 0
                    || entry.body_count != 0
                    || entry.declaration_count != 2
                    || entry.implementation_content != "NONE"
                {
                    return Err(reg(
                        "MULTILANG_PRIMARY_CLASS_INVALID",
                        "the only primary tooling source must be the evidence_labels header",
                    ));
                }
                primary += 1;
            }
            SuccessorDisposition::ImportedSourceProjection => {
                projections += 1;
                if route.tier == RouteTier::Declaration {
                    if entry.function_count != 0 || entry.body_count != 0 {
                        return Err(reg(
                            "MULTILANG_DECLARATION_BODY_FORBIDDEN",
                            format!(
                                "declaration-tier route must not claim bodies: {}",
                                entry.origin_id
                            ),
                        ));
                    }
                    if is_config {
                        if entry.representation_class != "DECLARATION_OR_INTERFACE"
                            || entry.implementation_content != "NONE"
                            || entry.field_count == 0
                            || entry.declaration_count != 1
                        {
                            return Err(reg(
                                "MULTILANG_CONFIG_DECLARATION_INVALID",
                                format!("config declaration shape invalid: {}", entry.origin_id),
                            ));
                        }
                    } else if entry.language != "CPP_HEADER"
                        || entry.representation_class != "DECLARATION_OR_INTERFACE"
                        || entry.declaration_count == 0
                    {
                        return Err(reg(
                            "MULTILANG_HEADER_DECLARATION_INVALID",
                            format!("header must remain declaration-only: {}", entry.origin_id),
                        ));
                    }
                } else if route.tier == RouteTier::Projection {
                    if entry.origin_id == "SRC-TS-STREAMING-COMPLETION-TRANSITION"
                        && (entry.function_count != 0
                            || entry.body_count != 2
                            || entry.declaration_count != 12
                            || entry.implementation_content != "CONSTANT_TABLES_ONLY")
                    {
                        return Err(reg(
                            "MULTILANG_STREAMING_PROJECTION_INVALID",
                            "streaming projection must remain constant tables/declarations",
                        ));
                    }
                } else {
                    return Err(reg(
                        "MULTILANG_ROUTE_TIER_MISMATCH",
                        format!(
                            "projection disposition is not allowed for route tier: {}",
                            entry.origin_id
                        ),
                    ));
                }
            }
            SuccessorDisposition::PreservedHistoricalAlternative => {
                if route.tier != RouteTier::Historical
                    || !is_historical
                    || entry.historical_active
                    || entry.already_imported
                    || !entry.overlap.contains("ACTIVE_FALSE")
                {
                    return Err(reg(
                        "MULTILANG_HISTORICAL_ACTIVE_OVERLAP",
                        format!(
                            "historical alternative must be inactive: {}",
                            entry.origin_id
                        ),
                    ));
                }
                historical_count += 1;
            }
            SuccessorDisposition::ReferenceOnly => {
                if route.tier != RouteTier::Reference {
                    return Err(reg(
                        "MULTILANG_ROUTE_TIER_MISMATCH",
                        format!(
                            "reference disposition is not allowed for route tier: {}",
                            entry.origin_id
                        ),
                    ));
                }
                not_imported_ids.insert(entry.origin_id.clone());
            }
            SuccessorDisposition::DeclarationOnlyNotImported => {
                if route.tier != RouteTier::Declaration
                    || entry.language != "CPP_HEADER"
                    || entry.function_count != 0
                    || entry.body_count != 0
                    || entry.declaration_count != 1
                    || route.path.is_some()
                {
                    return Err(reg(
                        "MULTILANG_PLACEHOLDER_PROMOTION",
                        format!("placeholder cannot be promoted: {}", entry.origin_id),
                    ));
                }
                not_imported_ids.insert(entry.origin_id.clone());
            }
        }

        match (&entry.physical_import, entry.already_imported) {
            (Some(physical), true) => {
                let import = import_by_origin.get(&entry.origin_id).ok_or_else(|| {
                    reg(
                        "MULTILANG_EXISTING_IMPORT_MISMATCH",
                        format!(
                            "existing import has no predecessor import: {}",
                            entry.origin_id
                        ),
                    )
                })?;
                if physical.mode != "EXISTING_IMMUTABLE"
                    || physical.path != import.destination.path
                    || physical.size != import.destination.size
                    || physical.sha256 != import.destination.sha256
                {
                    return Err(reg(
                        "MULTILANG_EXISTING_IMPORT_MISMATCH",
                        format!("existing import path/hash changed: {}", entry.origin_id),
                    ));
                }
                existing_count += 1;
                existing_paths.insert(physical.path.clone());
            }
            (Some(physical), false) => {
                if physical.mode != "EXACT_BYTE_COPY"
                    || physical.size != entry.original.size
                    || physical.sha256 != entry.original.sha256
                    || old_curated.contains(&physical.path)
                {
                    return Err(reg(
                        "MULTILANG_NEW_IMPORT_INVALID",
                        format!("new import binding/path invalid: {}", entry.origin_id),
                    ));
                }
                new_count += 1;
                new_paths.insert(physical.path.clone());
            }
            (None, true) => {
                return Err(reg(
                    "MULTILANG_EXISTING_IMPORT_MISSING",
                    format!(
                        "existing import has no physical reference: {}",
                        entry.origin_id
                    ),
                ));
            }
            (None, false) => {
                if !matches!(
                    entry.disposition,
                    SuccessorDisposition::ReferenceOnly
                        | SuccessorDisposition::DeclarationOnlyNotImported
                ) {
                    return Err(reg(
                        "MULTILANG_PHYSICAL_IMPORT_MISSING",
                        format!(
                            "non-reference row has no physical import: {}",
                            entry.origin_id
                        ),
                    ));
                }
            }
        }

        if let Some(physical) = &entry.physical_import {
            if !physical_paths.insert(physical.path.clone()) {
                return Err(reg(
                    "MULTILANG_DESTINATION_DUPLICATE",
                    format!("destination path is duplicated: {}", physical.path),
                ));
            }
            let destination = ArtifactRef {
                root_id: "CURATED_ROOT".to_string(),
                path: physical.path.clone(),
                size: physical.size,
                sha256: physical.sha256.clone(),
            };
            resolve_ref(roots, &destination)?;
            if verify_bytes {
                let origin = read_artifact(roots, &entry.original)?;
                let copied = read_artifact(roots, &destination)?;
                if origin != copied {
                    return Err(reg(
                        "MULTILANG_PHYSICAL_BYTE_DRIFT",
                        format!("copy bytes differ from origin: {}", physical.path),
                    ));
                }
            }
            let count = physical_language_counts
                .entry(entry.language.clone())
                .or_insert(LanguageCount {
                    total: 0,
                    new_count: 0,
                });
            count.total += 1;
            if !entry.already_imported {
                count.new_count += 1;
            }
        } else if entry.already_imported {
            return Err(reg(
                "MULTILANG_EXISTING_IMPORT_MISSING",
                format!(
                    "existing import has no physical reference: {}",
                    entry.origin_id
                ),
            ));
        }
    }

    if primary != EXPECTED_PRIMARY as u64
        || projections != EXPECTED_PROJECTIONS as u64
        || historical_count != EXPECTED_HISTORICAL as u64
        || existing_count != EXPECTED_EXISTING as u64
        || new_count != EXPECTED_NEW as u64
        || physical_paths.len() != EXPECTED_PHYSICAL
        || not_imported_ids.len() != EXPECTED_NOT_IMPORTED
    {
        return Err(reg(
            "MULTILANG_CATEGORY_COUNTS_MISMATCH",
            "successor category/physical counts are not the exact expected set",
        ));
    }
    if manifest.new_paths.iter().cloned().collect::<BTreeSet<_>>() != new_paths
        || duplicate_set(&manifest.new_paths)
        || manifest
            .existing_import_paths
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != existing_paths
        || duplicate_set(&manifest.existing_import_paths)
        || manifest
            .not_imported_origin_ids
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != not_imported_ids
        || duplicate_set(&manifest.not_imported_origin_ids)
    {
        return Err(reg(
            "MULTILANG_EXACT_SET_MISMATCH",
            "successor new/existing/not-imported exact sets are invalid",
        ));
    }
    let expected_summary = Summary {
        origins: EXPECTED_ORIGINS as u64,
        existing_imports: EXPECTED_EXISTING as u64,
        new_imports: EXPECTED_NEW as u64,
        physical_files: EXPECTED_PHYSICAL as u64,
        not_imported: EXPECTED_NOT_IMPORTED as u64,
        primary: EXPECTED_PRIMARY as u64,
        projections: EXPECTED_PROJECTIONS as u64,
        historical: EXPECTED_HISTORICAL as u64,
        origin_language_counts,
        physical_language_counts,
        predecessor_expected_curated_files: EXPECTED_PREDECESSOR_CURATED as u64,
        new_expected_curated_files: EXPECTED_NEW as u64,
        expected_curated_files: EXPECTED_CURATED as u64,
    };
    if manifest.summary != expected_summary {
        return Err(reg(
            "MULTILANG_SUMMARY_MISMATCH",
            "successor summary does not match computed category/language counts",
        ));
    }
    if manifest.summary.expected_curated_files as usize
        != old_curated.len() + manifest.new_paths.len()
    {
        return Err(reg(
            "MULTILANG_EXPECTED_CURATED_MISMATCH",
            "expected curated file count is not predecessor 112 plus new 19",
        ));
    }
    if manifest.limits.is_empty() {
        return Err(reg(
            "MULTILANG_LIMITS_MISSING",
            "successor must retain explicit limitations",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actual_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        let cwd = std::env::current_dir().unwrap();
        let curated = if cwd.join("metadata/source-closure.v1.json").exists() {
            cwd.clone()
        } else if cwd
            .join("github_src/metadata/source-closure.v1.json")
            .exists()
        {
            cwd.join("github_src")
        } else {
            let mut cursor = cwd.as_path();
            loop {
                if cursor.join("metadata/source-closure.v1.json").exists() {
                    break cursor.to_path_buf();
                }
                cursor = cursor.parent().expect("curated root not found");
            }
        };
        let research = curated
            .parent()
            .filter(|parent| parent.join("src/reconstructed").exists())
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| curated.clone());
        let parent = curated.join("metadata/source-closure.v1.json");
        let successor = curated.join(MULTILANG_MANIFEST_REL);
        (curated, research, parent, successor)
    }

    fn actual_roots() -> Roots {
        let (curated, research, _, _) = actual_paths();
        Roots { research, curated }
    }

    fn actual_parent_and_successor() -> (SourceManifest, Vec<u8>, SuccessorManifest) {
        let (_, _, parent_path, successor_path) = actual_paths();
        let (parent, parent_bytes) = load_manifest(&parent_path).unwrap();
        let value = crate::parse_strict_value(&fs::read(&successor_path).unwrap()).unwrap();
        let successor: SuccessorManifest = serde_json::from_value(value).unwrap();
        (parent, parent_bytes, successor)
    }

    fn trust_for(parent_bytes: &[u8], successor_bytes: &[u8]) -> Trust {
        Trust {
            manifest_sha256: Some(crate::sha256_hex(parent_bytes)),
            multilang_manifest_sha256: Some(crate::sha256_hex(successor_bytes)),
            successor_required: true,
            snapshot_id: ACCEPTED_SNAPSHOT_ID,
            graph_id: ACCEPTED_GRAPH_ID,
            payloads: &ACCEPTED_GRAPH_PAYLOADS,
        }
    }

    #[test]
    fn actual_successor_validates_exact_counts_and_copies() {
        let (_, _, parent_path, successor_path) = actual_paths();
        let (parent, parent_bytes) = load_manifest(&parent_path).unwrap();
        let successor_bytes = fs::read(&successor_path).unwrap();
        let trust = trust_for(&parent_bytes, &successor_bytes);
        let successor =
            load_candidate(&actual_roots(), &parent, &parent_bytes, &trust, true).unwrap();
        assert_eq!(successor.manifest.entries.len(), 34);
        assert_eq!(successor.manifest.summary.new_imports, 19);
        assert_eq!(successor.manifest.summary.existing_imports, 7);
        assert_eq!(successor.manifest.summary.physical_files, 26);
        assert_eq!(successor.manifest.summary.not_imported, 8);
        assert_eq!(successor.manifest.summary.primary, 1);
        assert_eq!(successor.manifest.summary.projections, 22);
        assert_eq!(successor.manifest.summary.historical, 3);
        assert_eq!(successor.manifest.summary.expected_curated_files, 131);
        assert_eq!(successor.expected_paths().len(), 19);
        assert!(successor.lookup_value("SRC-HPP-EVIDENCE-LABELS").is_some());
        assert!(
            successor
                .artifact_lookup_value("RESEARCH_ROOT", "src/reconstructed/config/SkillTree.ts")
                .is_some()
        );
        let config = successor
            .manifest
            .entries
            .iter()
            .find(|entry| entry.origin_id == "SRC-TS-CONFIG-BOSSPILINGBUFF")
            .unwrap();
        assert_eq!(
            config.origin_class,
            NormalizedOriginClass::DeclarationOrInterface
        );
        assert_eq!(config.predecessor_origin_class, "RECONSTRUCTED");
        assert_eq!(config.identity_scheme, "SOURCE_MODULE_PATH_ONLY");
        assert!(!config.evidence_references.is_empty());
        assert_eq!(config.authority.kind, "SOURCE_SCHEMA_ONLY");
        let historical = successor
            .manifest
            .entries
            .iter()
            .find(|entry| entry.origin_id == "SRC-JS-CHARACTER-HIT-R27")
            .unwrap();
        assert_eq!(
            historical.disposition,
            SuccessorDisposition::PreservedHistoricalAlternative
        );
        assert_eq!(
            historical.origin_class,
            NormalizedOriginClass::HistoricalAlternative
        );
        let metadata = successor
            .manifest
            .entries
            .iter()
            .find(|entry| entry.origin_id == "SRC-HPP-FEVENTSYSTEM")
            .unwrap();
        assert_eq!(metadata.origin_class, NormalizedOriginClass::MetadataOnly);
        assert_eq!(metadata.disposition, SuccessorDisposition::ReferenceOnly);
        assert!(metadata.physical_import.is_none());
    }

    #[test]
    fn exact_route_table_has_finite_34_routes_and_derived_counts() {
        assert_eq!(EXPECTED_ROUTES.len(), 34);
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter_map(|route| route.path)
                .collect::<BTreeSet<_>>()
                .len(),
            26
        );
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter(|route| route.disposition == SuccessorDisposition::ImportedSourceProjection)
                .count(),
            22
        );
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter(|route| route.disposition == SuccessorDisposition::ImportedPrimarySource)
                .count(),
            1
        );
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter(|route| route.disposition
                    == SuccessorDisposition::PreservedHistoricalAlternative)
                .count(),
            3
        );
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter(|route| route.disposition == SuccessorDisposition::ReferenceOnly)
                .count(),
            4
        );
        assert_eq!(
            EXPECTED_ROUTES
                .iter()
                .filter(
                    |route| route.disposition == SuccessorDisposition::DeclarationOnlyNotImported
                )
                .count(),
            4
        );
    }

    #[test]
    fn successor_pin_missing_and_wrong_fail_closed() {
        let (_, _, parent_path, successor_path) = actual_paths();
        let (parent, parent_bytes) = load_manifest(&parent_path).unwrap();
        let successor_bytes = fs::read(&successor_path).unwrap();
        let roots = actual_roots();
        let no_pin = Trust {
            manifest_sha256: Some(crate::sha256_hex(&parent_bytes)),
            multilang_manifest_sha256: None,
            successor_required: true,
            snapshot_id: ACCEPTED_SNAPSHOT_ID,
            graph_id: ACCEPTED_GRAPH_ID,
            payloads: &ACCEPTED_GRAPH_PAYLOADS,
        };
        assert_eq!(
            load_candidate(&roots, &parent, &parent_bytes, &no_pin, true)
                .unwrap_err()
                .code,
            "TRUST_PIN_UNAVAILABLE"
        );
        let wrong = Trust {
            multilang_manifest_sha256: Some("0".repeat(64)),
            ..no_pin
        };
        assert!(load_candidate(&roots, &parent, &parent_bytes, &wrong, true).is_err());
        let _ = successor_bytes;
    }

    #[test]
    fn predecessor_binding_and_origin_mapping_drift_are_rejected() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();
        let mut changed = successor.clone();
        changed.predecessor.sha256 = "0".repeat(64);
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        changed.entries[0].original.sha256 = "0".repeat(64);
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        changed.predecessor.origin_ids.pop();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());
    }

    #[test]
    fn balanced_promotion_reference_physical_and_wrong_tier_are_route_failures() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();
        let legitimate_physical = successor
            .entries
            .iter()
            .find(|entry| entry.origin_id == "SRC-TS-CONFIG-BOSSPILINGBUFF")
            .unwrap()
            .physical_import
            .clone()
            .unwrap();

        let mut changed = successor.clone();
        let loading = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-TS-LOADING-PIPELINE")
            .unwrap();
        loading.disposition = SuccessorDisposition::ImportedSourceProjection;
        loading.category = "PROJECTION".to_string();
        loading.origin_class = NormalizedOriginClass::RecoveredSourceProjection;
        loading.representation_class = "SOURCE_LIKE_PROJECTION".to_string();
        loading.physical_import = Some(legitimate_physical.clone());
        let config = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-TS-CONFIG-BOSSPILINGBUFF")
            .unwrap();
        config.disposition = SuccessorDisposition::ReferenceOnly;
        config.category = "NOT_IMPORTED".to_string();
        config.physical_import = None;
        config.already_imported = false;
        let error =
            validate_successor(&changed, &parent, &parent_bytes, &roots, false).unwrap_err();
        assert!(
            error.code.starts_with("MULTILANG_ROUTE_"),
            "balanced promotion must fail the route contract, got {}",
            error.code
        );

        let mut changed = successor.clone();
        let reference = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-HPP-FEVENTSYSTEM")
            .unwrap();
        reference.physical_import = Some(PhysicalImport {
            path: "src/native/include/research/evidence_labels.hpp".to_string(),
            size: 437,
            sha256: "d15408e960b6cab2b8dac567354de2f70180b6dbb416f6f2e5a1d984478b370c".to_string(),
            mode: "EXACT_BYTE_COPY".to_string(),
        });
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_ROUTE_PHYSICAL_FORBIDDEN"
        );

        let mut changed = successor.clone();
        let config = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-TS-CONFIG-BOSSPILINGBUFF")
            .unwrap();
        config.representation_class = "PRIMARY_TOOLING_DECLARATION".to_string();
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_ROUTE_REPRESENTATION_MISMATCH"
        );

        let mut changed = successor;
        let duplicate = changed
            .entries
            .iter()
            .find(|entry| entry.origin_id == "SRC-TS-CONFIG-BOSSRUSHBUFF")
            .unwrap()
            .clone();
        changed
            .entries
            .retain(|entry| entry.origin_id != "SRC-TS-CONFIG-BOSSPILINGBUFF");
        changed.entries.push(duplicate);
        assert_eq!(changed.entries.len(), 34);
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_ORIGIN_SET_INVALID"
        );
    }

    #[test]
    fn typed_fields_and_full_pin_reseal_are_rejected() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();

        let mut changed = successor.clone();
        changed.entries[0].authority.kind = "FORGED".to_string();
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_AUTHORITY_MISMATCH"
        );

        let mut changed = successor.clone();
        changed.entries[0].evidence_references.clear();
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_EVIDENCE_REFERENCE_MISMATCH"
        );

        let mut changed = successor.clone();
        changed.entries[0].identity_scheme = "FORGED".to_string();
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_IDENTITY_SCHEME_MISMATCH"
        );

        let mut changed = successor.clone();
        changed.entries[0].predecessor_origin_class = "FORGED".to_string();
        assert_eq!(
            validate_successor(&changed, &parent, &parent_bytes, &roots, false)
                .unwrap_err()
                .code,
            "MULTILANG_PREDECESSOR_ORIGIN_CLASS_MISMATCH"
        );

        let mut value = serde_json::to_value(&successor).unwrap();
        value["entries"][0]["identity_scheme"] = serde_json::json!("FORGED");
        let bytes = serde_json::to_vec(&value).unwrap();
        let trust = trust_for(&parent_bytes, &bytes);
        let error = load_candidate_bytes(&bytes, &roots, &parent, &parent_bytes, &trust, false)
            .unwrap_err();
        assert_eq!(error.code, "MULTILANG_IDENTITY_SCHEME_MISMATCH");

        let mut value = serde_json::to_value(&successor).unwrap();
        value["entries"][0]["authority"]["kind"] = serde_json::json!("FORGED");
        let bytes = serde_json::to_vec(&value).unwrap();
        let trust = trust_for(&parent_bytes, &bytes);
        let error = load_candidate_bytes(&bytes, &roots, &parent, &parent_bytes, &trust, false)
            .unwrap_err();
        assert_eq!(error.code, "MULTILANG_AUTHORITY_MISMATCH");

        let mut value = serde_json::to_value(&successor).unwrap();
        value["entries"][0]["evidence_references"] = serde_json::json!([]);
        let bytes = serde_json::to_vec(&value).unwrap();
        let trust = trust_for(&parent_bytes, &bytes);
        let error = load_candidate_bytes(&bytes, &roots, &parent, &parent_bytes, &trust, false)
            .unwrap_err();
        assert_eq!(error.code, "MULTILANG_EVIDENCE_REFERENCE_MISMATCH");

        let mut value = serde_json::to_value(&successor).unwrap();
        value["entries"][0]["physical_import"]["sha256"] = serde_json::json!("0".repeat(64));
        let bytes = serde_json::to_vec(&value).unwrap();
        let trust = trust_for(&parent_bytes, &bytes);
        let error = load_candidate_bytes(&bytes, &roots, &parent, &parent_bytes, &trust, false)
            .unwrap_err();
        assert!(
            error.code.starts_with("MULTILANG_"),
            "resealed wrong physical hash must fail the route/body contract, got {}",
            error.code
        );
        assert_ne!(error.code, "TRUST_MULTILANG_HASH_MISMATCH");
    }

    #[test]
    fn wrong_language_and_exact_set_drift_are_rejected() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();
        let mut changed = successor.clone();
        changed.entries[0].language = "JS".to_string();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        changed.entries.pop();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        let duplicate = changed.entries[0].clone();
        changed.entries.push(duplicate);
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor;
        changed.new_paths.pop();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());
    }

    #[test]
    fn destination_escape_and_placeholder_promotion_are_rejected() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();
        let mut changed = successor.clone();
        changed.entries[0].physical_import.as_mut().unwrap().path = "../escape.ts".to_string();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        let placeholder = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-HPP-RESOURCE-INTERFACES")
            .unwrap();
        placeholder.physical_import = Some(PhysicalImport {
            path: "src/projections/native/include/resource_interfaces.hpp".to_string(),
            size: 540,
            sha256: "0".repeat(64),
            mode: "EXACT_BYTE_COPY".to_string(),
        });
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor;
        let historical = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-JS-CHARACTER-HIT-R27")
            .unwrap();
        historical.historical_active = true;
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());
    }

    #[test]
    fn new_header_drift_and_existing_path_mutation_are_rejected() {
        let (parent, parent_bytes, successor) = actual_parent_and_successor();
        let roots = actual_roots();
        let mut changed = successor.clone();
        let header = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-HPP-EVIDENCE-LABELS")
            .unwrap();
        header.physical_import.as_mut().unwrap().sha256 = "0".repeat(64);
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        let missing = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-HPP-EVIDENCE-LABELS")
            .unwrap();
        missing.physical_import.as_mut().unwrap().path =
            "src/native/include/research/missing_evidence_labels.hpp".to_string();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor.clone();
        let existing = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-TS-DATALAYER-TRANSITION")
            .unwrap();
        existing.physical_import.as_mut().unwrap().path =
            "src/scripts/typescript/declarations/config/DataLayerTransition.ts".to_string();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());

        let mut changed = successor;
        let duplicate = changed
            .entries
            .iter_mut()
            .find(|entry| entry.origin_id == "SRC-JS-EVENT-CORE")
            .unwrap();
        duplicate.physical_import.as_mut().unwrap().path =
            "src/scripts/reconstructed/event/EventCSharpBridge.js".to_string();
        assert!(validate_successor(&changed, &parent, &parent_bytes, &roots, false).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn successor_destination_reparse_is_rejected() {
        let base = std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!("multilang-junction-{}", std::process::id()));
        let target = base.join("actual");
        let link = base.join("link");
        fs::create_dir_all(&target).unwrap();
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                link.to_str().unwrap(),
                target.to_str().unwrap(),
            ])
            .status();
        let mklink_ok = status.map(|status| status.success()).unwrap_or(false);
        if !mklink_ok {
            let _ = fs::remove_dir_all(&base);
        }
        assert!(mklink_ok, "mklink /J failed; junction fixture unavailable");
        let roots = Roots {
            research: base.clone(),
            curated: base.clone(),
        };
        let artifact = ArtifactRef {
            root_id: "CURATED_ROOT".to_string(),
            path: "link/file.ts".to_string(),
            size: 1,
            sha256: "a".repeat(64),
        };
        assert!(resolve_ref(&roots, &artifact).is_err());
        let _ = fs::remove_dir(&link);
        let _ = fs::remove_dir_all(&base);
    }
}
