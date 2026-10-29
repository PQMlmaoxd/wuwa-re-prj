// Build: BLD-WUWA-0001
// Unit: RCN-R1C-DATALAYER-0001
// Material claims: CLM-R1C-0001, CLM-R1C-0002, CLM-R1C-0003, CLM-R1C-0004, CLM-R1C-0005, CLM-R1C-0006, CLM-R1C-0007, CLM-R1C-0008, CLM-R1C-0009, CLM-R1C-0010, CLM-R1C-0011, CLM-R1C-0012, CLM-R1C-0013, CLM-R1C-0014, CLM-R1C-0015, CLM-R1C-0016, CLM-R1C-0017, CLM-R1C-0018, CLM-R1C-0019, CLM-R1C-0020, CLM-R1C-0021, CLM-R1C-0022, CLM-R1C-0023, CLM-R1C-0024, CLM-R1C-0025
// Classification: CLS-R1C-DATALAYER-0001
// Review: REV-R1C-0001
//
// R1c — DataLayer Transition Slice. Source-like reconstruction of the phase-9
// "Load.SetDataLayerAndLoadSubLevel" region of GameModeController.Load(l) and
// its directly reached DataLayer / sub-level collaborators.
//
// Reconstruction conventions (binding, per R1c authorization):
// - Every material property below is annotated with its governing claim ID and
//   [LABEL/STATUS]. Claims are OBSERVED (exact byte-evidenced tokens) unless
//   marked otherwise; native bodies, enum meanings, and signatures are UNKNOWN.
// - Minified identifiers are reconstructed aliases; original compiled names are
//   retained in the machine-readable alias map (ART-R1A-ALIASES-style,
//   ART-R1C-ALIASES-0001) with exact-assignment evidence.
// - "场景加载通知器" is kept as the literal reflected property name.
// - This file prioritizes semantic fidelity over compilability: types are
//   descriptive only; no ABI/signature inference is asserted.

// ============================================================================
// Section 1 — Phase-9 entry (parent continuation from accepted R1b)
// ============================================================================
// CLM-R1C-0001 [OBSERVED/OBSERVATION]
// The accepted R1b slice ends with the preload phase closed (progress 35,
// phase 8). The next region of the same Load(l) async task begins phase 9:
//   _.LoadDataLayerAndSubLevelProfiler.Restart()
//   cpp_1.FKuroPerfSightHelper.BeginExtTag("Load.SetDataLayerAndLoadSubLevel")
//   _.LoadingPhase = 9
// "Load.SetDataLayerAndLoadSubLevel" is a profiler tag, not a method name
// (R1a "Load.OpenLevel" precedent).

async function loadPhase9_SetDataLayerAndLoadSubLevel(gameModeModel: GameModeModel, mapConfig: MapConfig): Promise<void> {
    // CLM-R1C-0001 [OBSERVED/OBSERVATION]
    gameModeModel.loadingPhase = 9; // emits LoadingPhaseChange C# bridge event (R1a precedent)

    // ------------------------------------------------------------------------
    // Step 1 — DataLayer load (diff-based transaction)
    // CLM-R1C-0002 [OBSERVED/OBSERVATION]
    gameModeModel.loadDataLayerProfiler.restart();
    this.loadDataLayers(mapConfig); // alias of compiled `this.LoadDataLayers(l)`
    gameModeModel.loadDataLayerProfiler.stop();

    // ------------------------------------------------------------------------
    // Step 2 — Streaming-quality console variables
    // CLM-R1C-0007 [OBSERVED/OBSERVATION]
    this.updateStreamingQualityLevel();

    // ------------------------------------------------------------------------
    // Step 3 — Sub-level load barrier
    // CLM-R1C-0008 [OBSERVED/OBSERVATION]
    gameModeModel.loadSubLevelProfiler.start();
    await controllerHolder.subLevelController.checkLoadSubLevels(mapConfig);
    gameModeModel.loadSubLevelProfiler.stop();

    // ------------------------------------------------------------------------
    // Step 4 — Completion and continuation to phase 10
    // CLM-R1C-0025 [OBSERVED/OBSERVATION]
    loadingController.setProgress(ELoadingPhase.SETDATALAYER_AND_LOADSUBLEVEL_END_PROGRESS); // 40
    gameModeModel.loadingPhase = 10;
    cpp_1.FKuroPerfSightHelper.EndExtTag("Load.SetDataLayerAndLoadSubLevel");
    gameModeModel.loadDataLayerAndSubLevelProfiler.stop();

    // HARD STOP — R1c slice boundary.
    // CLM-R1C-0025 [OBSERVED/OBSERVATION]
    // The next region assigns _.LoadingPhase = 11 and begins the
    // CheckVoxelStreaming phase, which is out of scope for R1c.
}

