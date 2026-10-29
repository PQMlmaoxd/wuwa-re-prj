// Build: BLD-WUWA-0001
// Unit: RCN-R1DA-BINDINGMAP-0001
// Material claims: CLM-R1DA-0001, CLM-R1DA-0002, CLM-R1DA-0003, CLM-R1DA-0004, CLM-R1DA-0005, CLM-R1DA-0006, CLM-R1DA-0007, CLM-R1DA-0008, CLM-R1DA-0009, CLM-R1DA-0010, CLM-R1DA-0011
// Classification: CLS-R1DA-BINDINGMAP-0001
// Review: REV-R1DA-0001
//
// R1d-A — JS/Puerts → Native Binding Map (WorldPartition transition-facing surface)
//
// This file is a READABLE SEMANTIC SURFACE ONLY. It is NOT a compilable SDK and NOT a
// behavioral reconstruction. Every struct below is a forward declaration with
// property-level evidence annotations. Native ABIs and native function semantics are
// UNKNOWN [CLM-R1DA-0009 UNKNOWN/OBSERVATION] [CLM-R1DA-0010 UNKNOWN/OBSERVATION].
// All mapped code bodies are UNRESOLVED_NONPLAIN at rest
// [CLM-R1DA-0006 OBSERVED/OBSERVATION]; "encrypted/packed" is an INFERRED transform,
// not an observed fact.
//
// Evidence chain per identity:
//   accepted JS call claim → observed binding name (exe bytes)
//   → observed dispatch-table entry (code VA) → native body UNRESOLVED_NONPLAIN
//
// Prior-work correction (formal contradiction record):
//   [CLM-R1DA-0008 INFERRED/INFERENCE] earlier notes (wuwa-re/notes/IDA_ANCHORS.md,
//   WUWA-EXE-0003 era) misread alphabetically-sorted [code_ptr, name_ptr] tables as
//   [name_ptr, code_ptr], assigning each name the code pointer of the NEXT entry.
//   All code addresses in this file are from the byte-verified decode
//   [CLM-R1DA-0004 OBSERVED/OBSERVATION] [CLM-R1DA-0005 OBSERVED/OBSERVATION].

//
// Per-claim annotations (canonical label/status):
//   CLM-R1DA-0001 [OBSERVED/OBSERVATION]  — KR full binding strings byte-verified
//   CLM-R1DA-0002 [OBSERVED/OBSERVATION]  — LoadMapNotify stream-level binding strings
//   CLM-R1DA-0003 [OBSERVED/OBSERVATION]  — Tier-2 binding strings (SC + WPS)
//   CLM-R1DA-0004 [OBSERVED/OBSERVATION]  — 5 dispatch table regions decoded
//   CLM-R1DA-0005 [OBSERVED/OBSERVATION]  — code VA per target from table slots
//   CLM-R1DA-0006 [OBSERVED/OBSERVATION]  — all mapped bodies UNRESOLVED_NONPLAIN
//   CLM-R1DA-0007 [INFERRED/INFERENCE]    — Tier-2 admission from structural JS call sites
//   CLM-R1DA-0008 [INFERRED/INFERENCE]    — prior IDA_ANCHORS off-by-one superseded
//   CLM-R1DA-0009 [UNKNOWN/OBSERVATION]    — ABI unknown by scope
//   CLM-R1DA-0010 [UNKNOWN/OBSERVATION]    — native semantics unknown by scope
//   CLM-R1DA-0011 [INFERRED/INFERENCE]    — independence groups declared

#pragma once

// ============================================================================
// Tier 1 — directly reached from accepted R1c outgoing boundaries
// (RCN-R1C-DATALAYER-0001 / REV-R1C-0001)
// ============================================================================

// --- UKuroRenderingRuntimeBPPluginBPLibrary (KR) ----------------------------
// Full binding strings observed: 'UnrealEngine.UKuroRenderingRuntimeBPPluginBPLibrary::__<M>_Internal'
// [CLM-R1DA-0001 OBSERVED/OBSERVATION]
// Dispatch table @ 0x166f9d000, layout [code, name], alphabetically sorted.
struct KR_SetWorldPartitionDataLayerState_Binding;   // code VA 0x160e76d60
struct KR_SetWorldPartitionDataLayerState2_Binding;  // code VA 0x160e76dc0
struct KR_IsWorldPartitionDataLayerEnable_Binding;   // code VA 0x160e79180
// NOTE: JS call sites pass a numeric state code r = t ? 2 : (a ? 1 : 0) for State2;
// the meaning of 2/1/0 is UNKNOWN. Twin index table @ 0x166fa1900 references the
// same name with a different code pointer (0x160e8ff40) — second registration,
// relationship between the two registrations UNKNOWN.

// --- ULoadMapNotify (LMN) — reflected property 场景加载通知器 ----------------
// Full binding strings observed for the 4 stream-level methods + 7 related
// [CLM-R1DA-0002 OBSERVED/OBSERVATION]
// Dispatch table @ 0x167000710, layout [code, name], 11 entries, NULL-terminated.
struct LMN_BindLoadStreamLevel_Binding;   // code VA 0x160f26720
struct LMN_BindUnLoadStreamLevel_Binding; // code VA 0x160f26750
struct LMN_LoadStreamLevel_Binding;       // code VA 0x160f267b0
struct LMN_UnloadStreamLevel_Binding;     // code VA 0x160f26840
// Also in table (role support only): BindBeginLoadMap 0x160f26660,
// BindBeginTravelLoadMap 0x160f26690, BindEndLoadMap 0x160f266c0,
// BindEndLoadTransitionMap 0x160f266f0, Clear 0x160f26780,
// OnLoadStreamLevel 0x160f267e0, OnUnLoadStreamLevel 0x160f26810.
// JS side binds delegates ZEr/xBn via toManualReleaseDelegate (accepted R1a/R1c
// observation); delegate marshalling internals UNKNOWN.

