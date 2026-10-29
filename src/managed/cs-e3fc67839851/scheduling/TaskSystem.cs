// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TaskSystem TypeDef 0x020034d8.
// Direct CIL for this path is in evidence/performance/PERF02_DIRECT_CIL.txt.
// Generated MoveNext methods are retained as evidence; these bodies project their
// observed state-machine behavior into source-level control flow.
// SOURCE_VERIFIED means the corresponding canonical decompiled member was inspected.
// DIRECT_IL_VERIFIED means a complete successful direct-CIL response is stored.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using Cysharp.Threading.Tasks;

public class TaskSystem : Singleton<TaskSystem>
{
    private const int SELF_FEED_STREAK_THRESHOLD = 100;

    // OBSERVED_FROM_CIL/CANONICAL_SOURCE fields. Nullable annotations, local names,
    // and the source-level factoring below are projection metadata, not claims about
    // the original author's C# spelling.
    private Queue<TaskBase> IncomingQueue = new Queue<TaskBase>();
    private Queue<TaskBase> DrainBuffer = new Queue<TaskBase>();
    private TaskBase? CurrentTask;
    private readonly Dictionary<ETaskPriority, int> QueuedTaskCountsByInterruptPriority =
        new Dictionary<ETaskPriority, int>();
    private int SelfFeedStreak;

    public bool HasLoadingTask;
    public bool Running;
    private CustomPromise<bool>? RunPromise;
    private bool InitState;

    // MethodDef 0x0601c931; SOURCE_VERIFIED; metadata only (not directly captured).
    public UniTask<bool>? Promise => RunPromise?.Promise;

    // MethodDef 0x0601c937; SOURCE_VERIFIED; metadata only (not directly captured).
    private int HighestQueuedInterruptPriority
    {
        get
        {
            for (ETaskPriority priority = ETaskPriority.High;
                 priority >= ETaskPriority.Normal;
                 priority--)
            {
                if (QueuedTaskCountsByInterruptPriority.TryGetValue(priority, out int count) && count != 0)
                {
                    return (int)priority;
                }
            }

            return -1;
        }
    }

    // MethodDef 0x0601c93c; SOURCE_VERIFIED; metadata only (not directly captured).
    private int QueuedSize => IncomingQueue.Size + DrainBuffer.Size;

    // MethodDef 0x0601c932; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public bool Initialize()
    {
        InitState = true;
        return true;
    }