// ============================================================================
// Section 2 — DataLayer diff transaction
// ============================================================================

// CLM-R1C-0002 [OBSERVED/OBSERVATION]
// LoadDataLayers performs the DataLayer portion of the map transition.
static async loadDataLayers(mapConfig: MapConfig): Promise<void> {
    // CLM-R1C-0003 [OBSERVED/OBSERVATION]
    // Low-memory gate: IsLowMemoryDevice(), OR-ed with the Android
    // PlatformType==2 && PhysicalGBRam<=6 check. The
    // "DatalayerRuntime_HideInLowMemDevice" layer is ALWAYS set to the
    // negation of the low-memory flag (an outgoing render-module boundary).
    const isLowMemDevice: boolean = UE.KuroStaticLibrary.IsLowMemoryDevice()
        || (Info.platformType === 2 /* meaning of 2 UNKNOWN as enum name */
            && GameSettingsDeviceRender.physicalGBRam <= 6);
    renderModuleController.setWorldPartitionDataLayerState(
        "DatalayerRuntime_HideInLowMemDevice",
        !isLowMemDevice,
    );

    // CLM-R1C-0004 [OBSERVED/OBSERVATION]
    // bRs diff transaction: activate layers present in the map config but not
    // currently active; deactivate currently-active layers absent from the
    // config. Config resolution (DataLayerConfigById) is an outgoing ConfigDB
    // boundary — its implementation is UNKNOWN.
    const requestedConfigIds: number[] | undefined = mapConfig.bRs;
    if (requestedConfigIds) {
        const activeLayers: string[] = [...gameModeModel.getAllDataLayers()];
        for (const configId of requestedConfigIds) {
            const cfg = configDataLayerConfigById.getConfig(configId);
            if (!cfg) {
                // CLM-R1C-0004 [OBSERVED/OBSERVATION] missing config -> error log
                Log.error("GameMode", 3, "加载场景:加载DataLayer失败,不存在的配置Id", ["DataLayerId:", configId]);
                continue;
            }
            const layerName = cfg.dataLayer;
            if (gameModeModel.hasDataLayer(layerName)) {
                // Already active: remove from the pending-deactivate list.
                const idx = activeLayers.indexOf(layerName);
                if (idx >= 0) activeLayers.splice(idx, 1);
            } else {
                Log.info("GameMode", 3, "加载场景:加载DataLayer", ["DataLayer", layerName]);
                GameModeController.Afr(layerName); // activate — original compiled name
            }
        }
        // Anything still in the list is active but not requested: deactivate.
        for (const remaining of activeLayers) {
            GameModeController.Ufr(remaining); // deactivate — original compiled name
        }
    }
}

// CLM-R1C-0005 [OBSERVED/OBSERVATION]
// Afr (activate): model-set add + render-module activate.
static Afr(layerName: string): void {
    gameModeModel.addDataLayer(layerName);
    renderModuleController.setWorldPartitionDataLayerState(layerName, /* enable */ true);
}

// CLM-R1C-0006 [OBSERVED/OBSERVATION]
// Ufr (deactivate): model-set remove + render-module deactivate.
static Ufr(layerName: string): void {
    gameModeModel.removeDataLayer(layerName);
    renderModuleController.setWorldPartitionDataLayerState(layerName, /* enable */ false);
}

// ============================================================================
// Section 3 — Streaming-quality console variables
// ============================================================================

