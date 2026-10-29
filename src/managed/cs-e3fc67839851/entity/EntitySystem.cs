// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200009f.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/c2e71722616eb2e6e2355745496ea9d50678cb317736263da2a9dabff06d8fe5/source/_global/EntitySystem.cs
// Only the three selected MethodDefs are projected below. Unselected members
// are omitted rather than replaced with stubs.

using System.Collections.Generic;

public class EntitySystem : Singleton<EntitySystem>
{
    // OBSERVED canonical field context, retained in declaration order.
    private readonly TickEntityManager TickEntityManger = new TickEntityManager();

    private readonly TickEntityManager AfterTickEntityManger = new TickEntityManager();

    private TEntityDestroyHandle? DestroyHandle;

    // MethodDef 0x06000407; RVA 0x17fa4; code size 126.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:21-100.
    public T? Create<T>(int priority = 0, IEntityArgs? args = null) where T : Entity
    {
        T val = Singleton<ObjectSystem>.Instance.Create<T>();
        if (!val.Create(args))
        {
            Singleton<ObjectSystem>.Instance.Destroy(val);
            return null;
        }
        if (val.TickComponentManager.NeedTick)
        {
            TickEntityManger.Add(val, priority);
        }
        if (val.TickComponentManager.NeedAfterTick)
        {
            AfterTickEntityManger.Add(val, priority);
        }
        return val;
    }

    // MethodDef 0x0600040F; RVA 0x18184; code size 171.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:101-188.
    public bool Destroy<T>(T entity) where T : Entity
    {
        if (!Singleton<ObjectSystem>.Instance.Destroy(entity))
        {
            return false;
        }
        if (GameBudgetInterfaceController.IsOpen)
        {
            entity.UnregisterFromGameBudgetController();
        }
        if (entity.TickComponentManager.NeedTick)
        {
            TickEntityManger.Delete(entity.Id);
        }
        if (entity.TickComponentManager.NeedAfterTick)
        {
            AfterTickEntityManger.Delete(entity.Id);
        }
        if (!entity.End() || !entity.Clear())
        {
            return false;
        }
        DestroyHandle?.Invoke(entity);
        return true;
    }

    // MethodDef 0x06000415; RVA 0x183bd; code size 13.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:1-20.
    public void Tick(float delta)
    {
        TickEntityManger.Tick(delta);
    }
}
