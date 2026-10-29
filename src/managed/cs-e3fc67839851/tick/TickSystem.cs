// PERF-05: NONCANONICAL / RECONSTRUCTED source-like fragment.
// Static at-rest evidence only; this is not recovered original source.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x02000be8.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/9944f3678489a5589197ccbdbe2072c0dca9d04192a9c8d4b568a9ef65b3f6c3/source/_global/TickSystem.cs
// Direct CIL: evidence/performance/PERF05_DIRECT_CIL.txt.
// Only the selected PERF05 members are projected; other members remain in the
// canonical managed source context.
// Nullable annotations, local names, compiler-closure packing, and logging
// tuple/inline-array syntax are source-like projections, not original layout
// or non-byte-equivalent inline-array storage; they are not game allocations.

using System;
using System.Collections.Generic;
using CSharpScript.Typing;
using UnrealEngine;

public partial class TickSystem : Singleton<TickSystem>
{
    private const int SECOND_TO_MILLISECOND = 1000;

    public const int InvalidId = -1;

    private bool IsPausedInternal;

    public int PausedFrame = -1;

    private int Increment;

    private UKuroTickManager? KuroTickManager;

    private readonly Dictionary<int, Ticker> Tickers = new Dictionary<int, Ticker>();

    private readonly Dictionary<ETickingGroup, Action<float>> Delegates = new Dictionary<ETickingGroup, Action<float>>();

    // Keyed only by group while Groups retains separate priority buckets. A
    // later new priority bucket overwrites the stored group representative;
    // this projection does not infer ownership of earlier native callbacks.
    private readonly Dictionary<ETickingGroup, List<List<Ticker>?>> Groups = new Dictionary<ETickingGroup, List<List<Ticker>?>>();

    private readonly HashSet<int> PendingRemoveIds = new HashSet<int>();

    private bool IsIterating;

    private ETickingGroup IteratingGroup;

    private int IteratingPriority;

    // get_IsPaused MethodDef 0x06003241 and set_IsPaused MethodDef 0x06003242;
    // direct CIL capture failed twice for each accessor in ILSpy-MCP.
    // SOURCE_VERIFIED_ONLY from the canonical source body; no alternative IL
    // parser or FULL_TRUSTED_IL claim is made for either accessor.
    public bool IsPaused
    {
        get
        {
            if (IsPausedInternal)
            {
                return Singleton<Time>.Instance.Frame > PausedFrame;
            }
            return false;
        }
        set
        {
            IsPausedInternal = value;
            if (value)
            {
                PausedFrame = Singleton<Time>.Instance.Frame;
            }
        }
    }

    // MethodDef 0x06003244; RVA 0x21e0b; code size 37.
    // DIRECT_IL_VERIFIED. Only KuroTickManager, Tickers, and Groups are
    // touched; Delegates, PendingRemoveIds, Increment, and iteration state are
    // intentionally not cleared here.
    public void Initialize(UGameInstance gameInstance)
    {
        KuroTickManager = new UKuroTickManager(gameInstance);
        Tickers.Clear();
        Groups.Clear();
    }

    // MethodDef 0x06003245; RVA 0x21e34; code size 76.
    // DIRECT_IL_VERIFIED. Enumerator disposal is observed; only currently
    // stored group-keyed delegate representatives are released, so this does
    // not prove release of every native callback created for every priority.
    // No dictionary or manager cleanup beyond the selected calls is added.
    public void Destroy()
    {
        foreach (KeyValuePair<ETickingGroup, Action<float>> @delegate in Delegates)
        {
            @delegate.Deconstruct(out var _, out var value);
            DelegateUtils.ReleaseManualReleaseDelegate(value);
        }
        KuroTickManager.ClearTick();
    }