// CLM-R1C-0007 [OBSERVED/OBSERVATION]
static async updateStreamingQualityLevel(): Promise<void> {
    const worldSettings = GlobalData.world.getWorld().K2_GetWorldSettings();
    if (worldSettings?.bEnableWorldPartition) {
        // X = 0.8 when the current device render feature StreamLevel is 0,
        // else 1. Native meaning of the numeric stream levels is UNKNOWN.
        const feature = GameSettingsDeviceRender.getCurrentDeviceRenderFeature();
        const streamLevel = feature?.streamLevel ?? 0;
        const x = streamLevel === 0 ? 0.8 : 1;
        UE.KismetSystemLibrary.ExecuteConsoleCommand(
            GlobalData.world,
            `wp.Runtime.ModifyQualityLevelStreamingValue ${x}`,
        );
        UE.KismetSystemLibrary.ExecuteConsoleCommand(
            GlobalData.world,
            `wp.Runtime.CurStreamingQualityLevel ${x}`,
        );
    } else {
        Log.info(/* log only, no console commands */);
    }
}

// ============================================================================
// Section 4 — Sub-level load barrier
// ============================================================================

// CLM-R1C-0009 [OBSERVED/OBSERVATION]
// CheckLoadSubLevels builds a path -> boolean map. When MapConfig.URs is
// present, value = (path not in the j$_ preload list). Otherwise
// InstanceDungeon.SubLevels entries map to true. Empty map short-circuits.
static async checkLoadSubLevels(mapConfig: MapConfig): Promise<boolean> {
    const toLoad = new Map<string, boolean>();
    if (mapConfig.URs?.length) {
        for (const path of mapConfig.URs) {
            const preloadIdx = mapConfig.j$_.indexOf(path); // j$_ = preload list, original name
            toLoad.set(path, preloadIdx < 0);
        }
    } else {
        const dungeonSubLevels = gameModeModel.instanceDungeon?.subLevels;
        if (dungeonSubLevels) {
            for (const path of dungeonSubLevels) toLoad.set(path, true);
        }
    }
    if (toLoad.size === 0) return true;
    return await this.loadSubLevels(toLoad);
}

// CLM-R1C-0010 [OBSERVED/OBSERVATION]
// Two-phase barrier: entries with value=true (visible loads) push first,
// value=false after. Always returns true — individual failures surface
// through each SubLevel's LoadPromise semantics, not this return value.
static async loadSubLevels(toLoad: Map<string, boolean>): Promise<boolean> {
    const promises: Array<Promise<boolean>> = [];
    for (const [path, loadNow] of toLoad) {
        if (loadNow) promises.push(this.loadSubLevel(path, loadNow));
    }
    for (const [path, visibleOnly] of toLoad) {
        if (!visibleOnly) promises.push(this.loadSubLevel(path, visibleOnly));
    }
    await Promise.all(promises);
    return true;
}

// CLM-R1C-0011 [OBSERVED/OBSERVATION]
// Per-path barrier. Fresh loads go through the native LoadStreamLevel boundary
// and await the SubLevel's GameModePromise-backed LoadPromise.
static async loadSubLevel(path: string, visibleAfterLoad: boolean): Promise<boolean> {
    const subLevelModel = ModelManager.subLevelModel;
    let sub = subLevelModel.getPreloadSubLevel(path);
    if (sub) {
        subLevelModel.movePreloadSubLevelToSubLevel(path);
        if (sub.loadState === 1) {
            await sub.loadPromise.promise;
        }
        await sub.setLevelVisible(visibleAfterLoad, "LoadSubLevel");
        return true;
    }
    sub = subLevelModel.getSubLevel(path);
    if (sub) {
        if (sub.loadState === 2) {
            await sub.setLevelVisible(visibleAfterLoad, "LoadSubLevel");
            return true;
        }
    } else {
        sub = subLevelModel.addSubLevel(path, visibleAfterLoad);
    }
    if (sub.loadState === 0) {
        sub.loadState = 1;
        // CLM-R1C-0012 [OBSERVED/OBSERVATION]
        // Native boundary — returns a LinkId handle; body UNKNOWN.
        sub.linkId = GlobalData.gameInstance.场景加载通知器.LoadStreamLevel(
            FNameUtil.getDynamicFName(path),
            sub.loadVisibleParam,
            false,
        );
        Log.info("GameMode", /* "加载子关卡" */);
    }
    // CLM-R1C-0011 [OBSERVED/OBSERVATION]
    // UiTimeDilation tag wrap around the await (outgoing boundary; not
    // reconstructed — see EVD-R1C-0031).
    UiTimeDilation.addWaitSetTimeDilationTag("SubLevelController.LoadSubLevel" + sub.linkId);
    const result = await sub.loadPromise.promise;
    UiTimeDilation.deleteWaitSetTimeDilationTag("SubLevelController.LoadSubLevel" + sub.linkId);
    return result;
}

