// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x02000095.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/895b59601c20664f87486ee538582a59cc5822f7a32883a02c7d782a99fbf07c/source/_global/EntityComponent.cs
// Canonical field context is retained in declaration order. Only
// EntityComponent.Tick is projected below; unselected members are omitted.

using System;
using System.Collections.Generic;
using System.Reflection;

public abstract class EntityComponent : IComponentDependency, IEntityComponentCleaner
{
    // OBSERVED canonical field context, retained in declaration order.
    public const int DISABLE_REASON_LENGTH_LIMIT = 4;

    public HashSet<string>? UnResetPropertySet;

    private Entity? EntityInternal;

    private int DisableCount;

    private readonly Dictionary<int, string> DisableHandlesMap = new Dictionary<int, string>();

    private readonly Dictionary<EEntityDisableKey, int> DisableKeysMap = new Dictionary<EEntityDisableKey, int>();

    private readonly Stat? CreateStat;

    private readonly Stat? InitStat;

    private readonly Stat? ClearStat;

    private readonly Stat? StartStat;

    private readonly Stat? EndStat;

    private readonly Stat? ActivateStat;

    private readonly Stat? StatTick;

    private readonly Stat? StatForceTick;

    private readonly Stat? StatAfterTick;

    private readonly Stat? StatForceAfterTick;

    private bool ActiveInternal = true;

    private readonly bool NeedEnableInternal;

    private readonly bool NeedDisableInternal;

    private readonly bool NeedTickInternal;

    private readonly bool NeedForceTickInternal;

    private readonly bool NeedAfterTickInternal;

    private readonly bool NeedForceAfterTickInternal;

    private readonly bool NeedChangeTimeDilationInternal;

    // MethodDef 0x060003C0; RVA 0x16c74; code size 221.
    // DIRECT_IL_VERIFIED; filtered System.Exception handler and its logging
    // payload are observed. Evidence: PERF08_DIRECT_CIL.txt:1102-1205.
    public void Tick(float delta)
    {
        if (!Active)
        {
            return;
        }
        try
        {
            OnTick(delta);
        }
        catch (Exception ex) when (1 != 0)
        {
            Log instance = Singleton<Log>.Instance;
            global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
            global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("Id", Entity.Id);
            global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("entity", Entity.GetType().Name);
            global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("component", GetType().Name);
            global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("error", ex.Message);
            instance.ErrorWithStack((ELogModule)9, (ELogAuthor)1, "组件 Tick 异常", ex, global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
        }
    }
}
