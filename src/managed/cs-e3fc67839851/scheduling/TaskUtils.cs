// PERF-01: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TaskUtils TypeDef 0x020034d9.

using System;
using System.Collections.Generic;

public static class TaskUtils
{
    // TypeDef 0x020034d9; MethodDef 0x0601c940; original source _global/TaskUtils.cs:6-17.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c940.
    public static List<string> GetNodesWithNoDependencies(
        Dictionary<string, IGraphNodeWithDependencies> graphDependencyMap)
    {
        List<string> nodes = new List<string>();
        foreach (KeyValuePair<string, IGraphNodeWithDependencies> item in graphDependencyMap)
        {
            if (item.Value.DependsOn.Count == 0)
            {
                nodes.Add(item.Key);
            }
        }

        return nodes;
    }

    // TypeDef 0x020034d9; MethodDef 0x0601c941; original source _global/TaskUtils.cs:19-27.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c941.
    private static Dictionary<string, HashSet<string>> GetNewDependsOnMap(
        Dictionary<string, IGraphNodeWithDependencies> graphDependencyMap)
    {
        Dictionary<string, HashSet<string>> copy = new Dictionary<string, HashSet<string>>();
        foreach (KeyValuePair<string, IGraphNodeWithDependencies> item in graphDependencyMap)
        {
            copy[item.Key] = new HashSet<string>(item.Value.DependsOn);
        }

        return copy;
    }

    // TypeDef 0x020034d9; MethodDef 0x0601c942; original source _global/TaskUtils.cs:29-49.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#TaskUtils.TopologicalSort.
    private static List<string> TopologicalSort(
        Dictionary<string, IGraphNodeWithDependencies> graphDependencyMap,
        List<string> nodesWithNoDependencies)
    {
        List<string> result = new List<string>();
        Dictionary<string, HashSet<string>> newDependsOnMap = GetNewDependsOnMap(graphDependencyMap);
        List<string> pending = new List<string>(nodesWithNoDependencies);
        while (pending.Count > 0)
        {
            string nodeId = pending[pending.Count - 1];
            pending.RemoveAt(pending.Count - 1);
            result.Add(nodeId);
            foreach (string dependentId in graphDependencyMap[nodeId].DependedOnBy)
            {
                HashSet<string> dependsOn = newDependsOnMap[dependentId];
                dependsOn.Remove(nodeId);
                if (dependsOn.Count == 0)
                {
                    pending.Add(dependentId);
                }
            }
        }

        return result;
    }

    // TypeDef 0x020034d9; MethodDef 0x0601c943; original source _global/TaskUtils.cs:52-79.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#TaskUtils.GetNodeCumulativePriorities.
    // The reverse walk accumulates own priority + max(0, dependent cumulative scores).
    public static Dictionary<string, int> GetNodeCumulativePriorities(
        Dictionary<string, IGraphNodeWithDependencies> graphDependencyMap,
        List<string> nodesWithNoDependencies)
    {
        Dictionary<string, int> cumulativePriorities = new Dictionary<string, int>();
        List<string> topologicalOrder = TopologicalSort(graphDependencyMap, nodesWithNoDependencies);
        while (topologicalOrder.Count > 0)
        {
            string nodeId = topologicalOrder[topologicalOrder.Count - 1];
            topologicalOrder.RemoveAt(topologicalOrder.Count - 1);
            IGraphNodeWithDependencies node = graphDependencyMap[nodeId];
            int ownPriority = node.Priority.GetValueOrDefault();
            int maximumDependentPriority = 0;
            foreach (string dependentId in node.DependedOnBy)
            {
                if (!cumulativePriorities.ContainsKey(dependentId))
                {
                    throw new Exception("Expected to have already computed the cumulative priority for node " + dependentId);
                }

                int dependentPriority = cumulativePriorities[dependentId];
                if (dependentPriority > maximumDependentPriority)
                {
                    maximumDependentPriority = dependentPriority;
                }
            }

            cumulativePriorities[nodeId] = ownPriority + maximumDependentPriority;
        }

        return cumulativePriorities;
    }

    // TypeDef 0x020034d9; MethodDef 0x0601c944; original source _global/TaskUtils.cs:81-157.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c944.
    private static List<string> SearchForCycleDFS(
        Dictionary<string, IGraphNodeWithDependencies> graph,
        Dictionary<string, bool> visitMap,
        string nodeId)
    {
        List<StackElement> stack = new List<StackElement>
        {
            new StackElement
            {
                Node = nodeId,
                Traversing = false
            }
        };

        while (stack.Count > 0)
        {
            StackElement stackElement = stack[stack.Count - 1];
            if (!stackElement.Traversing)
            {
                if (visitMap.ContainsKey(stackElement.Node))
                {
                    if (visitMap[stackElement.Node])
                    {
                        List<string> traversingNodes = new List<string>();
                        foreach (StackElement item in stack)
                        {
                            if (item.Traversing)
                            {
                                traversingNodes.Add(item.Node);
                            }
                        }

                        int cycleStart = -1;
                        for (int i = 0; i < traversingNodes.Count; i++)
                        {
                            if (traversingNodes[i] == stackElement.Node)
                            {
                                cycleStart = i;
                                break;
                            }
                        }

                        if (cycleStart >= 0)
                        {
                            List<string> cycle = new List<string>();
                            for (int i = cycleStart; i < traversingNodes.Count; i++)
                            {
                                cycle.Add(traversingNodes[i]);
                            }

                            return cycle;
                        }

                        return traversingNodes;
                    }

                    stack.RemoveAt(stack.Count - 1);
                    continue;
                }

                visitMap[stackElement.Node] = true;
                stack[stack.Count - 1] = new StackElement
                {
                    Node = stackElement.Node,
                    Traversing = true
                };
                if (!graph.ContainsKey(stackElement.Node))
                {
                    throw new Exception("Could not find node \"" + stackElement.Node + "\" in the graph");
                }

                foreach (string dependentId in graph[stackElement.Node].DependedOnBy)
                {
                    stack.Add(new StackElement
                    {
                        Node = dependentId,
                        Traversing = false
                    });
                }
            }
            else
            {
                visitMap[stackElement.Node] = false;
                stack.RemoveAt(stack.Count - 1);
            }
        }

        return new List<string>();
    }

    // TypeDef 0x020034d9; MethodDef 0x0601c945; original source _global/TaskUtils.cs:159-185.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0601c945.
    public static OneOf<IGraphNodeWithCyclicDependency, IGraphNodeWithNoCyclicDependency> GraphHasCycles(
        Dictionary<string, IGraphNodeWithDependencies> graphDependencyMap)
    {
        Dictionary<string, bool> visitMap = new Dictionary<string, bool>();
        foreach (KeyValuePair<string, IGraphNodeWithDependencies> item in graphDependencyMap)
        {
            if (!visitMap.ContainsKey(item.Key))
            {
                List<string> cycle = SearchForCycleDFS(graphDependencyMap, visitMap, item.Key);
                if (cycle.Count > 0)
                {
                    return OneOf<IGraphNodeWithCyclicDependency, IGraphNodeWithNoCyclicDependency>.op_Implicit(
                        (IGraphNodeWithCyclicDependency)new GraphNodeWithCyclicDependency
                        {
                            HasCycle = true,
                            Cycle = cycle.ToArray()
                        });
                }
            }
        }

        return OneOf<IGraphNodeWithCyclicDependency, IGraphNodeWithNoCyclicDependency>.op_Implicit(
            (IGraphNodeWithNoCyclicDependency)new GraphNodeWithNoCyclicDependency
            {
                HasCycle = false
            });
    }
}
