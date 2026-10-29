// Build: BLD-WUWA-0001
// Unit: RCN-R1DB-STREAMING-0001
// Material claims: CLM-R1DB-0001, CLM-R1DB-0002, CLM-R1DB-0003, CLM-R1DB-0004, CLM-R1DB-0005, CLM-R1DB-0006, CLM-R1DB-0007, CLM-R1DB-0008, CLM-R1DB-0009, CLM-R1DB-0010, CLM-R1DB-0011, CLM-R1DB-0012, CLM-R1DB-0013, CLM-R1DB-0014, CLM-R1DB-0015, CLM-R1DB-0016, CLM-R1DB-0017, CLM-R1DB-0018, CLM-R1DB-0019, CLM-R1DB-0020, CLM-R1DB-0021, CLM-R1DB-0022
// Classification: CLS-R1DB-STREAMING-0001
// Review: REV-R1DB-0001
/*
 * R1d-B reconstruction: streaming-completion bridge (binding->native dispatch IDENTITY edge).
 *
 * This header records, for each transition-facing predicate, ONLY the accepted R1d-A
 * dispatch identity (double-underscore binding name + KR-WorldPartition dispatch-table
 * slot + static address). It does NOT reconstruct native bodies, ABI, signatures, or the
 * internal streaming algorithm — those remain UNKNOWN [CLM-R1DB-0019/0020 UNKNOWN/OBSERVATION].
 * All 26/26 target bodies were adjudicated UNRESOLVED_NONPLAIN by accepted R1d-A evidence;
 * no new body-visibility research was performed by R1d-B.
 *
 * RECON_META:
 *   CLM-R1DB-0012 [OBSERVED/OBSERVATION]  — TGT-R1DA-0009 IsStreamingCompletedForLayers2 is the sole
 *       completion predicate of the reconstructed phase-11/13 poll; role OBSERVED_POLL.
 *   CLM-R1DB-0013 [OBSERVED/OBSERVATION]  — TGT-R1DA-0012 EnableStreamingSource is the activation call
 *       at each poll setup; role OBSERVED_OPTIONAL_GATE.
 *   CLM-R1DB-0014 [OBSERVED/OBSERVATION]  — TGT-R1DA-0026 D_AddOrRemoveRenderAssetsQueryViewInfo is the
 *       phase-11 entry view registration; role OBSERVED_CONFIGURATION_QUERY.
 *   CLM-R1DB-0015 [OBSERVED/OBSERVATION]  — TGT-R1DA-0010 LogStreamingStuckInfo2 is the stall-diagnostics
 *       poll; role OBSERVED_POLL (diagnostic).
 *   CLM-R1DB-0016 [OBSERVED/OBSERVATION]  — TGT-R1DA-0016 IsStreamingCompletedWithPhysicsReady observed
 *       outside the transition path (CharacterActorComponent); recorded, not admitted.
 *   CLM-R1DB-0017 [OBSERVED/OBSERVATION]  — TGT-R1DA-0013 IsStreamingSourceEnabled observed outside the
 *       transition path (SceneItemPortalComponent); recorded, not admitted.
 *   CLM-R1DB-0018 [OBSERVED/OBSERVATION]  — zero-callsite predicates confirmed by open-paren static_search
 *       (bare IsStreamingCompleted( count 0 in all five polled files; TGT-0008/0014/0015/0017/0019/0021/0024
 *       have no authorized-corpus call sites); no aliasing asserted.
 *   CLM-R1DB-0019 [UNKNOWN/OBSERVATION]  — native semantics of transition-facing predicates UNKNOWN.
 *   CLM-R1DB-0020 [UNKNOWN/OBSERVATION]  — native ABI of transition-facing predicates UNKNOWN.
 *   CLM-R1DB-0021 [UNKNOWN/OBSERVATION]  — KURO_-prefixed completion-query relation to the transition UNKNOWN.
 *   CLM-R1DB-0022 [OBSERVED/OBSERVATION]  — GameModeController.CheckStreamingCompleted and
 *       TeleportStreamingHelper.CheckStreamingCompleted are distinct JS orchestrators over the same
 *       binding surface; never collapsed.
 *   CLM-R1DB-0005 [OBSERVED/OBSERVATION]  — ZQs poll engine shape (timer, completion, progress callback,
 *       early-abort callback) as statically observed.
 *   CLM-R1DB-0006 [OBSERVED/OBSERVATION]  — two-phase predicate use of IsStreamingCompletedForLayers2
 *       (phase A arg4=true/arg7=false; phase B arg4=false/arg7=true; same binding both phases).
 *   CLM-R1DB-0007 [OBSERVED/OBSERVATION]  — cadences: CHECK_STREAMING_INTERVAL=100 ms;
 *       LOG_STREAMING_STUCK_INTERVAL=60000 ms. No other cadence values exist in the reconstructed segment.
 *   CLM-R1DB-0008 [OBSERVED/OBSERVATION]  — PrintWorldPartitionDebugInfo -> LogStreamingStuckInfo2 stall path.
 *   CLM-R1DB-0009 [OBSERVED/OBSERVATION]  — voxel barrier flow incl. no-WP fast path.
 *   CLM-R1DB-0010 [OBSERVED/OBSERVATION]  — CheckStreamingCompleted barrier flow incl. dataLayers build,
 *       physics-included poll, no-WP fallback.
 *   CLM-R1DB-0011 [OBSERVED/OBSERVATION]  — bEnableWorldPartition gate is a reflected JS-read property,
 *       not one of the 26 bindings.
 *   CLM-R1DB-0001 [OBSERVED/OBSERVATION]  — phase-11 entry sequence bytes.
 *   CLM-R1DB-0002 [OBSERVED/OBSERVATION]  — phase-11 exit / 12 / 13 entry sequence bytes.
 *   CLM-R1DB-0003 [OBSERVED/OBSERVATION]  — phase-13 exit / 14 entry / StopIndependentStreaming boundary.
 *   CLM-R1DB-0004 [OBSERVED/OBSERVATION]  — ELoadingPhase progress constants 40/50/75/80/90/100.
 */

