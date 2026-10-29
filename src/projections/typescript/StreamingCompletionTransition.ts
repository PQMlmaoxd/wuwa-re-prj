// Build: BLD-WUWA-0001
// Unit: RCN-R1DB-STREAMING-0001
// Material claims: CLM-R1DB-0001, CLM-R1DB-0002, CLM-R1DB-0003, CLM-R1DB-0004, CLM-R1DB-0005, CLM-R1DB-0006, CLM-R1DB-0007, CLM-R1DB-0008, CLM-R1DB-0009, CLM-R1DB-0010, CLM-R1DB-0011, CLM-R1DB-0012, CLM-R1DB-0013, CLM-R1DB-0014, CLM-R1DB-0015, CLM-R1DB-0016, CLM-R1DB-0017, CLM-R1DB-0018, CLM-R1DB-0019, CLM-R1DB-0020, CLM-R1DB-0021, CLM-R1DB-0022
// Classification: CLS-R1DB-STREAMING-0001
// Review: REV-R1DB-0001
/*
 * R1d-B reconstruction: streaming-completion transition boundary (JS control-flow layer).
 *
 * Scope fence (R1d-B): this file reconstructs ONLY the JS control-flow and JS->binding
 * identity edges of the phase-11 (CheckVoxelStreaming) through phase-14 (CheckStreaming)
 * loading-transition segment, as statically observed in the authorized 25-file JS corpus
 * (GameModeController.js and corroborating files). Native bodies, ABI, and the internal
 * streaming algorithm are UNKNOWN and are NOT reconstructed here (see
 * StreamingCompletionBridge.hpp for the dispatch identity edge only).
 *
 * Method identities below are ONLY the ones the JS source actually defines:
 *   GameModeController.CheckVoxelStreamingCompleted / CheckStreamingCompleted / ZQs /
 *   PrintWorldPartitionDebugInfo. The PerfSight tag "Load.CheckVoxelStreaming" is an
 * *instrumentation tag*, never a JS method identity, and is not used as one here.
 *
 * RECON_META:
 *   CLM-R1DB-0001 [OBSERVED/OBSERVATION]  — phase-11 entry sequence: `_.LoadingPhase=11` at byte 22161 in
 *       GameModeController.js, ManuallyClearStreamingPool() then this.AddOrRemoveRenderAssetsQueryViewInfo(
 *       _.BornLocation, ResourceSystem.WAIT_RENDER_ASSET_DURATION), then await this.CheckVoxelStreamingCompleted(
 *       CHECK_VOXEL_STREAMING_END_PROGRESS - SETDATALAYER_AND_LOADSUBLEVEL_END_PROGRESS,
 *       CHECK_VOXEL_STREAMING_END_PROGRESS).
 *   CLM-R1DB-0002 [OBSERVED/OBSERVATION]  — phase-11 exit / phase-12 / phase-13 entry: EndExtTag("Load.CheckVoxelStreaming"),
 *       SetProgress(CHECK_VOXEL_STREAMING_END_PROGRESS), `_.LoadingPhase=12`, CheckVoxelStreamingSourceProfiler.Stop(),
 *       (conditional PreAwakeEntityDuringLoad/PreAwakeEntitiesFromPending), RegisterDataLayerChange(),
 *       CheckStreamingSourceProfiler.Restart(), BeginExtTag("Load.CheckStreaming"), `_.LoadingPhase=13`,
 *       await this.CheckStreamingCompleted(CHECK_STREAMING_END_PROGRESS - CHECK_VOXEL_STREAMING_END_PROGRESS,
 *       CHECK_STREAMING_END_PROGRESS).
 *   CLM-R1DB-0003 [OBSERVED/OBSERVATION]  — phase-13 exit / phase-14 entry: EndExtTag("Load.CheckStreaming"),
 *       ForceGarbageCollection(!1), ManuallyResetStreamingPool() (JS WorldController method, not a native binding),
 *       SetProgress(CHECK_STREAMING_END_PROGRESS), `_.LoadingPhase=14`, StopIndependentStreaming().
 *   CLM-R1DB-0004 [OBSERVED/OBSERVATION]  — ELoadingPhase.js progress constants:
 *       SETDATALAYER_AND_LOADSUBLEVEL_END_PROGRESS=40, CHECK_VOXEL_STREAMING_END_PROGRESS=50,
 *       CHECK_STREAMING_END_PROGRESS=75, CREATE_ENTITY_END_PROGRESS=80, WORLD_DONE_END_PROGRESS=90,
 *       LOAD_FINISHED_PROGRESS=100 (byte block @451..885).
 *   CLM-R1DB-0005 [OBSERVED/OBSERVATION]  — poll engine shape: GameModeController.ZQs(r, t, n, l, _ = false, i)
 *       registers a TimerSystem.Forever callback at CHECK_STREAMING_INTERVAL; on completion it does
 *       TimerSystem.Remove(M) then t.SetResult(true); progress callback l(cellProgress-unref) feeds
 *       LoadingController.AddProgress(Math.min(e*o, o), a); optional 6th-arg early-abort callback i.
 *   CLM-R1DB-0006 [OBSERVED/OBSERVATION]  — two-phase predicate use: ZQs phase A polls
 *       r.IsStreamingCompletedForLayers2(n, false, ResourceSystem.STREAMING_SOURCE_RADIUS, true, cellProgress, false)
 *       (arg4=true, arg7=false); once phase A completes and _ === true, phase B polls
 *       r.IsStreamingCompletedForLayers2(n, false, ResourceSystem.STREAMING_SOURCE_RADIUS, false, cellProgress, true)
 *       (arg4=false, arg7=true). Same binding both phases; boolean args differ.
 *   CLM-R1DB-0007 [OBSERVED/OBSERVATION]  — poll cadence: CHECK_STREAMING_INTERVAL=100 (ResourceSystem.js);
 *       stuck diagnostics every LOG_STREAMING_STUCK_INTERVAL=6e4 ms (GameModeController.js). No other cadences
 *       invented; all values are statically visible constants.
 *   CLM-R1DB-0008 [OBSERVED/OBSERVATION]  — stuck diagnostics binding: PrintWorldPartitionDebugInfo calls
 *       e.LogStreamingStuckInfo2(o, a, r, t) on the SourceComponent (TGT-R1DA-0010) when the stuck accumulator
 *       reaches LOG_STREAMING_STUCK_INTERVAL inside ZQs.
 *   CLM-R1DB-0009 [OBSERVED/OBSERVATION]  — voxel barrier flow: CheckVoxelStreamingCompleted reads
 *       K2_GetWorldSettings().bEnableWorldPartition; if false → VoxelStreamingCompleted.SetResult(true) +
 *       AddProgress(o, a) immediately (no native streaming waits); if true → resolve SourceComponent on the
 *       VoxelStreamingSource actor, EnableStreamingSource(), poll via ZQs, await r.Promise (r = t.VoxelStreamingCompleted).
 *   CLM-R1DB-0010 [OBSERVED/OBSERVATION]  — CheckStreamingCompleted barrier flow: builds dataLayers array from
 *       EnvironmentInfoUpdate(t, true, true) or (PIE) WaitHLODResInLoading config + WorldDefine.dataLayerRuntimeHLOD,
 *       plus AppendAllBaseDatalayers(l); EnableStreamingSource(); ZQs poll with 5th arg true (physics phase
 *       included); await t.Promise (t = e.StreamingCompleted). No-WP fallback: AddProgress(a, r) +
 *       StreamingCompleted.SetResult(true).
 *   CLM-R1DB-0011 [OBSERVED/OBSERVATION]  — the phase-11/13 entry gate is the reflected JS-read property
 *       bEnableWorldPartition (K2_GetWorldSettings), NOT one of the 26 accepted native bindings; the barrier
 *       fast-paths exist only in JS.
 *   CLM-R1DB-0012 [OBSERVED/OBSERVATION]  — TGT-R1DA-0009 IsStreamingCompletedForLayers2 is the sole completion
 *       predicate of the reconstructed transition poll; role OBSERVED_POLL.
 *   CLM-R1DB-0013 [OBSERVED/OBSERVATION]  — TGT-R1DA-0012 EnableStreamingSource is an activation call at each
 *       poll setup; role OBSERVED_OPTIONAL_GATE.
 *   CLM-R1DB-0014 [OBSERVED/OBSERVATION]  — TGT-R1DA-0026 D_AddOrRemoveRenderAssetsQueryViewInfo is the phase-11
 *       entry view registration (JS calls this.AddOrRemoveRenderAssetsQueryViewInfo which dispatches to the D_
 *       promise variant, WAIT_RENDER_ASSET_DURATION=42); role OBSERVED_CONFIGURATION_QUERY.
 *   CLM-R1DB-0015 [OBSERVED/OBSERVATION]  — TGT-R1DA-0010 LogStreamingStuckInfo2 is the stall-diagnostics poll
 *       inside ZQs; role OBSERVED_POLL (diagnostic only, never a completion predicate).
 *   CLM-R1DB-0016 [OBSERVED/OBSERVATION]  — TGT-R1DA-0016 IsStreamingCompletedWithPhysicsReady is observed in
 *       CharacterActorComponent (creature runtime physics wait), OUTSIDE the reconstructed transition path;
 *       relationship recorded only, never aliased.
 *   CLM-R1DB-0017 [OBSERVED/OBSERVATION]  — TGT-R1DA-0013 IsStreamingSourceEnabled is observed in
 *       SceneItemPortalComponent (portal flow), OUTSIDE the reconstructed transition path; relationship recorded only.
 *   CLM-R1DB-0018 [OBSERVED/OBSERVATION]  — zero-callsite predicates: bare IsStreamingCompleted( has 0 matches in
 *       all five polled files (open-paren static_search); TGT-0008/0014/0015/0017/0019/0021/0024 confirmed
 *       zero JS call sites in the authorized corpus; no aliasing of similarly named bindings is asserted.
 *   CLM-R1DB-0019 [UNKNOWN/OBSERVATION]  — native semantics of the transition-facing predicates are UNKNOWN
 *       (policy-derived; native-body reconstruction excluded by R1d-B authorization).
 *   CLM-R1DB-0020 [UNKNOWN/OBSERVATION]  — native ABI of the transition-facing predicates is UNKNOWN
 *       (policy-derived; native-body reconstruction excluded by R1d-B authorization).
 *   CLM-R1DB-0021 [UNKNOWN/OBSERVATION]  — relationship of the KURO_-prefixed completion queries to the
 *       reconstructed transition is UNKNOWN (zero JS call sites in authorized corpus; no aliasing asserted).
 *   CLM-R1DB-0022 [OBSERVED/OBSERVATION]  — distinct JS orchestrators: GameModeController.CheckStreamingCompleted
 *       (byte 70823) and TeleportStreamingHelper.CheckStreamingCompleted (byte 3249) are two different JS
 *       methods that both drive the same binding surface; they are never collapsed.
 */

