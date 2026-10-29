// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200009d.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/8dd686af907f9a24fa8cd71eea5f7b09240b61507f957cf7e86cddb1f861f313/source/_global/TickEntityManager.cs
// Only the eight selected MethodDefs are projected below. Unselected members
// are omitted rather than replaced with stubs.

using System.Collections.Generic;

public class TickEntityManager
{
    // OBSERVED canonical field context, retained in declaration order.
    private readonly Dictionary<int, TickEntityGroup> TickEntityMap = new Dictionary<int, TickEntityGroup>();

    private readonly List<TickEntityGroup> TickEntityQueue = new List<TickEntityGroup>();

    private readonly Dictionary<int, int> EntityPriority = new Dictionary<int, int>();

    private bool Ticking;

    private readonly List<Entity> CreateEntities = new List<Entity>();

    private readonly List<int> DestroyEntities = new List<int>();

    [StaticVariableRuleIgnore]
    private static readonly Stat ForceTickStat = Stat.Create("TickEntityManager.ForceTick");

    [StaticVariableRuleIgnore]
    private static readonly Stat TickStat = Stat.Create("TickEntityManager.Tick");

    [StaticVariableRuleIgnore]
    private static readonly Stat ForceAfterTickStat = Stat.Create("TickEntityManager.ForceAfterTick");

    [StaticVariableRuleIgnore]
    private static readonly Stat AfterTickStat = Stat.Create("TickEntityManager.AfterTick");

    private const int BINARY_SEARCH_THREADHOLD = 4;

    // MethodDef 0x060003F4; RVA 0x1796c; code size 48.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:189-225.
    public void Add(Entity entity, int priority)
    {
        EntityPriority[entity.Id] = priority;
        if (Ticking)
        {
            CreateEntities.Add(entity);
        }
        else
        {
            AddInternal(entity, priority);
        }
    }

    // MethodDef 0x060003F5; RVA 0x179a0; code size 74.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:226-274.
    private void AddInternal(Entity entity, int priority)
    {
        if (!TickEntityMap.TryGetValue(priority, out TickEntityGroup value))
        {
            value = new TickEntityGroup(priority);
            TickEntityMap[priority] = value;
            TickEntityQueue.Insert(SearchQueue(priority), value);
        }
        value.Entities[entity.Id] = entity;
    }

    // MethodDef 0x060003F6; RVA 0x179f8; code size 124.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:275-379.
    private int SearchQueue(int priority)
    {
        int count = TickEntityQueue.Count;
        if (count < BINARY_SEARCH_THREADHOLD)
        {
            for (int i = 0; i < count; i++)
            {
                if (TickEntityQueue[i].Priority < priority)
                {
                    return i;
                }
            }
            return count;
        }
        if (TickEntityQueue[0].Priority < priority)
        {
            return 0;
        }
        int num = 0;
        int num2 = count;
        while (num2 - num > 1)
        {
            int num3 = num + num2 >> 1;
            if (TickEntityQueue[num3].Priority > priority)
            {
                num = num3;
            }
            else
            {
                num2 = num3;
            }
        }
        return num2;
    }

    // MethodDef 0x060003F7; RVA 0x17a80; code size 47.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:380-421.
    public void Delete(int entityId)
    {
        if (EntityPriority.TryGetValue(entityId, out var value))
        {
            if (Ticking)
            {
                DestroyEntities.Add(entityId);
            }
            else
            {
                DeleteInternal(entityId, value);
            }
        }
    }

    // MethodDef 0x060003F8; RVA 0x17abb; code size 38.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:422-451.
    private void DeleteInternal(int entityId, int priority)
    {
        EntityPriority.Remove(entityId);
        TickEntityMap[priority].Entities.Remove(entityId);
    }

    // MethodDef 0x060003FA; RVA 0x17b9c; code size 149.
    // DIRECT_IL_VERIFIED; nested foreach disposal regions are observed. The
    // Ticking reset and deferred drain are after both loops, so an escaping
    // exception can leave them unreached.
    // Evidence: evidence/performance/PERF08_DIRECT_CIL.txt:452-548.
    public void Tick(float delta)
    {
        if (GameBudgetInterfaceController.IsOpen)
        {
            return;
        }
        Ticking = true;
        foreach (TickEntityGroup item in TickEntityQueue)
        {
            foreach (Entity value in item.Entities.Values)
            {
                if (value.Valid && value.IsInit)
                {
                    value.Tick(delta);
                }
            }
        }
        Ticking = false;
        CreateAndDestroyInternal();
    }

    // MethodDef 0x060003FD; RVA 0x17dd4; code size 182.
    // DIRECT_IL_VERIFIED; both list traversals have observed enumerator
    // disposal regions. Evidence: evidence/performance/PERF08_DIRECT_CIL.txt:549-657.
    private void CreateAndDestroyInternal()
    {
        foreach (Entity createEntity in CreateEntities)
        {
            if (createEntity.Valid && EntityPriority.TryGetValue(createEntity.Id, out var value))
            {
                AddInternal(createEntity, value);
            }
        }
        CreateEntities.Clear();
        foreach (int destroyEntity in DestroyEntities)
        {
            if (EntityPriority.TryGetValue(destroyEntity, out var value2))
            {
                DeleteInternal(destroyEntity, value2);
            }
        }
        DestroyEntities.Clear();
    }

    // MethodDef 0x060003FE; RVA 0x17eb4; code size 23.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:658-677.
    public void Clear()
    {
        TickEntityMap.Clear();
        TickEntityQueue.Clear();
    }
}
