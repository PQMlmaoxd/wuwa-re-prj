// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200009c.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/a58e64067460185cbf9759fe7ec41d98b10eda2213d567e5c9924d2c4040d54b/source/_global/TickEntityGroup.cs
// The selected constructor is retained with canonical field context.

using System.Collections.Generic;

public class TickEntityGroup
{
    // OBSERVED canonical field context, retained in declaration order.
    public readonly Dictionary<int, Entity> Entities = new Dictionary<int, Entity>();

    public readonly int Priority;

    // MethodDef 0x060003F3; RVA 0x17952; code size 25.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:1049-1072.
    public TickEntityGroup(int priority)
    {
        Priority = priority;
    }
}