// ---------------------------------------------------------------------------
// JS control-flow layer (observed byte-exact in GameModeController.js)
// ---------------------------------------------------------------------------

/** Phase-11/13 poll engine, as observed at GameModeController.js byte 68223 [CLM-R1DB-0005 OBSERVED/OBSERVATION]. */
declare const ZQs_poll_engine: {
  signature: "static ZQs(r, t, n, l, _ = false, i)";
  timer: "TimerSystem.Forever(callback, ResourceSystem.CHECK_STREAMING_INTERVAL)";
  completion: "TimerSystem.Remove(M); t.SetResult(true)";
  progressCallback: "l(cellProgress-unref) -> LoadingController.AddProgress(Math.min(e*o, o), a)";
  earlyAbort: "optional 6th-arg callback i; if i() truthy, poll stops without asserting streaming state";
  stuckAccumulator: "g += CHECK_STREAMING_INTERVAL each tick; at LOG_STREAMING_STUCK_INTERVAL emit stuck log";
};

/** Phase-A/phase-B predicate pattern inside ZQs [CLM-R1DB-0006 OBSERVED/OBSERVATION]. */
declare const ZQs_two_phase_predicate: {
  phaseA_nonPhysics: "r.IsStreamingCompletedForLayers2(n, false, STREAMING_SOURCE_RADIUS, true, cellProgress, false)";
  phaseB_physics: "r.IsStreamingCompletedForLayers2(n, false, STREAMING_SOURCE_RADIUS, false, cellProgress, true)";
  phaseB_gate: "entered only when ZQs 5th arg _ === true (voxel: false; phase-13 CheckStreaming: true)";
};

