/*
 * PERF04 PARTIAL SOURCE-LIKE FRAGMENT — NONCANONICAL / RECONSTRUCTED.
 * TypeDef: 0x0200347d.
 * Full type: global::SubLevel.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/4e917791d98d890279255c9e7fab873b11dfbc5ff3806dcc7db5c644bc77d5e9/source/_global/SubLevel.cs
 * Selected bodies: source/call-site projection of LoadVisibleParam
 * (0x0601C569), OnLevelLoad (0x0601C56D), generated
 * <OnLevelLoad>d__13.MoveNext (0x0604A493), and constructor (0x0601C56B).
 * Omitted-members node: SetLevelVisible, SetLevelVisibleFinished, disposal,
 * visibility helpers, and other members remain external; no behavior stubs
 * are supplied by this fragment.
 * `partial` marks reconstruction packaging only; it does not prove the
 * original declaration used the partial modifier.
 *
 * Constructor field values are represented from the selected constructor
 * evidence, not from an invented lifecycle. The getter has a source-only
 * semantic projection because all direct-CIL requests failed; it remains
 * UNKNOWN_DIRECT_CIL. Compiler-generated async lowering is projected once.
 * Source assembly: CSharpScript.dll SHA-256
 * e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.
 */

using Cysharp.Threading.Tasks;
using CSharpScript.Launcher.Platform;
using UnrealEngine;

public partial class SubLevel
{
    public ELoadLevelType LoadState;
    public ULevelStreaming? Level;
    public int LinkId;
    public int UnLoadLinkId;
    public readonly bool DependOnBeginPlayLogic;
    public readonly GameModePromise LoadPromise;
    public readonly GameModePromise UnLoadPromise;
    private bool InnerVisible;
    public readonly string Path;
    public readonly bool VisibleAfterLoad;
    private FSetVisibleFinishDelegate FinishDelegate;

    // MethodDef 0x0601C569; the direct-CIL request and all static retries failed.
    // UNKNOWN_DIRECT_CIL: evidence/performance/PERF04_DIRECT_CIL.txt:1977-1981,
    // 1995-1999, and 2001-2005. The body is source/call-site bounded only.
    public bool LoadVisibleParam
    {
        get
        {
            if (DependOnBeginPlayLogic)
            {
                return VisibleAfterLoad;
            }

            return true;
        }
    }

    // MethodDef 0x0601C56D; RVA 0x87c200; code size 63.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1632-1678.
    // Generated MoveNext 0x0604A493; RVA 0x1426e84; code size 435.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1680-1867.
    public async UniTask OnLevelLoad(ULevelStreaming? level)
    {
        if (level == null)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)81,
                (ELogAuthor)3,
                "SubLevelController:加载的子关卡不存在",
                new (string, object)[]
                {
                    ("LinkId", LinkId),
                    ("Level", Path),
                });
            LoadPromise.SetResult(true);
            return;
        }

        Level = level;
        InnerVisible = LoadVisibleParam;
        if (DependOnBeginPlayLogic)
        {
            LoadState = ELoadLevelType.Done;
            LoadPromise.SetResult(true);
            return;
        }

        ULevel loadedLevel = level.GetLoadedLevel();
        UKuroSubLevelVisibleSubsystem subSystem =
            UKuroSubLevelVisibleSubsystem.GetSubSystem(GlobalData.GameInstance);
        subSystem.AddLevel(LinkId, loadedLevel);
        subSystem.SetOneFrameExecuteCount(
            Singleton<Platform>.Instance.IsPcPlatform() ? 1000 : 200);
        await SetLevelVisible(VisibleAfterLoad, "OnLevelLoad", bWaitingLoading: false);
        LoadState = ELoadLevelType.Done;
        LoadPromise.SetResult(true);
    }

    // MethodDef 0x0601C56B; RVA 0x87c130; code size 91.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1927-1975.
    public SubLevel(string path, bool visibleAfterLoad)
    {
        LoadPromise = new GameModePromise();
        UnLoadPromise = new GameModePromise();
        Path = path;
        VisibleAfterLoad = visibleAfterLoad;
        FinishDelegate = DelegateUtils.ToManualReleaseDelegate<FSetVisibleFinishDelegate>(
            new System.Action<int>(SetLevelVisibleFinished));
        if (Path.Contains("_Audio"))
        {
            DependOnBeginPlayLogic = true;
        }
    }

    // SetLevelVisible, SetLevelVisibleFinished, and disposal members are omitted
    // from this five-fragment PERF-04 scope; calls retain their external boundary.
}
