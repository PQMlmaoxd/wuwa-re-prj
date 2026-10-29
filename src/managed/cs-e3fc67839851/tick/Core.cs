// PERF-05: NONCANONICAL / RECONSTRUCTED source-like fragment.
// Static at-rest evidence only; this is not recovered original source.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200007c.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/1bb927286cf97bdacde702c35cdca5d539da7c601d79a0ac3effd44db222faab/source/_global/Core.cs
// Direct CIL: evidence/performance/PERF05_DIRECT_CIL.txt.
// Initialize is a readable async projection, not a full state-machine
// lowering. Only the selected PERF05 members are projected.
// Nullable annotations, local names, compiler-closure packing, and logging
// tuple/inline-array syntax are source-like projections, not original layout
// or non-byte-equivalent inline-array storage; they are not game allocations.

using System;
using System.Collections.Generic;
using Aki.Config;
using CSharpScript.Core.Common;
using CSharpScript.Typing;
using Cysharp.Threading.Tasks;
using UnrealEngine;

public partial class Core : Singleton<Core>
{
    // SOURCE_VERIFIED field initializers; the generated constructor itself was
    // not selected as a new capture in PERF05.
    private readonly HashSet<Action<float>> PreTickFunctions = new HashSet<Action<float>>();

    private int NextTickedPriority = 2;

    public bool ForbiddenTickPriority;

    // MethodDef 0x060002ff; RVA 0x10908; code size 63.
    // DIRECT_IL_VERIFIED wrapper. The body below is the source-level async
    // projection of generated MoveNext 0x06046714 at RVA 0x12a9c58 with code
    // size 713, not a claim of original source text. The wrapper and MoveNext
    // are one represented source body for this bounded inventory. Registration
    // precedes WaitFrame #1, ConfigStatementRegister.Init, WaitFrame #2, and
    // the remaining managed subsystem initialization; no rollback is observed
    // if a later step returns false or throws.
    public async UniTask<bool> Initialize(UGameInstance gameInstance)
    {
        Singleton<Log>.Instance.Info((ELogModule)8, (ELogAuthor)1, Singleton<LogoDefine>.Instance.LOGO, default(ReadOnlySpan<(string, object)>));
        Singleton<Log>.Instance.Info((ELogModule)8, (ELogAuthor)1, Singleton<Info>.Instance.Version, default(ReadOnlySpan<(string, object)>));
        Singleton<Log>.Instance.SetLevel(
            Singleton<Info>.Instance.IsBuildShipping || Singleton<Info>.Instance.IsBuildTest
                ? ELogLevel.Info
                : ELogLevel.Debug);
        Stat.EnableCreateWithStack = Singleton<Info>.Instance.IsBuildDevelopmentOrDebug;
        Singleton<GameBudgetInterfaceController>.Instance.InitializeEnvironment(gameInstance.GetWorld());
        FKuroCycleCounter.InitializeEnvironment();
        Singleton<PerfSight>.Instance.Initialize();
        Singleton<Time>.Instance.Initialize();

        if (!Singleton<Net>.Instance.Initialize())
        {
            Singleton<Log>.Instance.Error((ELogModule)8, (ELogAuthor)64, "Net初始化失败", default(ReadOnlySpan<(string, object)>));
            return false;
        }

        Singleton<TickSystem>.Instance.Add(TickPriority2, "CorePriority2", ETickingGroup.TG_PrePhysics, tickEvenPaused: true, priority: 2);
        Singleton<TickSystem>.Instance.Add(TickPriority1, "CorePriority1", ETickingGroup.TG_PrePhysics, tickEvenPaused: true, priority: 1);
        Singleton<TickSystem>.Instance.Add(Tick, "Core", ETickingGroup.TG_PrePhysics, tickEvenPaused: true);
        Singleton<TickSystem>.Instance.Add(AfterTick, "Core", ETickingGroup.TG_PostPhysics, tickEvenPaused: true);

        await WaitFrame();
        if (!ConfigStatementRegister.Init())
        {
            Singleton<Log>.Instance.Error((ELogModule)8, (ELogAuthor)64, "ConfigStatement初始化失败", default(ReadOnlySpan<(string, object)>));
            return false;
        }

        await WaitFrame();
        Singleton<ObjectSystem>.Instance.Initialize();
        Singleton<EntitySystem>.Instance.Initialize();
        Singleton<EntityComponentSystem>.Instance.Initialize();
        Singleton<Application>.Instance.Initialize();
        Singleton<ActorSystem>.Instance.Initialize();
        Singleton<EffectEnvironment>.Instance.Initialize();
        return true;
    }

    // MethodDef 0x06000300; RVA 0x10953; code size 36.
    // DIRECT_IL_VERIFIED.
    public void TickPriority2(float delta)
    {
        if (!Singleton<Core>.Instance.ForbiddenTickPriority)
        {
            Singleton<Core>.Instance.NextTickedPriority = 1;
            Singleton<Time>.Instance.Tick(delta);
        }
    }

