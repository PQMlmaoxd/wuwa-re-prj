// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; AsyncTask TypeDef 0x020034c6.
// Direct CIL for this path is in evidence/performance/PERF02_DIRECT_CIL.txt.
// AsyncTask is an authored delegate wrapper, not compiler-lowering itself. Only
// its async OnRun member has a compiler-generated state machine.

using System;
using Cysharp.Threading.Tasks;

public class AsyncTask : TaskBase
{
    // OBSERVED_FROM_CIL/CANONICAL_SOURCE field.
    private readonly Func<UniTask<bool>> RunHandle;

    // MethodDef 0x0601c8d9; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public AsyncTask(
        string name,
        Func<UniTask<bool>> runHandle,
        Func<bool>? initHandle = null,
        Action<bool>? finishedCallback = null,
        TaskInterruptOptions? interruptOptions = null)
        : base(name, initHandle, finishedCallback, interruptOptions)
    {
        RunHandle = runHandle;
    }

    // MethodDef 0x0601c8da; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604a4e5 is also DIRECT_IL_VERIFIED.
    protected override async UniTask<bool> OnRun()
    {
        return await RunHandle();
    }
}
