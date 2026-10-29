// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; CustomPromise`1 TypeDef 0x0200003b;
// AkiUniTask`1 TypeDef 0x02000039.
// Direct CIL for this path is in evidence/performance/PERF02_DIRECT_CIL.txt.
// SOURCE_VERIFIED means the corresponding canonical decompiled member was inspected.
// DIRECT_IL_VERIFIED means a complete successful direct-CIL response is stored.

using System.Collections.Generic;
using System.Threading;
using Cysharp.Threading.Tasks;

public class AkiUniTask<T>
{
    // OBSERVED_FROM_CIL/CANONICAL_SOURCE field. GetTask appends a new waiter; the
    // list is not cleared or reused by the observed completion methods.
    private readonly List<UniTaskCompletionSource<T>> Tasks =
        new List<UniTaskCompletionSource<T>>(1);

    // MethodDef 0x060000f4; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public UniTask<T> GetTask()
    {
        UniTaskCompletionSource<T> task = new UniTaskCompletionSource<T>();
        Tasks.Add(task);
        return task.Task;
    }

    // MethodDef 0x060000f5; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Existing waiters are all fanned out.
    public void TrySetResult(T result)
    {
        foreach (UniTaskCompletionSource<T> task in Tasks)
        {
            task.TrySetResult(result);
        }
    }

    // MethodDef 0x060000f6; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public void TrySetCanceled()
    {
        foreach (UniTaskCompletionSource<T> task in Tasks)
        {
            task.TrySetCanceled(default(CancellationToken));
        }
    }
}

public class CustomPromise<T>
{
    // OBSERVED_FROM_CIL/CANONICAL_SOURCE fields. The implicit constructor is a
    // metadata member (0x06000101) but is not persisted as a separate projection
    // body here.
    private readonly AkiUniTask<T> Tcs = new AkiUniTask<T>();
    private EPromiseState State;
    private bool HasResult;
    private T Result;

    // MethodDef 0x060000fd; SOURCE_VERIFIED; UNKNOWN_DIRECT_CIL.
    // The direct disassembly request failed; the body below is bounded by canonical
    // source and field/call-site evidence, not by a fabricated parser result.
    public UniTask<T> Promise
    {
        get
        {
            if (HasResult)
            {
                return UniTask.FromResult(Result);
            }

            return Tcs.GetTask();
        }
    }

    // MethodDef 0x060000ff; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool IsFulfilled => State == EPromiseState.Fulfilled;
    // MethodDef 0x06000100; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool IsPending => State == EPromiseState.Pending;

    // MethodDef 0x060000fe; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Repeated completion is ignored by state.
    public void SetResult(T value)
    {
        if (State == EPromiseState.Pending)
        {
            HasResult = true;
            Result = value;
            State = EPromiseState.Fulfilled;
            Tcs.TrySetResult(value);
        }
    }
}
