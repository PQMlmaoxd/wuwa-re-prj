// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; SubLevelController TypeDef 0x020046e8.
// Direct CIL for the caller, generated closure, process bridge, and completion
// callback is in evidence/performance/PERF02_DIRECT_CIL.txt.
// Gameplay/model/process types referenced here remain external dependencies.
// SOURCE_VERIFIED means the corresponding canonical decompiled member was inspected.
// DIRECT_IL_VERIFIED means a complete successful direct-CIL response is stored.

using System;
using System.Collections.Generic;
using CSharpScript.Game.World.Controller.SubLevelAssistant;
using Cysharp.Threading.Tasks;

public class SubLevelController : ControllerWithAssistantBase<SubLevelController>
{
    private enum EAssistantType
    {
        SubLevelVisible
    }

    // OBSERVED_FROM_CANONICAL_SOURCE field; process/model types remain external.
    private Queue<SubLevelProcess>? TaskQueue;

    // MethodDef 0x0602f352; SOURCE_VERIFIED; metadata only (not directly captured).
    protected override bool OnInit()
    {
        bool result = base.OnInit();
        TaskQueue = new Queue<SubLevelProcess>();
        return result;
    }

    // MethodDef 0x0602f353; SOURCE_VERIFIED; metadata only (not directly captured).
    protected override bool OnClear()
    {
        bool result = base.OnClear();
        TaskQueue?.Clear();
        TaskQueue = null;
        return result;
    }

    // MethodDef 0x0602f357; SOURCE_VERIFIED; metadata only (not directly captured).
    private T GetAssistant<T>(EAssistantType type) where T : ControllerAssistantBase
    {
        Assistants.TryGetValue((int)type, out ControllerAssistantBase value);
        return value as T;
    }

    // MethodDef 0x0602f35a; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // The compiler-generated source closure is:
    //   SubLevelController/<>c__DisplayClass10_0/<<LoadOrUnloadSubLevel>b__0>d
    // and its callback is <>c__DisplayClass10_1.<LoadOrUnloadSubLevel>b__1.
    // Generated closure members 0x0604ae46 and 0x0604ae48, plus MoveNext
    // 0x06050750, are DIRECT_IL_VERIFIED. The tuple array in the failure log is a
    // projection-only allocation risk; direct generated code uses inline-array span
    // storage instead.
    public void LoadOrUnloadSubLevel(IList<string> subLevels, IList<string> hiddenSubLevels)
    {
        if (subLevels == null || subLevels.Count == 0)
        {
            return;
        }

        Dictionary<string, SubLevel> allSubLevels =
            ModelBase<SubLevelModel>.Instance.GetAllSubLevels();
        List<string> unloads = new List<string>();
        List<string> newLoads = new List<string>();
        HashSet<string> requested = new HashSet<string>();

        foreach (string subLevelPath in subLevels)
        {
            requested.Add(subLevelPath);
            SubLevel? subLevel = ModelBase<SubLevelModel>.Instance.GetSubLevel(subLevelPath);
            if (subLevel == null || subLevel.IsVisible != !hiddenSubLevels.Contains(subLevelPath))
            {
                newLoads.Add(subLevelPath);
            }
        }

        foreach (string key in allSubLevels.Keys)
        {
            if (!requested.Contains(key))
            {
                unloads.Add(key);
            }
        }

        if (unloads.Count == 0 && newLoads.Count == 0)
        {
            return;
        }

        ModelBase<GameModeModel>.Instance.AddLoadMapHandle("LoadOrUnloadSubLevel");

        // Source-level projection of the compiler-generated AsyncTask run handle.
        // No pump occurs in AddTask; Run is the separate call below.
        AsyncTask task = new AsyncTask(
            "LoadOrUnloadSubLevel",
            async () =>
            {
                CustomPromise<bool> promise = new CustomPromise<bool>();
                ChangeSubLevel(
                    unloads,
                    newLoads,
                    EScreenEffectType.ScreenEffectNone,
                    null,
                    null,
                    result =>
                    {
                        if (!result)
                        {
                            Singleton<Log>.Instance.Error(
                                (ELogModule)102,
                                (ELogAuthor)3,
                                "SubLevelController.加载或者卸载子关卡失败",
                                new (string, object)[]
                                {
                                    ("unloads", unloads),
                                    ("newLoads", newLoads)
                                });
                        }

                        ModelBase<GameModeModel>.Instance.RemoveLoadMapHandle("LoadOrUnloadSubLevel");
                        if (result && ModelBase<GameModeModel>.Instance.MapDone)
                        {
                            ControllerBase<CreatureController>.Instance.CreateEntityFromPending(
                                EAddEntityType.Normal);
                        }

                        promise.SetResult(result);
                    },
                    hiddenSubLevels);
                return await promise.Promise;
            });

        Singleton<TaskSystem>.Instance.AddTask(task);
        Singleton<TaskSystem>.Instance.Run();
    }

