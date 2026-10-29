// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TaskBase TypeDef 0x020034d3.
// Direct CIL for this path is in evidence/performance/PERF02_DIRECT_CIL.txt.
// SOURCE_VERIFIED means the corresponding canonical decompiled member was inspected.
// DIRECT_IL_VERIFIED means a complete successful direct-CIL response is stored.

using System;
using Cysharp.Threading.Tasks;

public abstract class TaskBase
{
    // OBSERVED_FROM_CIL/CANONICAL_SOURCE fields. The projection's nullable syntax
    // is source-like metadata, not a claim about original source spelling.
    protected readonly TaskInterruptOptions InterruptOptions;

    public bool Interrupted;

    protected string? LogPrefix = "";
    protected Func<bool>? InitHandle;
    // Stored in the observed type; no invocation appears in the captured Run body.
    protected Action<bool>? FinishedCallback;
    private readonly UniTaskCompletionSource<bool> TaskPromise;

    // MethodDef 0x0601c90a; SOURCE_VERIFIED; metadata only (not directly captured).
    public string Name { get; }
    // MethodDef 0x0601c90c; SOURCE_VERIFIED; metadata only (not directly captured).
    public ETaskPriority InterruptPriority => InterruptOptions.Priority;
    // MethodDef 0x0601c90d; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool Interruptible => InterruptOptions.Interruptible;
    // MethodDef 0x0601c911; SOURCE_VERIFIED; metadata only (not directly captured).
    public UniTask<bool> Promise => TaskPromise.Task;

    // MethodDef 0x0601c909; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public TaskBase(
        string name,
        Func<bool>? initHandle = null,
        Action<bool>? finishedCallback = null,
        TaskInterruptOptions? interruptOptions = null)
    {
        Name = name;
        InitHandle = initHandle;
        FinishedCallback = finishedCallback;
        InterruptOptions = interruptOptions ?? new TaskInterruptOptions();
        TaskPromise = new UniTaskCompletionSource<bool>();
    }

    // MethodDef 0x0601c90b; SOURCE_VERIFIED; metadata only (not directly captured).
    public void SetLogPrefix(string? logPrefix = null)
    {
        LogPrefix = logPrefix;
    }

    // MethodDef 0x0601c90e; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public void RequestInterrupt()
    {
        if (!Interrupted)
        {
            Interrupted = true;
            (string, object) reference = ("Name", Name);
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)40,
                "TaskBase:请求打断",
                new ReadOnlySpan<(string, object)>(ref reference));
            OnInterrupt();
        }
    }

    // MethodDef 0x0601c90f; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    protected virtual void OnInterrupt()
    {
        (string, object) reference = ("Name", Name);
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskBase:执行打断收尾",
            new ReadOnlySpan<(string, object)>(ref reference));
        InterruptOptions.OnInterrupt?.Invoke();
    }

    // MethodDef 0x0601c910; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public bool Init()
    {
        if (InitHandle != null && !InitHandle())
        {
            return false;
        }

        return OnInit();
    }

    // MethodDef 0x0601c912; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604a4ed is also DIRECT_IL_VERIFIED.
    // The inner catch covers exceptions from OnRun/its await, logs them, and the
    // finally resolves the per-task promise with the false runResult. Exceptions
    // from OnExit or from promise completion escape the outer catch instead; this
    // is not a blanket exception-to-false guarantee for every TaskBase operation.
    // RECONSTRUCTED allocation risk: the tuple array used by the error log is a
    // projection convenience; direct CIL uses an inline-array span local.
    public async UniTask<bool> Run()
    {
        bool runResult = false;
        try
        {
            try
            {
                runResult = await OnRun();
            }
            catch (Exception ex)
            {
                Singleton<Log>.Instance.Error(
                    (ELogModule)80,
                    (ELogAuthor)3,
                    "TaskBase:Run执行异常",
                    new (string, object)[]
                    {
                        ("Name", Name),
                        ("error", ex?.ToString() ?? "")
                    });
            }
            finally
            {
                OnExit();
                TaskPromise.TrySetResult(runResult);
            }

            return runResult;
        }
        catch
        {
            // Kept as the source-level counterpart of the generated outer catch.
            throw;
        }
    }

    // MethodDef 0x0601c913; SOURCE_VERIFIED; abstract signature only (not captured).
    protected abstract UniTask<bool> OnRun();

    // MethodDef 0x0601c914; SOURCE_VERIFIED; metadata only (not directly captured).
    protected virtual bool OnInit()
    {
        return true;
    }

    // MethodDef 0x0601c915; SOURCE_VERIFIED; metadata only (not directly captured).
    protected virtual void OnExit()
    {
    }
}
