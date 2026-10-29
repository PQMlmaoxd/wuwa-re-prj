/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e71 (global::ActiveBuffInternal)
 * Scope: nine selected whole method bodies only.
 * The six R11 modifier-lifecycle bodies are readable source views; the
 * enclosing class and unselected members/callees remain external.
 * See METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;

public partial class ActiveBuffInternal : IActiveBuff, IStaticVariableResetter
{
    // Observed canonical declarations used by this selected view.
    private const int MAX_POOL_COUNT = 100;
    private static List<ActiveBuffInternal>? _buffPool;

    private BuffDefinition ConfigInternal;
    private int HandleInternal = -1;
    private long? _preMessageId;
    private long _messageId = -1L;
    private string OwnerDebugName = "";
    private BaseBuffComponent? OwnerBuffCompInternal;
    private bool RemoveBuffWhenTimeout = true;
    private long InstigatorIdInternal;
    private int ServerIdInternal = -1;
    private long OriginalBuffIdInternal;
    private int? ConfigOverrideListenerId;
    protected Dictionary<int, float> BuffTimeScaleMap = new Dictionary<int, float>();
    protected float BuffTimeScale = 1f;
    protected TimerHandle? DurationTimer;
    private float DurationInternal = 1f;
    private readonly Stat ResetDurationTimerStat = Stat.Create("ActiveBuff.ResetDurationTimer");
    private float InnerCreateTimestamp;
    private float LastTimestamp;
    private float LastRemainingDuration;
    private float Timescale = 1f;
    private bool PrepareToDestroy;
    public Dictionary<int, HashSet<int>>? ExecutionStateData;
    private float PeriodUpdateTimestamp;
    private float LastRemainingPeriod;
    protected float PeriodInternal;
    private TimerHandle? PeriodTimer;
    private readonly Stat ResetPeriodTimerStat = Stat.Create("ActiveBuff.ResetPeriodTimer");
    private bool IsActiveInternal;

    [StaticVariableRuleIgnore]
    private static readonly Stat SetActivateEnableStat;

    [StaticVariableRuleIgnore]
    private static readonly Stat SetActivateDisableStat;

    protected int StackCountInternal;
    public int StackLimitCount;
    private int LevelInternal;
    private readonly List<(int AttributeId, int ModifierHandle)> NonStateModifierHandles = new List<(int, int)>();
    public readonly List<(IBuffModifierData Modifier, float? Snapshot)> StateModifiers = new List<(IBuffModifierData, float?)>();

    [StaticVariableRuleIgnore]
    private static readonly Stat ClearModifiersStat;

    [StaticVariableRuleIgnore]
    private static readonly Stat ResetModifiersStat;

    [StaticVariableRuleIgnore]
    private static readonly Stat SetStateModifiersStat;

    [StaticVariableRuleIgnore]
    private static readonly Stat SetNonStateModifierStat;

    [StaticVariableRuleIgnore]
    private static readonly Stat ModifyStateAttributeStat;

    public BuffDefinition Config => ConfigInternal;
    public int Handle => HandleInternal;
    public long? PreMessageId => _preMessageId;
    public long? MessageId => _messageId;
    public long? InstigatorId => InstigatorIdInternal;
    public long Id => OriginalBuffIdInternal;
    public int ServerId => ServerIdInternal;
    public long OriginalBuffId => OriginalBuffIdInternal;
    public float Duration => DurationInternal;
    public float CreateTimestamp => InnerCreateTimestamp;
    public float Period => PeriodInternal;
    public int StackCount => StackCountInternal;
    public int Level => LevelInternal;

    // Token 0x060186fd; original private, non-virtual.
    private void ResetDurationTimer(float remainingTime)
    {
        ClearDurationTimer();
        if (!(DurationInternal <= 0f))
        {
            LastRemainingDuration = remainingTime;
            LastTimestamp = GetCurrentTime();
            if (RemoveBuffWhenTimeout && Timescale > 0f)
            {
                float interval = LastRemainingDuration / Timescale * 1000f;
                Stat resetDurationTimerStat = ResetDurationTimerStat;
                DurationTimer = interval >= 20f
                    ? TimerSystem.Instance.Delay(DurationCallback, interval, resetDurationTimerStat, null, needCheckMaxInterval: false)
                    : TimerSystem.Instance.Next(DurationCallback, resetDurationTimerStat);
            }
        }
    }