    // This method is observed in the same caller path. Its process types and the
    // deeper ChangeSubLevelInternal body are intentionally external to PERF-02.
    // MethodDef 0x0602f35b; SOURCE_VERIFIED; metadata only (not directly captured).
    public void ChangeSubLevel(
        IList<string> unloadLevels,
        IList<string> levels,
        EScreenEffectType screenEffect,
        Vector? location = null,
        Rotator? rotator = null,
        Action<bool>? callback = null,
        IList<string>? hiddenSubLevels = null)
    {
        Dictionary<string, bool> levelsWithVisible = new Dictionary<string, bool>();
        foreach (string level in levels)
        {
            bool visible = (hiddenSubLevels?.IndexOf(level) ?? -1) < 0;
            levelsWithVisible[level] = visible;
        }

        SwitchSubLevelProcess element = new SwitchSubLevelProcess(
            new SwitchSubLevelParams
            {
                LevelsWithVisible = levelsWithVisible,
                UnloadLevels = unloadLevels,
                ScreenEffect = screenEffect,
                Location = location,
                Rotator = rotator,
                FinishCallback = callback
            });

        if (!TaskQueue.Empty)
        {
            TaskQueue.Push(element);
            return;
        }

        TaskQueue.Push(element);
        ExecuteSubLevelProcess();
    }

    // MethodDef 0x0602f367; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    private void ExecuteSubLevelProcess()
    {
        if (TaskQueue != null && !TaskQueue.Empty)
        {
            SubLevelProcess front = TaskQueue.Front;
            ExecuteSubLevelProcessImp(front);
        }
    }

    // MethodDef 0x0602f368; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604ae51 is also DIRECT_IL_VERIFIED.
    private async UniTask ExecuteSubLevelProcessImp(SubLevelProcess process)
    {
        bool result = false;
        if (process is SwitchSubLevelProcess switchProcess)
        {
            result = await ChangeSubLevelInternal(
                switchProcess.Params.UnloadLevels,
                switchProcess.Params.LevelsWithVisible,
                switchProcess.Params.ScreenEffect,
                switchProcess.Params.Location,
                switchProcess.Params.Rotator);
        }
        else if (process is SetSubLevelVisibleProcess visibleProcess)
        {
            result = await GetAssistant<SubLevelVisibleAssistant>(EAssistantType.SubLevelVisible)
                .SetSubLevelVisible(visibleProcess.Params);
        }

        OnExecuteFinish(result);
    }

    // ChangeSubLevelInternal is a canonical member whose deep loading body is
    // outside this bounded projection; the call above is not a fabricated stub.

    // MethodDef 0x0602f369; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    private void OnExecuteFinish(bool result)
    {
        SubLevelProcess subLevelProcess = TaskQueue?.Pop();
        if (subLevelProcess is SwitchSubLevelProcess switchProcess)
        {
            switchProcess.Params.FinishCallback?.Invoke(result);
        }
        else if (subLevelProcess is SetSubLevelVisibleProcess visibleProcess)
        {
            visibleProcess.Params.FinishCallback(result);
        }

        ExecuteSubLevelProcess();
    }
}
