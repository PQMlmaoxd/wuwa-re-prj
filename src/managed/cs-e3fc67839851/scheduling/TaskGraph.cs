// PERF-01: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TaskGraph TypeDef 0x020034d4.
// This file preserves the bounded managed scheduling path. It does not add
// worker, thread, native budget, or ThreadPool machinery not present in the evidence.

using System;
using System.Collections.Generic;
using Cysharp.Threading.Tasks;

public class TaskGraph : IStaticVariableResetter
{
    private readonly Dictionary<string, IGraphNodeWithDependencies> GraphDependencyMap =
        new Dictionary<string, IGraphNodeWithDependencies>();

    private readonly List<string> NodesWithNoDependencies = new List<string>();

    public static bool VisualizeTaskGraph;

    // TypeDef 0x020034d4; MethodDef 0x0601c916; original source _global/TaskGraph.cs:487-490.
    // CIL evidence: retained methods.jsonl record for 0x0601c916; the .cctor's RegisterAndExecute call is at CIL IL_0018; complete critical CIL is not required for this registration wrapper.
    static TaskGraph()
    {
        StaticVariableRegister.RegisterAndExecute((Action)CreateStaticDefaultValue, (Action)ResetStaticDefaultValue);
    }

    // TypeDef 0x020034d4; MethodDef 0x0601c917; original source _global/TaskGraph.cs:492-496.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#TaskGraph.Compare.
    // The comparison is deliberately recomputed on every call.
    private int Compare(string a, string b)
    {
        Dictionary<string, int> nodeCumulativePriorities =
            TaskUtils.GetNodeCumulativePriorities(GraphDependencyMap, NodesWithNoDependencies);
        return nodeCumulativePriorities[a] - nodeCumulativePriorities[b];
    }

    // TypeDef 0x020034d4; MethodDef 0x0601c918; original source _global/TaskGraph.cs:498-545.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c918.
    // A dependency tuple is (prerequisite, dependent).
    public TaskGraph(Dictionary<string, IGraphNode> nodeMap, List<(string, string)> dependencies)
    {
        foreach (KeyValuePair<string, IGraphNode> item in nodeMap)
        {
            GraphDependencyMap[item.Key] = new GraphNodeWithDependenciesImpl
            {
                Run = item.Value.Run,
                Priority = item.Value.Priority,
                DependsOn = new HashSet<string>(),
                DependedOnBy = new HashSet<string>(),
                Failed = false
            };
        }

        foreach ((string, string) dependency in dependencies)
        {
            string prerequisite = dependency.Item1;
            string dependent = dependency.Item2;
            IGraphNodeWithDependencies? prerequisiteNode = GraphDependencyMap.GetValueOrDefault(prerequisite);
            IGraphNodeWithDependencies? dependentNode = GraphDependencyMap.GetValueOrDefault(dependent);
            if (prerequisiteNode == null)
            {
                throw new Exception("检查dependencies参数传入的被依赖ID: " + prerequisite + ", 不在nodeMap中");
            }

            if (dependentNode == null)
            {
                throw new Exception("检查dependencies参数传入的依赖ID " + dependent + ", 不在nodeMap中");
            }

            prerequisiteNode.DependedOnBy.Add(dependent);
            dependentNode.DependsOn.Add(prerequisite);
        }

        NodesWithNoDependencies.AddRange(TaskUtils.GetNodesWithNoDependencies(GraphDependencyMap));
        if (NodesWithNoDependencies.Count == 0 && nodeMap.Count > 0)
        {
            throw new Exception("找不到Task执行起点, 可能有一个依赖链包含了所有的Task");
        }

        OneOf<IGraphNodeWithCyclicDependency, IGraphNodeWithNoCyclicDependency> cycleResult =
            TaskUtils.GraphHasCycles(GraphDependencyMap);
        if (cycleResult.IsT1 && cycleResult.AsT1.HasCycle)
        {
            throw new Exception("检测到了循环依赖:\n" + string.Join("\n", cycleResult.AsT1.Cycle));
        }

        if (cycleResult.IsT2 && cycleResult.AsT2.HasCycle)
        {
            throw new Exception("检测到了循环依赖:");
        }

        _ = VisualizeTaskGraph;
    }