// ============================================================================
// Tier 2 — transition completion surface
// [CLM-R1DA-0007 INFERRED/INFERENCE] admission by structural JS call-site
// evidence (2-phase non-physics/physics polling, stall-dump pattern), not name
// similarity. Names with zero JS occurrences carry role_status UNKNOWN.
// ============================================================================

// --- UWorldPartitionStreamingSourceComponent (SC) ---------------------------
// Table-X @ 0x167f9fa20, layout [code, name]. Region-3 re-registration table
// @ 0x167fa09b0 ([name, code] inverted) carries DIFFERENT code pointers for the
// same names — relationship between registrations UNKNOWN.
struct SC_EnableStreamingSource_Binding;        // 0x164f3baa0 (no full binding string; identity via table + JS) role_status OBSERVED
struct SC_DisableStreamingSource_Binding;       // 0x164f3b9b0 (no full binding string) role_status UNKNOWN (zero JS occurrences; binding identity/dispatch/address remain OBSERVED)
struct SC_IsStreamingCompletedForLayers2_Binding; // 0x164f3bb30 (R3: 0x164f3e110) role_status OBSERVED (gmc/DLT/IOS 2-phase polling)
struct SC_IsStreamingSourceEnabled_Binding;     // 0x164f3bba0 (R3: 0x164f3e4e0) role_status OBSERVED (SPC portal check)
struct SC_KURO_IsStreamingCompleted_Binding;    // 0x164f3bc00 (R3: 0x164f3e500) role_status UNKNOWN (zero JS occurrences)
struct SC_LogStreamingStuckInfo2_Binding;       // 0x164f3bc40 (R3: 0x164f3e8a0) role_status OBSERVED (gmc/IOS stall-dump)

// --- UWorldPartitionSubsystem (WPS) ----------------------------------------
// Run-1 table @ 0x167f9f670, layout [code, name], 13 entries. Region-3 carries a
// second, differently-addressed registration set for most names.
struct WPS_AddOrRemoveRenderAssetsQueryViewInfo_Binding;       // 0x164f3b6b0 (R3 0x164f3d520)
struct WPS_D_AddOrRemoveRenderAssetsQueryViewInfo_Binding;     // 0x164f3b6e0 (R3 0x164f3d630) role_status OBSERVED (gmc/IOS promise-variant)
struct WPS_FlushUnloadingStreamingCells_Binding;              // 0x164f3b710 (R3 0x164f3d780) role_status OBSERVED (TSH teleport flush)
struct WPS_IsAllStreamingCompleted_Binding;                    // 0x164f3b7c0 (R3 0x164f3d850) role_status UNKNOWN (zero JS occurrences)
struct WPS_IsRenderAssetsStreamingCompleted_Binding;          // 0x164f3b800 (R3 0x164f3d8f0) role_status OBSERVED (gmc/DLT/IOS render poll)
struct WPS_IsRenderAssetsStreamingCompletedForQuerySources_Binding; // 0x164f3b830 (R3 0x164f3dc20) role_status UNKNOWN (zero JS occurrences; binding identity/dispatch/address remain OBSERVED)
struct WPS_IsStreamingCompleted_Binding;                       // 0x164f3b8e0 (R3 0x164f3de40) role_status UNKNOWN (no dedicated JS site mapped)
struct WPS_IsStreamingCompletedWithPhysicsReady_Binding;      // 0x164f3b940 (R3 0x164f3e370) role_status OBSERVED (CAC physics wait)
struct WPS_IsStreamingEnable_Binding;                          // 0x164f3b980 (R3 0x161931590 — DIFFERENT module region) role_status UNKNOWN (zero JS occurrences)
struct WPS_KURO_IsStreamingCompleted_Binding;                  // 0x164f3bad0 (R3 0x164f3e600) role_status UNKNOWN (zero JS occurrences)
struct WPS_MarkActorStreamingCellsDirty_Binding;               // 0x164f3bb00 (R3 0x164f3ea70) role_status UNKNOWN (zero JS occurrences)
struct WPS_ReloadActorStreamingCells_Binding;                  // 0x164f3bb70 (R3 0x164f3eb60) role_status OBSERVED (CC OnLeaveLevel)
struct WPS_SetStreamingEnable_Binding;                         // 0x164f3bbd0 (R3 0x164f3eb80) role_status OBSERVED (TSH)

// ============================================================================
// Scope fence
// ============================================================================
// NOT represented here (out of R1d-A scope): native body semantics, ABI,
// streaming-cell algorithms, WorldPartition internals, FastGeo, resource
// systems, runtime behavior. Independence groups for this phase's evidence:
// local-pak3-js, local-exe-analysis-copy [CLM-R1DA-0011 INFERRED/INFERENCE].
// classification: CLM-R1DA-0099 + CLS-R1DA-BINDINGMAP-0001 [UNKNOWN/OBSERVATION]