// ---------------------------------------------------------------------------
// Dispatch identity records (accepted R1d-A; nothing below is an implementation)
// ---------------------------------------------------------------------------

/* Sole completion predicate of the transition poll.
 * Identity: UnrealEngine.UWorldPartitionStreamingSourceComponent::IsStreamingCompletedForLayers2
 * (accepted observed binding name form: ...::__IsStreamingCompletedForLayers2_Internal)
 * Dispatch table: KR-WorldPartition streaming-source component table; static address 0x164f3bb30.
 * role_status OBSERVED_POLL [CLM-R1DB-0012 OBSERVED/OBSERVATION].
 * Native ABI: UNKNOWN. Native semantics: UNKNOWN [CLM-R1DB-0019/0020 UNKNOWN/OBSERVATION]. */
struct WorldPartitionStreamingSourceComponent_IsStreamingCompletedForLayers2_Binding;

/* Activation gate at each poll setup.
 * Identity: UWorldPartitionStreamingSourceComponent::EnableStreamingSource; static address 0x164f3baa0.
 * role_status OBSERVED_OPTIONAL_GATE [CLM-R1DB-0013 OBSERVED/OBSERVATION].
 * Native ABI: UNKNOWN. Native semantics: UNKNOWN. */
struct WorldPartitionStreamingSourceComponent_EnableStreamingSource_Binding;

/* Phase-11 entry view registration (promise variant).
 * Identity: UWorldPartitionSubsystem::D_AddOrRemoveRenderAssetsQueryViewInfo; static address 0x164f3b6e0.
 * role_status OBSERVED_CONFIGURATION_QUERY [CLM-R1DB-0014 OBSERVED/OBSERVATION].
 * Native ABI: UNKNOWN. Native semantics: UNKNOWN. */
struct WorldPartitionSubsystem_D_AddOrRemoveRenderAssetsQueryViewInfo_Binding;

/* Stall diagnostics inside the poll.
 * Identity: UWorldPartitionStreamingSourceComponent::LogStreamingStuckInfo2; static address 0x164f3bc40.
 * role_status OBSERVED_POLL (diagnostic only) [CLM-R1DB-0015 OBSERVED/OBSERVATION].
 * Native ABI: UNKNOWN. Native semantics: UNKNOWN. */
struct WorldPartitionStreamingSourceComponent_LogStreamingStuckInfo2_Binding;

// ---------------------------------------------------------------------------
// Outside-path observations (recorded; not admitted to the transition predicate set)
// ---------------------------------------------------------------------------

/* TGT-R1DA-0016 UWorldPartitionSubsystem::IsStreamingCompletedWithPhysicsReady; 0x164f3b940.
 * Observed in CharacterActorComponent creature runtime wait — outside the transition path.
 * [CLM-R1DB-0016 OBSERVED/OBSERVATION] Native ABI/semantics UNKNOWN. */
struct WorldPartitionSubsystem_IsStreamingCompletedWithPhysicsReady_Binding;

/* TGT-R1DA-0013 UWorldPartitionStreamingSourceComponent::IsStreamingSourceEnabled; 0x164f3bba0.
 * Observed in SceneItemPortalComponent portal flow — outside the transition path.
 * [CLM-R1DB-0017 OBSERVED/OBSERVATION] Native ABI/semantics UNKNOWN. */
struct WorldPartitionStreamingSourceComponent_IsStreamingSourceEnabled_Binding;

/* Zero-callsite targets (no authorized-corpus JS call sites; relationship UNKNOWN; no aliasing):
 * TGT-R1DA-0008 SC::KURO_IsStreamingCompleted 0x164f3bc00
 * TGT-R1DA-0014 Sub::KURO_IsStreamingCompleted 0x164f3bad0
 * TGT-R1DA-0015 Sub::IsStreamingCompleted 0x164f3b8e0
 * TGT-R1DA-0017 Sub::IsStreamingEnable 0x164f3b980
 * TGT-R1DA-0019 Sub::MarkActorStreamingCellsDirty 0x164f3bb00
 * TGT-R1DA-0021 Sub::IsAllStreamingCompleted 0x164f3b7c0
 * TGT-R1DA-0024 Sub::IsRenderAssetsStreamingCompletedForQuerySources 0x164f3b830
 * [CLM-R1DB-0018 OBSERVED/OBSERVATION] [CLM-R1DB-0021 UNKNOWN/OBSERVATION] */

// ---------------------------------------------------------------------------
// Scope fence
// ---------------------------------------------------------------------------
//
// R1d-B reconstructs how the loading transition ASKS whether streaming is ready
// (JS control flow + binding identity edges). The binding->native relationship below
// each struct above is an IDENTITY claim (name + table + static address) accepted from
// R1d-A; it is NOT an implementation claim. The internal streaming algorithm remains
// unknown.
//
// classification: CLM-R1DB-0099 + CLS-R1DB-STREAMING-0001 [UNKNOWN/OBSERVATION]
