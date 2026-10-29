// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200008b.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/4081e00c39d7504b86499e63de087ae8000c11666ee8fcbf800942a74e0d5af9/source/_global/TickComponentManager.cs
// Only the five selected MethodDefs are projected below. Unselected members
// are omitted rather than replaced with stubs.

using System.Collections.Generic;

public class TickComponentManager
{
    // OBSERVED canonical field context, retained in declaration order.
    private float DeltaTotal;

    private int TickIntervalCount;

    private float DeltaTotalAfter;

    private int TickIntervalCountAfter;

    private bool NeedTickInternal;

    private bool NeedAfterTickInternal;

    private readonly List<TickComponentInfo> TickComponents = new List<TickComponentInfo>();

    private readonly List<TickComponentInfo> ForceTickComponents = new List<TickComponentInfo>();

    private readonly List<TickComponentInfo> AfterTickComponents = new List<TickComponentInfo>();

    private readonly List<TickComponentInfo> ForceAfterTickComponents = new List<TickComponentInfo>();

    private int ComponentCount;

    // MethodDef 0x06000325; RVA 0x12dd4; code size 164.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:712-806.
    public void Add(EntityComponent entityComponent, int? priority)
    {
        if (entityComponent.NeedTick || entityComponent.NeedForceTick || entityComponent.NeedAfterTick || entityComponent.NeedForceAfterTick)
        {
            TickComponentInfo item = new TickComponentInfo(entityComponent, ComponentCount, priority.GetValueOrDefault());
            if (entityComponent.NeedTick || entityComponent.NeedForceTick)
            {
                TickComponents.Add(item);
            }
            if (entityComponent.NeedForceTick)
            {
                ForceTickComponents.Add(item);
            }
            if (entityComponent.NeedAfterTick || entityComponent.NeedForceAfterTick)
            {
                AfterTickComponents.Add(item);
            }
            if (entityComponent.NeedForceAfterTick)
            {
                ForceAfterTickComponents.Add(item);
            }
            ComponentCount++;
        }
    }

    // MethodDef 0x06000322; RVA 0x12d73; code size 42.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:807-841.
    private static int Compare(TickComponentInfo a, TickComponentInfo b)
    {
        if (a.Priority == b.Priority)
        {
            return a.Index - b.Index;
        }
        return b.Priority - a.Priority;
    }

    // MethodDef 0x06000326; RVA 0x12e84; code size 227.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:842-945.
    // The compiler-generated Comparison cache is not a selected member; the
    // direct method-group calls below are its source-like projection.
    public void Sort()
    {
        NeedTickInternal = TickComponents.Count > 0 || ForceTickComponents.Count > 0;
        NeedAfterTickInternal = AfterTickComponents.Count > 0 || ForceAfterTickComponents.Count > 0;
        TickComponents.Sort(Compare);
        ForceTickComponents.Sort(Compare);
        AfterTickComponents.Sort(Compare);
        ForceAfterTickComponents.Sort(Compare);
    }

    // MethodDef 0x06000328; RVA 0x12fcc; code size 141.
    // DIRECT_IL_VERIFIED; the foreach enumerator disposal region is observed.
    // Evidence: evidence/performance/PERF08_DIRECT_CIL.txt:946-1031.
    public void Tick(int intervalTime, float delta)
    {
        DeltaTotal += delta;
        TickIntervalCount++;
        if (!GameBudgetInterfaceController.IsOpen && TickIntervalCount < intervalTime)
        {
            return;
        }
        foreach (TickComponentInfo tickComponent in TickComponents)
        {
            if (tickComponent.Component.NeedTick)
            {
                tickComponent.Component.Tick(DeltaTotal);
            }
        }
        DeltaTotal = 0f;
        TickIntervalCount = 0;
    }

    // MethodDef 0x06000324; RVA 0x12dc4; code size 12.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:1032-1048.
    public void ClearDelta()
    {
        DeltaTotal = 0f;
    }
}