// ============================================================================
// Section 5 — Native callback chain (stream-level completion)
// ============================================================================

// CLM-R1C-0013 [OBSERVED/OBSERVATION]
// WorldGlobal binds the native load-stream delegates at initialization.
// ZEr is the load-completion callback; original compiled names retained.
WorldGlobal.ZEr = (linkId: number, levelName: string, levelStreaming: LevelStreaming): void => {
    Log.info("LoadStream完成。");
    if (GlobalData.world?.isValid()) {
        SubLevelController.OnLoadSubLevel(linkId, levelName, levelStreaming);
    }
};

// CLM-R1C-0014 [OBSERVED/OBSERVATION]
// Dispatch by LinkId; original compiled name OnLoadSubLevel retained.
static onLoadSubLevel(linkId: number, levelName: string, levelStreaming: LevelStreaming): void {
    let sub: SubLevel | undefined;
    for (const [, candidate] of subLevelModel.getAllPreloadSubLevels()) {
        if (candidate.linkId === linkId) { sub = candidate; break; }
    }
    if (!sub) {
        for (const [, candidate] of subLevelModel.getAllSubLevels()) {
            if (candidate.linkId === linkId) { sub = candidate; break; }
        }
    }
    if (!sub) {
        // Re-unload case: the unload set may hold this LinkId.
        const unloading = subLevelModel.getUnloadSubLevel(levelName);
        if (unloading) {
            unloading.loadPromise?.setResult(true);
            GlobalData.gameInstance.场景加载通知器.UnloadStreamLevel(
                FNameUtil.getDynamicFName(unloading.path), true);
        } else {
            Log.warn("加载的子关卡不存在");
        }
        return;
    }
    sub.onLevelLoad(levelStreaming).then(() => {
        Log.info("子关卡加载完成");
    });
}

// ============================================================================
// Section 6 — SubLevel state machine
// ============================================================================

// CLM-R1C-0015 [OBSERVED/OBSERVATION]
// SubLevel initial state. Original compiled field names retained in the alias
// map (Yz1 visible flag, Pd1 visibility promise, etc.).
class SubLevel {
    public path: string;
    public visibleAfterLoad: boolean;
    public loadState: 0 | 1 | 2 = 0; // numeric states observed 0/1/2 only; idle/loading/loaded naming RECONSTRUCTED, not source-named
    public level: LevelStreaming | undefined = undefined;
    public linkId: number = 0;
    public unLoadLinkId: number = 0;
    public dependOnBeginPlayLogic = false;
    public loadPromise = new GameModePromise();
    public unLoadPromise = new GameModePromise();

    constructor(path: string, visibleAfterLoad: boolean) {
        this.path = path;
        this.visibleAfterLoad = visibleAfterLoad;
        // _Audio sub-levels opt into begin-play-driven visibility.
        if (path.includes("_Audio")) this.dependOnBeginPlayLogic = true;
    }

    // LoadVisibleParam getter — original name retained.
    // CLM-R1C-0015 [OBSERVED/OBSERVATION]
    get loadVisibleParam(): boolean {
        return !this.dependOnBeginPlayLogic || this.visibleAfterLoad;
    }

    // CLM-R1C-0016 [OBSERVED/OBSERVATION]
    // Always resolves the LoadPromise with true, including the error path
    // (error only logged). No false/reject branch exists in the observed bytes.
    async onLevelLoad(levelStreaming: LevelStreaming): Promise<void> {
        if (levelStreaming) {
            this.level = levelStreaming;
            this.visibleFlag = this.loadVisibleParam; // Yz1
            if (!this.dependOnBeginPlayLogic) {
                const loadedLevel = levelStreaming.getLoadedLevel();
                const subsystem = UE.KuroSubLevelVisibleSubsystem.getSubSystem(GlobalData.gameInstance);
                subsystem.addLevel(this.linkId, loadedLevel);
                subsystem.setOneFrameExecuteCount(
                    IsPc ? PC_ONEFRAME_MAXSET_COUNT : MOBILE_ONEFRAME_MAXSET_COUNT);
                await this.setLevelVisible(this.visibleAfterLoad, "OnLevelLoad", false);
            }
            this.loadState = 2;
        } else {
            Log.error(/* invalid level streaming */);
        }
        this.loadPromise.setResult(true); // always true — see claim
    }

