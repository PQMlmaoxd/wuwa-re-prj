// PERF08: NONCANONICAL / RECONSTRUCTED / PARTIAL source-like fragment.
// Static at-rest managed reconstruction only; this is not recovered developer
// source and is not a standalone compilation unit.
// Source assembly: private/r5b1/materialized/CSharpScript.dll
// SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; TypeDef 0x0200008c.
// Canonical source:
// src/reconstructed/managed/full/CSharpScript/shards/a0282a0149f13eb23702d07a51b5cc465737fad7e124ac1d11802cd3691e4336/source/_global/Entity.cs
// Canonical field context is retained in declaration order. Only Entity.Tick
// is projected below; unselected members are omitted rather than stubbed.

using System;
using System.Collections.Generic;
using System.Reflection;
using System.Runtime.InteropServices;
using CSharpScript.Core.Common;
using UnrealEngine;

public abstract class Entity : ObjectBase, IGameBudgetManagedObject, IStaticVariableResetter
{
    // OBSERVED canonical field context, retained in declaration order.
    public const int DISABLE_REASON_LENGTH_LIMIT = 4;

    public const int MILLIONSECOND_PER_SECOND = 1000;

    private Stat? CreateStat;

    private Stat? InitDataStat;

    private Stat? InitStat;

    protected Stat? InitStatTdType;

    private Stat? ClearStat;

    protected Stat? ClearStatTdType;

    private Stat? StartStat;

    protected Stat? StartStatTdType;

    private Stat? EndStat;

    protected Stat? EndStatTdType;

    private Stat? ActivateStat;

    private Stat? PostActivateStat;

    protected Stat? ActivateStatTdType;

    private Stat? TickStat;

    protected Stat? TickStatTdType;

    private Stat? AfterTickStat;

    protected Stat? AfterTickStatTdType;

    private EExecutedFlag FlagInternal;

    public List<EntityComponent> Components = new List<EntityComponent>();

    public List<EntityComponent>? OnWasRecentlyRenderComponents;

    public List<EntityComponent>? OnBudgetTickEnableChangeComponents;

    public TickComponentManager TickComponentManager = new TickComponentManager();

    private int DisableCount;

    private float DeltaSeconds;

    private int TickInterval;

    public int LastTickFrame;

    private bool IsEncloseSpaceInternal;

    private float TimeDilationInternal = 1f;

    private static TsGameBudgetGroupConfig? _staticGameBudgetConfigInternal;

    private uint GameBudgetManagedTokenInternal;

    private TsGameBudgetGroupConfig? GameBudgetConfigInternal;

    private float DistanceWithCameraInternal = -1f;

    private readonly List<EntityComponent?> ComponentsRegistry = new List<EntityComponent>();

    private readonly Dictionary<int, string> DisableHandlesMap = new Dictionary<int, string>();

    private readonly Dictionary<EEntityDisableKey, int> DisableKeysMap = new Dictionary<EEntityDisableKey, int>();

    private object PendingEnable;

    private static Dictionary<string, Stat?[]>? _statMap;

    private GCHandle GameBudgetGCHandle;

    // MethodDef 0x06000363; RVA 0x15490; code size 46.
    // DIRECT_IL_VERIFIED: evidence/performance/PERF08_DIRECT_CIL.txt:678-711.
    public virtual void Tick(float delta)
    {
        if (!Active)
        {
            TickComponentManager.ClearDelta();
        }
        else
        {
            TickComponentManager.Tick(TickInterval, delta * TimeDilationInternal);
        }
    }
}