    // MethodDef 0x06003247; RVA 0x21eb0; code size 348.
    // DIRECT_IL_VERIFIED. The generated callback is MethodDef 0x06046782,
    // RVA 0x12ab000, code size 252. It preserves the captured
    // this/group/priority/set variables and iterates the live List<Ticker>.
    // Tickers is inserted before priority indexing, bucket creation, and native
    // registration; a negative priority or later fault can retain that partial
    // entry because no rollback/finally path is observed.
    // The generated body clears IsIterating at IL_00a2; its finally spans only
    // pending-id enumerator disposal (IL_00dd-IL_00ea), and Clear is IL_00f6.
    public Ticker? Add(
        Action<float>? handle,
        string name,
        ETickingGroup group = ETickingGroup.TG_PrePhysics,
        bool tickEvenPaused = false,
        int priority = 0,
        bool ignoreSelfCenterMode = false)
    {
        if (handle == null)
        {
            Log instance = Singleton<Log>.Instance;
            (string, object) reference = ("handle", handle);
            instance.Error((ELogModule)21, (ELogAuthor)1, "处理方法不存在", new ReadOnlySpan<(string, object)>(ref reference));
            return null;
        }

        int id = ++Increment;
        Ticker ticker = new Ticker(id, handle, group, priority, name, 0, tickEvenPaused, ignoreSelfCenterMode);
        Tickers[id] = ticker;

        Groups.TryGetValue(group, out List<List<Ticker>?>? value);
        if (value == null)
        {
            value = new List<List<Ticker>?>();
            Groups[group] = value;
        }

        while (value.Count <= priority)
        {
            value.Add(null);
        }

        List<Ticker>? set = value[priority];
        if (set != null)
        {
            // The captured list is live; its Count is re-read by the generated
            // callback, so an in-dispatch append can become part of that same
            // traversal. No explicit re-entry guard is reconstructed here.
            set.Add(ticker);
            return ticker;
        }

        set = new List<Ticker>();
        set.Add(ticker);
        value[priority] = set;
        Action<float> action = delegate(float delta)
        {
            IsIterating = true;
            IteratingGroup = group;
            IteratingPriority = priority;
            float deltaMilliseconds = delta * 1000f;
            for (int i = 0; i < set.Count; i++)
            {
                Ticker ticker2 = set[i];
                if (!ticker2.PendingRemove && (!IsPaused || ticker2.TickEvenPaused))
                {
                    float tickerDelta = ticker2.IgnoreSelfCenterMode
                        ? deltaMilliseconds * Singleton<Time>.Instance.InverseSelfCenteredTimeDilation
                        : deltaMilliseconds;
                    Handle(ticker2, tickerDelta);
                }
            }

            IsIterating = false;
            foreach (int pendingRemoveId in PendingRemoveIds)
            {
                Remove(pendingRemoveId);
            }
            PendingRemoveIds.Clear();
        };
        Delegates[group] = action;
        KuroTickManager.AddTick(group, DelegateUtils.ToManualReleaseDelegate<FTickHandler>(action), priority);
        return ticker;
    }