    // TypeDef 0x020034d4; MethodDef 0x0601c919; original source _global/TaskGraph.cs:547-560.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c919.
    // The async state machine is represented here by its
    // source-level shape; generated MoveNext methods are retained in the evidence.
    public async UniTask Run(IRunOptions? options = null)
    {
        int? concurrency = options?.Concurrency;
        if (concurrency.HasValue && concurrency < 0)
        {
            throw new Exception($"Concurrency需要是正数或者不传, 现在: {options?.Concurrency}");
        }

        PriorityQueue<string> priorityQueue = new PriorityQueue<string>(Compare);
        foreach (string nodeId in NodesWithNoDependencies)
        {
            priorityQueue.Push(nodeId);
        }

        int currentlyRunningTaskCount = 0;
        UniTaskCompletionSource tcs = new UniTaskCompletionSource();
        List<Exception> errors = new List<Exception>();

        // Generated MethodDef 0x0604a4f0 is the wrapper (original source _global/TaskGraph.cs:176-190).
        // Its actual nested MoveNext body is MethodDef 0x060506e6 (original source:29-137); complete CIL:
        // evidence/performance/bounded_il_critical.cil.txt#TaskGraph.ScheduleTask.MoveNext.
        async UniTask ScheduleTask()
        {
            string? taskToRunId = priorityQueue.Pop();
            if (taskToRunId == null)
            {
                throw new Exception("尝试调度一个Task的时候, 发现没货了!");
            }

            IGraphNodeWithDependencies taskToRun = GraphDependencyMap[taskToRunId];
            try
            {
                currentlyRunningTaskCount++;
                if (!taskToRun.Failed)
                {
                    await taskToRun.Run();
                }
            }
            catch (Exception ex)
            {
                taskToRun.Failed = true;
                // Observed semantic payload: ("error", ex.Message), ("taskToRun", taskToRun).
                // This direct array-to-span argument is a source-like tuple-span
                // projection; the compiler-generated inline-array storage/lowering is
                // not reconstructed and the array is not a claim about original heap
                // allocation.
                Singleton<Log>.Instance.Error(
                    (ELogModule)39,
                    (ELogAuthor)73,
                    "TaskGraph执行异常",
                    new (string, object)[]
                    {
                        ("error", ex.Message),
                        ("taskToRun", taskToRun)
                    });
                throw;
            }
            finally
            {
                // Finalization is controlled only by the exact nullable gate below:
                // explicit false suppresses decrement/release even after SUCCESS.
                if (options?.ContinueEvenFail ?? !taskToRun.Failed)
                {
                    currentlyRunningTaskCount--;
                    foreach (string dependentId in taskToRun.DependedOnBy)
                    {
                        IGraphNodeWithDependencies dependent = GraphDependencyMap[dependentId];
                        if (taskToRun.Failed)
                        {
                            dependent.Failed = true;
                        }

                        dependent.DependsOn.Remove(taskToRunId);
                        if (dependent.DependsOn.Count == 0)
                        {
                            priorityQueue.Push(dependentId);
                        }
                    }
                }
            }
        }

        // Generated MethodDef 0x0604a4f1; original source _global/TaskGraph.cs:192-237;
        // complete CIL: evidence/performance/bounded_il_critical.cil.txt#TaskGraph.TrySchedulingTasks.
        void TrySchedulingTasks()
        {
            if (!UniTaskStatusExtensions.IsCompleted(tcs.Task.Status))
            {
                if (priorityQueue.Empty && currentlyRunningTaskCount == 0)
                {
                    if (errors.Count == 0)
                    {
                        tcs.TrySetResult();
                    }
                    else
                    {
                        tcs.TrySetException((Exception)new CompositeError(errors));
                    }
                }
                else
                {
                    while (!priorityQueue.Empty &&
                           (!concurrency.HasValue || currentlyRunningTaskCount < concurrency))
                    {
                        // Success continuation MethodDef 0x0604a4f2; original source _global/TaskGraph.cs:217-220.
                        // Error continuation MethodDef 0x0604a4f3; original source _global/TaskGraph.cs:220-232 and 239-250;
                        // complete CIL:
                        // evidence/performance/bounded_il_critical.cil.txt#TaskGraph.error_continuation.
                        UniTaskExtensions.Forget(
                            UniTaskExtensions.ContinueWith(
                                ScheduleTask(),
                                (Action)delegate { TrySchedulingTasks(); }),
                            (Action<Exception>)delegate(Exception ex)
                            {
                                errors.Add(ex);
                                if (options != null && options.ContinueEvenFail.GetValueOrDefault())
                                {
                                    TrySchedulingTasks();
                                }
                                else
                                {
                                    tcs.TrySetException(ex);
                                }
                            },
                            true);
                    }
                }
            }
        }

        TrySchedulingTasks();
        await tcs.Task;
    }

    // TypeDef 0x020034d4; MethodDef 0x0601c91a; original source _global/TaskGraph.cs:562-564.
    // CIL evidence: retained methods.jsonl record for 0x0601c91a.
    public static void CreateStaticDefaultValue()
    {
    }

    // TypeDef 0x020034d4; MethodDef 0x0601c91b; original source _global/TaskGraph.cs:566-568.
    // CIL evidence: retained methods.jsonl record for 0x0601c91b.
    public static void ResetStaticDefaultValue()
    {
        VisualizeTaskGraph = false;
    }
}