    // MethodDef 0x0601c933; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604a4f8 is also DIRECT_IL_VERIFIED.
    // The await boundaries and RunPromise ownership are compiler-lowered in the
    // original; the direct generated body remains the authority for those details.
    public async UniTask<bool> Run()
    {
        if (!InitState)
        {
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)40,
                "TaskSystem:Run未初始化,直接返回",
                default(ReadOnlySpan<(string, object)>));
            return false;
        }

        if (Running)
        {
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)40,
                "TaskSystem:已在执行,本次Run视为抢占请求",
                default(ReadOnlySpan<(string, object)>));
            TryInterruptCurrent();
            return await RunPromise!.Promise;
        }

        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskSystem:启动新一轮执行",
            default(ReadOnlySpan<(string, object)>));
        Running = true;
        RunPromise = new CustomPromise<bool>();
        return await RunInternal();
    }

    // MethodDef 0x0601c934; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // AddTask initializes/enqueues only. It does not pump.
    // RECONSTRUCTED allocation risk: the tuple array passed to logging below is a
    // projection convenience; the canonical generated body uses inline-array span
    // storage, so this source-like allocation is not an observed runtime cost.
    public void AddTask(TaskBase task)
    {
        if (!InitState)
        {
            return;
        }

        if (!task.Init())
        {
            Log instance = Singleton<Log>.Instance;
            (string, object) reference = ("Name", task.Name);
            instance.Info(
                (ELogModule)80,
                (ELogAuthor)40,
                "TaskSystem:Task初始化失败,不入队",
                new ReadOnlySpan<(string, object)>(ref reference));
            return;
        }

        IncomingQueue.Push(task);
        IncreaseQueuedCount(task.InterruptPriority);
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)3,
            "TaskSystem:入队Task",
            new (string, object)[]
            {
                ("Name", task.Name),
                ("InterruptPriority", task.InterruptPriority),
                ("剩余Task个数", QueuedSize)
            });
    }

    // MethodDef 0x0601c935; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Clear drops queues; it does not resolve RunPromise itself.
    public void Clear()
    {
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskSystem:Clear",
            default(ReadOnlySpan<(string, object)>));
        InitState = false;
        ClearInternal();
    }

    // MethodDef 0x0601c936; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // The current delegate is not aborted here.
    private void ClearInternal()
    {
        Log instance = Singleton<Log>.Instance;
        (string, object) reference = ("剩余Task个数", QueuedSize);
        instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskSystem:清理内部状态",
            new ReadOnlySpan<(string, object)>(ref reference));
        IncomingQueue.Clear();
        DrainBuffer.Clear();
        QueuedTaskCountsByInterruptPriority.Clear();
        CurrentTask = null;
        SelfFeedStreak = 0;
        HasLoadingTask = false;
        Running = false;
    }

    // MethodDef 0x0601c938; SOURCE_VERIFIED; metadata only (not directly captured).
    private void IncreaseQueuedCount(ETaskPriority priority)
    {
        int count = QueuedTaskCountsByInterruptPriority.TryGetValue(priority, out int value) ? value : 0;
        QueuedTaskCountsByInterruptPriority[priority] = count + 1;
    }

    // MethodDef 0x0601c939; SOURCE_VERIFIED; metadata only (not directly captured).
    private void DecreaseQueuedCount(ETaskPriority priority)
    {
        int count = (QueuedTaskCountsByInterruptPriority.TryGetValue(priority, out int value) ? value : 0) - 1;
        QueuedTaskCountsByInterruptPriority[priority] = count > 0 ? count : 0;
    }

    // MethodDef 0x0601c93a; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    private void TryInterruptCurrent()
    {
        TaskBase? currentTask = CurrentTask;
        if (currentTask != null &&
            currentTask.Interruptible &&
            HighestQueuedInterruptPriority > (int)currentTask.InterruptPriority)
        {
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)3,
                "TaskSystem:打断Task",
                new (string, object)[]
                {
                    ("Name", currentTask.Name),
                    ("当前优先级", currentTask.InterruptPriority),
                    ("最高等待优先级", HighestQueuedInterruptPriority)
                });
            currentTask.RequestInterrupt();
        }
    }

    // MethodDef 0x0601c93b; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Incoming tasks remain in the other buffer.
    private void SwapBuffers()
    {
        Queue<TaskBase> drainBuffer = DrainBuffer;
        DrainBuffer = IncomingQueue;
        IncomingQueue = drainBuffer;
        (string, object) reference = ("本轮任务数", DrainBuffer.Size);
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskSystem:交换缓冲区,开始本轮排空",
            new ReadOnlySpan<(string, object)>(ref reference));
    }

    // MethodDef 0x0601c93d; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604a4fc is also DIRECT_IL_VERIFIED.
    private async UniTask YieldFrame()
    {
        Singleton<Log>.Instance.Info(
            (ELogModule)80,
            (ELogAuthor)40,
            "TaskSystem:让出一帧打破自喂",
            default(ReadOnlySpan<(string, object)>));
        CustomPromise<bool> waiter = new CustomPromise<bool>();
        TimerSystem.GameplayTimeInstance.Next(
            delegate(float _)
            {
                waiter.SetResult(true);
            },
            null,
            "TaskSystem.SelfFeedYield");
        await waiter.Promise;
    }

    // MethodDef 0x0601c93e; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    // Generated MoveNext 0x0604a4fa is also DIRECT_IL_VERIFIED.
    // Reconstructed control names such as completedNormally and firstPass are not
    // metadata claims. The nested finally clears CurrentTask/Running and sets the
    // RunPromise to false when an exception leaves the drain loop. The outer catch
    // then completes RunInternal's UniTask exceptionally; it does not ClearInternal.
    private async UniTask<bool> RunInternal()
    {
        bool completedNormally = false;
        try
        {
            (string, object) startReference = ("Task个数", QueuedSize);
            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)3,
                "TaskSystem:开始",
                new ReadOnlySpan<(string, object)>(ref startReference));

            try
            {
                bool firstPass = true;
                while (true)
                {
                    // The initial entry reaches the incoming-buffer check
                    // directly. SelfFeedStreak is not incremented for this
                    // first swap; the threshold counts only post-task feed.
                    if (firstPass)
                    {
                        firstPass = false;
                        if (IncomingQueue.Size > 0)
                        {
                            SwapBuffers();
                            continue;
                        }

                        completedNormally = true;
                        break;
                    }

                    if (DrainBuffer.Size > 0)
                    {
                        TaskBase task = DrainBuffer.Pop()!;
                        DecreaseQueuedCount(task.InterruptPriority);
                        CurrentTask = task;
                        Singleton<Log>.Instance.Info(
                            (ELogModule)80,
                            (ELogAuthor)3,
                            "TaskSystem:执行Task前",
                            new (string, object)[]
                            {
                                ("Name", task.Name),
                                ("剩余Task个数", QueuedSize)
                            });

                        UniTask<bool> taskRun = task.Run();
                        TryInterruptCurrent();
                        bool result = await taskRun;
                        CurrentTask = null;
                        Singleton<Log>.Instance.Info(
                            (ELogModule)80,
                            (ELogAuthor)3,
                            "TaskSystem:执行Task后",
                            new (string, object)[]
                            {
                                ("Name", task.Name),
                                ("Result", result),
                                ("剩余Task个数", QueuedSize)
                            });

                        if (InitState)
                        {
                            continue;
                        }
                    }

                    if (!InitState)
                    {
                        completedNormally = true;
                        break;
                    }

                    if (IncomingQueue.Size > 0)
                    {
                        if (++SelfFeedStreak >= SELF_FEED_STREAK_THRESHOLD)
                        {
                            Singleton<Log>.Instance.Error(
                                (ELogModule)80,
                                (ELogAuthor)3,
                                "TaskSystem:疑似无限自喂Task,让出一帧打破同帧死循环",
                                new (string, object)[]
                                {
                                    ("SelfFeedStreak", SelfFeedStreak),
                                    ("剩余Task个数", IncomingQueue.Size)
                                });
                            await YieldFrame();
                            SelfFeedStreak = 0;
                        }
                    }
                    else
                    {
                        SelfFeedStreak = 0;
                    }

                    // This is deliberately after the yield reset. The generated
                    // MoveNext rechecks IncomingQueue and swaps only when work
                    // remains; it does not add another initialization guard.
                    if (IncomingQueue.Size > 0)
                    {
                        SwapBuffers();
                        continue;
                    }

                    completedNormally = true;
                    break;
                }
            }
            finally
            {
                CurrentTask = null;
                Running = false;
                if (!completedNormally)
                {
                    RunPromise!.SetResult(false);
                }
            }

            if (!InitState)
            {
                (string, object) reference = ("InitState", InitState);
                Singleton<Log>.Instance.Info(
                    (ELogModule)80,
                    (ELogAuthor)3,
                    "TaskSystem:结束",
                    new ReadOnlySpan<(string, object)>(ref reference));
                RunPromise!.SetResult(false);
                ClearInternal();
                return false;
            }

            Singleton<Log>.Instance.Info(
                (ELogModule)80,
                (ELogAuthor)3,
                "TaskSystem:结束",
                default(ReadOnlySpan<(string, object)>));
            RunPromise!.SetResult(true);
            ClearInternal();
            return true;
        }
        catch
        {
            // The generated outer catch completes the async builder exceptionally;
            // the inner finally has already handled RunPromise=false when reached,
            // while this outer catch itself does not call ClearInternal.
            throw;
        }
    }
}