    // MethodDef 0x06003248; RVA 0x22018; code size 530.
    // DIRECT_IL_VERIFIED. Active matching-group/priority removals are deferred.
    // When the selected priority list becomes empty, the observed path removes
    // the entire group entry and group-keyed delegate/native registration; it
    // does not test whether other priority lists for that group are nonempty.
    // The native call has only the group argument, so this source-like fragment
    // cannot infer which priority registration is removed.
    public bool Remove(int id)
    {
        Tickers.TryGetValue(id, out Ticker? value);
        if (value == null)
        {
            Log instance = Singleton<Log>.Instance;
            (string, object) reference = ("id", id);
            instance.Error((ELogModule)21, (ELogAuthor)1, "编号不存在", new ReadOnlySpan<(string, object)>(ref reference));
            return false;
        }

        if (IsIterating && IteratingGroup == value.Group && IteratingPriority == value.Priority)
        {
            value.PendingRemove = true;
            PendingRemoveIds.Add(id);
            return true;
        }

        Tickers.Remove(id);
        Groups.TryGetValue(value.Group, out List<List<Ticker>?>? value2);
        if (value2 == null)
        {
            Log instance2 = Singleton<Log>.Instance;
            instance2.Error((ELogModule)21, (ELogAuthor)1, "分组不存在", new (string, object)[]
            {
                ("id", id),
                ("group", value.Group),
            });
            return false;
        }

        if (value2.Count <= value.Priority || value2[value.Priority] == null)
        {
            Log instance3 = Singleton<Log>.Instance;
            instance3.Error((ELogModule)21, (ELogAuthor)6, "优先级不存在", new (string, object)[]
            {
                ("id", id),
                ("group", value.Group),
                ("priority", value.Priority),
            });
            return false;
        }

        List<Ticker> list = value2[value.Priority]!;
        list.Remove(value);
        if (list.Count == 0)
        {
            Groups.Remove(value.Group);
            Delegates.TryGetValue(value.Group, out Action<float>? value3);
            Delegates.Remove(value.Group);
            KuroTickManager.RemoveTick(value.Group);
            DelegateUtils.ReleaseManualReleaseDelegate(value3);
        }
        return true;
    }

    // MethodDef 0x0600324b; RVA 0x222e0; code size 475; exception region:
    // callback Invoke is protected by a System.Exception catch. The telemetry
    // and clock calls remain outside that catch exactly as observed.
    private void Handle(Ticker ticker, float delta)
    {
        if (ticker.Pause)
        {
            return;
        }

        float callbackDelta = delta;
        if (ticker.TickIntervalMs > 0)
        {
            ticker.CoolDown += delta;
            if (ticker.CoolDown < ticker.TickIntervalMs)
            {
                return;
            }
            float remainder = ticker.CoolDown % ticker.TickIntervalMs;
            callbackDelta = ticker.CoolDown - remainder;
            ticker.CoolDown = remainder;
        }

        ticker.Count++;
        double milliseconds = KuroTime.GetMilliseconds64();
        try
        {
            ticker.Handle(callbackDelta);
        }
        catch (Exception ex)
        {
            Singleton<Log>.Instance.ErrorWithStack(
                (ELogModule)21,
                (ELogAuthor)1,
                "处理方法执行异常",
                ex,
                new (string, object)[]
                {
                    ("id", ticker.Id),
                    ("error", ex.Message),
                });
        }

        if (!Singleton<PerfSight>.Instance.IsEnable || ticker.Group != 0)
        {
            return;
        }

        int maxFps = UKuroRenderingRuntimeBPPluginBPLibrary.GetMaxFps();
        double frameBudgetMilliseconds = 1000.0 / maxFps;
        float gameThreadTime = UKuroRenderingRuntimeBPPluginBPLibrary.GetGameThreadTime();
        float renderThreadTime = UKuroRenderingRuntimeBPPluginBPLibrary.GetRenderThreadTime();
        if (gameThreadTime > frameBudgetMilliseconds * 2.0 || renderThreadTime > frameBudgetMilliseconds * 2.0)
        {
            double elapsedMilliseconds = KuroTime.GetMilliseconds64() - milliseconds;
            if (ticker.Name == "Core")
            {
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_Core", (float)elapsedMilliseconds);
            }
            else if (ticker.Name == "Game")
            {
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_Game", (float)elapsedMilliseconds);
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_GameThreadTime", gameThreadTime);
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_RenderThreadTime", renderThreadTime);
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_RHIThreadTime", UKuroRenderingRuntimeBPPluginBPLibrary.GetRHIThreadTime());
                FKuroPerfSightHelper.PostValueFloat1("CustomPerformance", "[PrePhysics]TickSystem_PresentTime", UKuroRenderingRuntimeBPPluginBPLibrary.GetSwapBufferTime());
            }
        }
    }
}
