// PERF-03: NONCANONICAL / RECONSTRUCTED source-like fragment.
// Source assembly: CSharpScript.dll SHA-256
// e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TimerSystemInstance TypeDef
// 0x02000bed. Direct CIL: evidence/performance/PERF03_DIRECT_CIL.txt.
// Only Next, DoAdd, Tick, and DoRemove are projected. Constructors, helpers,
// and other members remain in the canonical managed source context.

using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Reflection;

public class TimerSystemInstance
{
    public double Now;
    public Dictionary<int, Timer> Timers;
    public PriorityQueue<Timer> Queue;
    public Dictionary<MethodInfo, Stat> StatWeakMap;
    private readonly ConcurrentQueue<(TimerHandle handle, string? reason)> PendingRemoveQueue;
    private readonly FinalizationRegistry<Timer, TimerHandle> Registry;
    private readonly Action<TimerHandle?, string?>? DisposeHandle_FinalizerThread;

    // MethodDef 0x06003271; RVA 0x22ab8; code size 21.
    // DIRECT_IL_VERIFIED. Observed arguments are interval 1 / loop 1 / dilation
    // 1; clock units and next-frame timing are not proven.
    public TimerHandle? Next(TTimerAction action, Stat? stat = null, string? reason = null)
    {
        return DoAdd(action, 1f, 1, 1f, stat, reason);
    }

    // MethodDef 0x0600327e; RVA 0x23140; code size 216.
    // DIRECT_IL_VERIFIED. SOURCE_VERIFIED Stat signature:
    // CreateWithStack(string namePrefix, int skipStackIndex = 0,
    // int maxStackCount = 5). The CIL loads the explicit optional arguments 2
    // and 5; the source-like call retains those arguments.
    private TimerHandle? DoAdd(
        TTimerAction action,
        float interval,
        int loop,
        float dilation,
        Stat? stat,
        string? reason)
    {
        if (action == null)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)22,
                (ELogAuthor)1,
                "定时器处理方法不存在",
                new (string, object)[] { ("handle", action) });
            return null;
        }

        Stat? statFromMethod = null;
        if (stat == null && !StatWeakMap.TryGetValue(action.Method, out statFromMethod))
        {
            statFromMethod = Stat.CreateWithStack("TimerAction", 2, 5);
            StatWeakMap.Add(action.Method, statFromMethod);
        }

        double now = Now;
        TimerHandle timerHandle = new TimerHandle(this);
        Timer timer = new Timer(
            timerHandle.Id,
            action,
            interval,
            loop,
            dilation,
            timerHandle,
            stat ?? statFromMethod,
            reason);
        timer.Now = now;
        timer.Next = now + (double)(interval / dilation);
        Timers[timerHandle.Id] = timer;
        Queue.Push(timer);
        Registry.Register(timer, timerHandle, reason, DisposeHandle_FinalizerThread);
        return timerHandle;
    }

    // MethodDef 0x0600326b; RVA 0x228c4; code size 278.
    // DIRECT_IL_VERIFIED. Now advances first; due items are popped before the
    // state check. Normal timers run/requeue or go through DoRemove. Finalizer
    // removals are drained after the due-timer loop.
    public void Tick(float delta)
    {
        double now = Now + (double)delta;
        Now = now;
        PriorityQueue<Timer> queue = Queue;

        while (!queue.Empty)
        {
            Timer top = queue.Top;
            if (top == null || top.Next > now)
            {
                break;
            }

            queue.Pop();
            if (top.State != EState.Normal)
            {
                continue;
            }

            if (top.Do())
            {
                queue.Push(top);
            }
            else
            {
                DoRemove(top);
            }
        }

        while (!PendingRemoveQueue.IsEmpty)
        {
            if (PendingRemoveQueue.TryDequeue(out (TimerHandle handle, string? reason) result))
            {
                Singleton<Log>.Instance.Error(
                    (ELogModule)22,
                    (ELogAuthor)3,
                    "移除TimerHandle已经被GC的定时器",
                    new (string, object)[]
                    {
                        ("timer", result.handle?.Id),
                        ("reason", result.reason),
                    });
                Remove(result.handle);
            }
        }
    }

    // MethodDef 0x0600327f; RVA 0x23224; code size 46.
    // DIRECT_IL_VERIFIED. Registry unregistration precedes Trash/Clear and the
    // timer dictionary removal; the method always returns true.
    private bool DoRemove(Timer timer)
    {
        Registry.Unregister(timer);
        timer.State = EState.Trash;
        timer.Clear();
        Timers.Remove(timer.Id);
        return true;
    }
}