/** Phase-11 entry sequence [CLM-R1DB-0001 OBSERVED/OBSERVATION]. */
declare const phase11_entry: {
  loadingPhaseAssign: "_.LoadingPhase = 11";
  clearPool: "ControllerHolder.WorldController.ManuallyClearStreamingPool()";
  viewRegistration: "this.AddOrRemoveRenderAssetsQueryViewInfo(_.BornLocation, ResourceSystem.WAIT_RENDER_ASSET_DURATION)";
  await: "await this.CheckVoxelStreamingCompleted(CHECK_VOXEL_STREAMING_END_PROGRESS - SETDATALAYER_AND_LOADSUBLEVEL_END_PROGRESS, CHECK_VOXEL_STREAMING_END_PROGRESS)";
};

/** Phase-11 exit through phase-13 entry [CLM-R1DB-0002 OBSERVED/OBSERVATION]. */
declare const phase11_exit_12_13: {
  gcAndProgress: "WorldController.ForceGarbageCollection(!1); LoadingController.SetProgress(CHECK_VOXEL_STREAMING_END_PROGRESS)";
  phase12: "_.LoadingPhase = 12; EndExtTag('Load.CheckVoxelStreaming'); _.CheckVoxelStreamingSourceProfiler.Stop()";
  preAwake: "conditional PreAwakeEntityDuringLoad -> PreAwakeEntitiesFromPending(Proto_SceneInit)";
  dataLayerChange: "_.RegisterDataLayerChange()";
  phase13: "_.CheckStreamingSourceProfiler.Restart(); BeginExtTag('Load.CheckStreaming'); _.LoadingPhase = 13";
  await: "await this.CheckStreamingCompleted(CHECK_STREAMING_END_PROGRESS - CHECK_VOXEL_STREAMING_END_PROGRESS, CHECK_STREAMING_END_PROGRESS)";
};