    // Token 0x06018708; original protected, non-virtual.
    protected void DurationCallback(float delta)
    {
        if (IsValid())
        {
            GetOwnerBuffComponent()?.RemoveBuffWhenTimeout(this);
        }
    }

    // Token 0x06018710; original public, non-virtual.
    public void SetStackCount(int newStack, EBuffStackPeriodResetOverride stackPeriodResetPolicy)
    {
        BuffDefinition config = Config;
        int oldStack = StackCountInternal;
        StackCountInternal = newStack;
        BaseTagComponent tagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
        if (tagComponent == null)
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity? owner = GetOwner();
            ReadOnlySpan<(string, object)> details =
            [
                ("handle", Handle),
                ("buffId", Id),
                ("持有者", OwnerBuffCompInternal?.GetDebugName())
            ];
            logger.Error(CombatLog.EDebugModule.Buff, owner, "buff更改层数时无法获取到持有者", details);
            return;
        }

        if (stackPeriodResetPolicy == EBuffStackPeriodResetOverride.Default
            && config.StackPeriodResetPolicy == EBuffStackPeriodResetPolicy.Refresh)
        {
            SetPeriod();
        }
        ResetModifiers();
        if (!IsActiveInternal)
        {
            return;
        }

        int[] grantedTags = config.GrantedTags ?? Array.Empty<int>();
        foreach (int tagId in grantedTags)
        {
            if (GameplayTagUtils.GetGameplayTagById(tagId).HasValue)
            {
                // Modifier/tag behavior is delegated to the observed external component.
                tagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, newStack - oldStack);
            }
        }
    }

    // Token 0x060186e0; original private, non-virtual; canonical source lines 172-215.
    private ActiveBuffInternal Init(BuffDefinition config, int handle, long? instigatorId, BaseBuffComponent ownerBuffComp, int serverId, long? preMessageId, long? messageId, int level, int stackCount, float? duration, ApplyGEType applyType, long originalBuffId)
    {
        ConfigInternal = config;
        HandleInternal = handle;
        InstigatorIdInternal = instigatorId.GetValueOrDefault();
        OwnerBuffCompInternal = ownerBuffComp;
        OwnerDebugName = ownerBuffComp?.GetDebugName() ?? "unknown";
        RemoveBuffWhenTimeout = applyType == ApplyGEType.Common && ownerBuffComp.NeedCheck(config);
        ServerIdInternal = serverId;
        _messageId = messageId ?? ModelBase<CombatMessageModel>.Instance.GenMessageId();
        _preMessageId = preMessageId;
        LevelInternal = level;
        StackCountInternal = stackCount;
        IsActiveInternal = false;
        PrepareToDestroy = false;
        ExecutionStateData?.Clear();
        ExecutionStateData = null;
        ConfigOverrideListenerId = null;
        OriginalBuffIdInternal = originalBuffId;
        StackLimitCount = config.StackLimitCount;
        InnerCreateTimestamp = GetCurrentTime();
        SetDuration(duration);
        SetPeriod();
        if (IsInstantBuff())
        {
            IBuffModifierData[] modifiers = Config.Modifiers;
            foreach (IBuffModifierData buffModifierData in modifiers)
            {
                EAttributeType attributeId = buffModifierData.AttributeId;
                if (CharacterAttributeTypes.stateAttributeIds.Contains(attributeId))
                {
                    SetStateModifier(buffModifierData);
                }
                else
                {
                    SetNonStateModifier(buffModifierData);
                }
            }
        }
        else
        {
            ResetModifiers();
        }
        return this;
    }

    // Token 0x060186e1; original public, non-virtual; canonical source lines 218-240.
    public void Destroy()
    {
        if (IsActive())
        {
            BaseTagComponent tagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
            if (tagComponent != null && tagComponent.Valid)
            {
                int[] grantedTags = Config.GrantedTags ?? Array.Empty<int>();
                foreach (int tagId in grantedTags)
                {
                    tagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, -StackCount);
                }
            }
        }
        PrepareToDestroy = true;
        ClearDurationTimer();
        ClearPeriodCallback();
        ClearModifiers();
        ClearBuffTimeScale();
        ExecutionStateData?.Clear();
        ExecutionStateData = null;
        StackCountInternal = 0;
    }

    // Token 0x0601870e; original public, non-virtual; canonical source lines 692-748.
    public bool SetActivate(bool isActive)
    {
        if (IsActiveInternal == isActive)
        {
            return false;
        }
        IsActiveInternal = isActive;
        BaseTagComponent tagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
        if (tagComponent == null)
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity? owner = GetOwner();
            ReadOnlySpan<(string, object)> details =
            [
                ("handle", Handle),
                ("buffId", Id),
                ("持有者", OwnerBuffCompInternal?.GetDebugName())
            ];
            logger.Error(CombatLog.EDebugModule.Buff, owner, "buff更改激活状态时无法获取到持有者", details);
            return false;
        }
        if (isActive)
        {
            ResetModifiers();
            int[] grantedTags = Config.GrantedTags ?? Array.Empty<int>();
            foreach (int tagId in grantedTags)
            {
                if (GameplayTagUtils.GetGameplayTagById(tagId).HasValue)
                {
                    tagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, StackCount);
                }
            }
            if (PeriodInternal > 0f)
            {
                switch (Config.PeriodicInhibitionPolicy)
                {
                case EBuffPeriodicInhibitionPolicy.ResetAndExecute:
                    ResetPeriodTimer(0.02f);
                    break;
                case EBuffPeriodicInhibitionPolicy.Reset:
                    ResetPeriodTimer(PeriodInternal);
                    break;
                }
            }
        }
        else
        {
            ClearModifiers();
            int[] grantedTags = Config.GrantedTags ?? Array.Empty<int>();
            foreach (int tagId in grantedTags)
            {
                if (GameplayTagUtils.GetGameplayTagById(tagId).HasValue)
                {
                    tagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, -StackCount);
                }
            }
        }
        return true;
    }

    // Token 0x06018712; original public, non-virtual; canonical source lines 786-799.
    public void ClearModifiers()
    {
        StateModifiers.Clear();
        BaseAttributeComponent baseAttributeComponent = GetOwner()?.GetComponent<BaseAttributeComponent>();
        if (NonStateModifierHandles.Count <= 0 || baseAttributeComponent == null)
        {
            return;
        }
        foreach (var nonStateModifierHandle in NonStateModifierHandles)
        {
            baseAttributeComponent.RemoveModifier((EAttributeType)nonStateModifierHandle.AttributeId, nonStateModifierHandle.ModifierHandle);
        }
        NonStateModifierHandles.Clear();
    }

    // Token 0x06018713; original public, non-virtual; canonical source lines 801-821.
    public void ResetModifiers()
    {
        ClearModifiers();
        if (!IsActiveInternal)
        {
            return;
        }
        IBuffModifierData[] modifiers = Config.Modifiers;
        foreach (IBuffModifierData buffModifierData in modifiers)
        {
            EAttributeType attributeId = buffModifierData.AttributeId;
            if (CharacterAttributeTypes.stateAttributeIds.Contains(attributeId))
            {
                SetStateModifier(buffModifierData);
            }
            else
            {
                SetNonStateModifier(buffModifierData);
            }
        }
    }

    // Token 0x06018715; original private, non-virtual; canonical source lines 853-931.
    private void SetNonStateModifier(IBuffModifierData modifier)
    {
        int stackFactor = StackCountInternal == 0 ? 1 : StackCountInternal;
        BaseAttributeComponent baseAttributeComponent = GetOwner()?.GetComponent<BaseAttributeComponent>();
        if (baseAttributeComponent == null)
        {
            return;
        }
        int modifierHandle = 0;
        EAttributeType attributeId = modifier.AttributeId;
        float levelValue = AbilityUtils.GetLevelValue(modifier.Value1, LevelInternal, 0f);
        float levelValue2 = AbilityUtils.GetLevelValue(modifier.Value2, LevelInternal, 0f);
        switch (modifier.CalculationPolicy[0])
        {
        case 0:
        case 1:
        case 3:
            modifierHandle = baseAttributeComponent.AddModifier(attributeId, new CharacterAttributeTypes.AttributeModifier
            {
                Type = (ECalculationPolicyType)modifier.CalculationPolicy[0],
                Value1 = levelValue * (float)stackFactor
            });
            break;
        case 2:
        case 4:
        case 9:
        {
            ECalculationPolicyType type = (ECalculationPolicyType)modifier.CalculationPolicy[0];
            int sourceAttributeId = modifier.CalculationPolicy[1];
            int sourceType = modifier.CalculationPolicy[2];
            EAttributeBasedFloatCalculationType sourceCalculationType = (EAttributeBasedFloatCalculationType)modifier.CalculationPolicy[3];
            bool snapshotSource = modifier.CalculationPolicy.Length > 4 && modifier.CalculationPolicy[4] != 0;
            float? min = modifier.CalculationPolicy.Length > 5 ? new float?(modifier.CalculationPolicy[5]) : null;
            float? ratio = modifier.CalculationPolicy.Length > 6 ? new float?(modifier.CalculationPolicy[6]) : null;
            float? max = modifier.CalculationPolicy.Length > 7 ? new float?(modifier.CalculationPolicy[7]) : null;
            bool floorAfterRatio = modifier.CalculationPolicy.Length > 8 && modifier.CalculationPolicy[8] == 1;
            BaseAttributeComponent sourceAttributeSet = sourceType == 1 ? GetInstigatorAttributeSet() : GetOwnerAttributeSet();
            long? sourceEntity = sourceType == 1 ? InstigatorId : new long?(0L);
            if (sourceAttributeSet == null || !sourceEntity.HasValue)
            {
                CombatLog logger = Singleton<CombatLog>.Instance;
                Entity? owner = GetOwner();
                ReadOnlySpan<(string, object)> details =
                [
                    ("buffId", Id),
                    ("handle", Handle),
                    ("持有者", OwnerBuffCompInternal?.GetDebugName()),
                    ("施加者", InstigatorId),
                    ("attrId", attributeId)
                ];
                logger.Error(CombatLog.EDebugModule.Buff, owner, "持续型buff设置属性modifier时缺少来源", details);
                return;
            }
            modifierHandle = baseAttributeComponent.AddModifier(attributeId, new CharacterAttributeTypes.AttributeModifier
            {
                Type = type,
                Value1 = levelValue * (float)stackFactor,
                Value2 = levelValue2 * (float)stackFactor,
                SourceEntity = sourceEntity.Value,
                SourceAttributeId = (EAttributeType)sourceAttributeId,
                SourceCalculationType = sourceCalculationType,
                SnapshotSource = snapshotSource ? new float?(AbilityUtils.GetAttrValue(sourceAttributeSet, (EAttributeType)sourceAttributeId, sourceCalculationType)) : null,
                Min = min,
                Ratio = ratio,
                Max = max,
                FloorAfterRatio = floorAfterRatio
            });
            break;
        }
        case 5:
        case 6:
        {
            CombatLog logger = Singleton<CombatLog>.Instance;
            Entity? owner = GetOwner();
            ReadOnlySpan<(string, object)> details =
            [
                ("buffId", Id)
            ];
            logger.Error(CombatLog.EDebugModule.Buff, owner, "不能对非状态属性使用时间膨胀类属性修改", details);
            break;
        }
        }
        NonStateModifierHandles.Add(((int)attributeId, modifierHandle));
    }
}