    // CLM-R1C-0017 [OBSERVED/OBSERVATION]
    async setLevelVisible(visible: boolean, caller: string, waitForLoad = true): Promise<void> {
        if (this.level && this.level.isLevelLoaded()) {
            if (waitForLoad && this.loadState === 1) {
                await this.loadPromise.promise;
            }
            if (this.dependOnBeginPlayLogic) {
                await this.beginPlayVisibility(this.level, visible, caller); // zz1
            } else {
                await this.actorVisibility(visible, caller); // Jz1
            }
        } else {
            Log.error("SetLevelVisible:失败,Level未加载完毕");
        }
    }

    // CLM-R1C-0018 [OBSERVED/OBSERVATION]
    // zz1 — begin-play variant: level-shown/hidden delegates on the
    // LevelStreaming plus SetShouldBeVisible; completion via CustomPromise.
    private async beginPlayVisibility(level: LevelStreaming, visible: boolean, _caller: string): Promise<void> {
        if (level.isLevelVisible() !== visible) {
            // OnLevelShown / OnLevelHidden delegate wiring + SetShouldBeVisible
            // + CustomPromise await — native delegate semantics UNKNOWN.
            /* observed: delegates added, SetShouldBeVisible(visible), await custom promise */
        }
    }

    // CLM-R1C-0018 [OBSERVED/OBSERVATION]
    // Jz1 — non-beginplay variant: KuroSubLevelVisibleSubsystem native call.
    private async actorVisibility(visible: boolean, _caller: string): Promise<void> {
        if (this.visibleFlag !== visible) {
            this.visibilityPromise = new CustomPromise(); // Pd1
            // Native boundary — returns whether the actors set is empty.
            UE.KuroSubLevelVisibleSubsystem.getSubSystem(GlobalData.gameInstance)
                .setLevelActorsVisible(this.linkId, visible, this.visibilityDelegate); // xd1
            await this.visibilityPromise.promise;
        }
    }
}

// ============================================================================
// Section 7 — SubLevelModel registry
// ============================================================================

// CLM-R1C-0019 [OBSERVED/OBSERVATION]
// Three maps: SW_ preload sub-levels, da sub-levels, MW_ unload sub-levels
// (original compiled field names retained in the alias map).
class SubLevelModel {
    private preloadSubLevels = new Map<string, SubLevel>();  // SW_
    private subLevels = new Map<string, SubLevel>();          // da
    private unloadSubLevels = new Map<string, SubLevel>();     // MW_

    addSubLevel(path: string, visibleAfterLoad: boolean): SubLevel {
        if (!this.subLevels.has(path)) {
            const sub = new SubLevel(path, visibleAfterLoad);
            this.subLevels.set(path, sub);
            return sub;
        }
        Log.error("重复添加子关卡");
        return this.subLevels.get(path)!;
    }

    movePreloadSubLevelToSubLevel(path: string): void {
        const pre = this.getPreloadSubLevel(path);
        if (pre) {
            this.removePreloadSubLevel(path);
            this.subLevels.set(pre.path, pre);
        }
    }

    getPreloadSubLevel(path: string): SubLevel | undefined { /* SW_ lookup */ }
    getSubLevel(path: string): SubLevel | undefined { /* da lookup */ }
    getUnloadSubLevel(path: string): SubLevel | undefined { /* MW_ lookup */ }
}

// ============================================================================
// Section 8 — GameModeModel DataLayer set
// ============================================================================

// CLM-R1C-0020 [OBSERVED/OBSERVATION]
// The DataLayerSet backs the diff transaction (original method names
// AddDataLayer/RemoveDataLayer/HasDataLayer/GetAllDataLayers retained).
class GameModeModel {
    public dataLayerSet = new Set<string>();

    addDataLayer(layerName: string): boolean {
        if (this.dataLayerSet.has(layerName)) {
            Log.debug("[GameModeModel.AddDataLayer] 重复添加DataLayer。", ["Path", layerName]);
            return false;
        }
        this.dataLayerSet.add(layerName);
        return true;
    }

    // CLM-R1C-0021 [OBSERVED/OBSERVATION]
    removeDataLayer(layerName: string): boolean {
        if (!this.hasDataLayer(layerName)) return false;
        this.dataLayerSet.delete(layerName);
        return true;
    }

