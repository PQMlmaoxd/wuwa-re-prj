// PERF-05: NONCANONICAL / RECONSTRUCTED source-like fragment.
// Static at-rest evidence only; this is not recovered original source.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; Ticker TypeDef 0x02000be7.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/00f4c236dff9d012d5a0bce2f569c41e48aae5ae2fe7659df853c9568870b260/source/_global/Ticker.cs
// Direct CIL: evidence/performance/PERF05_DIRECT_CIL.txt.
// Nullable annotations, local names, compiler-closure packing, and logging
// tuple/inline-array syntax are source-like projections, not original layout
// or non-byte-equivalent inline-array storage; they are not game allocations.

using System;
using UnrealEngine;

public partial class Ticker
{
    public int Count;
    public float CoolDown;
    public bool Pause;
    public readonly Stat? StatObj;
    public readonly int Id;
    public readonly Action<float> Handle;
    public readonly ETickingGroup Group;
    public readonly int Priority;
    public readonly string Name;
    public int TickIntervalMs;
    public bool TickEvenPaused;
    public bool IgnoreSelfCenterMode;
    public bool PendingRemove;

    // MethodDef 0x06003240; RVA 0x21d54; code size 105.
    // DIRECT_IL_VERIFIED. The Stat call is retained as the canonical
    // source-verified meaningful stat creation; optional empty arguments in
    // the deep IL signature are not treated as a new allocation claim.
    public Ticker(
        int id,
        Action<float> handle,
        ETickingGroup group,
        int priority,
        string name,
        int tickIntervalMs = 0,
        bool tickEvenPaused = false,
        bool ignoreSelfCenterMode = false)
    {
        Id = id;
        Handle = handle;
        Group = group;
        Priority = priority;
        Name = name;
        TickIntervalMs = tickIntervalMs;
        TickEvenPaused = tickEvenPaused;
        IgnoreSelfCenterMode = ignoreSelfCenterMode;
        StatObj = Stat.CreateNoFlameGraph("In TickSystem." + Name);
    }
}
