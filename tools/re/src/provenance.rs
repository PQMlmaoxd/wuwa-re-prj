//! Strict, offline provenance errata model for reconstructed source anchors.
//!
//! Phase A deliberately does not activate errata in the loader. This module only
//! parses and validates proposed corrections against the accepted method map,
//! the frozen catalog, and the actual source bytes.

use crate::bulk::{CATALOG_REL, GAMEPLAY_COHORT, INDEX_REL};
use crate::{
    ActiveReceipt, Manifest, MethodIdentity, SafePathState, inspect_relative, parse_strict_value,
    sha256_hex,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub const ERRATA_SCHEMA: &str = "wuwa-reconstruction-provenance-errata-v1";
pub const ERRATA_VERSION: u32 = 1;
pub const ERRATA_CAMPAIGN: &str = "bulk-curate01";
pub const METHOD_EVIDENCE_MAP: &str =
    "src/reconstructed/managed/semantic/gameplay/METHOD_EVIDENCE_MAP.md";
/// The only versioned errata path the effective-view loader reads. The file is
/// intentionally absent until the primary approves a reviewed document.
pub const ERRATA_REL: &str = "metadata/curation/bulk-curate01.provenance-errata.v1.json";
pub const ERRATA_DIR_REL: &str = "metadata/curation";
pub const ERRATA_FILE_NAME: &str = "bulk-curate01.provenance-errata.v1.json";
pub const ERRATA_FILE_PREFIX: &str = "bulk-curate01.provenance-errata.v";
pub const ERRATA_FILE_SUFFIX: &str = ".json";
pub const PROVENANCE_PROFILE: &str = "ordinary-gameplay-292";
pub const COMPLETION_PROFILE: &str = "accepted-completion-533";
pub const REGISTRY_REL: &str =
    "tools/re/local-recovery/full-corpus-preserve01/phase-c/graph/artifacts.links.jsonl";
pub const REGISTRY_BYTES: u64 = 54_417_149;
pub const REGISTRY_SHA256: &str =
    "2f56bb62855a2bf4e6f5f8bad0c77cefdf315994b716ec6fd0c1dd1fd6fb33b4";
pub const V5_RECEIPT_REL: &str = "metadata/imports.v5.json";
pub const V5_RECEIPT_SHA256: &str =
    "7c990cc73ea4050f7b20b00b7c8de2aef3f51bd179c516f12e0102b4bc5d11fd";
pub const MIGRATION_REL: &str = "metadata/migration.json";
pub const PERFORMANCE_SOURCE_REL: &str = "metadata/performance-source.v1.json";
pub const SCHEDULING_SOURCE_REL: &str = "metadata/scheduling-source.v1.json";
pub const TASK_SOURCE_REL: &str = "metadata/task-execution-source.v1.json";
pub const COMPLETION_REPORT_SHA256: [(&str, u64, &str); 6] = [
    (
        "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md",
        31875,
        "13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27",
    ),
    (
        "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md",
        32768,
        "df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65",
    ),
    (
        "docs/performance/PERF_CIL12_FAILURE_LIFECYCLE.md",
        69303,
        "603b35228299d913042d8eb7359d538bfc80c0a38bf38cf13f5adaff8af93d15",
    ),
    (
        "docs/performance/PERF_CIL15_CONCRETE_INVOKE.md",
        28478,
        "e74337d26207cd558321d54d5182fb526951ea1134ca6a49fdd11f4ea5df75ca",
    ),
    (
        "docs/performance/PERF_CIL16_WRAPPER_RECOVERY.md",
        49257,
        "fb7a311cc5cf35c88df273ef197baaea3480513400fc192b56e0d98b4cda0058",
    ),
    (
        "docs/performance/PERF11_PENDING_EVENT_DRAIN.md",
        31986,
        "2575195f24ec70044a75ab59b95fdc3d3c83a29b23f4c373952fe987681a5872",
    ),
];
/// Phase C keeps the whole-file public digest unpinned until the primary has
/// reviewed the exact publication bytes. Loader acceptance still requires
/// PRIMARY_APPROVED entries, the frozen catalog/index digests, and structural
/// target binding; semantic acceptance additionally requires the research root
/// through `validate_for_activation`. Phase D may set this constant to the
/// reviewed digest without changing any loader behavior.
pub const PINNED_PUBLIC_ERRATA_SHA256: Option<&str> =
    Some("b3a4145017a23b3b7642bc3cd7a0a64848c656a21b6709034383ecca9a1c17a5");

#[derive(Debug, Clone, Copy)]
pub struct AssociatedBinding {
    pub record_id: &'static str,
    pub type_def: &'static str,
    pub method_def: &'static str,
    pub map_line: u64,
    pub source_path: &'static str,
    pub exact: Option<(u64, u64)>,
    pub required_tokens: &'static [&'static str],
    pub parent_method_def: Option<&'static str>,
    pub parent_map_line: Option<u64>,
    pub context_lines: &'static [u64],
    pub basis: &'static str,
}