    // CLM-R1C-0022 [OBSERVED/OBSERVATION]
    hasDataLayer(layerName: string): boolean {
        return this.dataLayerSet.has(layerName);
    }

    // CLM-R1C-0023 [OBSERVED/OBSERVATION]
    getAllDataLayers(): Set<string> {
        return this.dataLayerSet;
    }
}

// ============================================================================
// Section 9 — RenderModuleController DataLayer state layer
// ============================================================================

// CLM-R1C-0021 [OBSERVED/OBSERVATION]
// Hard-coded dependency maps for the 30FGL DataLayer family (original field
// names hardCodeDataLayerDependencies / hardCodeReversedDataLayerDependencies).
const hardCodeDataLayerDependencies = new Map<string, Set<string>>([
    ["DataLayerRuntime_DLTask_30FGL03", new Set(["DataLayerRuntime_DLTask_30FGL02"])],
    ["DataLayerRuntime_DLTask_30FGL04", new Set(["DataLayerRuntime_DLTask_30FGL02"])],
    ["DataLayerRuntime_DLTask_30FGL05", new Set(["DataLayerRuntime_DLTask_30FGL02"])],
]);
const hardCodeReversedDataLayerDependencies = new Map<string, Set<string>>([
    ["DataLayerRuntime_DLTask_30FGL02", new Set([
        "DataLayerRuntime_DLTask_30FGL03",
        "DataLayerRuntime_DLTask_30FGL04",
        "DataLayerRuntime_DLTask_30FGL05",
    ])],
]);

class RenderModuleController {
    // CLM-R1C-0022 [OBSERVED/OBSERVATION]
    // Activation gate: when enabling a layer whose hard-coded dependencies
    // are not all enabled, activation is blocked and the layer is parked in
    // the DependenciesNotMatch set for rechecking on later calls.
    static setWorldPartitionDataLayerState(
        layerName: string,
        enable: boolean,
        viaAlternativePath = false, // `a` — meaning UNKNOWN
    ): void {
        if (enable && !this.dependenciesSatisfied(layerName)) { // Ufg
            Log.info(
                "[SetWorldPartitionDataLayerState]激活DataLayer前，发现依赖不满足，停止激活DataLayer，并加进DependenciesNotMatchDataLayerSet",
                ["dataLayerName", layerName],
            );
            renderModuleModel.addDependenciesNotMatchDataLayer(layerName);
            return; // no activation performed
        } else if (renderModuleModel.isDependenciesNotMatchDataLayer(layerName)) {
            renderModuleModel.removeDependenciesNotMatchDataLayer(layerName);
        }
        Log.info(`正式执行${enable ? "激活" : "停用"}DataLayer`, ["dataLayerName", layerName]);
        // CLM-R1C-0023 [OBSERVED/OBSERVATION]
        // State code: 2 = enable, 1 = alternative path, 0 = disable.
        // The numeric meaning is UNKNOWN beyond the observed mapping.
        const stateCode = enable ? 2 : (viaAlternativePath ? 1 : 0);
        // Native boundary — body UNKNOWN.
        UE.KuroRenderingRuntimeBPPluginBPLibrary.SetWorldPartitionDataLayerState2(
            GlobalData.world,
            FNameUtil.getDynamicFName(layerName),
            stateCode,
        );
        this.cascadeDependencyState(layerName, enable, viaAlternativePath); // xfg
    }