    // MethodDef 0x06000301; RVA 0x10978; code size 37.
    // DIRECT_IL_VERIFIED.
    public void TickPriority1(float delta)
    {
        if (Singleton<Core>.Instance.NextTickedPriority == 1)
        {
            Singleton<Core>.Instance.NextTickedPriority = 0;
        }
        else
        {
            Singleton<Core>.Instance.ForbiddenTickPriority = true;
        }
    }

    // MethodDef 0x06000302; RVA 0x109a0; code size 347; enumerator dispose
    // is observed for PreTickFunctions. HashSet enumeration order is not an
    // insertion-order guarantee; per-bucket List addition order is observed,
    // while cross-priority native delivery order remains unknown. No
    // per-callback catch is added.
    public void Tick(float delta)
    {
        if (Singleton<Core>.Instance.NextTickedPriority != 0)
        {
            Singleton<Time>.Instance.Tick(delta);
            Singleton<Core>.Instance.ForbiddenTickPriority = true;
        }
        Singleton<Core>.Instance.NextTickedPriority = -1;
        Singleton<CycleCounter>.Instance.RefreshState();
        float seconds = delta / 1000f;
        Singleton<Net>.Instance.Tick(seconds);

        if (!Singleton<TickSystem>.Instance.IsPaused)
        {
            foreach (Action<float> preTickFunction in Singleton<Core>.Instance.PreTickFunctions)
            {
                preTickFunction(delta);
            }
        }

        TimerSystem.Instance.Tick(delta);
        TimerSystem.FlowTimeInstance.Tick(delta * Singleton<Time>.Instance.TimeDilation * Singleton<Time>.Instance.FlowTimeDilation);
        TimerSystem.GameplayTimeInstance.Tick(delta * Singleton<Time>.Instance.InverseSelfCenteredTimeDilation);

        double serverTimeStamp = Singleton<Time>.Instance.ServerTimeStamp;
        double serverDelta = serverTimeStamp - Singleton<Time>.Instance.LastServerTime;
        Singleton<Time>.Instance.LastServerTime = serverTimeStamp;
        TimerSystem.RealTimeInstance.Tick((float)serverDelta);

        if (!Singleton<TickSystem>.Instance.IsPaused)
        {
            Singleton<EffectEnvironment>.Instance.Tick(delta, Singleton<Info>.Instance.World);
            if (Singleton<Info>.Instance.EnableForceTick)
            {
                Singleton<EntitySystem>.Instance.ForceTick(delta);
            }
            GameBudgetInterfaceController.UpdateBudgetTime(delta);
        }

        UKuroGameBudgetAllocatorCSharpInterface.TickOutside(seconds);
        if (!Singleton<TickSystem>.Instance.IsPaused)
        {
            Singleton<EntitySystem>.Instance.Tick(delta);
        }
    }

    // MethodDef 0x06000303; RVA 0x10b18; code size 93.
    // DIRECT_IL_VERIFIED. Priority state resets precede the remaining work.
    public void AfterTick(float delta)
    {
        Singleton<Core>.Instance.NextTickedPriority = 2;
        Singleton<Core>.Instance.ForbiddenTickPriority = false;
        if (!Singleton<TickSystem>.Instance.IsPaused && Singleton<Info>.Instance.EnableForceTick)
        {
            Singleton<EntitySystem>.Instance.ForceAfterTick(delta);
        }
        UKuroGameBudgetAllocatorCSharpInterface.AfterTickOutside(delta / 1000f);
        if (!Singleton<TickSystem>.Instance.IsPaused)
        {
            Singleton<EntitySystem>.Instance.AfterTick(delta);
        }
    }

    // MethodDef 0x06000304; RVA 0x10b84; code size 55.
    // DIRECT_IL_VERIFIED. HashSet membership controls duplicate registration.
    public void RegisterPreTick(Action<float> func)
    {
        if (PreTickFunctions.Contains(func))
        {
            Singleton<Log>.Instance.Warn((ELogModule)8, (ELogAuthor)7, "[Core.RegisterPreTickFunctions] 已经注册过PreTickfunc", default(ReadOnlySpan<(string, object)>));
        }
        else
        {
            PreTickFunctions.Add(func);
        }
    }

    // MethodDef 0x06000305; RVA 0x10bc8; code size 55.
    // DIRECT_IL_VERIFIED.
    public void UnRegisterPreTick(Action<float> func)
    {
        if (!PreTickFunctions.Contains(func))
        {
            Singleton<Log>.Instance.Warn((ELogModule)8, (ELogAuthor)7, "[Core.UnRegisterPreTick] 未注册的PreTickfunc", default(ReadOnlySpan<(string, object)>));
        }
        else
        {
            PreTickFunctions.Remove(func);
        }
    }

    // MethodDef 0x06000306; RVA 0x10c0c; code size 54.
    // Generated callback MethodDef 0x06046713; RVA 0x12a9c47; code size 13.
    // DIRECT_IL_VERIFIED. The returned TimerHandle is deliberately discarded.
    // The generated closure receives the timer delta and discards it.
    private UniTask WaitFrame()
    {
        UniTaskCompletionSource tcs = new UniTaskCompletionSource();
        TimerSystem.Instance.Next(delegate(float _)
        {
            tcs.TrySetResult();
        });
        return tcs.Task;
    }
}
