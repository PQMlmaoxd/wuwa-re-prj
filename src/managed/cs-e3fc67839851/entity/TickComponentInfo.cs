// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200008a.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/a58e64067460185cbf9759fe7ec41d98b10eda2213d567e5c9924d2c4040d54b/source/_global/TickComponentInfo.cs
// The selected constructor is retained with canonical field context.

public class TickComponentInfo
{
    // OBSERVED canonical field context, retained in declaration order.
    public readonly EntityComponent Component;

    public readonly int Index;

    public readonly int Priority;

    // MethodDef 0x0600031E; RVA 0x12d3e; code size 28.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:1073-1101.
    public TickComponentInfo(EntityComponent component, int index, int priority)
    {
        Component = component;
        Index = index;
        Priority = priority;
    }
}