/** Phase-13 exit / phase-14 entry / hard-stop boundary [CLM-R1DB-0003 OBSERVED/OBSERVATION]. */
declare const phase13_exit_14: {
  gc: "WorldController.ForceGarbageCollection(!1)",
  resetPool: "_.ManuallyResetStreamingPool()  // JS WorldController method; NOT a native binding",
  progressAndPhase: "LoadingController.SetProgress(CHECK_STREAMING_END_PROGRESS); _.LoadingPhase = 14",
  endTag: "EndExtTag('Load.CheckStreaming'); _.CheckStreamingSourceProfiler.Stop()",
  stopIndependent: "_.StopIndependentStreaming()",
  hardStop: "R1d-B hard stop: first directly observable post-streaming boundary is phase 14 -> 15 (CreateEntities NOT reconstructed here)",
};

/** ELoadingPhase.js progress constants [CLM-R1DB-0004 OBSERVED/OBSERVATION]. */
export const progressConstants = {
  SETDATALAYER_AND_LOADSUBLEVEL_END_PROGRESS: 40,
  CHECK_VOXEL_STREAMING_END_PROGRESS: 50,
  CHECK_STREAMING_END_PROGRESS: 75,
  CREATE_ENTITY_END_PROGRESS: 80,
  WORLD_DONE_END_PROGRESS: 90,
  LOAD_FINISHED_PROGRESS: 100,
} as const;

/** ResourceSystem.js poll/stuck cadences [CLM-R1DB-0007 OBSERVED/OBSERVATION]. */
export const pollCadence = {
  CHECK_STREAMING_INTERVAL: 100,
  LOG_STREAMING_STUCK_INTERVAL: 60000,
} as const;

// ---------------------------------------------------------------------------
// Barrier objects (JS-side promise-like completion barriers)
// ---------------------------------------------------------------------------

/**
 * CheckVoxelStreamingCompleted flow [CLM-R1DB-0009 OBSERVED/OBSERVATION].
 * The bEnableWorldPartition gate is a reflected JS-read property, not a binding [CLM-R1DB-0011 OBSERVED/OBSERVATION].
 */
export declare function CheckVoxelStreamingCompletedFlow(): {
  gate: "GlobalData.World.GetWorld().K2_GetWorldSettings().bEnableWorldPartition";
  gateFalse: "t.UseWorldPartition = false; t.VoxelStreamingCompleted.SetResult(true); LoadingController.AddProgress(o, a)";
  gateTrue: "resolve SourceComponent on VoxelStreamingSource; EnableStreamingSource(); ZQs(e, r, undefined, progress, _=false); await r.Promise";
  barrier: "GameModeModel.VoxelStreamingCompleted (JS promise-like barrier; r.Promise awaited)";
};

/**
 * CheckStreamingCompleted flow [CLM-R1DB-0010 OBSERVED/OBSERVATION].
 */