    // CLM-R1C-0024 [OBSERVED/OBSERVATION]
    // xfg cascade: on activate, reversed-dependencies parked in the NotMatch
    // set whose own dependencies are now satisfied get re-activated; on
    // deactivate, enabled reversed-dependencies get parked and deactivated.
    static cascadeDependencyState(layerName: string, enable: boolean, viaAlternativePath: boolean): void {
        const stateCode = enable ? 2 : (viaAlternativePath ? 1 : 0);
        const reversed = hardCodeReversedDataLayerDependencies.get(layerName);
        if (enable) {
            if (reversed?.size) {
                for (const related of reversed) {
                    if (renderModuleModel.isDependenciesNotMatchDataLayer(related)
                        && this.dependenciesSatisfied(related)) {
                        Log.info("激活DataLayer后，发现有本该激活的RelatedDataLayer依赖都已满足，激活relatedDataLayer",
                            ["dataLayerName", layerName], ["relatedDataLayer", related]);
                        renderModuleModel.removeDependenciesNotMatchDataLayer(related);
                        UE.KuroRenderingRuntimeBPPluginBPLibrary.SetWorldPartitionDataLayerState2(
                            GlobalData.world,
                            FNameUtil.getDynamicFName(related),
                            stateCode,
                        );
                    }
                }
            }
        } else {
            if (reversed?.size) {
                for (const related of reversed) {
                    if (this.isWorldPartitionDataLayerEnable(related)) {
                        renderModuleModel.addDependenciesNotMatchDataLayer(related);
                        UE.KuroRenderingRuntimeBPPluginBPLibrary.SetWorldPartitionDataLayerState2(
                            GlobalData.world,
                            FNameUtil.getDynamicFName(related),
                            stateCode,
                        );
                    }
                }
            }
        }
    }

    // CLM-R1C-0022 [OBSERVED/OBSERVATION]
    // Ufg — dependencies-satisfied check over the hard-coded map.
    static dependenciesSatisfied(layerName: string): boolean {
        const deps = hardCodeDataLayerDependencies.get(layerName);
        let satisfied = true;
        if (deps?.size) {
            for (const dep of deps) {
                if (!(UE.KuroRenderingRuntimeBPPluginBPLibrary.IsWorldPartitionDataLayerEnable(
                        GlobalData.world, FNameUtil.getDynamicFName(dep)))) {
                    satisfied = false;
                    break;
                }
            }
        }
        return satisfied;
    }

    // CLM-R1C-0019 (partially; see EVD-R1C-0019) [OBSERVED/OBSERVATION]
    static isWorldPartitionDataLayerEnable(layerName: string): boolean {
        return UE.KuroRenderingRuntimeBPPluginBPLibrary.IsWorldPartitionDataLayerEnable(
                   GlobalData.world, FNameUtil.getDynamicFName(layerName))
            || renderModuleModel?.isDependenciesNotMatchDataLayer(layerName) === true;
    }
}

// ============================================================================
// Section 10 — UNKNOWN boundaries (recorded, not reconstructed)
// ============================================================================
//
// The following are outgoing native/reflection boundaries observed by exact
// name in the admitted bytes. Their implementations, signatures, enum
// meanings, and behaviors are UNKNOWN and intentionally not reconstructed:
//
//   - UE.KuroRenderingRuntimeBPPluginBPLibrary.SetWorldPartitionDataLayerState2(world, fname, code)
//   - UE.KuroRenderingRuntimeBPPluginBPLibrary.IsWorldPartitionDataLayerEnable(world, fname)
//   - GlobalData.GameInstance.场景加载通知器.LoadStreamLevel(fname, visibleParam, false) -> linkId
//   - GlobalData.GameInstance.场景加载通知器.UnloadStreamLevel(fname, true)
//   - BindLoadStreamLevel / BindUnLoadStreamLevel (delegate registration)
//   - UE.KuroSubLevelVisibleSubsystem.GetSubSystem(...) -> AddLevel / RemoveLevel /
//     SetLevelActorsVisible / SetOneFrameExecuteCount
//   - UE.KuroStaticLibrary.IsLowMemoryDevice()
//   - GameSettingsDeviceRender.PhysicalGBRam / .GetCurrentDeviceRenderFeature()
//   - Info.PlatformType (numeric platform values — enum names UNKNOWN)
//   - K2_GetWorldSettings()?.bEnableWorldPartition
//   - UE.KismetSystemLibrary.ExecuteConsoleCommand(world, command)
//   - DataLayerConfigById.configDataLayerConfigById.GetConfig(id) (ConfigDB boundary)
//   - LevelStreaming: IsLevelLoaded / IsLevelVisible / OnLevelShown / OnLevelHidden /
//     SetShouldBeVisible / GetLoadedLevel
//   - UiTimeDilation.AddWaitSetTimeDilationTag / DeleteWaitSetTimeDilationTag
//   - cpp_1.FKuroPerfSightHelper.BeginExtTag / EndExtTag
//   - GameModePromise setResult semantics (accepted R1a CLM-R1A-0005 governs)

// classification: CLM-R1C-0099 + CLS-R1C-DATALAYER-0001 [UNKNOWN/OBSERVATION]
