/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e55 (global::BaseBuffComponent)
 * Scope: seven selected whole method bodies only.
 * Unknown dependencies remain external; AddBuffInner is an explicit
 * PARTIAL_CHAIN boundary. See METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using CSharpScript.Game.Common.Event;

public partial class BaseBuffComponent : EntityComponent, IBuffComponent, IStaticVariableResetter
{
    // Observed state members used by the selected view.
    protected readonly Dictionary<int, ActiveBuffInternal> BuffContainer = new Dictionary<int, ActiveBuffInternal>();
    protected readonly Dictionary<long, HashSet<int>> BuffIdToHandleMap = new Dictionary<long, HashSet<int>>();
    protected readonly HashSet<int> BuffGarbageSet = new HashSet<int>();
    protected static readonly HashSet<long> NoLogBuffSet;
    public ExtraEffectManager? BuffEffectManager { get; set; }

    // BuffLock's setter performs the original deferred garbage bookkeeping; it
    // is an external unselected member and is not restated here.

    // Token 0x0601852c; original public, virtual.
    public virtual void AddBuff(long buffId, AddBuffParam buffParams)
    {
        if (buffId <= 0)
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("buffId", buffId),
                ("创建者", buffParams.InstigatorId),
                ("持有者", GetDebugName())
            ];
            logger.Error(CombatLog.EDebugModule.Buff, owner, "尝试添加buff时传入了不合法的buffId", details);
            return;
        }

        BaseBuffComponent instigatorBuffComponent =
            ModelBase<CreatureModel>.Instance.GetEntity(buffParams.InstigatorId)?.Entity?.GetComponent<BaseBuffComponent>();
        buffId = (instigatorBuffComponent?.GetReplaceBuffId(buffId, buffParams)).GetValueOrDefault(buffId);
        int[] customTypes = ModelBase<GameModeModel>.Instance.InstanceDungeon?.GetCustomTypesArray();
        bool isSpecialDungeon = customTypes != null && Array.IndexOf(customTypes, 5) >= 0;
        if ((!buffParams.PreMessageId.HasValue || buffParams.PreMessageId == 0)
            && !BuffIdUtils.CheckBuffInSpecialList(buffId)
            && !isSpecialDungeon)
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("原因", buffParams.Reason),
                ("buffId", buffId)
            ];
            logger.Error(CombatLog.EDebugModule.Buff, owner, "加Buff上文无效", details);
        }
        else
        {
            buffParams.Level = buffParams.Level ?? instigatorBuffComponent?.GetBuffLevel(buffId);
            if (HasBuffAuthority())
            {
                AddBuffLocal(buffId, buffParams);
            }
            else if (NeedAddBuffOrder(buffId))
            {
                AddBuffOrder(buffId, buffParams);
            }
        }
    }

    // Token 0x0601852d; original public, virtual.
    public virtual int AddBuffLocal(long buffId, AddBuffParam buffParams)
    {
        long instigatorId = buffParams.InstigatorId;
        int? level = buffParams.Level;
        int? outerStackCount = buffParams.OuterStackCount;
        ApplyGEType applyType = buffParams.ApplyType.GetValueOrDefault();
        long? preMessageId = buffParams.PreMessageId;
        long? messageId = buffParams.MessageId;
        float? duration = buffParams.Duration ?? ActiveBuffConfigs.USE_INTERNAL_DURATION;
        int serverId = buffParams.ServerId.GetValueOrDefault(-1);
        bool isIterable = buffParams.IsIterable.GetValueOrDefault(true);
        bool isServerOrder = buffParams.IsServerOrder.GetValueOrDefault();
        string reason = buffParams.Reason;
        long? bulletMessageId = buffParams.BulletMessageId;
        bool bornBuff = buffParams.BornBuff.GetValueOrDefault();
        if (!HasBuffAuthority())
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity owner = base.Entity;
            ReadOnlySpan<(string, object)> details =
            [
                ("buffId", buffId),
                ("施加者", instigatorId),
                ("持有者", GetDebugName()),
                ("前置行为id", preMessageId),
                ("原因", reason)
            ];
            logger.Warn(CombatLog.EDebugModule.Buff, owner, "[local] 模拟端不本地添加buff", details);
            return -1;
        }

        BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(buffId, reason);
        if (buffDefinition == null)
        {
            return -1;
        }

        BaseBuffComponent instigatorBuffComponent =
            ModelBase<CreatureModel>.Instance.GetEntity(instigatorId)?.Entity?.GetComponent<BaseBuffComponent>();
        level = level ?? (instigatorBuffComponent?.GetBuffLevel(buffId)).GetValueOrDefault(1);

        // PARTIAL_CHAIN: AddBuffInner is outside the selected source recovery;
        // preserve the original call and its argument/evaluation order.
        return AddBuffInner(
            buffId,
            buffDefinition,
            instigatorId,
            level.Value,
            outerStackCount,
            applyType,
            preMessageId,
            messageId,
            duration,
            null,
            serverId,
            reason,
            fromServer: false,
            isIterable,
            isServerOrder,
            null,
            bulletMessageId,
            bornBuff);
    }

    // Token 0x0601853c; metadata virtual; original public access.
    public virtual void RemoveBuffWhenTimeout(ActiveBuffInternal buff)
    {
        int stackExpirationRemoveNumber = buff.Config.StackExpirationRemoveNumber;
        RemoveBuffInner(buff.Handle, stackExpirationRemoveNumber, isPrematureRemoval: false, "时间结束自然移除");
        if (stackExpirationRemoveNumber > 0 && buff.IsValid())
        {
            buff.SetDuration();
        }
    }

    // Token 0x0601853d; original public, virtual.
    public virtual int RemoveBuffInner(int handle, int removeStackCount, bool isPrematureRemoval, string? reason, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
    {
        if (!(GetBuffByHandle(handle) is ActiveBuffInternal activeBuff))
        {
            return 0;
        }

        BuffLock++;
        ReadOnlySpan<(string, object)> details =
        [
            ("buffId", activeBuff.Id),
            ("持有者", GetDebugName()),
            ("handle", activeBuff?.Handle),
            ("说明", activeBuff?.Config.Desc),
            ("原因", reason)
        ];
        if (!NoLogBuffSet.Contains(activeBuff.Id))
        {
            Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.Buff, base.Entity, "本地移除buff", details);
        }

        int stackCount = activeBuff.StackCount;
        int newStack = removeStackCount > 0 ? Math.Max(0, stackCount - removeStackCount) : 0;
        if (newStack <= 0)
        {
            try
            {
                OnBuffRemoved(activeBuff, isPrematureRemoval, reason, isServerRequest, preMessageId, instigatorId);
            }
            catch (Exception exception)
            {
                Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff移除时发生异常", exception, details);
            }
        }
        else
        {
            if (!isPrematureRemoval && HasBuffRoutineExpirationLock(activeBuff.Id))
            {
                newStack = stackCount;
            }
            try
            {
                OnBuffStackDecreased(activeBuff, stackCount, newStack, isPrematureRemoval, reason, EBuffStackPeriodResetOverride.Default, instigatorId);
            }
            catch (Exception exception)
            {
                Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff层数降低时发生异常", exception, details);
            }
        }

        // BuffGarbageSet/deferred physical removal is governed by BuffLock's
        // external setter; this method does not claim immediate reclamation.
        BuffLock--;
        return stackCount - newStack;
    }

    // Token 0x0601854c; original protected, virtual.
    protected virtual void OnBuffAdded(ActiveBuffInternal buff, int? stackCount, ApplyGEType applyType, long? preMessageId, float? duration, bool? isActive, int serverId, bool fromServer, bool isIterable, bool isServerOrder, string? reason)
    {
        if (buff == null)
        {
            return;
        }

        BuffDefinition config = buff.Config;
        if (buff.IsInstantBuff())
        {
            ApplyPeriodExecution(buff);
        }
        else
        {
            int handle = buff.Handle;
            BuffContainer[handle] = buff;
            if (!BuffIdToHandleMap.TryGetValue(buff.Id, out HashSet<int> handles))
            {
                handles = new HashSet<int>();
                BuffIdToHandleMap[buff.Id] = handles;
            }
            handles.Add(handle);
            MarkListenerBuff(buff);
            BuffEffectManager?.OnBuffAdded(buff);
        }

        if (!NeedCheck(buff.Config))
        {
            if (!isActive.HasValue)
            {
                CombatLog logger = Singleton<CombatLog>.Instance;
                Entity owner = base.Entity;
                ReadOnlySpan<(string, object)> details =
                [
                    ("buffId", buff.Id),
                    ("handle", buff.Handle)
                ];
                logger.Error(CombatLog.EDebugModule.Buff, owner, "buff激活状态未知", details);
            }
            else
            {
                OnBuffActiveChanged(buff, isActive.Value);
            }
        }
        else
        {
            OnBuffActiveChanged(buff, CheckActivate(buff.Config, buff.GetInstigator()));
        }

        if (buff.IsActive() && config != null && config.Period > 0f && config.ExecutePeriodicOnAdd)
        {
            buff.ResetPeriodTimer(0.0200000014f);
        }
        if (Singleton<Info>.Instance.IsBuildDevelopmentOrDebug)
        {
            base.Entity.GetComponent<CharacterStatisticsComponent>()?.OnBuffAdded(buff);
            base.Entity.GetComponent<CharacterGasDebugComponent>()?.OnBuffAdded(buff);
        }
        CheckWhenBuffChanged(buff.Id);
        foreach (ExtraEffectParameters effectInfo in buff.Config.EffectInfos)
        {
            effectInfo.ExecutionEffect?.OnBuffAddedCallback(buff, isIterable);
        }
        buff.OnTimeScaleChanged(GetTimeScale(), IsPaused());
        ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.BuffStackChanged, buff.Id, buff.Id, 0, buff.StackCount, base.Entity);
        EmitBuffStackInstigatorChanged(buff.Id, 0, buff.StackCount, buff.InstigatorId);
    }

    // Token 0x0601854e; original protected, virtual.
    protected virtual void OnBuffRemoved(ActiveBuffInternal buff, bool isPrematureRemoval, string? reason, bool? isServerRequest = null, long? preMessageId = null, long? instigatorId = null)
    {
        if (buff == null)
        {
            return;
        }

        int handle = buff.Handle;
        int stackCount = buff.StackCount;
        foreach (ExtraEffectParameters effectInfo in buff.Config.EffectInfos)
        {
            effectInfo.ExecutionEffect?.OnBuffRemovedCallback(buff);
        }
        RemoveListenerBuff(buff);
        BuffGarbageSet.Add(handle);
        buff.Destroy();
        CheckWhenBuffChanged(buff.Id);
        BuffEffectManager.OnBuffRemoved(buff, isPrematureRemoval);
        CharacterBuffComponent instigatorBuffComponent = buff.GetInstigatorBuffComponent();
        if ((instigatorBuffComponent == null || instigatorBuffComponent.Valid) && HasBuffAuthority())
        {
            long[] expirationEffects = !isPrematureRemoval
                ? buff.Config?.RoutineExpirationEffects
                : buff.Config?.PrematureExpirationEffects;
            if (expirationEffects != null)
            {
                foreach (long buffId in expirationEffects)
                {
                    AddIterativeBuff(buffId, buff, null, isIterable: true, $"因为Buff{buff.Id}移除导致的添加");
                }
            }

            long[] buffsAddedByStackCountOnRemoved = buff.Config.BuffsAddedByStackCountOnRemoved;
            if (isPrematureRemoval
                && buffsAddedByStackCountOnRemoved != null
                && stackCount - 1 >= 0
                && stackCount - 1 < buffsAddedByStackCountOnRemoved.Length)
            {
                long buffId = buffsAddedByStackCountOnRemoved[stackCount - 1];
                if (buffId != 0L)
                {
                    AddIterativeBuff(buffId, buff, null, isIterable: true, $"buff{buff.Id}移除时根据buff层数获取指定buff");
                }
            }
        }

        ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.BuffStackChanged, buff.Id, buff.Id, stackCount, 0, base.Entity);
        EmitBuffStackInstigatorChanged(buff.Id, stackCount, 0, instigatorId);
    }

    // Token 0x06018550; original protected, virtual.
    protected virtual void OnBuffStackIncreased(ActiveBuffInternal buff, int oldStack, int newStack, long? instigatorId, int level, int? outerStackCount, ApplyGEType applyType, long? preMessageId, float? duration, int serverId, bool isIterable, bool isServerOrder, string? reason, EBuffStackDurationOverride stackDurationRefreshPolicy = EBuffStackDurationOverride.Default, EBuffStackPeriodResetOverride stackPeriodResetPolicy = EBuffStackPeriodResetOverride.Default)
    {
        if (buff == null)
        {
            return;
        }

        buff.SetStackCount(newStack, stackPeriodResetPolicy);
        if (stackDurationRefreshPolicy == EBuffStackDurationOverride.Default
            && buff.Config.StackDurationRefreshPolicy == EBuffStackDurationRefreshPolicy.Refresh
            && (NeedCheck(buff.Config) || applyType != 0))
        {
            buff.SetDuration(duration);
        }

        BuffDefinition config = buff.Config;
        int handle = buff.Handle;
        if (config == null)
        {
            return;
        }
        _ = buff.Id;
        _ = 0;
        BuffEffectManager.OnStackIncreased(buff, newStack, oldStack, instigatorId);
        if (HasBuffAuthority())
        {
            int stackLimit = buff.StackLimitCount > 0 ? buff.StackLimitCount : int.MaxValue;
            if (oldStack >= stackLimit && newStack >= stackLimit)
            {
                long[] overflowEffects = config.OverflowEffects;
                if (overflowEffects != null && overflowEffects.Length != 0)
                {
                    foreach (long buffId in overflowEffects)
                    {
                        AddIterativeBuff(buffId, buff, null, isIterable: true, $"Buff层数溢出时迭代添加新buff（前置buff Id={buff.Id}, handle={handle}）");
                    }
                }
                if (buff.Config.ClearStackOnOverflow)
                {
                    RemoveBuffByHandle(handle, -1, "Buff层数溢出时清除");
                }
            }
        }
        ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.BuffStackChanged, buff.Id, buff.Id, oldStack, newStack, base.Entity);
        EmitBuffStackInstigatorChanged(buff.Id, oldStack, newStack, instigatorId);
    }
}