pub const ASSOCIATED_BINDINGS: [AssociatedBinding; 18] = [
    AssociatedBinding {
        record_id: "G_BASEDAMAGECOMPONENT",
        type_def: "0x02002E65",
        method_def: "0x06018666",
        map_line: 4545,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseDamageComponent.cs",
        exact: Some((373, 387)),
        required_tokens: &["0x06018666", "BaseDamageComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R26 accepted callback row binds the literal MethodDef to BaseDamageComponent.cs:373-387",
    },
    AssociatedBinding {
        record_id: "G_BASEDAMAGECOMPONENT",
        type_def: "0x02002E65",
        method_def: "0x06018667",
        map_line: 4546,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseDamageComponent.cs",
        exact: Some((410, 421)),
        required_tokens: &["0x06018667", "BaseDamageComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R26 accepted callback row binds the literal MethodDef to BaseDamageComponent.cs:410-421",
    },
    AssociatedBinding {
        record_id: "G_BASEDAMAGECOMPONENT",
        type_def: "0x02002E65",
        method_def: "0x06018668",
        map_line: 4547,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseDamageComponent.cs",
        exact: Some((427, 433)),
        required_tokens: &["0x06018668", "BaseDamageComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R26 accepted callback row binds the literal MethodDef to BaseDamageComponent.cs:427-433",
    },
    AssociatedBinding {
        record_id: "G_BASEDAMAGECOMPONENT",
        type_def: "0x020092FB",
        method_def: "0x06049EB8",
        map_line: 1812,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseDamageComponent.cs",
        exact: None,
        required_tokens: &["0x06049eb8", "0x06018652"],
        parent_method_def: Some("0x06018652"),
        parent_map_line: None,
        context_lines: &[],
        basis: "R18 generated response callback row binds the literal MethodDef to parent GetServerDamage 0x06018652 in the same record",
    },
    AssociatedBinding {
        record_id: "G_ROLETEAMCOMPONENT",
        type_def: "0x02003206",
        method_def: "0x0601A91E",
        map_line: 1188,
        source_path: "src/reconstructed/managed/semantic/gameplay/RoleTeamComponent.cs",
        exact: None,
        required_tokens: &["0x0601a91e", "RoleTeamComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R15 callback table row binds the literal MethodDef to RoleTeamComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_ROLETEAMCOMPONENT",
        type_def: "0x02003206",
        method_def: "0x0601A91F",
        map_line: 1189,
        source_path: "src/reconstructed/managed/semantic/gameplay/RoleTeamComponent.cs",
        exact: None,
        required_tokens: &["0x0601a91f", "RoleTeamComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R15 callback table row binds the literal MethodDef to RoleTeamComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_ROLETEAMCOMPONENT",
        type_def: "0x02003206",
        method_def: "0x0601A920",
        map_line: 1190,
        source_path: "src/reconstructed/managed/semantic/gameplay/RoleTeamComponent.cs",
        exact: None,
        required_tokens: &["0x0601a920", "RoleTeamComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R15 callback table row binds the literal MethodDef to RoleTeamComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_SCENETEAMCONTROLLER",
        type_def: "0x02008C6E",
        method_def: "0x06049786",
        map_line: 1191,
        source_path: "src/reconstructed/managed/semantic/gameplay/SceneTeamController.cs",
        exact: None,
        required_tokens: &["0x06049786", "SceneTeamController.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R15 callback table row binds the literal MethodDef to SceneTeamController.cs",
    },
    AssociatedBinding {
        record_id: "G_CALCULATION_FORMULAONEINVOCATION",
        type_def: "0x02009315",
        method_def: "0x06049F93",
        map_line: 8129,
        source_path: "src/reconstructed/managed/semantic/gameplay/Calculation.FormulaOneInvocation.cs",
        exact: Some((83, 105)),
        required_tokens: &["0x06049f93", "Calculation.FormulaOneInvocation.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R39 generated association row binds the literal MethodDef to Calculation.FormulaOneInvocation.cs:83-105",
    },
    AssociatedBinding {
        record_id: "G_CHARACTERPASSIVESKILLCOMPONENT",
        type_def: "0x0200931F",
        method_def: "0x06049FB0",
        map_line: 100,
        source_path: "src/reconstructed/managed/semantic/gameplay/CharacterPassiveSkillComponent.cs",
        exact: None,
        required_tokens: &["0x06049fb0", "CharacterPassiveSkillComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R14 LearnPassiveSkill row binds the literal MethodDef to CharacterPassiveSkillComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_CHARACTERPASSIVESKILLCOMPONENT",
        type_def: "0x0200931F",
        method_def: "0x06049FB1",
        map_line: 100,
        source_path: "src/reconstructed/managed/semantic/gameplay/CharacterPassiveSkillComponent.cs",
        exact: None,
        required_tokens: &["0x06049fb1", "CharacterPassiveSkillComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R14 LearnPassiveSkill row binds the literal checker MethodDef to CharacterPassiveSkillComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_BASESKILLCOMPONENT",
        type_def: "0x020093B5",
        method_def: "0x0604A128",
        map_line: 717,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseSkillComponent.cs",
        exact: Some((297, 321)),
        required_tokens: &["0x0604a128", "BaseSkillComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R21 wrapper row binds MoveNext 4A128 to BaseSkillComponent.cs:297-321",
    },
    AssociatedBinding {
        record_id: "G_BASESKILLCOMPONENT",
        type_def: "0x020093B7",
        method_def: "0x0604A133",
        map_line: 716,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseSkillComponent.cs",
        exact: Some((247, 295)),
        required_tokens: &["0x0604a133", "BaseSkillComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R21 wrapper row binds MoveNext 4A133 to BaseSkillComponent.cs:247-295",
    },
    AssociatedBinding {
        record_id: "G_BASESKILLCOMPONENT",
        type_def: "0x020093B3",
        method_def: "0x0604A125",
        map_line: 962,
        source_path: "src/reconstructed/managed/semantic/gameplay/BaseSkillComponent.cs",
        exact: None,
        required_tokens: &["0x0604a125", "BaseSkillComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R14 generated association row binds the literal MethodDef to BaseSkillComponent.cs",
    },
    AssociatedBinding {
        record_id: "G_TSANIMNOTIFYSTATECOUNTERATTACK",
        type_def: "0x0200720E",
        method_def: "0x060467D4",
        map_line: 2636,
        source_path: "src/reconstructed/managed/semantic/gameplay/TsAnimNotifyStateCounterAttack.cs",
        exact: None,
        required_tokens: &["0x060467d4", "parent offset `585`"],
        parent_method_def: Some("0x06004697"),
        parent_map_line: Some(2621),
        context_lines: &[2635],
        basis: "R20 callback paragraph binds the generated MethodDef to parent End 0x06004697 and TsAnimNotifyStateCounterAttack.cs",
    },
    AssociatedBinding {
        record_id: "G_TSTASKUSESKILL",
        type_def: "0x020071D8",
        method_def: "0x060467B8",
        map_line: 5644,
        source_path: "src/reconstructed/managed/semantic/gameplay/TsTaskUseSkill.cs",
        exact: None,
        required_tokens: &["0x060467b8", "parent row 265"],
        parent_method_def: Some("0x06004157"),
        parent_map_line: Some(5628),
        context_lines: &[],
        basis: "R32 callback table binds the generated MethodDef to parent ReceiveTickAI 0x06004157 and TsTaskUseSkill.cs",
    },
    AssociatedBinding {
        record_id: "G_SKILL",
        type_def: "0x020093C6",
        method_def: "0x0604A189",
        map_line: 963,
        source_path: "src/reconstructed/managed/semantic/gameplay/Skill.cs",
        exact: None,
        required_tokens: &["0x0604a189", "Skill.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R14 generated association row binds the literal MethodDef to Skill.cs",
    },
    AssociatedBinding {
        record_id: "G_CHARACTERHITCOMPONENT",
        type_def: "0x02003044",
        method_def: "0x06019509",
        map_line: 1462,
        source_path: "src/reconstructed/managed/semantic/gameplay/CharacterHitComponent.cs",
        exact: None,
        required_tokens: &["0x06019509", "CharacterHitComponent.cs"],
        parent_method_def: None,
        parent_map_line: None,
        context_lines: &[],
        basis: "R17 callback table row binds the literal MethodDef to CharacterHitComponent.cs",
    },
];

#[derive(Debug, Clone, Copy)]
pub struct PerformanceBinding {
    pub record_id: &'static str,
    pub type_def: &'static str,
    pub method_def: &'static str,
    pub report_path: &'static str,
    pub report_bytes: u64,
    pub report_sha256: &'static str,
    pub target_lines: &'static [u64],
    pub source_lines: &'static [u64],
    pub source_token: &'static str,
    pub basis: &'static str,
}

pub const PERFORMANCE_BINDINGS: [PerformanceBinding; 13] = [
    PerformanceBinding {
        record_id: "P_ASYNCTASK",
        type_def: "0x02009689",
        method_def: "0x0604A4E5",
        report_path: "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md",
        report_bytes: 31875,
        report_sha256: "13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27",
        target_lines: &[168],
        source_lines: &[168],
        source_token: "AsyncTask.cs",
        basis: "PERF02 AsyncTask.cs row names OnRun 0x0601c8da and generated MoveNext 0x0604a4e5",
    },
    PerformanceBinding {
        record_id: "P_SUBLEVELCONTROLLER_EXECUTION",
        type_def: "0x0200A853",
        method_def: "0x0604AE46",
        report_path: "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md",
        report_bytes: 31875,
        report_sha256: "13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27",
        target_lines: &[171],
        source_lines: &[171],
        source_token: "SubLevelController.cs",
        basis: "PERF02 SubLevelController.cs row names LoadOrUnloadSubLevel 0x0602f35a and generated closures 4Ae46/4Ae48/50750",
    },
    PerformanceBinding {
        record_id: "P_SUBLEVELCONTROLLER_EXECUTION",
        type_def: "0x0200A854",
        method_def: "0x0604AE48",
        report_path: "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md",
        report_bytes: 31875,
        report_sha256: "13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27",
        target_lines: &[171],
        source_lines: &[171],
        source_token: "SubLevelController.cs",
        basis: "PERF02 SubLevelController.cs row names LoadOrUnloadSubLevel 0x0602f35a and generated closures 4Ae46/4Ae48/50750",
    },
    PerformanceBinding {
        record_id: "P_SUBLEVELCONTROLLER_EXECUTION",
        type_def: "0x0200CE43",
        method_def: "0x06050750",
        report_path: "docs/performance/PERF02_TASKSYSTEM_EXECUTION.md",
        report_bytes: 31875,
        report_sha256: "13a30daa68160d1dd59f7072daacb4dd7bb096228a3a988c47819f0db2d11b27",
        target_lines: &[171],
        source_lines: &[171],
        source_token: "SubLevelController.cs",
        basis: "PERF02 SubLevelController.cs row names LoadOrUnloadSubLevel 0x0602f35a and generated closures 4Ae46/4Ae48/50750",
    },
    PerformanceBinding {
        record_id: "P_CORE",
        type_def: "0x02007187",
        method_def: "0x06046713",
        report_path: "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md",
        report_bytes: 32768,
        report_sha256: "df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65",
        target_lines: &[91],
        source_lines: &[106],
        source_token: "WaitFrame",
        basis: "PERF05 Core.cs row binds WaitFrame and its generated callback 0x06046713",
    },
    PerformanceBinding {
        record_id: "P_CORE",
        type_def: "0x02007188",
        method_def: "0x06046714",
        report_path: "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md",
        report_bytes: 32768,
        report_sha256: "df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65",
        target_lines: &[82],
        source_lines: &[106],
        source_token: "Initialize",
        basis: "PERF05 Core.cs row binds the async Initialize projection and generated MoveNext 0x06046714",
    },
    PerformanceBinding {
        record_id: "P_TICKSYSTEM",
        type_def: "0x020071BE",
        method_def: "0x06046782",
        report_path: "docs/performance/PERF05_UPDATE_TICK_ARCHITECTURE.md",
        report_bytes: 32768,
        report_sha256: "df50418d6cc9c21a05aae29b3e6cab7a550148bd13cf23093d728d6cf6770f65",
        target_lines: &[79],
        source_lines: &[105],
        source_token: "generated Add callback",
        basis: "PERF05 TickSystem.cs row binds Add and its generated callback 0x06046782",
    },
    PerformanceBinding {
        record_id: "P_EVENTARGS_INVOKE_ENUM2",
        type_def: "0x020000A9",
        method_def: "0x06000458",
        report_path: "docs/performance/PERF_CIL15_CONCRETE_INVOKE.md",
        report_bytes: 28478,
        report_sha256: "e74337d26207cd558321d54d5182fb526951ea1134ca6a49fdd11f4ea5df75ca",
        target_lines: &[19],
        source_lines: &[9],
        source_token: "EventArgs.Invoke.Enum2.cs",
        basis: "PERF_CIL15 selected method row and source path bind MethodDef 0x06000458",
    },
    PerformanceBinding {
        record_id: "P_EVENTSYSTEM_EMITWITHTARGET_ENUM2",
        type_def: "0x0200706E",
        method_def: "0x06045B73",
        report_path: "docs/performance/PERF_CIL16_WRAPPER_RECOVERY.md",
        report_bytes: 49257,
        report_sha256: "fb7a311cc5cf35c88df273ef197baaea3480513400fc192b56e0d98b4cda0058",
        target_lines: &[18],
        source_lines: &[460],
        source_token: "EventSystem.EmitWithTarget.Enum2.cs",
        basis: "PERF_CIL16 method row and SOURCE_PATH bind MethodDef 0x06045B73",
    },
    PerformanceBinding {
        record_id: "P_RANGECOMPONENT_LIFECYCLERECOVERY",
        type_def: "0x020048E7",
        method_def: "0x06030B43",
        report_path: "docs/performance/PERF_CIL12_FAILURE_LIFECYCLE.md",
        report_bytes: 69303,
        report_sha256: "603b35228299d913042d8eb7359d538bfc80c0a38bf38cf13f5adaff8af93d15",
        target_lines: &[75],
        source_lines: &[416],
        source_token: "RangeComponent.LifecycleRecovery.cs",
        basis: "PERF_CIL12 OnEnd row and SOURCE bind MethodDef 0x06030B43",
    },
    PerformanceBinding {
        record_id: "P_RANGECOMPONENT_LIFECYCLERECOVERY",
        type_def: "0x020048E7",
        method_def: "0x06030BA4",
        report_path: "docs/performance/PERF_CIL12_FAILURE_LIFECYCLE.md",
        report_bytes: 69303,
        report_sha256: "603b35228299d913042d8eb7359d538bfc80c0a38bf38cf13f5adaff8af93d15",
        target_lines: &[149],
        source_lines: &[416],
        source_token: "RangeComponent.LifecycleRecovery.cs",
        basis: "PERF_CIL12 ClearComponent row and SOURCE bind MethodDef 0x06030BA4",
    },
    PerformanceBinding {
        record_id: "P_PENDINGEVENTBOUNDARIES",
        type_def: "0x02000BED",
        method_def: "0x0600326C",
        report_path: "docs/performance/PERF11_PENDING_EVENT_DRAIN.md",
        report_bytes: 31986,
        report_sha256: "2575195f24ec70044a75ab59b95fdc3d3c83a29b23f4c373952fe987681a5872",
        target_lines: &[72],
        source_lines: &[38],
        source_token: "PendingEventBoundaries.cs",
        basis: "PERF11 Has row and source path bind MethodDef 0x0600326C",
    },
    PerformanceBinding {
        record_id: "P_PENDINGEVENTBOUNDARIES",
        type_def: "0x02000BED",
        method_def: "0x06003273",
        report_path: "docs/performance/PERF11_PENDING_EVENT_DRAIN.md",
        report_bytes: 31986,
        report_sha256: "2575195f24ec70044a75ab59b95fdc3d3c83a29b23f4c373952fe987681a5872",
        target_lines: &[73],
        source_lines: &[38],
        source_token: "PendingEventBoundaries.cs",
        basis: "PERF11 Remove row and source path bind MethodDef 0x06003273",
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceTrust {
    pub public_errata_sha256: Option<String>,
}

impl ProvenanceTrust {
    pub fn production() -> Self {
        Self {
            public_errata_sha256: PINNED_PUBLIC_ERRATA_SHA256.map(str::to_owned),
        }
    }

    pub fn fixture(sha256: impl Into<String>) -> Self {
        Self {
            public_errata_sha256: Some(sha256.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceError {
    pub code: String,
    pub path: Option<String>,
    pub record_id: Option<String>,
}

impl ProvenanceError {
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
            path: Some(path.to_owned()),
            record_id: None,
        }
    }

    fn record(code: &str, record_id: &str) -> Self {
        let mut error = Self::new(code);
        error.record_id = Some(record_id.to_owned());
        error
    }
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.code)?;
        if let Some(path) = &self.path {
            write!(formatter, " ({path})")?;
        }
        if let Some(record_id) = &self.record_id {
            write!(formatter, " [{record_id}]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RootKind {
    CuratedRoot,
    ResearchRoot,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Precision {
    ExactRange,
    FileMethodAssociation,
    Unresolved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceClass {
    AcceptedMethodMap,
    SemanticSource,
    CanonicalSource,
    RawMethod,
    Report,
    #[serde(rename = "ASSOCIATED_SOURCE")]
    AssociatedSource,
    #[serde(rename = "PERFORMANCE_SOURCE")]
    PerformanceSource,
    #[serde(rename = "LEGACY_DESCRIPTOR")]
    LegacyDescriptor,
    Unresolved,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CorrectionClass {
    OutOfBounds,
    WrongMethodBody,
    WrongSourceArtifact,
    StaleHistoricalRange,
    InsufficientEvidence,
    AmbiguousRepresentation,
    #[serde(rename = "NOT_A_CORRECTION")]
    NotACorrection,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrataAction {
    Correct,
    Refine,
    Confirm,
}

impl Default for ErrataAction {
    fn default() -> Self {
        Self::Correct
    }
}

impl ErrataAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Correct => "CORRECT",
            Self::Refine => "REFINE",
            Self::Confirm => "CONFIRM",
        }
    }
}

impl RootKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CuratedRoot => "CURATED_ROOT",
            Self::ResearchRoot => "RESEARCH_ROOT",
        }
    }
}

impl Precision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExactRange => "EXACT_RANGE",
            Self::FileMethodAssociation => "FILE_METHOD_ASSOCIATION",
            Self::Unresolved => "UNRESOLVED",
        }
    }
}

impl EvidenceClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AcceptedMethodMap => "ACCEPTED_METHOD_MAP",
            Self::SemanticSource => "SEMANTIC_SOURCE",
            Self::CanonicalSource => "CANONICAL_SOURCE",
            Self::RawMethod => "RAW_METHOD",
            Self::Report => "REPORT",
            Self::AssociatedSource => "ASSOCIATED_SOURCE",
            Self::PerformanceSource => "PERFORMANCE_SOURCE",
            Self::LegacyDescriptor => "LEGACY_DESCRIPTOR",
            Self::Unresolved => "UNRESOLVED",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Outcome {
    ResolvedExact,
    ResolvedConservative,
    UnresolvedInsufficientEvidence,
    ConflictingEvidence,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewStatus {
    PrimaryReviewPending,
    PrimaryApproved,
    PrimaryRejected,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FrozenFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SupportRef {
    pub path: String,
    pub locator: String,
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct IncorrectAnchor {
    pub path: String,
    pub locator: String,
    pub artifact_sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ImportedAnchor {
    pub root: RootKind,
    pub path: String,
    pub whole_file_sha256: String,
    pub method_identity: MethodIdentity,
    pub precision: Precision,
    pub start_line: Option<u64>,
    pub end_line: Option<u64>,
    pub scope: String,
    pub support_refs: Vec<SupportRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceAnchor {
    pub root: RootKind,
    pub path: String,
    pub artifact_sha256: String,
    pub method_identity: MethodIdentity,
    pub evidence_class: EvidenceClass,
    pub precision: Precision,
    pub start_line: Option<u64>,
    pub end_line: Option<u64>,
    pub accepted_map_row: Option<u64>,
    pub accepted_map_line: Option<u64>,
    pub basis: String,
    pub support_refs: Vec<SupportRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBasis {
    pub accepted_map_path: String,
    pub accepted_map_sha256: String,
    pub accepted_map_row: u64,
    pub accepted_map_line: u64,
    pub basis: String,
    pub proof_refs: Vec<SupportRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProvenanceProfile {
    #[default]
    OrdinaryGameplayMap,
    RegistryFileAssociation,
    AssociatedGameplayMap,
    PerformanceEvidenceAssociation,
    LegacyV5SourceAssociation,
    MigrationRecordAssociation,
    Unresolved,
}

impl ProvenanceProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OrdinaryGameplayMap => "ORDINARY_GAMEPLAY_MAP",
            Self::RegistryFileAssociation => "REGISTRY_FILE_ASSOCIATION",
            Self::AssociatedGameplayMap => "ASSOCIATED_GAMEPLAY_MAP",
            Self::PerformanceEvidenceAssociation => "PERFORMANCE_EVIDENCE_ASSOCIATION",
            Self::LegacyV5SourceAssociation => "LEGACY_V5_SOURCE_ASSOCIATION",
            Self::MigrationRecordAssociation => "MIGRATION_RECORD_ASSOCIATION",
            Self::Unresolved => "UNRESOLVED",
        }
    }

    pub fn is_ordinary(&self) -> bool {
        *self == Self::OrdinaryGameplayMap
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct AssociationProof {
    #[serde(default)]
    pub map_line: Option<u64>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub literal_method_def: Option<String>,
    #[serde(default)]
    pub report_refs: Vec<SupportRef>,
    #[serde(default)]
    pub raw_refs: Vec<SupportRef>,
    #[serde(default)]
    pub basis: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErrataEntry {
    pub target_catalog_record_id: String,
    pub expected_original_record_hash: String,
    pub method_identity: MethodIdentity,
    #[serde(default)]
    pub action: ErrataAction,
    pub incorrect_anchors: Vec<IncorrectAnchor>,
    pub corrected_imported_source_anchor: ImportedAnchor,
    pub corrected_evidence_source_anchors: Vec<EvidenceAnchor>,
    pub evidence_basis: EvidenceBasis,
    pub correction_class: CorrectionClass,
    pub review_status: ReviewStatus,
    pub outcome: Outcome,
    #[serde(default)]
    pub profile: ProvenanceProfile,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_catalog_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub association_proof: Option<AssociationProof>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ErrataDocument {
    pub schema: String,
    pub version: u32,
    pub campaign: String,
    pub catalog: FrozenFile,
    pub index: FrozenFile,
    pub accepted_map: FrozenFile,
    pub entries: Vec<ErrataEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bound_catalogs: Vec<FrozenFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub v5_receipt: Option<FrozenFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceState {
    NotApplicableBulkCorrection,
    Absent,
    PendingReview,
    StructurallyApplied,
}

impl ProvenanceState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotApplicableBulkCorrection => "NOT_APPLICABLE_BULK_CORRECTION",
            Self::Absent => "ABSENT",
            Self::PendingReview => "PENDING_PRIMARY_REVIEW",
            Self::StructurallyApplied => "STRUCTURALLY_APPLIED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodProvenanceStatus {
    Corrected,
    Confirmed,
    Refined,
}

impl MethodProvenanceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Corrected => "CORRECTED",
            Self::Confirmed => "CONFIRMED",
            Self::Refined => "REFINED",
        }
    }
}

/// Effective per-method provenance in the loaded view. Curated imported paths
/// never enter the flat RESEARCH_ROOT evidence-reference bag; they are exposed
/// here as their own typed object. The unmarked numeric reference that made the
/// old anchor false is retained only as an explicit legacy record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceProvenance {
    ProfileEntry {
        status: MethodProvenanceStatus,
        action: ErrataAction,
        imported: ImportedAnchor,
        evidence_anchors: Vec<EvidenceAnchor>,
        legacy_original_references: Vec<IncorrectAnchor>,
    },
    NotValidated,
}

impl SourceProvenance {
    pub fn status_label(&self) -> &'static str {
        match self {
            Self::ProfileEntry { status, .. } => status.as_str(),
            Self::NotValidated => "NOT_VALIDATED_BY_CURRENT_PROVENANCE_PROFILE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProvenanceView {
    pub state: ProvenanceState,
    pub errata_sha256: Option<String>,
    pub entries: u64,
    pub resolved_entries: u64,
    pub corrected: u64,
    pub confirmed: u64,
    pub refined: u64,
    pub activation_block: Option<String>,
    pub document: Option<ErrataDocument>,
}

impl ProvenanceView {
    pub fn not_applicable() -> Self {
        Self {
            state: ProvenanceState::NotApplicableBulkCorrection,
            errata_sha256: None,
            entries: 0,
            resolved_entries: 0,
            corrected: 0,
            confirmed: 0,
            refined: 0,
            activation_block: None,
            document: None,
        }
    }

    pub fn absent() -> Self {
        Self {
            state: ProvenanceState::Absent,
            ..Self::not_applicable()
        }
    }

    pub fn covered(&self) -> u64 {
        if self.state == ProvenanceState::StructurallyApplied {
            self.resolved_entries
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedMapRow {
    pub row: u64,
    pub map_line: u64,
    pub type_def: String,
    pub method_def: String,
    pub semantic_path: String,
    pub start_line: Option<u64>,
    pub end_line: Option<u64>,
    pub single_line: Option<u64>,
    pub canonical_path: Option<String>,
    pub canonical_start_line: Option<u64>,
    pub canonical_end_line: Option<u64>,
    pub raw_locator: Option<String>,
    pub body_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptedMethodMap {
    pub rows: Vec<AcceptedMapRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyHashReport {
    pub rows: u64,
    pub checked: u64,
    pub mismatched: u64,
}

impl AcceptedMethodMap {
    pub fn row_for(&self, type_def: &str, method_def: &str) -> Option<&AcceptedMapRow> {
        self.rows.iter().find(|row| {
            row.type_def.eq_ignore_ascii_case(type_def)
                && row.method_def.eq_ignore_ascii_case(method_def)
        })
    }

    /// Rehash dedicated map body-hash columns against the accepted semantic
    /// source bytes. The hash is only read from the header-bound body-hash
    /// column; a 64-hex shard identifier in a canonical path is never treated
    /// as a body digest.
    pub fn validate_body_hashes(
        &self,
        research_root: &Path,
    ) -> Result<BodyHashReport, ProvenanceError> {
        let mut checked = 0;
        let rows = self
            .rows
            .iter()
            .filter(|row| row.body_sha256.is_some())
            .count() as u64;
        for row in &self.rows {
            let Some(expected) = &row.body_sha256 else {
                continue;
            };
            let (Some(start), Some(end)) = (row.start_line, row.end_line) else {
                return Err(ProvenanceError::new("BODY_HASH_WITHOUT_TWO_END_RANGE"));
            };
            let bytes = checked_file_bytes(research_root, &row.semantic_path)?;
            let actual = body_slice_sha256_lf(&bytes, start, end)?;
            checked += 1;
            if &actual != expected {
                return Err(ProvenanceError::at(
                    "BODY_HASH_MISMATCH",
                    &row.semantic_path,
                ));
            }
        }
        Ok(BodyHashReport {
            rows,
            checked,
            mismatched: 0,
        })
    }
}

/// True when a path string is safe to emit into semantic metadata. Absolute,
/// drive-qualified, backslash, traversal, and control-character paths are
/// rejected by the existing shared relative-path contract.
pub fn validate_relative_metadata_path(path: &str) -> Result<(), ProvenanceError> {
    crate::validate_relative_path(path)
        .map_err(|_| ProvenanceError::at("UNSAFE_RELATIVE_PATH", path))
}

/// Read a file only after the shared no-follow ancestor/leaf check. This is the
/// same Windows reparse-point contract used by the corpus loader.
pub fn checked_file_bytes(root: &Path, relative: &str) -> Result<Vec<u8>, ProvenanceError> {
    validate_relative_metadata_path(relative)?;
    match inspect_relative(root, relative) {
        Ok(SafePathState::File) => {
            fs::read(root.join(relative)).map_err(|_| ProvenanceError::at("READ_ERROR", relative))
        }
        Ok(SafePathState::Missing) => Err(ProvenanceError::at("MISSING_PATH", relative)),
        Ok(SafePathState::Directory) => Err(ProvenanceError::at("PATH_NOT_REGULAR", relative)),
        Err(error) => Err(ProvenanceError::at(&error.code, relative)),
    }
}

/// Canonical compact JSON bytes for a serde_json value: objects are recursively
/// sorted by key, no insignificant whitespace, UTF-8. This is the basis for
/// `expected_original_record_hash`; it is not the raw descriptor-file hash.
pub fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, ProvenanceError> {
    serde_json::to_vec(&canonicalize_value(value.clone()))
        .map_err(|_| ProvenanceError::new("CANONICAL_JSON_SERIALIZE_FAILED"))
}

pub fn canonical_record_hash(record: &Value) -> Result<String, ProvenanceError> {
    Ok(sha256_hex(&canonical_json_bytes(record)?))
}

/// File.ReadAllLines-compatible 1-based inclusive body slicing, with line
/// endings normalized to LF and exactly one trailing LF for the documented
/// body-slice digest only. This value is never a whole-source-file hash.
pub fn body_slice_sha256_lf(
    bytes: &[u8],
    start_line: u64,
    end_line: u64,
) -> Result<String, ProvenanceError> {
    if start_line == 0 || end_line < start_line {
        return Err(ProvenanceError::new("INVALID_SOURCE_RANGE"));
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ProvenanceError::new("SOURCE_NOT_UTF8"))?;
    let lines: Vec<&str> = text.lines().collect();
    let start = usize::try_from(start_line - 1).unwrap_or(usize::MAX);
    let end = usize::try_from(end_line).unwrap_or(usize::MAX);
    if start >= lines.len() || end > lines.len() {
        return Err(ProvenanceError::new("SOURCE_RANGE_OUT_OF_BOUNDS"));
    }
    let mut slice = lines[start..end].join("\n");
    slice.push('\n');
    Ok(sha256_hex(slice.as_bytes()))
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

/// Parse accepted method tables only through their real header/separator
/// boundaries. A numbered pipe row without a bound `TypeDef / MethodDef`
/// column is never accepted. Canonical cells are retained as separate
/// evidence; they can never become the imported semantic range.
pub fn parse_method_map(bytes: &[u8]) -> Result<AcceptedMethodMap, ProvenanceError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ProvenanceError::new("MAP_NOT_UTF8"))?;
    let mut rows = Vec::new();
    for table in parse_map_tables(text) {
        let Some(identity_index) = table
            .header
            .iter()
            .position(|header| header_is_identity(header))
        else {
            continue;
        };
        let Some(identity_cell) = table.cells.get(identity_index) else {
            continue;
        };
        let Some((type_def, method_def)) = parse_type_method_pair(identity_cell) else {
            continue;
        };
        let Some(row_number) = table
            .cells
            .first()
            .and_then(|cell| cell.trim().trim_matches('`').parse::<u64>().ok())
            .filter(|number| *number > 0)
        else {
            continue;
        };
        if rows.iter().any(|row: &AcceptedMapRow| {
            row.type_def.eq_ignore_ascii_case(&type_def)
                && row.method_def.eq_ignore_ascii_case(&method_def)
        }) {
            return Err(ProvenanceError::new("DUPLICATE_MAP_METHOD"));
        }
        let overlay_file = table.header.iter().enumerate().find_map(|(index, header)| {
            if header_is_overlay_method(header) {
                table
                    .cells
                    .get(index)
                    .and_then(|cell| overlay_method_file(cell))
            } else {
                None
            }
        });
        let (semantic_path, range) = semantic_anchor(&table, overlay_file.as_deref())?;
        let (canonical_path, canonical_start, canonical_end) = canonical_anchor(&table)?;
        let raw_locator = table
            .header
            .iter()
            .enumerate()
            .find(|(_, header)| header_is_raw_reference(header))
            .and_then(|(index, _)| table.cells.get(index))
            .map(|cell| cell.trim().trim_matches('`').trim().to_owned())
            .filter(|cell| !cell.is_empty());
        let body_sha256 = table
            .header
            .iter()
            .position(|header| header_has_body_sha(header))
            .and_then(|index| table.cells.get(index))
            .and_then(|cell| exact_lower_hex_64(cell.trim().trim_matches('`').trim()));
        let (start_line, end_line, single_line) = range;
        rows.push(AcceptedMapRow {
            row: row_number,
            map_line: u64::try_from(table.physical_line).unwrap_or(u64::MAX),
            type_def: normalize_hex(&type_def),
            method_def: normalize_hex(&method_def),
            semantic_path,
            start_line,
            end_line,
            single_line,
            canonical_path,
            canonical_start_line: canonical_start,
            canonical_end_line: canonical_end,
            raw_locator,
            body_sha256,
        });
    }
    if rows.is_empty() {
        return Err(ProvenanceError::new("MAP_HAS_NO_ACCEPTED_ROWS"));
    }
    Ok(AcceptedMethodMap { rows })
}

#[derive(Debug, Clone)]
struct MapTableRow {
    header: Vec<String>,
    physical_line: usize,
    cells: Vec<String>,
}

fn parse_map_tables(text: &str) -> Vec<MapTableRow> {
    let lines: Vec<&str> = text.lines().collect();
    let mut rows = Vec::new();
    let mut header: Vec<String> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !line.trim_start().starts_with('|') {
            header.clear();
            continue;
        }
        let cells = split_markdown_cells(line);
        if index + 1 < lines.len() && lines[index + 1].trim_start().starts_with('|') {
            let next = split_markdown_cells(lines[index + 1]);
            if is_separator_row(&next) {
                header = cells;
                continue;
            }
        }
        if is_separator_row(&cells) || header.is_empty() {
            continue;
        }
        rows.push(MapTableRow {
            header: header.clone(),
            physical_line: index + 1,
            cells,
        });
    }
    rows
}

fn split_markdown_cells(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut in_backticks = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            current.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            current.push(character);
            continue;
        }
        if character == '`' {
            in_backticks = !in_backticks;
            current.push(character);
            continue;
        }
        if character == '|' && !in_backticks {
            cells.push(current.trim().to_owned());
            current.clear();
            continue;
        }
        current.push(character);
    }
    cells.push(current.trim().to_owned());
    cells
}

fn is_separator_row(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells.iter().all(|cell| {
            let trimmed = cell.trim();
            !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|character| character == '-' || character == ':' || character == ' ')
                && trimmed.chars().any(|character| character == '-')
        })
}

fn normalized_header(header: &str) -> String {
    header
        .to_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

fn header_is_identity(header: &str) -> bool {
    let normalized = normalized_header(header);
    normalized.contains("methoddef")
        && (normalized.contains("typedef") || normalized.contains("type"))
}

fn header_is_overlay_method(header: &str) -> bool {
    normalized_header(header).contains("overlaymethod")
}

fn header_is_semantic(header: &str) -> bool {
    let normalized = normalized_header(header);
    !normalized.contains("canonical")
        && (normalized.contains("semantic")
            || normalized.contains("overlay")
            || normalized.contains("selectedbody")
            || normalized.contains("evidencelevel")
            || normalized.contains("freshcil")
            || normalized.contains("currentindependentcoverage"))
}

fn header_is_raw_reference(header: &str) -> bool {
    let normalized = normalized_header(header);
    !normalized.contains("canonical")
        && !normalized.contains("semantic")
        && (normalized.contains("raw") || normalized.contains("methodsrow"))
}

fn header_has_body_sha(header: &str) -> bool {
    let normalized = normalized_header(header);
    normalized.contains("bodysha256")
        || normalized.contains("bodysourcesha")
        || normalized.contains("sourceslicesha")
        || normalized.contains("sourceslicsha")
        || normalized == "bodysha"
}

fn exact_lower_hex_64(value: &str) -> Option<String> {
    if value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Some(value.to_owned())
    } else {
        None
    }
}

fn parse_type_method_pair(cell: &str) -> Option<(String, String)> {
    let type_positions = token_positions(cell, "0x02");
    let method_positions = token_positions(cell, "0x06");
    if type_positions.len() != 1 || method_positions.len() != 1 {
        return None;
    }
    let type_end = type_positions[0].0 + 10;
    if type_end > method_positions[0].0 {
        return None;
    }
    let between = cell[type_end..method_positions[0].0]
        .trim()
        .trim_matches('`')
        .trim();
    if between != "/" {
        return None;
    }
    Some((type_positions[0].1.clone(), method_positions[0].1.clone()))
}

fn token_positions(cell: &str, prefix: &str) -> Vec<(usize, String)> {
    let bytes = cell.as_bytes();
    let mut positions = Vec::new();
    let mut index = 0;
    while index + 10 <= bytes.len() {
        if cell[index..].starts_with(prefix)
            && bytes[index + 2..index + 10]
                .iter()
                .all(|byte| byte.is_ascii_hexdigit())
            && (index == 0 || !bytes[index - 1].is_ascii_hexdigit() || bytes[index - 1] == b'x')
        {
            let token = &cell[index..index + 10];
            if token[2..].bytes().all(|byte| byte.is_ascii_hexdigit()) {
                positions.push((index, token.to_ascii_lowercase()));
                index += 10;
                continue;
            }
        }
        index += 1;
    }
    positions
}

fn overlay_method_file(cell: &str) -> Option<String> {
    let marker = ".cs::";
    let index = cell.find(marker)?;
    let start = cell[..index]
        .rfind(|character: char| {
            character.is_whitespace() || matches!(character, '`' | '(' | ')' | ';' | '/')
        })
        .map_or(0, |index| index + 1);
    let file = cell[start..index + 3].trim_matches('`');
    if file.ends_with(".cs") && file.len() > 3 {
        Some(file.to_owned())
    } else {
        None
    }
}

fn semantic_anchor(
    table: &MapTableRow,
    overlay_file: Option<&str>,
) -> Result<(String, (Option<u64>, Option<u64>, Option<u64>)), ProvenanceError> {
    let mut candidates: Vec<(String, (Option<u64>, Option<u64>, Option<u64>))> = Vec::new();
    for (index, cell) in table.cells.iter().enumerate() {
        let header = table.header.get(index).map(String::as_str).unwrap_or("");
        let header_normalized = normalized_header(header);
        let canonical_column = header_normalized.contains("canonical");
        if cell.contains("src/reconstructed/managed/semantic/gameplay/") {
            if canonical_column {
                return Err(ProvenanceError::new("SEMANTIC_ANCHOR_IN_CANONICAL_COLUMN"));
            }
            if let Some(found) = parse_full_semantic_cell(cell) {
                candidates.push(found);
            }
            continue;
        }
        if canonical_column {
            continue;
        }
        if header_is_semantic(header) {
            if let Some(found) = parse_short_semantic_cell(cell) {
                candidates.push(found);
                continue;
            }
            if overlay_file.is_some() {
                if let Some(found) = parse_overlay_range(cell) {
                    candidates.push((
                        format!(
                            "src/reconstructed/managed/semantic/gameplay/{}",
                            overlay_file.unwrap_or_default()
                        ),
                        found,
                    ));
                }
            }
        } else if cell.to_ascii_lowercase().contains("overlay") && overlay_file.is_some() {
            if let Some(found) = parse_overlay_range(cell) {
                candidates.push((
                    format!(
                        "src/reconstructed/managed/semantic/gameplay/{}",
                        overlay_file.unwrap_or_default()
                    ),
                    found,
                ));
            }
        }
    }
    if let Some(file) = overlay_file {
        if candidates.is_empty() {
            candidates.push((
                format!("src/reconstructed/managed/semantic/gameplay/{file}"),
                (None, None, None),
            ));
        }
    }
    if candidates.is_empty() {
        return Err(ProvenanceError::new("SEMANTIC_ANCHOR_UNRESOLVED"));
    }
    let path = candidates[0].0.clone();
    if candidates.iter().any(|candidate| candidate.0 != path) {
        return Err(ProvenanceError::new("CONFLICTING_SEMANTIC_ANCHORS"));
    }
    let mut range = candidates[0].1;
    for candidate in candidates.iter().skip(1) {
        if candidate.1 != (None, None, None) {
            if range != (None, None, None) && range != candidate.1 {
                return Err(ProvenanceError::new("CONFLICTING_SEMANTIC_RANGES"));
            }
            range = candidate.1;
        }
    }
    Ok((path, range))
}

fn canonical_anchor(
    table: &MapTableRow,
) -> Result<(Option<String>, Option<u64>, Option<u64>), ProvenanceError> {
    let mut path = None;
    let mut start = None;
    let mut end = None;
    for (index, cell) in table.cells.iter().enumerate() {
        let header = table.header.get(index).map(String::as_str).unwrap_or("");
        if !normalized_header(header).contains("canonical") {
            continue;
        }
        let marker = "src/reconstructed/managed/full/";
        if let Some(found) = cell.find(marker) {
            let tail = &cell[found..];
            let cs_end = tail.find(".cs").map(|index| index + 3);
            if let Some(cs_end) = cs_end {
                let candidate = &tail[..cs_end];
                if candidate.contains('\\') {
                    return Err(ProvenanceError::new("CANONICAL_ANCHOR_INVALID"));
                }
                path = Some(candidate.to_owned());
                let (candidate_start, candidate_end, _) = parse_range_tail(&tail[cs_end..]);
                start = candidate_start;
                end = candidate_end;
            }
        } else if path.is_some() && start.is_none() {
            let (candidate_start, candidate_end, single) = parse_range_tail(cell);
            if candidate_start.is_some() && candidate_end.is_some() && single.is_none() {
                start = candidate_start;
                end = candidate_end;
            }
        }
    }
    Ok((path, start, end))
}

fn parse_full_semantic_cell(
    cell: &str,
) -> Option<(String, (Option<u64>, Option<u64>, Option<u64>))> {
    let marker = "src/reconstructed/managed/semantic/gameplay/";
    let start = cell.find(marker)?;
    let remainder = &cell[start..];
    let cs_end = remainder.find(".cs")? + 3;
    let path = &remainder[..cs_end];
    if path.contains('\\') {
        return None;
    }
    Some((path.to_owned(), parse_range_tail(&remainder[cs_end..])))
}

fn parse_short_semantic_cell(
    cell: &str,
) -> Option<(String, (Option<u64>, Option<u64>, Option<u64>))> {
    if cell.contains('/') || cell.contains("src/") {
        return None;
    }
    let index = cell.find(".cs:")?;
    let file_start = cell[..index]
        .rfind(|character: char| {
            character.is_whitespace() || matches!(character, '`' | '(' | ')' | ';' | ',')
        })
        .map_or(0, |index| index + 1);
    let file = &cell[file_start..index + 3];
    if !file.ends_with(".cs") {
        return None;
    }
    Some((
        format!("src/reconstructed/managed/semantic/gameplay/{file}"),
        parse_range_tail(&cell[index + 3..]),
    ))
}

fn parse_overlay_range(cell: &str) -> Option<(Option<u64>, Option<u64>, Option<u64>)> {
    let index = cell.to_ascii_lowercase().find("overlay")?;
    let after = &cell[index + "overlay".len()..];
    let colon = after.find(':')?;
    Some(parse_range_tail(&after[colon..]))
}

fn parse_range_tail(tail: &str) -> (Option<u64>, Option<u64>, Option<u64>) {
    let mut rest = tail.trim_start_matches(|character: char| {
        character.is_whitespace() || matches!(character, '`' | ':' | ';' | '(' | ')')
    });
    if let Some(after_getter) = rest.strip_prefix("getter") {
        rest = after_getter.trim_start();
    } else if let Some(after_setter) = rest.strip_prefix("setter") {
        rest = after_setter.trim_start();
    }
    let digits: String = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return (None, None, None);
    }
    let start = digits.parse::<u64>().ok();
    let after_start = &rest[digits.len()..];
    if let Some(after_dash) = after_start.strip_prefix('-') {
        let end_digits: String = after_dash
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect();
        if let Some(end) = end_digits.parse::<u64>().ok() {
            return (Some(start.unwrap_or(0)), Some(end), None);
        }
    }
    (None, None, start)
}

fn normalize_hex(value: &str) -> String {
    format!("0x{}", value[2..].to_ascii_uppercase())
}

fn valid_hex_word(value: &str) -> bool {
    value.len() == 10
        && value.starts_with("0x")
        && value[2..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl ErrataDocument {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProvenanceError> {
        let value = parse_strict_value(bytes).map_err(|error| ProvenanceError::new(&error.code))?;
        let document: Self = serde_json::from_value(value)
            .map_err(|_| ProvenanceError::new("ERRATA_SCHEMA_INVALID"))?;
        document.validate_shape()?;
        Ok(document)
    }

    pub fn validate_shape(&self) -> Result<(), ProvenanceError> {
        if self.schema != ERRATA_SCHEMA {
            return Err(ProvenanceError::new("ERRATA_SCHEMA_MISMATCH"));
        }
        if self.version != ERRATA_VERSION {
            return Err(ProvenanceError::new("ERRATA_VERSION_INVALID"));
        }
        if self.campaign != ERRATA_CAMPAIGN {
            return Err(ProvenanceError::new("ERRATA_CAMPAIGN_MISMATCH"));
        }
        validate_frozen_file(&self.catalog)?;
        validate_frozen_file(&self.index)?;
        validate_frozen_file(&self.accepted_map)?;
        let mut bound_paths = std::collections::BTreeSet::new();
        for frozen in &self.bound_catalogs {
            validate_frozen_file(frozen)?;
            if !bound_paths.insert(frozen.path.clone()) {
                return Err(ProvenanceError::at("DUPLICATE_BOUND_CATALOG", &frozen.path));
            }
        }
        if let Some(v5) = &self.v5_receipt {
            validate_frozen_file(v5)?;
            if v5.path != V5_RECEIPT_REL {
                return Err(ProvenanceError::new("V5_RECEIPT_PATH_INVALID"));
            }
        }
        if self.accepted_map.path != METHOD_EVIDENCE_MAP {
            return Err(ProvenanceError::new("ERRATA_MAP_PATH_INVALID"));
        }
        if self.entries.is_empty() {
            return Err(ProvenanceError::new("ERRATA_ENTRIES_EMPTY"));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut previous: Option<(String, String, String, String, String)> = None;
        for entry in &self.entries {
            validate_entry_shape(entry)?;
            let key = canonical_target_key(entry);
            if !seen.insert(key.clone()) {
                return Err(ProvenanceError::record("COMPETING_ERRATA_TARGET", &key.0));
            }
            if let Some(previous_key) = &previous {
                if key < *previous_key {
                    return Err(ProvenanceError::record("ERRATA_OUT_OF_ORDER", &key.0));
                }
            }
            previous = Some(key);
        }
        Ok(())
    }

    /// Validate every proposed correction against actual frozen bytes and the
    /// accepted method map. This rederives the supported path/range from the
    /// map; a matching hash alone cannot validate a wrong method body.
    pub fn validate_against_roots(
        &self,
        curated_root: &Path,
        research_root: &Path,
    ) -> Result<(), ProvenanceError> {
        self.validate_shape()?;
        if self
            .entries
            .iter()
            .any(|entry| entry.review_status == ReviewStatus::PrimaryRejected)
        {
            return Err(ProvenanceError::new("REVIEW_REJECTED"));
        }
        let catalog_bytes = checked_frozen_file(curated_root, &self.catalog)?;
        let index_bytes = checked_frozen_file(curated_root, &self.index)?;
        let map_bytes = checked_frozen_file(research_root, &self.accepted_map)?;
        let _ = index_bytes;
        let catalog = parse_strict_value(&catalog_bytes)
            .map_err(|error| ProvenanceError::new(&error.code))?;
        let accepted_map = parse_method_map(&map_bytes)?;
        accepted_map.validate_body_hashes(research_root)?;
        let records = catalog
            .get("records")
            .and_then(Value::as_array)
            .ok_or_else(|| ProvenanceError::new("CATALOG_RECORDS_INVALID"))?;
        let map_sha = sha256_hex(&map_bytes);
        for entry in &self.entries {
            if entry.evidence_basis.accepted_map_sha256 != map_sha {
                return Err(ProvenanceError::record(
                    "EVIDENCE_BASIS_MAP_SHA_MISMATCH",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        let bound = self.load_bound_catalogs(curated_root)?;
        let v5_entries = if self
            .entries
            .iter()
            .any(|entry| entry.profile == ProvenanceProfile::LegacyV5SourceAssociation)
        {
            Some(load_v5_entries(curated_root)?)
        } else {
            None
        };
        let needs_registry = self.entries.iter().any(|entry| {
            matches!(
                entry.profile,
                ProvenanceProfile::RegistryFileAssociation
                    | ProvenanceProfile::LegacyV5SourceAssociation
                    | ProvenanceProfile::MigrationRecordAssociation
            )
        });
        let registry = if needs_registry {
            Some(load_registry(curated_root, &self.entries)?)
        } else {
            None
        };
        for entry in &self.entries {
            match entry.profile {
                ProvenanceProfile::OrdinaryGameplayMap => validate_entry_against_catalog(
                    entry,
                    records,
                    &accepted_map,
                    &map_bytes,
                    curated_root,
                    research_root,
                )?,
                ProvenanceProfile::RegistryFileAssociation => validate_registry_entry(
                    entry,
                    records,
                    curated_root,
                    research_root,
                    registry.as_ref().expect("registry loaded"),
                )?,
                ProvenanceProfile::AssociatedGameplayMap => validate_associated_entry(
                    entry,
                    records,
                    &map_bytes,
                    curated_root,
                    research_root,
                )?,
                ProvenanceProfile::PerformanceEvidenceAssociation => {
                    validate_performance_entry(entry, records, curated_root, research_root)?
                }
                ProvenanceProfile::LegacyV5SourceAssociation
                | ProvenanceProfile::MigrationRecordAssociation => {
                    let Some(path) = &entry.target_catalog_path else {
                        return Err(ProvenanceError::new("LEGACY_TARGET_CATALOG_MISSING"));
                    };
                    let target_records = bound
                        .get(path)
                        .ok_or_else(|| ProvenanceError::at("BOUND_CATALOG_MISSING", path))?;
                    validate_legacy_entry(
                        entry,
                        target_records,
                        curated_root,
                        research_root,
                        registry.as_ref().expect("registry loaded"),
                        v5_entries.as_ref(),
                    )?;
                }
                ProvenanceProfile::Unresolved => {
                    let target_records = if let Some(path) = &entry.target_catalog_path {
                        bound
                            .get(path)
                            .ok_or_else(|| ProvenanceError::at("BOUND_CATALOG_MISSING", path))?
                    } else {
                        records
                    };
                    let _ = validate_target_structure(entry, target_records)?;
                }
            }
        }
        Ok(())
    }

    /// Acceptance/activation gate. This is deliberately separate from proposal
    /// inspection so pending or unresolved accounting rows cannot be activated.
    pub fn validate_for_activation(
        &self,
        curated_root: &Path,
        research_root: &Path,
    ) -> Result<(), ProvenanceError> {
        self.validate_shape()?;
        for entry in &self.entries {
            validate_entry_activation_policy(entry)?;
        }
        self.validate_against_roots(curated_root, research_root)
    }

    /// Structural validation against the frozen catalog/index and canonical
    /// target records. This deliberately does not inspect the accepted method
    /// table or the research root.
    pub fn validate_structural(&self, curated_root: &Path) -> Result<(), ProvenanceError> {
        self.validate_shape()?;
        if self.catalog.path != CATALOG_REL {
            return Err(ProvenanceError::new("ERRATA_CATALOG_PATH_INVALID"));
        }
        if self.index.path != INDEX_REL {
            return Err(ProvenanceError::new("ERRATA_INDEX_PATH_INVALID"));
        }
        let catalog_bytes = checked_frozen_file(curated_root, &self.catalog)?;
        let _index_bytes = checked_frozen_file(curated_root, &self.index)?;
        let catalog = parse_strict_value(&catalog_bytes)
            .map_err(|error| ProvenanceError::new(&error.code))?;
        let records = catalog
            .get("records")
            .and_then(Value::as_array)
            .ok_or_else(|| ProvenanceError::new("CATALOG_RECORDS_INVALID"))?;
        let bound = self.load_bound_catalogs(curated_root)?;
        if self
            .entries
            .iter()
            .any(|entry| entry.profile == ProvenanceProfile::LegacyV5SourceAssociation)
            && self.v5_receipt.is_none()
        {
            return Err(ProvenanceError::new("V5_RECEIPT_BINDING_MISSING"));
        }
        if let Some(v5) = &self.v5_receipt {
            let bytes = checked_frozen_file(curated_root, v5)?;
            if bytes.len() as u64 != 61_237 || sha256_hex(&bytes) != V5_RECEIPT_SHA256 {
                return Err(ProvenanceError::new("V5_RECEIPT_DIGEST_MISMATCH"));
            }
        }
        for entry in &self.entries {
            let target_records = match entry.profile {
                ProvenanceProfile::LegacyV5SourceAssociation
                | ProvenanceProfile::MigrationRecordAssociation => {
                    let Some(path) = &entry.target_catalog_path else {
                        return Err(ProvenanceError::new("LEGACY_TARGET_CATALOG_MISSING"));
                    };
                    bound
                        .get(path)
                        .ok_or_else(|| ProvenanceError::at("BOUND_CATALOG_MISSING", path))?
                }
                _ => records,
            };
            validate_entry_structure(entry, target_records)?;
        }
        Ok(())
    }

    fn load_bound_catalogs(
        &self,
        curated_root: &Path,
    ) -> Result<BTreeMap<String, Vec<Value>>, ProvenanceError> {
        let mut bound = BTreeMap::new();
        for frozen in &self.bound_catalogs {
            let expected = match frozen.path.as_str() {
                MIGRATION_REL => crate::BASELINE_MANIFEST_SHA256,
                PERFORMANCE_SOURCE_REL => crate::PERFORMANCE_SOURCE_V1_SHA256,
                SCHEDULING_SOURCE_REL => crate::SCHEDULING_SOURCE_V1_SHA256,
                TASK_SOURCE_REL => crate::TASK_EXECUTION_SOURCE_V1_SHA256,
                _ => {
                    return Err(ProvenanceError::at(
                        "BOUND_CATALOG_PATH_INVALID",
                        &frozen.path,
                    ));
                }
            };
            if frozen.sha256 != expected {
                return Err(ProvenanceError::at(
                    "BOUND_CATALOG_DIGEST_MISMATCH",
                    &frozen.path,
                ));
            }
            let bytes = checked_frozen_file(curated_root, frozen)?;
            let value =
                parse_strict_value(&bytes).map_err(|error| ProvenanceError::new(&error.code))?;
            let records = value
                .get("records")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ProvenanceError::at("BOUND_CATALOG_RECORDS_INVALID", &frozen.path)
                })?;
            let mut flattened = Vec::with_capacity(records.len());
            for record in records {
                flattened.push(
                    record
                        .get("record")
                        .cloned()
                        .unwrap_or_else(|| record.clone()),
                );
            }
            bound.insert(frozen.path.clone(), flattened);
        }
        Ok(bound)
    }

    /// Structural validation plus the publication/activation policy. The
    /// effective-view loader separates the two so a structurally valid but
    /// pending document is reported as a blocked state, not silently applied
    /// and not misreported as a syntax/schema failure.
    pub fn validate_structural_for_activation(
        &self,
        curated_root: &Path,
    ) -> Result<(), ProvenanceError> {
        self.validate_structural(curated_root)?;
        for entry in &self.entries {
            validate_entry_activation_policy(entry)?;
        }
        Ok(())
    }
}

fn validate_entry_activation_policy(entry: &ErrataEntry) -> Result<(), ProvenanceError> {
    if entry.review_status != ReviewStatus::PrimaryApproved {
        return Err(ProvenanceError::record(
            "ACTIVATION_REVIEW_NOT_APPROVED",
            &entry.target_catalog_record_id,
        ));
    }
    if !matches!(
        entry.outcome,
        Outcome::ResolvedExact | Outcome::ResolvedConservative
    ) {
        return Err(ProvenanceError::record(
            "ACTIVATION_OUTCOME_UNRESOLVED_OR_CONFLICTING",
            &entry.target_catalog_record_id,
        ));
    }
    if entry.corrected_imported_source_anchor.precision == Precision::Unresolved
        || entry
            .corrected_evidence_source_anchors
            .iter()
            .any(|anchor| {
                anchor.evidence_class == EvidenceClass::Unresolved
                    || anchor.precision == Precision::Unresolved
            })
    {
        return Err(ProvenanceError::record(
            "ACTIVATION_UNRESOLVED_ANCHOR",
            &entry.target_catalog_record_id,
        ));
    }
    Ok(())
}

fn target_record_and_method<'a>(
    entry: &ErrataEntry,
    records: &'a [Value],
) -> Result<(&'a Value, &'a Value), ProvenanceError> {
    let record = records
        .iter()
        .find(|record| {
            record.get("id").and_then(Value::as_str)
                == Some(entry.target_catalog_record_id.as_str())
        })
        .ok_or_else(|| {
            ProvenanceError::record("TARGET_RECORD_MISSING", &entry.target_catalog_record_id)
        })?;
    let methods_value = record.get("methods").ok_or_else(|| {
        ProvenanceError::record("TARGET_METHODS_INVALID", &entry.target_catalog_record_id)
    })?;
    let methods: &[Value] = if let Some(array) = methods_value.as_array() {
        array
    } else if let Some(array) = methods_value.get("current").and_then(Value::as_array) {
        array
    } else {
        return Err(ProvenanceError::record(
            "TARGET_METHODS_INVALID",
            &entry.target_catalog_record_id,
        ));
    };
    let method = methods
        .iter()
        .find(|method| {
            method_type_token(method).eq_ignore_ascii_case(&entry.method_identity.type_def)
                && method_def_token(method).eq_ignore_ascii_case(&entry.method_identity.method_def)
        })
        .ok_or_else(|| {
            ProvenanceError::record("TARGET_METHOD_MISSING", &entry.target_catalog_record_id)
        })?;
    Ok((record, method))
}

fn method_type_token(method: &Value) -> String {
    let direct = value_string(method, "type_def");
    if !direct.is_empty() {
        return direct;
    }
    method
        .get("symbol_key")
        .map(|key| value_string(key, "typeDef"))
        .unwrap_or_default()
}

fn method_def_token(method: &Value) -> String {
    let direct = value_string(method, "method_def");
    if !direct.is_empty() {
        return direct;
    }
    let direct = value_string(method, "methodDef");
    if !direct.is_empty() {
        return direct;
    }
    method
        .get("symbol_key")
        .map(|key| value_string(key, "methodDef"))
        .unwrap_or_default()
}

fn record_source_path(record: &Value) -> String {
    let direct = value_string(record, "source_path");
    if !direct.is_empty() {
        return direct;
    }
    value_string(record, "original_path")
}

fn record_source_sha(record: &Value) -> String {
    let direct = value_string(record, "source_sha256");
    if !direct.is_empty() {
        return direct;
    }
    value_string(record, "original_sha256")
}

fn record_source_size(record: &Value) -> u64 {
    record
        .get("source_size")
        .and_then(Value::as_u64)
        .or_else(|| record.get("original_size").and_then(Value::as_u64))
        .unwrap_or(0)
}

fn record_destination_path(record: &Value) -> String {
    let direct = value_string(record, "destination_path");
    if !direct.is_empty() {
        return direct;
    }
    value_string(record, "proposed_path")
}

fn validate_entry_structure(entry: &ErrataEntry, records: &[Value]) -> Result<(), ProvenanceError> {
    let (record, method) = validate_target_structure(entry, records)?;
    let source_path = record_source_path(record);
    let imported = &entry.corrected_imported_source_anchor;
    match entry.profile {
        ProvenanceProfile::OrdinaryGameplayMap => {
            if value_string(record, "cohort") != GAMEPLAY_COHORT
                || value_string(method, "role") != "ORDINARY"
            {
                return Err(ProvenanceError::record(
                    "ERRATA_TARGET_OUTSIDE_ORDINARY_GAMEPLAY_PROFILE",
                    &entry.target_catalog_record_id,
                ));
            }
            let original_source_lines_null =
                method.get("source_lines").map_or(true, Value::is_null);
            if original_source_lines_null
                && imported.precision == Precision::ExactRange
                && entry.action != ErrataAction::Refine
            {
                return Err(ProvenanceError::record(
                    "NEW_PLACEMENT_MUST_BE_REFINE",
                    &entry.target_catalog_record_id,
                ));
            }
            let mut has_map_anchor = false;
            let mut has_semantic_anchor = false;
            for anchor in &entry.corrected_evidence_source_anchors {
                match anchor.evidence_class {
                    EvidenceClass::AcceptedMethodMap => {
                        if anchor.path != METHOD_EVIDENCE_MAP {
                            return Err(ProvenanceError::record(
                                "STRUCTURAL_MAP_ANCHOR_PATH_INVALID",
                                &entry.target_catalog_record_id,
                            ));
                        }
                        has_map_anchor = true;
                    }
                    EvidenceClass::SemanticSource => {
                        if anchor.path != source_path {
                            return Err(ProvenanceError::record(
                                "STRUCTURAL_SEMANTIC_ANCHOR_PATH_INVALID",
                                &entry.target_catalog_record_id,
                            ));
                        }
                        has_semantic_anchor = true;
                    }
                    _ => {}
                }
            }
            if !has_map_anchor || !has_semantic_anchor {
                return Err(ProvenanceError::record(
                    "MANDATORY_EVIDENCE_ANCHOR_MISSING",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        ProvenanceProfile::RegistryFileAssociation => {
            let cohort = value_string(record, "cohort");
            let role = value_string(method, "role");
            if !(cohort == GAMEPLAY_COHORT && role == "ORDINARY")
                && !(cohort == crate::bulk::PERFORMANCE_COHORT
                    && (role == "ORDINARY" || role == "ASSOCIATED"))
            {
                return Err(ProvenanceError::record(
                    "REGISTRY_TARGET_OUTSIDE_FILE_PROFILE",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        ProvenanceProfile::AssociatedGameplayMap => {
            if value_string(record, "cohort") != GAMEPLAY_COHORT
                || value_string(method, "role") != "ASSOCIATED"
            {
                return Err(ProvenanceError::record(
                    "ASSOCIATED_TARGET_NOT_GAMEPLAY_ASSOCIATION",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        ProvenanceProfile::PerformanceEvidenceAssociation => {
            let role = value_string(method, "role");
            if value_string(record, "cohort") != crate::bulk::PERFORMANCE_COHORT
                || (role != "ORDINARY" && role != "ASSOCIATED")
            {
                return Err(ProvenanceError::record(
                    "PERFORMANCE_TARGET_NOT_ORDINARY",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        ProvenanceProfile::LegacyV5SourceAssociation
        | ProvenanceProfile::MigrationRecordAssociation => {}
        ProvenanceProfile::Unresolved => {}
    }
    Ok(())
}

fn validate_target_structure<'a>(
    entry: &ErrataEntry,
    records: &'a [Value],
) -> Result<(&'a Value, &'a Value), ProvenanceError> {
    let (record, method) = target_record_and_method(entry, records)?;
    let actual_hash = canonical_record_hash(record)?;
    if actual_hash != entry.expected_original_record_hash {
        return Err(ProvenanceError::record(
            "STALE_TARGET_RECORD_HASH",
            &entry.target_catalog_record_id,
        ));
    }
    let record_identity =
        method_identity_from_record(record, method, &entry.target_catalog_record_id)?;
    if record_identity != entry.method_identity {
        return Err(ProvenanceError::record(
            "TARGET_METHOD_IDENTITY_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    verify_incorrect_anchors(entry, method)?;
    let source_path = record_source_path(record);
    let destination_path = record_destination_path(record);
    let source_sha = record_source_sha(record);
    let imported = &entry.corrected_imported_source_anchor;
    if imported.path != destination_path
        || imported.whole_file_sha256 != source_sha
        || imported.method_identity != entry.method_identity
    {
        return Err(ProvenanceError::record(
            "STRUCTURAL_SOURCE_BINDING_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    for anchor in &entry.incorrect_anchors {
        if anchor.path != source_path {
            return Err(ProvenanceError::record(
                "INCORRECT_ANCHOR_NOT_OVERLAY_SOURCE",
                &entry.target_catalog_record_id,
            ));
        }
    }
    Ok((record, method))
}

#[derive(Debug, Clone)]
struct RegistryRow {
    root_id: String,
    path: String,
    size: u64,
    sha256: String,
    roles: Vec<String>,
}

#[derive(Debug, Default)]
struct RegistryIndex {
    rows: BTreeMap<String, Vec<RegistryRow>>,
}

impl RegistryIndex {
    fn exact(
        &self,
        method_id: &str,
        root_id: &str,
        path: &str,
        sha256: &str,
        size: u64,
        allowed_roles: &[&str],
    ) -> bool {
        self.rows.get(method_id).is_some_and(|rows| {
            rows.iter().any(|row| {
                row.root_id == root_id
                    && row.path == path
                    && row.sha256 == sha256
                    && row.size == size
                    && row
                        .roles
                        .iter()
                        .any(|role| allowed_roles.iter().any(|allowed| role == allowed))
            })
        })
    }
}

fn completion_method_id(identity: &MethodIdentity) -> String {
    let text = format!(
        "{}\0{}\0{}\0{}",
        identity.full_authority_sha256,
        identity.mvid,
        identity.type_def.to_ascii_lowercase(),
        identity.method_def.to_ascii_lowercase()
    );
    sha256_hex(text.as_bytes())
}

fn load_registry(
    curated_root: &Path,
    entries: &[ErrataEntry],
) -> Result<RegistryIndex, ProvenanceError> {
    validate_relative_metadata_path(REGISTRY_REL)?;
    match inspect_relative(curated_root, REGISTRY_REL) {
        Ok(SafePathState::File) => {}
        Ok(SafePathState::Missing) => {
            return Err(ProvenanceError::at("REGISTRY_MISSING", REGISTRY_REL));
        }
        Ok(SafePathState::Directory) => {
            return Err(ProvenanceError::at("REGISTRY_NOT_REGULAR", REGISTRY_REL));
        }
        Err(error) => return Err(ProvenanceError::at(&error.code, REGISTRY_REL)),
    }
    let mut needed = std::collections::BTreeSet::new();
    for entry in entries {
        if matches!(
            entry.profile,
            ProvenanceProfile::RegistryFileAssociation
                | ProvenanceProfile::LegacyV5SourceAssociation
                | ProvenanceProfile::MigrationRecordAssociation
        ) {
            needed.insert(completion_method_id(&entry.method_identity));
        }
    }
    let file = fs::File::open(curated_root.join(REGISTRY_REL))
        .map_err(|_| ProvenanceError::at("REGISTRY_READ_ERROR", REGISTRY_REL))?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut total_bytes = 0_u64;
    let mut line = Vec::new();
    let mut index = RegistryIndex::default();
    loop {
        line.clear();
        let read = reader
            .read_until(b'\n', &mut line)
            .map_err(|_| ProvenanceError::at("REGISTRY_READ_ERROR", REGISTRY_REL))?;
        if read == 0 {
            break;
        }
        total_bytes += read as u64;
        hasher.update(&line);
        if !line.windows(12).any(|window| window == b"\"method_ids\"") {
            continue;
        }
        let Ok(value) = serde_json::from_slice::<Value>(&line) else {
            return Err(ProvenanceError::at("REGISTRY_ROW_INVALID", REGISTRY_REL));
        };
        let Some(artifact) = value.get("artifact") else {
            continue;
        };
        let row = RegistryRow {
            root_id: value_string(artifact, "root_id"),
            path: value_string(artifact, "path"),
            size: artifact.get("size").and_then(Value::as_u64).unwrap_or(0),
            sha256: value_string(artifact, "sha256"),
            roles: value
                .get("roles")
                .and_then(Value::as_array)
                .map(|roles| {
                    roles
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
        };
        let Some(method_ids) = value.get("method_ids").and_then(Value::as_array) else {
            continue;
        };
        for method_id in method_ids.iter().filter_map(Value::as_str) {
            if needed.contains(method_id) {
                index
                    .rows
                    .entry(method_id.to_owned())
                    .or_default()
                    .push(row.clone());
            }
        }
    }
    if total_bytes != REGISTRY_BYTES {
        return Err(ProvenanceError::at("REGISTRY_SIZE_MISMATCH", REGISTRY_REL));
    }
    let digest = hasher.finalize();
    let observed: String = digest
        .as_slice()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if observed != REGISTRY_SHA256 {
        return Err(ProvenanceError::at(
            "REGISTRY_SHA256_MISMATCH",
            REGISTRY_REL,
        ));
    }
    Ok(index)
}

fn load_v5_entries(curated_root: &Path) -> Result<BTreeMap<String, Value>, ProvenanceError> {
    let bytes = checked_file_bytes(curated_root, V5_RECEIPT_REL)?;
    if bytes.len() as u64 != 61_237 || sha256_hex(&bytes) != V5_RECEIPT_SHA256 {
        return Err(ProvenanceError::at(
            "V5_RECEIPT_DIGEST_MISMATCH",
            V5_RECEIPT_REL,
        ));
    }
    let value = parse_strict_value(&bytes).map_err(|error| ProvenanceError::new(&error.code))?;
    let imports = value
        .get("imports")
        .and_then(Value::as_array)
        .ok_or_else(|| ProvenanceError::new("V5_IMPORTS_INVALID"))?;
    let mut entries = BTreeMap::new();
    for entry in imports {
        let id = value_string(entry, "id");
        if id.is_empty() {
            return Err(ProvenanceError::new("V5_IMPORT_ID_MISSING"));
        }
        entries.insert(id, entry.clone());
    }
    Ok(entries)
}

fn check_file_digest(
    root: &Path,
    path: &str,
    expected_sha: &str,
    expected_size: Option<u64>,
    prefix: &str,
) -> Result<Vec<u8>, ProvenanceError> {
    let bytes = checked_file_bytes(root, path)?;
    if let Some(size) = expected_size
        && bytes.len() as u64 != size
    {
        return Err(ProvenanceError::at(
            &format!("{prefix}_SIZE_MISMATCH"),
            path,
        ));
    }
    if sha256_hex(&bytes) != expected_sha {
        return Err(ProvenanceError::at(
            &format!("{prefix}_SHA256_MISMATCH"),
            path,
        ));
    }
    Ok(bytes)
}

fn has_evidence_anchor(
    entry: &ErrataEntry,
    class: EvidenceClass,
    path: &str,
    sha256: &str,
) -> bool {
    entry
        .corrected_evidence_source_anchors
        .iter()
        .any(|anchor| {
            anchor.evidence_class == class
                && anchor.path == path
                && anchor.artifact_sha256 == sha256
                && anchor.method_identity == entry.method_identity
        })
}

fn validate_registry_entry(
    entry: &ErrataEntry,
    records: &[Value],
    curated_root: &Path,
    research_root: &Path,
    registry: &RegistryIndex,
) -> Result<(), ProvenanceError> {
    let (record, _method) = validate_target_structure(entry, records)?;
    let source_path = record_source_path(record);
    let source_sha = record_source_sha(record);
    let source_size = record_source_size(record);
    let method_id = completion_method_id(&entry.method_identity);
    if !registry.exact(
        &method_id,
        "RESEARCH_ROOT",
        &source_path,
        &source_sha,
        source_size,
        &["PERFORMANCE_SOURCE"],
    ) {
        return Err(ProvenanceError::record(
            "REGISTRY_FILE_ASSOCIATION_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    check_file_digest(
        research_root,
        &source_path,
        &source_sha,
        Some(source_size),
        "REGISTRY_SOURCE",
    )?;
    check_file_digest(
        curated_root,
        &entry.corrected_imported_source_anchor.path,
        &entry.corrected_imported_source_anchor.whole_file_sha256,
        None,
        "REGISTRY_IMPORTED",
    )?;
    if !has_evidence_anchor(
        entry,
        EvidenceClass::PerformanceSource,
        &source_path,
        &source_sha,
    ) {
        return Err(ProvenanceError::record(
            "REGISTRY_SOURCE_ANCHOR_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    validate_completion_evidence(
        entry,
        CompletionProofSpec {
            source_path: &source_path,
            source_sha: &source_sha,
            source_size,
            map_sha: &entry.evidence_basis.accepted_map_sha256,
            map_line: None,
            report: None,
            registry: true,
            v5: false,
            exact: None,
        },
        curated_root,
        research_root,
    )
}

fn validate_legacy_entry(
    entry: &ErrataEntry,
    records: &[Value],
    curated_root: &Path,
    research_root: &Path,
    registry: &RegistryIndex,
    v5_entries: Option<&BTreeMap<String, Value>>,
) -> Result<(), ProvenanceError> {
    let (record, _method) = validate_target_structure(entry, records)?;
    let source_path = record_source_path(record);
    let source_sha = record_source_sha(record);
    let source_size = record_source_size(record);
    let destination_path = record_destination_path(record);
    let method_id = completion_method_id(&entry.method_identity);
    if entry.profile == ProvenanceProfile::LegacyV5SourceAssociation {
        let Some(v5_entries) = v5_entries else {
            return Err(ProvenanceError::new("V5_RECEIPT_BINDING_MISSING"));
        };
        let Some(v5_entry) = v5_entries.get(&entry.target_catalog_record_id) else {
            return Err(ProvenanceError::record(
                "V5_IMPORT_ENTRY_MISSING",
                &entry.target_catalog_record_id,
            ));
        };
        let v5_methods = if let Some(methods) = v5_entry.get("methods").and_then(Value::as_array) {
            methods.clone()
        } else if let Some(identity) = v5_entry.get("method_identity") {
            vec![json_value_with_identity(identity)]
        } else {
            return Err(ProvenanceError::record(
                "V5_IMPORT_METHODS_INVALID",
                &entry.target_catalog_record_id,
            ));
        };
        let found = v5_methods.iter().any(|method| {
            let identity = method.get("method_identity").unwrap_or(method);
            value_string(identity, "typeDef").eq_ignore_ascii_case(&entry.method_identity.type_def)
                && value_string(identity, "methodDef")
                    .eq_ignore_ascii_case(&entry.method_identity.method_def)
        });
        if !found
            || value_string(v5_entry, "original_path") != source_path
            || value_string(v5_entry, "imported_path") != destination_path
            || value_string(v5_entry, "original_sha256") != source_sha
            || value_string(v5_entry, "imported_sha256") != source_sha
            || v5_entry.get("original_size").and_then(Value::as_u64) != Some(source_size)
        {
            return Err(ProvenanceError::record(
                "V5_IMPORT_BINDING_MISMATCH",
                &entry.target_catalog_record_id,
            ));
        }
    }
    let research_linked = registry.exact(
        &method_id,
        "RESEARCH_ROOT",
        &source_path,
        &source_sha,
        source_size,
        &["CURATED_ORIGINAL", "PERFORMANCE_SOURCE"],
    );
    let curated_linked = registry.exact(
        &method_id,
        "CURATED_ROOT",
        &destination_path,
        &source_sha,
        source_size,
        &["CURATED_IMPORTED"],
    );
    if !research_linked && !curated_linked {
        return Err(ProvenanceError::record(
            "REGISTRY_FILE_ASSOCIATION_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    check_file_digest(
        research_root,
        &source_path,
        &source_sha,
        Some(source_size),
        "LEGACY_SOURCE",
    )?;
    check_file_digest(
        curated_root,
        &destination_path,
        &entry.corrected_imported_source_anchor.whole_file_sha256,
        None,
        "LEGACY_IMPORTED",
    )?;
    if !has_evidence_anchor(
        entry,
        EvidenceClass::LegacyDescriptor,
        &source_path,
        &source_sha,
    ) && !has_evidence_anchor(
        entry,
        EvidenceClass::PerformanceSource,
        &source_path,
        &source_sha,
    ) {
        return Err(ProvenanceError::record(
            "LEGACY_SOURCE_ANCHOR_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    validate_completion_evidence(
        entry,
        CompletionProofSpec {
            source_path: &source_path,
            source_sha: &source_sha,
            source_size,
            map_sha: &entry.evidence_basis.accepted_map_sha256,
            map_line: None,
            report: None,
            registry: true,
            v5: entry.profile == ProvenanceProfile::LegacyV5SourceAssociation,
            exact: None,
        },
        curated_root,
        research_root,
    )
}

struct CompletionProofSpec<'a> {
    source_path: &'a str,
    source_sha: &'a str,
    source_size: u64,
    map_sha: &'a str,
    map_line: Option<u64>,
    report: Option<(&'static str, u64, &'static str)>,
    registry: bool,
    v5: bool,
    exact: Option<(u64, u64)>,
}

fn validate_associated_entry(
    entry: &ErrataEntry,
    records: &[Value],
    map_bytes: &[u8],
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let binding = ASSOCIATED_BINDINGS
        .iter()
        .find(|binding| {
            binding.record_id == entry.target_catalog_record_id
                && binding
                    .type_def
                    .eq_ignore_ascii_case(&entry.method_identity.type_def)
                && binding
                    .method_def
                    .eq_ignore_ascii_case(&entry.method_identity.method_def)
        })
        .ok_or_else(|| {
            ProvenanceError::record(
                "ASSOCIATED_BINDING_UNRESOLVED",
                &entry.target_catalog_record_id,
            )
        })?;
    let (record, _method) = validate_target_structure(entry, records)?;
    if record_source_path(record) != binding.source_path {
        return Err(ProvenanceError::record(
            "ASSOCIATED_BINDING_SOURCE_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let proof = entry
        .association_proof
        .as_ref()
        .ok_or_else(|| ProvenanceError::new("ASSOCIATION_PROOF_MISSING"))?;
    if proof.map_line != Some(binding.map_line)
        || proof
            .literal_method_def
            .as_deref()
            .is_none_or(|literal| !literal.eq_ignore_ascii_case(binding.method_def))
    {
        return Err(ProvenanceError::record(
            "ASSOCIATED_BINDING_SHAPE_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let text = std::str::from_utf8(map_bytes).map_err(|_| ProvenanceError::new("MAP_NOT_UTF8"))?;
    let lines: Vec<&str> = text.lines().collect();
    let index = usize::try_from(binding.map_line.saturating_sub(1)).unwrap_or(usize::MAX);
    let line = lines
        .get(index)
        .ok_or_else(|| ProvenanceError::new("ASSOCIATED_BINDING_LINE_OUT_OF_BOUNDS"))?;
    let lower = line.to_ascii_lowercase();
    let mut binding_text = lower.clone();
    for context_line in binding.context_lines {
        let context_index = usize::try_from(context_line.saturating_sub(1)).unwrap_or(usize::MAX);
        let context = lines
            .get(context_index)
            .ok_or_else(|| ProvenanceError::new("ASSOCIATED_BINDING_CONTEXT_OUT_OF_BOUNDS"))?;
        binding_text.push('\n');
        binding_text.push_str(&context.to_ascii_lowercase());
    }
    for token in binding.required_tokens {
        if !binding_text.contains(&token.to_ascii_lowercase()) {
            return Err(ProvenanceError::record(
                "ASSOCIATED_BINDING_TOKEN_MISSING",
                &entry.target_catalog_record_id,
            ));
        }
    }
    let source_basename = binding
        .source_path
        .rsplit('/')
        .next()
        .unwrap_or(binding.source_path);
    let imported = &entry.corrected_imported_source_anchor;
    let imported_bytes = check_file_digest(
        curated_root,
        &imported.path,
        &imported.whole_file_sha256,
        None,
        "ASSOCIATION_IMPORTED",
    )?;
    if let Some((start, end)) = binding.exact {
        let range_text = format!("{source_basename}:{start}-{end}").to_ascii_lowercase();
        if !lower.contains(&range_text)
            || imported.precision != Precision::ExactRange
            || imported.start_line != Some(start)
            || imported.end_line != Some(end)
        {
            return Err(ProvenanceError::record(
                "ASSOCIATED_BINDING_RANGE_MISMATCH",
                &entry.target_catalog_record_id,
            ));
        }
        validate_actual_range(
            &imported_bytes,
            imported.start_line,
            imported.end_line,
            &imported.path,
            "ASSOCIATION_IMPORTED",
        )?;
    } else {
        if imported.precision != Precision::FileMethodAssociation
            || imported.start_line.is_some()
            || imported.end_line.is_some()
        {
            return Err(ProvenanceError::record(
                "ASSOCIATED_BINDING_PRECISION_MISMATCH",
                &entry.target_catalog_record_id,
            ));
        }
        let row_has_source = lower.contains(&source_basename.to_ascii_lowercase());
        if !row_has_source {
            match (binding.parent_map_line, binding.parent_method_def) {
                (Some(parent_line), Some(parent_method)) => {
                    let parent_index =
                        usize::try_from(parent_line.saturating_sub(1)).unwrap_or(usize::MAX);
                    let parent = lines.get(parent_index).ok_or_else(|| {
                        ProvenanceError::new("ASSOCIATED_BINDING_PARENT_LINE_OUT_OF_BOUNDS")
                    })?;
                    let parent_lower = parent.to_ascii_lowercase();
                    if !parent_lower.contains(&source_basename.to_ascii_lowercase())
                        || !parent_lower.contains(&parent_method.to_ascii_lowercase())
                    {
                        return Err(ProvenanceError::record(
                            "ASSOCIATED_BINDING_PARENT_MISMATCH",
                            &entry.target_catalog_record_id,
                        ));
                    }
                }
                (None, Some(parent_method)) => {
                    if !lower.contains(&parent_method.to_ascii_lowercase())
                        || !record_has_method(record, parent_method)
                    {
                        return Err(ProvenanceError::record(
                            "ASSOCIATED_BINDING_PARENT_MISMATCH",
                            &entry.target_catalog_record_id,
                        ));
                    }
                }
                _ => {
                    return Err(ProvenanceError::record(
                        "ASSOCIATED_BINDING_SOURCE_MISMATCH",
                        &entry.target_catalog_record_id,
                    ));
                }
            }
        }
        if let Some(parent_method) = binding.parent_method_def
            && !record_has_method(record, parent_method)
        {
            return Err(ProvenanceError::record(
                "ASSOCIATED_BINDING_PARENT_MISSING",
                &entry.target_catalog_record_id,
            ));
        }
    }
    let source_sha = record_source_sha(record);
    let source_size = record_source_size(record);
    validate_completion_evidence(
        entry,
        CompletionProofSpec {
            source_path: &binding.source_path,
            source_sha: &source_sha,
            source_size,
            map_sha: &entry.evidence_basis.accepted_map_sha256,
            map_line: Some(binding.map_line),
            report: None,
            registry: false,
            v5: false,
            exact: binding.exact,
        },
        curated_root,
        research_root,
    )
}

fn validate_performance_entry(
    entry: &ErrataEntry,
    records: &[Value],
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let binding = PERFORMANCE_BINDINGS
        .iter()
        .find(|binding| {
            binding.record_id == entry.target_catalog_record_id
                && binding
                    .type_def
                    .eq_ignore_ascii_case(&entry.method_identity.type_def)
                && binding
                    .method_def
                    .eq_ignore_ascii_case(&entry.method_identity.method_def)
        })
        .ok_or_else(|| {
            ProvenanceError::record(
                "PERFORMANCE_BINDING_UNRESOLVED",
                &entry.target_catalog_record_id,
            )
        })?;
    let (record, _method) = validate_target_structure(entry, records)?;
    let source_path = record_source_path(record);
    let source_basename = source_path.rsplit('/').next().unwrap_or(&source_path);
    let proof = entry
        .association_proof
        .as_ref()
        .ok_or_else(|| ProvenanceError::new("ASSOCIATION_PROOF_MISSING"))?;
    if proof.report_refs.is_empty()
        || proof.report_refs.iter().any(|reference| {
            reference.path != binding.report_path
                || reference.artifact_sha256.as_deref() != Some(binding.report_sha256)
        })
    {
        return Err(ProvenanceError::record(
            "PERFORMANCE_BINDING_REPORT_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let report_bytes = check_file_digest(
        research_root,
        binding.report_path,
        binding.report_sha256,
        Some(binding.report_bytes),
        "PERFORMANCE_REPORT",
    )?;
    let text = String::from_utf8_lossy(&report_bytes);
    let lines: Vec<&str> = text.lines().collect();
    let target_md = entry.method_identity.method_def.to_ascii_lowercase();
    let mut target_found = false;
    for line_number in binding.target_lines {
        let index = usize::try_from(line_number.saturating_sub(1)).unwrap_or(usize::MAX);
        if lines
            .get(index)
            .is_some_and(|line| line.to_ascii_lowercase().contains(&target_md))
        {
            target_found = true;
        }
    }
    if !target_found {
        return Err(ProvenanceError::record(
            "PERFORMANCE_BINDING_TARGET_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    let source_token = binding.source_token.to_ascii_lowercase();
    let source_basename_lower = source_basename.to_ascii_lowercase();
    let mut source_found = false;
    for line_number in binding.source_lines {
        let index = usize::try_from(line_number.saturating_sub(1)).unwrap_or(usize::MAX);
        if lines.get(index).is_some_and(|line| {
            let lower = line.to_ascii_lowercase();
            lower.contains(&source_token) && lower.contains(&source_basename_lower)
        }) {
            source_found = true;
        }
    }
    if !source_found {
        return Err(ProvenanceError::record(
            "PERFORMANCE_BINDING_SOURCE_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    let imported = &entry.corrected_imported_source_anchor;
    if imported.precision != Precision::FileMethodAssociation
        || imported.start_line.is_some()
        || imported.end_line.is_some()
    {
        return Err(ProvenanceError::record(
            "PERFORMANCE_BINDING_PRECISION_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let source_sha = record_source_sha(record);
    let source_size = record_source_size(record);
    validate_completion_evidence(
        entry,
        CompletionProofSpec {
            source_path: &source_path,
            source_sha: &source_sha,
            source_size,
            map_sha: &entry.evidence_basis.accepted_map_sha256,
            map_line: None,
            report: Some((
                binding.report_path,
                binding.report_bytes,
                binding.report_sha256,
            )),
            registry: false,
            v5: false,
            exact: None,
        },
        curated_root,
        research_root,
    )
}

fn validate_completion_evidence(
    entry: &ErrataEntry,
    spec: CompletionProofSpec<'_>,
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let imported = &entry.corrected_imported_source_anchor;
    if imported.precision
        != if spec.exact.is_some() {
            Precision::ExactRange
        } else {
            Precision::FileMethodAssociation
        }
        || imported.start_line != spec.exact.map(|range| range.0)
        || imported.end_line != spec.exact.map(|range| range.1)
    {
        return Err(ProvenanceError::record(
            "COMPLETION_IMPORTED_PRECISION_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    for reference in &imported.support_refs {
        validate_completion_support_ref(entry, reference, &spec, curated_root, research_root)?;
    }
    let mut has_map = false;
    let mut has_semantic = false;
    let mut has_report = false;
    for anchor in &entry.corrected_evidence_source_anchors {
        if anchor.method_identity != entry.method_identity || anchor.root != RootKind::ResearchRoot
        {
            return Err(ProvenanceError::record(
                "COMPLETION_ANCHOR_IDENTITY_OR_ROOT_INVALID",
                &entry.target_catalog_record_id,
            ));
        }
        if anchor.path == METHOD_EVIDENCE_MAP {
            if spec.map_line.is_none()
                || anchor.evidence_class != EvidenceClass::AcceptedMethodMap
                || anchor.artifact_sha256 != spec.map_sha
                || anchor.precision != Precision::FileMethodAssociation
                || anchor.start_line.is_some()
                || anchor.end_line.is_some()
                || anchor.accepted_map_line != spec.map_line
            {
                return Err(ProvenanceError::record(
                    "COMPLETION_MAP_ANCHOR_MISMATCH",
                    &entry.target_catalog_record_id,
                ));
            }
            has_map = true;
        } else if anchor.path == spec.source_path {
            let class_ok = matches!(
                anchor.evidence_class,
                EvidenceClass::SemanticSource
                    | EvidenceClass::PerformanceSource
                    | EvidenceClass::LegacyDescriptor
            );
            let range_ok = match spec.exact {
                Some((start, end)) => {
                    anchor.precision == Precision::ExactRange
                        && anchor.start_line == Some(start)
                        && anchor.end_line == Some(end)
                }
                None => {
                    anchor.precision == Precision::FileMethodAssociation
                        && anchor.start_line.is_none()
                        && anchor.end_line.is_none()
                }
            };
            if !class_ok || !range_ok || anchor.artifact_sha256 != spec.source_sha {
                return Err(ProvenanceError::record(
                    "COMPLETION_SOURCE_ANCHOR_MISMATCH",
                    &entry.target_catalog_record_id,
                ));
            }
            let bytes = check_file_digest(
                research_root,
                &anchor.path,
                &anchor.artifact_sha256,
                Some(spec.source_size),
                "COMPLETION_SOURCE",
            )?;
            validate_actual_range(
                &bytes,
                anchor.start_line,
                anchor.end_line,
                &anchor.path,
                "COMPLETION_SOURCE",
            )?;
            has_semantic = true;
        } else if let Some((report_path, report_bytes, report_sha)) = spec.report {
            if anchor.path != report_path
                || anchor.evidence_class != EvidenceClass::Report
                || anchor.artifact_sha256 != report_sha
                || anchor.precision != Precision::FileMethodAssociation
                || anchor.start_line.is_some()
                || anchor.end_line.is_some()
            {
                return Err(ProvenanceError::record(
                    "COMPLETION_REPORT_ANCHOR_MISMATCH",
                    &entry.target_catalog_record_id,
                ));
            }
            check_file_digest(
                research_root,
                report_path,
                report_sha,
                Some(report_bytes),
                "COMPLETION_REPORT",
            )?;
            has_report = true;
        } else {
            return Err(ProvenanceError::record(
                "COMPLETION_EVIDENCE_ANCHOR_UNSUPPORTED",
                &entry.target_catalog_record_id,
            ));
        }
        for reference in &anchor.support_refs {
            validate_completion_support_ref(entry, reference, &spec, curated_root, research_root)?;
        }
    }
    for reference in &entry.evidence_basis.proof_refs {
        validate_completion_support_ref(entry, reference, &spec, curated_root, research_root)?;
    }
    if !has_semantic
        || (spec.map_line.is_some() && !has_map)
        || (spec.report.is_some() && !has_report)
    {
        return Err(ProvenanceError::record(
            "COMPLETION_REQUIRED_ANCHOR_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    Ok(())
}

fn validate_completion_support_ref(
    entry: &ErrataEntry,
    reference: &SupportRef,
    spec: &CompletionProofSpec<'_>,
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let path = reference.path.as_str();
    let (root, expected_sha, expected_size) = if path == spec.source_path {
        (research_root, spec.source_sha, Some(spec.source_size))
    } else if path == METHOD_EVIDENCE_MAP {
        (research_root, spec.map_sha, None)
    } else if let Some((report_path, report_bytes, report_sha)) = spec.report {
        if path == report_path {
            (research_root, report_sha, Some(report_bytes))
        } else if spec.registry && path == REGISTRY_REL {
            (curated_root, REGISTRY_SHA256, Some(REGISTRY_BYTES))
        } else if spec.v5 && path == V5_RECEIPT_REL {
            (curated_root, V5_RECEIPT_SHA256, Some(61_237))
        } else {
            return Err(ProvenanceError::record(
                "COMPLETION_SUPPORT_REFERENCE_UNVALIDATED",
                &entry.target_catalog_record_id,
            ));
        }
    } else if spec.registry && path == REGISTRY_REL {
        (curated_root, REGISTRY_SHA256, Some(REGISTRY_BYTES))
    } else if spec.v5 && path == V5_RECEIPT_REL {
        (curated_root, V5_RECEIPT_SHA256, Some(61_237))
    } else {
        return Err(ProvenanceError::record(
            "COMPLETION_SUPPORT_REFERENCE_UNVALIDATED",
            &entry.target_catalog_record_id,
        ));
    };
    if reference.artifact_sha256.as_deref() != Some(expected_sha) {
        return Err(ProvenanceError::record(
            "COMPLETION_SUPPORT_REFERENCE_SHA_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    if path != METHOD_EVIDENCE_MAP && path != REGISTRY_REL {
        check_file_digest(
            root,
            path,
            expected_sha,
            expected_size,
            "COMPLETION_SUPPORT",
        )?;
    }
    Ok(())
}

fn record_has_method(record: &Value, method_def: &str) -> bool {
    let Some(methods_value) = record.get("methods") else {
        return false;
    };
    let methods = if let Some(array) = methods_value.as_array() {
        array
    } else if let Some(array) = methods_value.get("current").and_then(Value::as_array) {
        array
    } else {
        return false;
    };
    methods
        .iter()
        .any(|method| method_def_token(method).eq_ignore_ascii_case(method_def))
}

pub fn apply_effective_view(
    root: &Path,
    manifest: &mut Manifest,
    active: Option<&ActiveReceipt>,
    trust: &ProvenanceTrust,
) -> Result<ProvenanceView, ProvenanceError> {
    let Some(active) = active else {
        return Ok(ProvenanceView::not_applicable());
    };
    if active.version != 6 {
        return Ok(ProvenanceView::not_applicable());
    }
    let recognized = recognized_errata_files(root)?;
    if recognized.is_empty() {
        mark_uncovered(manifest);
        return Ok(ProvenanceView::absent());
    }
    if recognized.len() != 1 || recognized[0] != ERRATA_FILE_NAME {
        return Err(ProvenanceError::new("COMPETING_ERRATA_VERSIONS"));
    }
    let bytes = checked_file_bytes(root, ERRATA_REL)?;
    let errata_sha256 = sha256_hex(&bytes);
    let Some(expected_sha256) = trust.public_errata_sha256.as_deref() else {
        return Err(ProvenanceError::new("PUBLIC_ERRATA_DIGEST_NOT_PINNED"));
    };
    if expected_sha256 != errata_sha256 {
        return Err(ProvenanceError::at(
            "PUBLIC_ERRATA_DIGEST_MISMATCH",
            ERRATA_REL,
        ));
    }
    let document = ErrataDocument::parse(&bytes)?;
    document.validate_structural(root)?;
    let activation_block = document
        .entries
        .iter()
        .find_map(|entry| validate_entry_activation_policy(entry).err())
        .map(|error| error.code);
    let corrected = document
        .entries
        .iter()
        .filter(|entry| entry.action == ErrataAction::Correct)
        .count() as u64;
    let confirmed = document
        .entries
        .iter()
        .filter(|entry| entry.action == ErrataAction::Confirm)
        .count() as u64;
    let refined = document
        .entries
        .iter()
        .filter(|entry| entry.action == ErrataAction::Refine)
        .count() as u64;
    let resolved_entries = document
        .entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.outcome,
                Outcome::ResolvedExact | Outcome::ResolvedConservative
            )
        })
        .count() as u64;
    if let Some(activation_block) = activation_block {
        mark_uncovered(manifest);
        return Ok(ProvenanceView {
            state: ProvenanceState::PendingReview,
            errata_sha256: Some(errata_sha256),
            entries: document.entries.len() as u64,
            resolved_entries,
            corrected,
            confirmed,
            refined,
            activation_block: Some(activation_block),
            document: Some(document),
        });
    }
    let (corrected, confirmed, refined) = apply_document(&document, manifest)?;
    mark_uncovered(manifest);
    Ok(ProvenanceView {
        state: ProvenanceState::StructurallyApplied,
        errata_sha256: Some(errata_sha256),
        entries: document.entries.len() as u64,
        resolved_entries,
        corrected,
        confirmed,
        refined,
        activation_block: None,
        document: Some(document),
    })
}

pub fn completion_binding_manifest() -> Value {
    serde_json::json!({
        "associated": ASSOCIATED_BINDINGS.iter().map(|binding| serde_json::json!({
            "fullkey": format!("{}|{}|{}", binding.record_id, binding.type_def, binding.method_def),
            "profile": "ASSOCIATED_GAMEPLAY_MAP",
            "map_line": binding.map_line,
            "source_path": binding.source_path,
            "precision": if binding.exact.is_some() { "EXACT_RANGE" } else { "FILE_METHOD_ASSOCIATION" },
            "start_line": binding.exact.map(|range| range.0),
            "end_line": binding.exact.map(|range| range.1),
            "required_tokens": binding.required_tokens,
            "parent_method_def": binding.parent_method_def,
            "parent_map_line": binding.parent_map_line,
            "context_lines": binding.context_lines,
            "basis": binding.basis,
            "derived_from": "ACCEPTED_METHOD_EVIDENCE_MAP_LINES"
        })).collect::<Vec<_>>(),
        "performance": PERFORMANCE_BINDINGS.iter().map(|binding| serde_json::json!({
            "fullkey": format!("{}|{}|{}", binding.record_id, binding.type_def, binding.method_def),
            "profile": "PERFORMANCE_EVIDENCE_ASSOCIATION",
            "report_path": binding.report_path,
            "report_bytes": binding.report_bytes,
            "report_sha256": binding.report_sha256,
            "target_lines": binding.target_lines,
            "source_lines": binding.source_lines,
            "source_token": binding.source_token,
            "basis": binding.basis,
            "derived_from": "ACCEPTED_PERFORMANCE_REPORT_LINES"
        })).collect::<Vec<_>>()
    })
}

fn json_value_with_identity(identity: &Value) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("method_identity".to_owned(), identity.clone());
    Value::Object(object)
}

fn recognized_errata_files(root: &Path) -> Result<Vec<String>, ProvenanceError> {
    match inspect_relative(root, ERRATA_DIR_REL) {
        Ok(SafePathState::Missing) => return Ok(Vec::new()),
        Ok(SafePathState::Directory) => {}
        Ok(SafePathState::File) => {
            return Err(ProvenanceError::at(
                "ERRATA_DIRECTORY_INVALID",
                ERRATA_DIR_REL,
            ));
        }
        Err(error) => return Err(ProvenanceError::at(&error.code, ERRATA_DIR_REL)),
    }
    let entries = fs::read_dir(root.join(ERRATA_DIR_REL))
        .map_err(|_| ProvenanceError::at("ERRATA_DIRECTORY_READ_FAILED", ERRATA_DIR_REL))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|_| ProvenanceError::at("ERRATA_DIRECTORY_READ_FAILED", ERRATA_DIR_REL))?;
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if let Some(version) = name
            .strip_prefix(ERRATA_FILE_PREFIX)
            .and_then(|rest| rest.strip_suffix(ERRATA_FILE_SUFFIX))
            && !version.is_empty()
            && version.bytes().all(|byte| byte.is_ascii_digit())
        {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

fn apply_document(
    document: &ErrataDocument,
    manifest: &mut Manifest,
) -> Result<(u64, u64, u64), ProvenanceError> {
    let mut corrected = 0_u64;
    let mut confirmed = 0_u64;
    let mut refined = 0_u64;
    for entry in &document.entries {
        let record = manifest
            .records
            .iter_mut()
            .find(|record| record.id == entry.target_catalog_record_id)
            .ok_or_else(|| {
                ProvenanceError::record("TARGET_RECORD_MISSING", &entry.target_catalog_record_id)
            })?;
        let method = record
            .methods
            .current
            .iter_mut()
            .find(|method| {
                method
                    .symbol_key
                    .type_def
                    .eq_ignore_ascii_case(&entry.method_identity.type_def)
                    && method
                        .symbol_key
                        .method_def
                        .eq_ignore_ascii_case(&entry.method_identity.method_def)
            })
            .ok_or_else(|| {
                ProvenanceError::record("TARGET_METHOD_MISSING", &entry.target_catalog_record_id)
            })?;
        if method.symbol_key != entry.method_identity {
            return Err(ProvenanceError::record(
                "EFFECTIVE_VIEW_IDENTITY_MISMATCH",
                &entry.target_catalog_record_id,
            ));
        }
        if method.source_provenance.is_some() {
            return Err(ProvenanceError::record(
                "COMPETING_ERRATA_TARGET_IN_EFFECTIVE_VIEW",
                &entry.target_catalog_record_id,
            ));
        }
        let mut legacy_original_references = Vec::new();
        for anchor in &entry.incorrect_anchors {
            let before = method.evidence_references.len();
            method.evidence_references.retain(|reference| {
                !(reference.path == anchor.path
                    && reference.locator == anchor.locator
                    && reference.artifact_sha256 == anchor.artifact_sha256)
            });
            if method.evidence_references.len() == before {
                return Err(ProvenanceError::record(
                    "INCORRECT_ANCHOR_NOT_IN_EFFECTIVE_VIEW",
                    &entry.target_catalog_record_id,
                ));
            }
            legacy_original_references.push(anchor.clone());
        }
        let status = match entry.action {
            ErrataAction::Correct => {
                corrected += 1;
                MethodProvenanceStatus::Corrected
            }
            ErrataAction::Confirm => {
                confirmed += 1;
                MethodProvenanceStatus::Confirmed
            }
            ErrataAction::Refine => {
                refined += 1;
                MethodProvenanceStatus::Refined
            }
        };
        method.source_provenance = Some(SourceProvenance::ProfileEntry {
            status,
            action: entry.action.clone(),
            imported: entry.corrected_imported_source_anchor.clone(),
            evidence_anchors: entry.corrected_evidence_source_anchors.clone(),
            legacy_original_references,
        });
    }
    Ok((corrected, confirmed, refined))
}

fn mark_uncovered(manifest: &mut Manifest) {
    for record in &mut manifest.records {
        for method in &mut record.methods.current {
            if method.source_provenance.is_none() {
                method.source_provenance = Some(SourceProvenance::NotValidated);
            }
        }
    }
}

fn canonical_target_key(entry: &ErrataEntry) -> (String, String, String, String, String) {
    (
        entry.target_catalog_record_id.trim().to_ascii_uppercase(),
        entry
            .method_identity
            .full_authority_sha256
            .to_ascii_lowercase(),
        entry.method_identity.mvid.to_ascii_lowercase(),
        entry.method_identity.type_def.to_ascii_uppercase(),
        entry.method_identity.method_def.to_ascii_uppercase(),
    )
}

fn validate_frozen_file(file: &FrozenFile) -> Result<(), ProvenanceError> {
    validate_relative_metadata_path(&file.path)?;
    if file.bytes == 0 {
        return Err(ProvenanceError::new("FROZEN_FILE_BYTES_INVALID"));
    }
    validate_lower_sha(&file.sha256, "FROZEN_FILE_SHA_INVALID")
}

fn validate_entry_shape(entry: &ErrataEntry) -> Result<(), ProvenanceError> {
    if entry.target_catalog_record_id.trim().is_empty() {
        return Err(ProvenanceError::new("TARGET_RECORD_ID_EMPTY"));
    }
    validate_lower_sha(&entry.expected_original_record_hash, "RECORD_HASH_INVALID")?;
    validate_method_identity(&entry.method_identity)?;
    if entry.action == ErrataAction::Correct && entry.incorrect_anchors.is_empty() {
        return Err(ProvenanceError::new("CORRECT_REQUIRES_INCORRECT_ANCHOR"));
    }
    let mut seen = std::collections::BTreeSet::new();
    for anchor in &entry.incorrect_anchors {
        validate_relative_metadata_path(&anchor.path)?;
        if anchor.locator.trim().is_empty() {
            return Err(ProvenanceError::new("INCORRECT_ANCHOR_LOCATOR_EMPTY"));
        }
        let key = (
            anchor.path.clone(),
            anchor.locator.clone(),
            anchor.artifact_sha256.clone(),
        );
        if !seen.insert(key) {
            return Err(ProvenanceError::new("DUPLICATE_INCORRECT_ANCHOR"));
        }
        if let Some(hash) = &anchor.artifact_sha256 {
            validate_lower_sha(hash, "INCORRECT_ANCHOR_SHA_INVALID")?;
        }
    }
    validate_imported_anchor(&entry.corrected_imported_source_anchor)?;
    if entry.profile != ProvenanceProfile::Unresolved
        && entry.corrected_evidence_source_anchors.is_empty()
    {
        return Err(ProvenanceError::new("EVIDENCE_ANCHORS_EMPTY"));
    }
    for anchor in &entry.corrected_evidence_source_anchors {
        validate_evidence_anchor(anchor)?;
    }
    if entry.evidence_basis.accepted_map_path != METHOD_EVIDENCE_MAP {
        return Err(ProvenanceError::new("EVIDENCE_BASIS_MAP_PATH_INVALID"));
    }
    validate_relative_metadata_path(&entry.evidence_basis.accepted_map_path)?;
    validate_lower_sha(
        &entry.evidence_basis.accepted_map_sha256,
        "EVIDENCE_BASIS_MAP_SHA_INVALID",
    )?;
    let basis_nonzero_required = entry.profile == ProvenanceProfile::OrdinaryGameplayMap;
    if (basis_nonzero_required
        && (entry.evidence_basis.accepted_map_row == 0
            || entry.evidence_basis.accepted_map_line == 0))
        || entry.evidence_basis.basis.trim().is_empty()
    {
        return Err(ProvenanceError::new("EVIDENCE_BASIS_INVALID"));
    }
    for reference in &entry.evidence_basis.proof_refs {
        validate_relative_metadata_path(&reference.path)?;
        if reference.locator.trim().is_empty() {
            return Err(ProvenanceError::new("PROOF_REFERENCE_INVALID"));
        }
        if let Some(hash) = &reference.artifact_sha256 {
            validate_lower_sha(hash, "PROOF_REFERENCE_SHA_INVALID")?;
        }
    }
    if let Some(path) = &entry.target_catalog_path {
        validate_relative_metadata_path(path)?;
    }
    if let Some(proof) = &entry.association_proof {
        if let Some(line) = proof.map_line
            && line == 0
        {
            return Err(ProvenanceError::new("ASSOCIATION_MAP_LINE_INVALID"));
        }
        if let Some(literal) = &proof.literal_method_def {
            if !valid_hex_word(literal) {
                return Err(ProvenanceError::new("ASSOCIATION_LITERAL_METHOD_INVALID"));
            }
        }
        for reference in proof.report_refs.iter().chain(proof.raw_refs.iter()) {
            validate_relative_metadata_path(&reference.path)?;
            if reference.locator.trim().is_empty() {
                return Err(ProvenanceError::new("ASSOCIATION_REFERENCE_INVALID"));
            }
            if let Some(hash) = &reference.artifact_sha256 {
                validate_lower_sha(hash, "ASSOCIATION_REFERENCE_SHA_INVALID")?;
            }
        }
        if proof.basis.trim().is_empty() {
            return Err(ProvenanceError::new("ASSOCIATION_BASIS_INVALID"));
        }
    }
    match entry.profile {
        ProvenanceProfile::OrdinaryGameplayMap => {}
        ProvenanceProfile::RegistryFileAssociation => {
            if entry.target_catalog_path.is_some() || entry.association_proof.is_some() {
                return Err(ProvenanceError::new("REGISTRY_PROFILE_SHAPE_INVALID"));
            }
            if entry.corrected_imported_source_anchor.precision == Precision::ExactRange {
                return Err(ProvenanceError::new(
                    "REGISTRY_PROFILE_CANNOT_CLAIM_EXACT_RANGE",
                ));
            }
        }
        ProvenanceProfile::AssociatedGameplayMap => {
            let Some(proof) = &entry.association_proof else {
                return Err(ProvenanceError::new("ASSOCIATION_PROOF_MISSING"));
            };
            if proof.map_line.is_none() || proof.literal_method_def.is_none() {
                return Err(ProvenanceError::new("ASSOCIATION_MAP_PROOF_INCOMPLETE"));
            }
            if entry.target_catalog_path.is_some() {
                return Err(ProvenanceError::new("ASSOCIATION_TARGET_CATALOG_INVALID"));
            }
        }
        ProvenanceProfile::PerformanceEvidenceAssociation => {
            let Some(proof) = &entry.association_proof else {
                return Err(ProvenanceError::new("ASSOCIATION_PROOF_MISSING"));
            };
            if proof.report_refs.is_empty() {
                return Err(ProvenanceError::new("PERFORMANCE_REPORT_PROOF_MISSING"));
            }
            if entry.corrected_imported_source_anchor.precision == Precision::ExactRange {
                return Err(ProvenanceError::new(
                    "PERFORMANCE_PROFILE_CANNOT_CLAIM_EXACT_RANGE",
                ));
            }
            if entry.target_catalog_path.is_some() {
                return Err(ProvenanceError::new("PERFORMANCE_TARGET_CATALOG_INVALID"));
            }
        }
        ProvenanceProfile::LegacyV5SourceAssociation
        | ProvenanceProfile::MigrationRecordAssociation => {
            if entry.target_catalog_path.is_none() {
                return Err(ProvenanceError::new("LEGACY_TARGET_CATALOG_MISSING"));
            }
            if entry.association_proof.is_some() {
                return Err(ProvenanceError::new("LEGACY_ASSOCIATION_PROOF_INVALID"));
            }
            if entry.corrected_imported_source_anchor.precision != Precision::FileMethodAssociation
            {
                return Err(ProvenanceError::new("LEGACY_PRECISION_MUST_BE_FILE"));
            }
        }
        ProvenanceProfile::Unresolved => {
            if entry.corrected_imported_source_anchor.precision != Precision::Unresolved {
                return Err(ProvenanceError::new("UNRESOLVED_PROFILE_PRECISION_INVALID"));
            }
        }
    }
    match entry.outcome {
        Outcome::ResolvedExact => {
            if entry.corrected_imported_source_anchor.precision != Precision::ExactRange {
                return Err(ProvenanceError::new(
                    "EXACT_OUTCOME_REQUIRES_EXACT_IMPORTED_ANCHOR",
                ));
            }
        }
        Outcome::ResolvedConservative => match entry.corrected_imported_source_anchor.precision {
            Precision::ExactRange => {
                return Err(ProvenanceError::new(
                    "CONSERVATIVE_OUTCOME_HAS_EXACT_IMPORTED_ANCHOR",
                ));
            }
            Precision::Unresolved => {
                return Err(ProvenanceError::new(
                    "CONSERVATIVE_OUTCOME_HAS_UNRESOLVED_ANCHOR",
                ));
            }
            Precision::FileMethodAssociation => {}
        },
        Outcome::UnresolvedInsufficientEvidence => {
            if entry.corrected_imported_source_anchor.precision != Precision::Unresolved {
                return Err(ProvenanceError::new(
                    "UNRESOLVED_OUTCOME_REQUIRES_UNRESOLVED_IMPORTED_ANCHOR",
                ));
            }
        }
        Outcome::ConflictingEvidence => {}
    }
    Ok(())
}

fn validate_imported_anchor(anchor: &ImportedAnchor) -> Result<(), ProvenanceError> {
    if anchor.root != RootKind::CuratedRoot {
        return Err(ProvenanceError::new("IMPORTED_ANCHOR_ROOT_INVALID"));
    }
    validate_relative_metadata_path(&anchor.path)?;
    validate_lower_sha(&anchor.whole_file_sha256, "IMPORTED_ANCHOR_SHA_INVALID")?;
    validate_method_identity(&anchor.method_identity)?;
    validate_precision_range(anchor.precision.clone(), anchor.start_line, anchor.end_line)?;
    if anchor.scope.trim().is_empty() {
        return Err(ProvenanceError::new("IMPORTED_ANCHOR_SCOPE_EMPTY"));
    }
    if anchor.support_refs.is_empty() {
        return Err(ProvenanceError::new("IMPORTED_ANCHOR_SUPPORT_EMPTY"));
    }
    for reference in &anchor.support_refs {
        validate_relative_metadata_path(&reference.path)?;
        if reference.locator.trim().is_empty() {
            return Err(ProvenanceError::new("SUPPORT_REFERENCE_INVALID"));
        }
        if let Some(hash) = &reference.artifact_sha256 {
            validate_lower_sha(hash, "SUPPORT_REFERENCE_SHA_INVALID")?;
        }
    }
    Ok(())
}

fn validate_evidence_anchor(anchor: &EvidenceAnchor) -> Result<(), ProvenanceError> {
    if anchor.root != RootKind::ResearchRoot {
        return Err(ProvenanceError::new("EVIDENCE_ANCHOR_ROOT_INVALID"));
    }
    validate_relative_metadata_path(&anchor.path)?;
    validate_lower_sha(&anchor.artifact_sha256, "EVIDENCE_ANCHOR_SHA_INVALID")?;
    validate_method_identity(&anchor.method_identity)?;
    validate_precision_range(anchor.precision.clone(), anchor.start_line, anchor.end_line)?;
    if anchor.basis.trim().is_empty() {
        return Err(ProvenanceError::new("EVIDENCE_ANCHOR_BASIS_EMPTY"));
    }
    for reference in &anchor.support_refs {
        validate_relative_metadata_path(&reference.path)?;
        if reference.locator.trim().is_empty() {
            return Err(ProvenanceError::new("SUPPORT_REFERENCE_INVALID"));
        }
        if let Some(hash) = &reference.artifact_sha256 {
            validate_lower_sha(hash, "SUPPORT_REFERENCE_SHA_INVALID")?;
        }
    }
    Ok(())
}

fn validate_precision_range(
    precision: Precision,
    start: Option<u64>,
    end: Option<u64>,
) -> Result<(), ProvenanceError> {
    match precision {
        Precision::ExactRange => {
            let (start, end) = start
                .zip(end)
                .ok_or_else(|| ProvenanceError::new("EXACT_RANGE_MISSING_ENDPOINTS"))?;
            if start == 0 || end < start {
                return Err(ProvenanceError::new("EXACT_RANGE_INVALID"));
            }
        }
        Precision::FileMethodAssociation | Precision::Unresolved => {
            if start.is_some() || end.is_some() {
                return Err(ProvenanceError::new("NONEXACT_RANGE_HAS_ENDPOINTS"));
            }
        }
    }
    Ok(())
}

fn validate_method_identity(identity: &MethodIdentity) -> Result<(), ProvenanceError> {
    validate_lower_sha(
        &identity.full_authority_sha256,
        "METHOD_AUTHORITY_SHA_INVALID",
    )?;
    validate_mvid(&identity.mvid)?;
    if !valid_hex_word(&identity.type_def) || !valid_hex_word(&identity.method_def) {
        return Err(ProvenanceError::new("METHOD_KEY_INVALID"));
    }
    Ok(())
}

fn validate_lower_sha(value: &str, code: &str) -> Result<(), ProvenanceError> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ProvenanceError::new(code));
    }
    Ok(())
}

fn validate_mvid(value: &str) -> Result<(), ProvenanceError> {
    let bytes = value.as_bytes();
    if bytes.len() != 36
        || ![8, 13, 18, 23].iter().all(|index| bytes[*index] == b'-')
        || bytes
            .iter()
            .enumerate()
            .filter(|(index, _)| ![8, 13, 18, 23].contains(index))
            .any(|(_, byte)| !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase())
    {
        return Err(ProvenanceError::new("METHOD_MVID_INVALID"));
    }
    Ok(())
}

fn checked_frozen_file(root: &Path, frozen: &FrozenFile) -> Result<Vec<u8>, ProvenanceError> {
    let bytes = checked_file_bytes(root, &frozen.path)?;
    if bytes.len() as u64 != frozen.bytes {
        return Err(ProvenanceError::at(
            "FROZEN_FILE_SIZE_MISMATCH",
            &frozen.path,
        ));
    }
    if sha256_hex(&bytes) != frozen.sha256 {
        return Err(ProvenanceError::at(
            "FROZEN_FILE_HASH_MISMATCH",
            &frozen.path,
        ));
    }
    Ok(bytes)
}

fn validate_entry_against_catalog(
    entry: &ErrataEntry,
    records: &[Value],
    accepted_map: &AcceptedMethodMap,
    map_bytes: &[u8],
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let (record, method) = target_record_and_method(entry, records)?;
    let actual_hash = canonical_record_hash(record)?;
    if actual_hash != entry.expected_original_record_hash {
        return Err(ProvenanceError::record(
            "STALE_TARGET_RECORD_HASH",
            &entry.target_catalog_record_id,
        ));
    }
    let record_identity =
        method_identity_from_record(record, method, &entry.target_catalog_record_id)?;
    if record_identity != entry.method_identity {
        return Err(ProvenanceError::record(
            "TARGET_METHOD_IDENTITY_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    verify_incorrect_anchors(entry, method)?;
    let row = accepted_map
        .row_for(
            &entry.method_identity.type_def,
            &entry.method_identity.method_def,
        )
        .ok_or_else(|| {
            ProvenanceError::record("NO_ACCEPTED_SEMANTIC_ROW", &entry.target_catalog_record_id)
        })?;
    if entry.evidence_basis.accepted_map_row != row.row
        || entry.evidence_basis.accepted_map_line != row.map_line
        || entry.evidence_basis.accepted_map_sha256 != sha256_hex(map_bytes)
    {
        return Err(ProvenanceError::record(
            "EVIDENCE_BASIS_STALE",
            &entry.target_catalog_record_id,
        ));
    }
    let expected_precision = if row.start_line.is_some() && row.end_line.is_some() {
        Precision::ExactRange
    } else {
        Precision::FileMethodAssociation
    };
    validate_record_source_binding(entry, record, row, curated_root, research_root)?;
    validate_support_refs(
        research_root,
        &entry.corrected_imported_source_anchor.support_refs,
        row,
        "IMPORTED_SUPPORT",
    )?;
    validate_support_refs(
        research_root,
        &entry.evidence_basis.proof_refs,
        row,
        "PROOF_SUPPORT",
    )?;
    let imported = &entry.corrected_imported_source_anchor;
    if imported.method_identity != entry.method_identity {
        return Err(ProvenanceError::record(
            "IMPORTED_METHOD_IDENTITY_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let imported_bytes = checked_file_bytes(curated_root, &imported.path)?;
    if sha256_hex(&imported_bytes) != imported.whole_file_sha256 {
        return Err(ProvenanceError::at(
            "IMPORTED_ANCHOR_HASH_MISMATCH",
            &imported.path,
        ));
    }
    if value_string(record, "destination_path") != imported.path {
        return Err(ProvenanceError::record(
            "IMPORTED_PATH_NOT_RECORD_DESTINATION",
            &entry.target_catalog_record_id,
        ));
    }
    if imported.precision != expected_precision {
        return Err(ProvenanceError::record(
            "IMPORTED_PRECISION_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    validate_actual_range(
        &imported_bytes,
        imported.start_line,
        imported.end_line,
        &imported.path,
        "IMPORTED_ANCHOR",
    )?;
    if let (Some(start), Some(end)) = (row.start_line, row.end_line) {
        if imported.start_line != Some(start) || imported.end_line != Some(end) {
            return Err(ProvenanceError::record(
                "IMPORTED_RANGE_DOES_NOT_MATCH_ACCEPTED_MAP",
                &entry.target_catalog_record_id,
            ));
        }
    }
    verify_evidence_anchors(entry, row, map_bytes, research_root)?;
    match entry.outcome {
        Outcome::ResolvedExact => {
            if expected_precision != Precision::ExactRange {
                return Err(ProvenanceError::record(
                    "EXACT_OUTCOME_WITHOUT_EXACT_MAP",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        Outcome::ResolvedConservative => {
            if imported.precision == Precision::ExactRange {
                return Err(ProvenanceError::record(
                    "CONSERVATIVE_OUTCOME_HAS_EXACT_IMPORTED_ANCHOR",
                    &entry.target_catalog_record_id,
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_record_source_binding(
    entry: &ErrataEntry,
    record: &Value,
    row: &AcceptedMapRow,
    curated_root: &Path,
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let source_path = value_string(record, "source_path");
    let destination_path = value_string(record, "destination_path");
    let source_sha = value_string(record, "source_sha256");
    if source_path.is_empty()
        || destination_path.is_empty()
        || !source_sha_is_valid(&source_sha)
        || row.semantic_path != source_path
        || entry.corrected_imported_source_anchor.path != destination_path
    {
        return Err(ProvenanceError::record(
            "RECORD_SOURCE_BINDING_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    let source_bytes = checked_file_bytes(research_root, &source_path)?;
    let destination_bytes = checked_file_bytes(curated_root, &destination_path)?;
    if source_bytes != destination_bytes
        || sha256_hex(&source_bytes) != source_sha
        || sha256_hex(&destination_bytes) != source_sha
    {
        return Err(ProvenanceError::record(
            "IMPORTED_SOURCE_BYTES_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    if let Some(size) = record.get("source_size").and_then(Value::as_u64)
        && size != source_bytes.len() as u64
    {
        return Err(ProvenanceError::record(
            "RECORD_SOURCE_SIZE_MISMATCH",
            &entry.target_catalog_record_id,
        ));
    }
    Ok(())
}

fn source_sha_is_valid(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_support_refs(
    research_root: &Path,
    references: &[SupportRef],
    row: &AcceptedMapRow,
    prefix: &str,
) -> Result<(), ProvenanceError> {
    for reference in references {
        validate_relative_metadata_path(&reference.path)?;
        if reference.locator.trim().is_empty() {
            return Err(ProvenanceError::new("SUPPORT_REFERENCE_LOCATOR_EMPTY"));
        }
        let allowed = reference.path == METHOD_EVIDENCE_MAP
            || reference.path == row.semantic_path
            || row
                .canonical_path
                .as_deref()
                .is_some_and(|path| path == reference.path);
        if !allowed {
            return Err(ProvenanceError::at(
                &format!("{prefix}_REFERENCE_NOT_CHECKED"),
                &reference.path,
            ));
        }
        let bytes = checked_file_bytes(research_root, &reference.path)?;
        if let Some(expected) = &reference.artifact_sha256
            && sha256_hex(&bytes) != *expected
        {
            return Err(ProvenanceError::at(
                &format!("{prefix}_HASH_MISMATCH"),
                &reference.path,
            ));
        }
    }
    Ok(())
}

fn verify_incorrect_anchors(entry: &ErrataEntry, method: &Value) -> Result<(), ProvenanceError> {
    let references = method
        .get("evidence_references")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            ProvenanceError::record("TARGET_REFERENCES_INVALID", &entry.target_catalog_record_id)
        })?;
    for anchor in &entry.incorrect_anchors {
        let found = references.iter().any(|reference| {
            value_string(reference, "path") == anchor.path
                && value_string(reference, "locator") == anchor.locator
                && value_option_string(reference, "artifact_sha256") == anchor.artifact_sha256
        });
        if !found {
            return Err(ProvenanceError::record(
                "INCORRECT_ANCHOR_NOT_IN_CATALOG",
                &entry.target_catalog_record_id,
            ));
        }
    }
    Ok(())
}

fn verify_evidence_anchors(
    entry: &ErrataEntry,
    row: &AcceptedMapRow,
    map_bytes: &[u8],
    research_root: &Path,
) -> Result<(), ProvenanceError> {
    let map_sha = sha256_hex(map_bytes);
    let mut map_anchor_found = false;
    let mut semantic_anchor_found = false;
    for anchor in &entry.corrected_evidence_source_anchors {
        validate_support_refs(research_root, &anchor.support_refs, row, "EVIDENCE_SUPPORT")?;
        if anchor.method_identity != entry.method_identity {
            return Err(ProvenanceError::record(
                "EVIDENCE_METHOD_IDENTITY_MISMATCH",
                &entry.target_catalog_record_id,
            ));
        }
        let bytes = checked_file_bytes(research_root, &anchor.path)?;
        if sha256_hex(&bytes) != anchor.artifact_sha256 {
            return Err(ProvenanceError::at(
                "EVIDENCE_ANCHOR_HASH_MISMATCH",
                &anchor.path,
            ));
        }
        validate_actual_range(
            &bytes,
            anchor.start_line,
            anchor.end_line,
            &anchor.path,
            "EVIDENCE_ANCHOR",
        )?;
        match anchor.evidence_class {
            EvidenceClass::AcceptedMethodMap => {
                if anchor.path != METHOD_EVIDENCE_MAP
                    || anchor.artifact_sha256 != map_sha
                    || anchor.accepted_map_row != Some(row.row)
                    || anchor.accepted_map_line != Some(row.map_line)
                    || anchor.precision != Precision::FileMethodAssociation
                {
                    return Err(ProvenanceError::record(
                        "ACCEPTED_MAP_ANCHOR_INVALID",
                        &entry.target_catalog_record_id,
                    ));
                }
                map_anchor_found = true;
            }
            EvidenceClass::SemanticSource => {
                if anchor.path != row.semantic_path {
                    return Err(ProvenanceError::record(
                        "SEMANTIC_ANCHOR_PATH_MISMATCH",
                        &entry.target_catalog_record_id,
                    ));
                }
                if row.start_line.is_some() {
                    if anchor.precision != Precision::ExactRange
                        || anchor.start_line != row.start_line
                        || anchor.end_line != row.end_line
                    {
                        return Err(ProvenanceError::record(
                            "SEMANTIC_ANCHOR_RANGE_MISMATCH",
                            &entry.target_catalog_record_id,
                        ));
                    }
                } else if anchor.precision != Precision::FileMethodAssociation
                    || anchor.start_line.is_some()
                    || anchor.end_line.is_some()
                {
                    return Err(ProvenanceError::record(
                        "SEMANTIC_ANCHOR_FALSE_EXACT_CLAIM",
                        &entry.target_catalog_record_id,
                    ));
                }
                semantic_anchor_found = true;
            }
            EvidenceClass::CanonicalSource => {
                let Some(canonical_path) = row.canonical_path.as_deref() else {
                    return Err(ProvenanceError::record(
                        "CANONICAL_ANCHOR_NOT_ACCEPTED",
                        &entry.target_catalog_record_id,
                    ));
                };
                if anchor.path != canonical_path {
                    return Err(ProvenanceError::record(
                        "CANONICAL_ANCHOR_PATH_MISMATCH",
                        &entry.target_catalog_record_id,
                    ));
                }
                if anchor.precision == Precision::ExactRange {
                    if row.canonical_start_line.is_none()
                        || row.canonical_end_line.is_none()
                        || anchor.start_line != row.canonical_start_line
                        || anchor.end_line != row.canonical_end_line
                    {
                        return Err(ProvenanceError::record(
                            "CANONICAL_ANCHOR_RANGE_MISMATCH",
                            &entry.target_catalog_record_id,
                        ));
                    }
                } else if anchor.start_line.is_some() || anchor.end_line.is_some() {
                    return Err(ProvenanceError::record(
                        "CANONICAL_ANCHOR_FALSE_EXACT_CLAIM",
                        &entry.target_catalog_record_id,
                    ));
                }
            }
            EvidenceClass::RawMethod
            | EvidenceClass::Report
            | EvidenceClass::AssociatedSource
            | EvidenceClass::PerformanceSource
            | EvidenceClass::LegacyDescriptor => {
                return Err(ProvenanceError::record(
                    "UNVERIFIED_EVIDENCE_ANCHOR",
                    &entry.target_catalog_record_id,
                ));
            }
            EvidenceClass::Unresolved => {
                return Err(ProvenanceError::record(
                    "EVIDENCE_ANCHOR_UNRESOLVED",
                    &entry.target_catalog_record_id,
                ));
            }
        }
    }
    if !map_anchor_found || !semantic_anchor_found {
        return Err(ProvenanceError::record(
            "MANDATORY_EVIDENCE_ANCHOR_MISSING",
            &entry.target_catalog_record_id,
        ));
    }
    Ok(())
}

fn validate_actual_range(
    bytes: &[u8],
    start: Option<u64>,
    end: Option<u64>,
    path: &str,
    prefix: &str,
) -> Result<(), ProvenanceError> {
    let Some(start) = start else {
        return Ok(());
    };
    let Some(end) = end else {
        return Ok(());
    };
    let text = std::str::from_utf8(bytes)
        .map_err(|_| ProvenanceError::at(&format!("{prefix}_NOT_UTF8"), path))?;
    let line_count = text.lines().count() as u64;
    if start == 0 || end < start || end > line_count {
        return Err(ProvenanceError::at(
            &format!("{prefix}_RANGE_OUT_OF_BOUNDS"),
            path,
        ));
    }
    Ok(())
}

fn method_identity_from_record(
    record: &Value,
    method: &Value,
    record_id: &str,
) -> Result<MethodIdentity, ProvenanceError> {
    let mut authority = value_string(record, "authority_sha256");
    if authority.is_empty() {
        authority = value_string(record, "authority_binary_sha256");
    }
    let mvid = value_string(record, "authority_mvid");
    let type_def = method_type_token(method);
    let method_def = method_def_token(method);
    if authority.is_empty() || mvid.is_empty() || type_def.is_empty() || method_def.is_empty() {
        return Err(ProvenanceError::record(
            "TARGET_AUTHORITY_INVALID",
            record_id,
        ));
    }
    Ok(MethodIdentity {
        full_authority_sha256: authority,
        mvid,
        type_def,
        method_def,
    })
}

fn value_string(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn value_option_string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_owned)
}
