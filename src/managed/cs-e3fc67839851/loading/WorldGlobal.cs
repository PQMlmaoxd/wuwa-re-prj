/*
 * PERF04 PARTIAL SOURCE-LIKE FRAGMENT — NONCANONICAL / RECONSTRUCTED.
 * TypeDef: 0x020034e0.
 * Full type: global::WorldGlobal.
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/7d83dcfb7d0d91a0b591380471198ea186d64db500a33252949f27daed462de6/source/_global/WorldGlobal.cs
 * Selected bodies: Initialize (0x0601C95F) and LoadSubLevel (0x0601C97C).
 * Omitted-members node: static reset/clear and other map callbacks remain
 * external; native/provider implementation bodies are UNKNOWN and are not
 * represented by stubs.
 * `partial` marks reconstruction packaging only; it does not prove the
 * original declaration used the partial modifier.
 *
 * Delegate wrappers and lifetime are represented only at the managed binding
 * boundary. Source assembly: CSharpScript.dll SHA-256
 * e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
 * MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3.
 */

using System;
using CSharpScript.Game.World.Controller;
using UnrealEngine;

public partial class WorldGlobal : IStaticVariableResetter
{
    private static FBeginTravelLoadMapDelegate? BeginTravelLoadMapDelegate;
    private static FEndLoadTransitionMapDelegate? EndLoadTransitionMapHandler;
    private static FBeginLoadMapDelegate? BeginLoadMapHandler;
    private static FEndLoadMapDelegate? EndLoadMapHandler;
    private static FLoadStreamLevelDelegate? LoadStreamLevelHandler;
    private static FUnLoadStreamLevelDelegate? UnLoadStreamLevelHandler;

    // MethodDef 0x0601C95F; RVA 0x89052c; code size 380.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1448-1574.
    public static void Initialize()
    {
        GlobalData.GameInstance.场景加载通知器 = new ULoadMapNotify(GlobalData.GameInstance);
        GlobalData.GameInstance.场景加载通知器.Clear();

        BeginTravelLoadMapDelegate =
            DelegateUtils.ToManualReleaseDelegate<FBeginTravelLoadMapDelegate>(
                new Action<string>(BeginTravelLoadMap));
        GlobalData.GameInstance.场景加载通知器.BindBeginTravelLoadMap(BeginTravelLoadMapDelegate);

        EndLoadTransitionMapHandler =
            DelegateUtils.ToManualReleaseDelegate<FEndLoadTransitionMapDelegate>(
                new Action<string>(AfterEnterTransitionMap));
        GlobalData.GameInstance.场景加载通知器.BindEndLoadTransitionMap(EndLoadTransitionMapHandler);

        BeginLoadMapHandler =
            DelegateUtils.ToManualReleaseDelegate<FBeginLoadMapDelegate>(
                new Action<string>(BeginLoadMap));
        GlobalData.GameInstance.场景加载通知器.BindBeginLoadMap(BeginLoadMapHandler);

        EndLoadMapHandler =
            DelegateUtils.ToManualReleaseDelegate<FEndLoadMapDelegate>(
                new Action<string>(EndLoadMap));
        GlobalData.GameInstance.场景加载通知器.BindEndLoadMap(EndLoadMapHandler);

        LoadStreamLevelHandler =
            DelegateUtils.ToManualReleaseDelegate<FLoadStreamLevelDelegate>(
                new Action<int, FName, ULevelStreaming>(LoadSubLevel));
        GlobalData.GameInstance.场景加载通知器.BindLoadStreamLevel(LoadStreamLevelHandler);

        UnLoadStreamLevelHandler =
            DelegateUtils.ToManualReleaseDelegate<FUnLoadStreamLevelDelegate>(
                new Action<int, FName>(UnLoadSubLevel));
        GlobalData.GameInstance.场景加载通知器.BindUnLoadStreamLevel(UnLoadStreamLevelHandler);
    }

    // MethodDef 0x0601C97C; RVA 0x890d54; code size 96.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF04_DIRECT_CIL.txt:1576-1630.
    private static void LoadSubLevel(int linkId, FName levelName, ULevelStreaming? level)
    {
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)3,
            "LoadStream完成。",
            new (string, object)[] { ("LevelName", levelName.ToString()) });

        if (GlobalData.World == null || !GlobalData.World.IsValid())
        {
            return;
        }

        ControllerBase<SubLevelController>.Instance.OnLoadSubLevel(
            linkId,
            levelName.ToString(),
            level);
    }

    // BeginTravelLoadMap, AfterEnterTransitionMap, BeginLoadMap, EndLoadMap, and
    // UnLoadSubLevel are existing class members outside this selected method set.
}