export declare function CheckStreamingCompletedFlow(): {
  gate: "e.UseWorldPartition (set by the earlier voxel gate)";
  dataLayers:
    "EnvironmentInfoUpdate(t, true, true) result OR (PIE) WaitHLODResInLoading config + WorldDefine.dataLayerRuntimeHLOD; plus AppendAllBaseDatalayers(l)";
  poll: "EnableStreamingSource(); ZQs(n, t, l, progress, _=true)  // physics phase included; await t.Promise";
  fallback: "no-WP tail: AddProgress(a, r); StreamingCompleted.SetResult(true)";
  barrier: "GameModeModel.StreamingCompleted (JS promise-like barrier; t.Promise awaited)";
};

// ---------------------------------------------------------------------------
// Predicate identity edges (JS -> binding identity only; never native implementation)
// ---------------------------------------------------------------------------

/**
 * TGT-R1DA-0009 UWorldPartitionStreamingSourceComponent::IsStreamingCompletedForLayers2
 * [CLM-R1DB-0012 OBSERVED/OBSERVATION] — sole completion predicate, role OBSERVED_POLL.
 * Dispatch identity (KR-WorldPartition table slot, static address 0x164f3bb30) is accepted
 * from R1d-A; native ABI and semantics remain UNKNOWN [CLM-R1DB-0019/0020 UNKNOWN/OBSERVATION].
 */
export declare const completionPredicate: {
  binding: "UnrealEngine.UWorldPartitionStreamingSourceComponent::IsStreamingCompletedForLayers2";
  dispatch: "accepted R1d-A dispatch entry, static address 0x164f3bb30";
  nativeAbi: "UNKNOWN";
  nativeSemantics: "UNKNOWN";
};

/**
 * TGT-R1DA-0012 EnableStreamingSource [CLM-R1DB-0013 OBSERVED/OBSERVATION] — OBSERVED_OPTIONAL_GATE.
 * TGT-R1DA-0026 D_AddOrRemoveRenderAssetsQueryViewInfo [CLM-R1DB-0014 OBSERVED/OBSERVATION] — OBSERVED_CONFIGURATION_QUERY.
 * TGT-R1DA-0010 LogStreamingStuckInfo2 [CLM-R1DB-0015 OBSERVED/OBSERVATION] — OBSERVED_POLL diagnostic.
 */
export declare const transitionPredicateSet: {
  completion: "TGT-R1DA-0009 IsStreamingCompletedForLayers2";
  activation: "TGT-R1DA-0012 EnableStreamingSource";
  viewRegistration: "TGT-R1DA-0026 D_AddOrRemoveRenderAssetsQueryViewInfo";
  stallDiagnostics: "TGT-R1DA-0010 LogStreamingStuckInfo2";
  nativeAbi: "UNKNOWN";
  nativeSemantics: "UNKNOWN";
};

/**
 * Outside-path observations recorded without re-adjudication:
 * TGT-R1DA-0016 IsStreamingCompletedWithPhysicsReady — CharacterActorComponent creature wait
 * [CLM-R1DB-0016 OBSERVED/OBSERVATION]; TGT-R1DA-0013 IsStreamingSourceEnabled — portal flow
 * [CLM-R1DB-0017 OBSERVED/OBSERVATION]; zero-callsite targets confirmed by open-paren
 * static_search [CLM-R1DB-0018 OBSERVED/OBSERVATION]; KURO_-prefixed query relation UNKNOWN
 * [CLM-R1DB-0021 UNKNOWN/OBSERVATION]; distinct same-named JS orchestrators never collapsed
 * [CLM-R1DB-0022 OBSERVED/OBSERVATION].
 */
export declare const outsidePathRelationships: {
  physicsReady: "TGT-R1DA-0016 — CharacterActorComponent runtime wait (outside transition)";
  sourceEnabled: "TGT-R1DA-0013 — portal flow (outside transition)";
  zeroCallsite: "TGT-0008/0014/0015/0017/0019/0021/0024 — no authorized-corpus call sites";
  kuroRelation: "UNKNOWN";
  distinctOrchestrators: "GameModeController.CheckStreamingCompleted vs TeleportStreamingHelper.CheckStreamingCompleted — distinct JS methods";
};

// Scope fence: R1d-B reconstructs the JS control-flow layer and the JS->binding identity
// edge only. The binding->native dispatch edge is recorded in StreamingCompletionBridge.hpp
// as an identity (accepted R1d-A address), never as an implementation. Native bodies, ABI,
// and the internal streaming algorithm are UNKNOWN and out of scope.
// classification: CLM-R1DB-0099 + CLS-R1DB-STREAMING-0001 [UNKNOWN/OBSERVATION]
