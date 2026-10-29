using System;
using System.Collections.Generic;
using Aki.Protocol;
using CSharpScript.Game.Module.CombatMessage;

public class ActiveBuffInternal : IActiveBuff, IStaticVariableResetter
{
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

	public static List<ActiveBuffInternal> BuffPool => _buffPool;

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

	static ActiveBuffInternal()
	{
		SetActivateEnableStat = Stat.Create("ActiveBuffInternal.SetActivate_Enable");
		SetActivateDisableStat = Stat.Create("ActiveBuffInternal.SetActivate_Disable");
		ClearModifiersStat = Stat.Create("ActiveBuffInternal.ClearModifiers");
		ResetModifiersStat = Stat.Create("ActiveBuffInternal.ResetModifiers");
		SetStateModifiersStat = Stat.Create("ActiveBuffInternal.SetStateModifier");
		SetNonStateModifierStat = Stat.Create("ActiveBuffInternal.SetNonStateModifier");
		ModifyStateAttributeStat = Stat.Create("ActiveBuffInternal.ModifyStateAttribute");
		StaticVariableRegister.RegisterAndExecute((Action)CreateStaticDefaultValue, (Action)ResetStaticDefaultValue);
	}

	private ActiveBuffInternal(BuffDefinition configInternal)
	{
		ConfigInternal = configInternal;
	}

	public static ActiveBuffInternal AllocBuff(BuffDefinition config, int handle, long? instigatorId, BaseBuffComponent ownerBuffComp, int serverId, long? preMessageId, long? messageId, int level, int stackCount, float? duration, ApplyGEType applyType, long originalBuffId)
	{
		ActiveBuffInternal activeBuffInternal = null;
		if (BuffPool.Count > 0)
		{
			activeBuffInternal = BuffPool[BuffPool.Count - 1];
			BuffPool.RemoveAt(BuffPool.Count - 1);
		}
		if (activeBuffInternal == null)
		{
			activeBuffInternal = new ActiveBuffInternal(config);
		}
		activeBuffInternal.Init(config, handle, instigatorId, ownerBuffComp, serverId, preMessageId, messageId, level, stackCount, duration, applyType, originalBuffId);
		return activeBuffInternal;
	}

	public static void ReleaseBuff(ActiveBuffInternal buff)
	{
		if (buff.IsValid())
		{
			buff.Destroy();
		}
		if (buff != null && BuffPool.Count < 100)
		{
			BuffPool.Add(buff);
		}
	}

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

	public void Destroy()
	{
		if (IsActive())
		{
			BaseTagComponent baseTagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
			if (baseTagComponent != null && baseTagComponent.Valid)
			{
				int[] array = Config.GrantedTags ?? Array.Empty<int>();
				foreach (int tagId in array)
				{
					baseTagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, -StackCount);
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

	public bool IsValid()
	{
		if (!PrepareToDestroy)
		{
			return (GetOwnerBuffComponent()?.GetExactEntity()?.Valid).GetValueOrDefault();
		}
		return false;
	}

	public Entity? GetOwner()
	{
		return OwnerBuffCompInternal?.Entity;
	}

	public string GetOwnerDebugName()
	{
		return OwnerDebugName;
	}

	public IBuffComponent? GetOwnerBuffComponent()
	{
		return OwnerBuffCompInternal;
	}

	public Entity? GetInstigator()
	{
		if (!InstigatorId.HasValue)
		{
			return null;
		}
		return ModelBase<CreatureModel>.Instance.GetEntity(InstigatorId.Value)?.Entity;
	}

	public CharacterBuffComponent? GetInstigatorBuffComponent()
	{
		if (!InstigatorId.HasValue)
		{
			return null;
		}
		return ModelBase<CreatureModel>.Instance.GetEntity(InstigatorId.Value)?.Entity?.GetComponent<CharacterBuffComponent>();
	}

	public CharacterActorComponent? GetInstigatorActorComponent()
	{
		if (!InstigatorId.HasValue)
		{
			return null;
		}
		return ModelBase<CreatureModel>.Instance.GetEntity(InstigatorId.Value)?.Entity?.GetComponent<CharacterActorComponent>();
	}

	public BaseAttributeComponent? GetInstigatorAttributeSet()
	{
		if (!InstigatorId.HasValue)
		{
			return null;
		}
		return ModelBase<CreatureModel>.Instance.GetEntity(InstigatorId.Value)?.Entity?.GetComponent<BaseAttributeComponent>();
	}

	public BaseAttributeComponent? GetOwnerAttributeSet()
	{
		return OwnerBuffCompInternal?.GetEntity()?.GetComponent<BaseAttributeComponent>();
	}

	public int? GetConfigOverrideListenerId()
	{
		return ConfigOverrideListenerId;
	}

	public void SetConfigOverrideListenerId(int listenerId)
	{
		ConfigOverrideListenerId = listenerId;
	}

	public void OnConfigOverrideChanged()
	{
		OwnerBuffCompInternal?.RemoveBuffByHandle(Handle, -1, "配置覆盖条件变化");
	}

	public bool IsInstantBuff()
	{
		return Config.DurationPolicy == EBuffDurationType.Instant;
	}

	public void SetBuffTimeScale(int buffHandle, float timeScale)
	{
		BuffTimeScaleMap[buffHandle] = timeScale;
		RefreshBuffTimeScale();
	}

	public void RemoveBuffTimeScale(int buffHandle)
	{
		if (BuffTimeScaleMap.ContainsKey(buffHandle))
		{
			BuffTimeScaleMap.Remove(buffHandle);
			RefreshBuffTimeScale();
		}
	}

	private void RefreshBuffTimeScale()
	{
		float num = 1f;
		foreach (float value in BuffTimeScaleMap.Values)
		{
			num *= value;
		}
		if (num != BuffTimeScale)
		{
			BuffTimeScale = num;
			float newSpeed = OwnerBuffCompInternal?.GetTimeScale() ?? 1f;
			bool forcePaused = OwnerBuffCompInternal?.IsPaused() ?? false;
			OnTimeScaleChanged(newSpeed, forcePaused);
		}
	}

	private void ClearBuffTimeScale()
	{
		BuffTimeScale = 1f;
		BuffTimeScaleMap.Clear();
	}

	private void ClearDurationTimer()
	{
		if (DurationTimer != null)
		{
			if (TimerSystem.Instance.Has(DurationTimer))
			{
				TimerSystem.Instance.Remove(DurationTimer);
			}
			DurationTimer = null;
		}
	}

	private void ResetDurationTimer(float remainingTime)
	{
		ClearDurationTimer();
		if (!(DurationInternal <= 0f))
		{
			LastRemainingDuration = remainingTime;
			LastTimestamp = GetCurrentTime();
			if (RemoveBuffWhenTimeout && Timescale > 0f)
			{
				float num = LastRemainingDuration / Timescale * 1000f;
				Stat resetDurationTimerStat = ResetDurationTimerStat;
				DurationTimer = ((num >= 20f) ? TimerSystem.Instance.Delay(DurationCallback, num, resetDurationTimerStat, null, needCheckMaxInterval: false) : TimerSystem.Instance.Next(DurationCallback, resetDurationTimerStat));
			}
		}
	}

	protected float GetCurrentTime()
	{
		return (float)Singleton<Time>.Instance.Now;
	}

	public void SetDuration(float? duration = null)
	{
		BuffDefinition config = Config;
		float num = 0.034f;
		if (config.DurationPolicy == EBuffDurationType.Infinite)
		{
			num = -1f;
		}
		else if (duration.HasValue)
		{
			num = duration.Value;
		}
		else if (config.DurationMagnitude.Length == 0 || config.DurationCalculationPolicy.Length == 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity? owner = GetOwner();
			string message = $"Buff 配置为hasDuration 但未配置DurationMagnitude或DurationCalculationPolicy, 强制将时间设为{0.034f}";
			(string, object) reference = ("BuffId", Id);
			instance.Error(CombatLog.EDebugModule.Buff, owner, message, new ReadOnlySpan<(string, object)>(ref reference));
			num = 0.034f;
		}
		else
		{
			float levelValue = AbilityUtils.GetLevelValue(Config.DurationMagnitude, Level, 0f);
			float num2 = OwnerBuffCompInternal?.CalculateDurationRate(Id, GetInstigatorBuffComponent()) ?? 1f;
			int[] durationCalculationPolicy = config.DurationCalculationPolicy;
			switch ((durationCalculationPolicy != null) ? new int?(durationCalculationPolicy[0]) : null)
			{
			case 1:
			{
				if (durationCalculationPolicy.Length < 4)
				{
					CombatLog instance2 = Singleton<CombatLog>.Instance;
					Entity? owner2 = GetOwner();
					string message2 = $"Buff 配置为hasDuration 但未配置DurationMagnitude或DurationCalculationPolicy, 将被重设为{0.034f}";
					global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("BuffId", Id);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", Handle);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("释放者", InstigatorId);
					instance2.Error(CombatLog.EDebugModule.Buff, owner2, message2, global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
					num = 0.034f;
					break;
				}
				float levelValue2 = AbilityUtils.GetLevelValue(config.DurationMagnitude2, Level, 0f);
				int attrId = durationCalculationPolicy[1];
				EBuffAttributeCaptureSource eBuffAttributeCaptureSource = (EBuffAttributeCaptureSource)durationCalculationPolicy[2];
				EAttributeBasedFloatCalculationType attrCalcPolicy = (EAttributeBasedFloatCalculationType)durationCalculationPolicy[3];
				BaseAttributeComponent baseAttributeComponent = ((eBuffAttributeCaptureSource == EBuffAttributeCaptureSource.Instigator) ? GetInstigatorAttributeSet() : GetOwnerAttributeSet());
				if (baseAttributeComponent == null)
				{
					CombatLog instance3 = Singleton<CombatLog>.Instance;
					Entity? owner3 = GetOwner();
					string message3 = $"Buff 找不到周期计算属性来源, 周期将被重设为{0.034f}";
					global::<>y__InlineArray4<(string, object)> buffer2 = default(global::<>y__InlineArray4<(string, object)>);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 0) = ("BuffId", Id);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 1) = ("handle", Handle);
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
					global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer2, 3) = ("释放者", InstigatorId);
					instance3.Error(CombatLog.EDebugModule.Buff, owner3, message3, global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer2, 4));
					num = 0.034f;
				}
				else
				{
					num = Math.Max((AbilityUtils.GetAttrValue(baseAttributeComponent, (EAttributeType)attrId, attrCalcPolicy) * levelValue * 0.0001f + levelValue2) * num2, 0.034f);
				}
				break;
			}
			default:
				num = levelValue * num2;
				break;
			}
		}
		DurationInternal = num;
		ResetDurationTimer(num);
	}

	public void SetRemainDuration(float remainDuration)
	{
		if (Config.DurationPolicy == EBuffDurationType.HasDuration)
		{
			ResetDurationTimer((remainDuration > 0f) ? remainDuration : 0.034f);
		}
	}

	public float GetRemainDuration()
	{
		if (!IsValid())
		{
			return 0f;
		}
		switch (Config?.DurationPolicy)
		{
		case EBuffDurationType.Infinite:
			return -1f;
		case EBuffDurationType.Instant:
			return 0f;
		default:
		{
			float num = (GetCurrentTime() - LastTimestamp) * Timescale / 1000f;
			if (!(DurationInternal < 0f))
			{
				return Math.Max(LastRemainingDuration - num, 0f);
			}
			return -1f;
		}
		}
	}

	public HashSet<int> GetOrCreateExecutionState(int index)
	{
		if (ExecutionStateData == null)
		{
			ExecutionStateData = new Dictionary<int, HashSet<int>>();
		}
		if (!ExecutionStateData.TryGetValue(index, out HashSet<int> value))
		{
			value = new HashSet<int>();
			ExecutionStateData[index] = value;
		}
		return value;
	}

	protected void RefreshPeriodInternal()
	{
		PeriodInternal = Config.Period;
		if (!(PeriodInternal > 0f))
		{
			return;
		}
		float num = OwnerBuffCompInternal?.CalculatePeriodRate(Id, GetInstigatorBuffComponent()) ?? 1f;
		PeriodInternal *= num;
		if (Config.HasBuffPeriodExecution)
		{
			if (PeriodInternal < 0.2f)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity? owner = GetOwner();
				string message = $"目前限制带周期型额外效果的Buff周期最短为{0.2f}，配置周期{PeriodInternal}，已强制修改周期";
				(string, object) reference = ("BuffId", Id);
				instance.Error(CombatLog.EDebugModule.Buff, owner, message, new ReadOnlySpan<(string, object)>(ref reference));
				PeriodInternal = 0.2f;
			}
		}
		else if (PeriodInternal < 0.034f)
		{
			CombatLog instance2 = Singleton<CombatLog>.Instance;
			Entity? owner2 = GetOwner();
			string message2 = $"目前限制Buff周期最短为{0.034f}，配置周期{PeriodInternal}，已强制修改周期";
			(string, object) reference2 = ("BuffId", Id);
			instance2.Error(CombatLog.EDebugModule.Buff, owner2, message2, new ReadOnlySpan<(string, object)>(ref reference2));
			PeriodInternal = 0.034f;
		}
	}

	protected bool SetPeriod()
	{
		RefreshPeriodInternal();
		ResetPeriodTimer(PeriodInternal);
		return true;
	}

	public void OnTimeScaleChanged(float newSpeed, bool forcePaused)
	{
		if (forcePaused)
		{
			newSpeed = 0f;
		}
		else if (!Config.DurationAffectedByBulletTime)
		{
			newSpeed = GetOwnerBuffComponent()?.GetLogicTimeScale() ?? 1f;
		}
		else
		{
			bool flag = false;
			foreach (ExtraEffectParameters effectInfo in Config.EffectInfos)
			{
				if (effectInfo.ExtraEffectId == EExtraEffectId.ModifyLife || effectInfo.ExtraEffectId == EExtraEffectId.PeriodicExtraEffect || effectInfo.ExtraEffectId == EExtraEffectId.SetTimeScale)
				{
					flag = true;
					break;
				}
			}
			if (flag)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity? owner = GetOwner();
				(string, object) reference = ("buffId", Id);
				instance.Warn(CombatLog.EDebugModule.Buff, owner, "带额外效果4、5、17的buff不应受子弹顿帧影响", new ReadOnlySpan<(string, object)>(ref reference));
				newSpeed = GetOwner()?.TimeDilation ?? 1f;
			}
		}
		newSpeed *= BuffTimeScale;
		if (newSpeed != Timescale)
		{
			float currentTime = GetCurrentTime();
			float timescale = Timescale;
			Timescale = newSpeed;
			if (DurationInternal > 0f)
			{
				float num = (currentTime - LastTimestamp) * timescale / 1000f;
				float remainingTime = LastRemainingDuration - num;
				ResetDurationTimer(remainingTime);
			}
			if (PeriodInternal > 0f)
			{
				float num2 = (currentTime - PeriodUpdateTimestamp) * timescale / 1000f;
				float remainingTime2 = LastRemainingPeriod - num2;
				ResetPeriodTimer(remainingTime2);
			}
		}
	}

	protected void DurationCallback(float delta)
	{
		if (IsValid())
		{
			GetOwnerBuffComponent()?.RemoveBuffWhenTimeout(this);
		}
	}

	private void ClearPeriodCallback()
	{
		if (PeriodTimer != null)
		{
			if (TimerSystem.Instance.Has(PeriodTimer))
			{
				TimerSystem.Instance.Remove(PeriodTimer);
			}
			PeriodTimer = null;
		}
	}

	public void ResetPeriodTimer(float remainingTime)
	{
		ClearPeriodCallback();
		if (PeriodInternal <= 0f)
		{
			return;
		}
		LastRemainingPeriod = remainingTime;
		PeriodUpdateTimestamp = GetCurrentTime();
		if (Timescale > 0f)
		{
			float num = LastRemainingPeriod / Timescale * 1000f;
			Stat resetPeriodTimerStat = ResetPeriodTimerStat;
			PeriodTimer = ((num >= 20f) ? TimerSystem.Instance.Delay(delegate
			{
				PeriodCallback();
			}, num, resetPeriodTimerStat, null, needCheckMaxInterval: false) : TimerSystem.Instance.Next(delegate
			{
				PeriodCallback();
			}, resetPeriodTimerStat));
		}
	}

	private void PeriodCallback()
	{
		if (!IsValid())
		{
			return;
		}
		float periodInternal = PeriodInternal;
		float value = GetRemainPeriod().Value;
		int num = (int)Math.Floor(1f - value / periodInternal);
		float remainingTime = (value % periodInternal + periodInternal) % periodInternal;
		RefreshPeriodInternal();
		if (!IsActive())
		{
			ResetPeriodTimer(remainingTime);
			return;
		}
		ResetPeriodTimer(remainingTime);
		IBuffComponent ownerBuffComponent = GetOwnerBuffComponent();
		for (int i = 0; i < num; i++)
		{
			ownerBuffComponent?.ApplyPeriodExecution(this);
		}
	}

	public float? GetRemainPeriod()
	{
		float num = (GetCurrentTime() - PeriodUpdateTimestamp) * Timescale;
		if (PeriodInternal < 0f)
		{
			return null;
		}
		return LastRemainingPeriod - num / 1000f;
	}

	public bool IsActive()
	{
		return IsActiveInternal;
	}

	public bool SetActivate(bool isActive)
	{
		if (IsActiveInternal == isActive)
		{
			return false;
		}
		IsActiveInternal = isActive;
		BaseTagComponent baseTagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
		if (baseTagComponent == null)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity? owner = GetOwner();
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("handle", Handle);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("buffId", Id);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
			instance.Error(CombatLog.EDebugModule.Buff, owner, "buff更改激活状态时无法获取到持有者", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return false;
		}
		if (isActive)
		{
			ResetModifiers();
			int[] array = Config.GrantedTags ?? Array.Empty<int>();
			foreach (int tagId in array)
			{
				if (GameplayTagUtils.GetGameplayTagById(tagId).HasValue)
				{
					baseTagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, StackCount);
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
			int[] array = Config.GrantedTags ?? Array.Empty<int>();
			foreach (int tagId2 in array)
			{
				if (GameplayTagUtils.GetGameplayTagById(tagId2).HasValue)
				{
					baseTagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId2, -StackCount);
				}
			}
		}
		return true;
	}

	public void SetStackCount(int newStack, EBuffStackPeriodResetOverride stackPeriodResetPolicy)
	{
		BuffDefinition config = Config;
		int stackCountInternal = StackCountInternal;
		StackCountInternal = newStack;
		BaseTagComponent baseTagComponent = GetOwnerBuffComponent()?.GetExactEntity()?.CheckGetComponent<BaseTagComponent>();
		if (baseTagComponent == null)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity? owner = GetOwner();
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("handle", Handle);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("buffId", Id);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
			instance.Error(CombatLog.EDebugModule.Buff, owner, "buff更改层数时无法获取到持有者", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return;
		}
		if (stackPeriodResetPolicy == EBuffStackPeriodResetOverride.Default && config.StackPeriodResetPolicy == EBuffStackPeriodResetPolicy.Refresh)
		{
			SetPeriod();
		}
		ResetModifiers();
		if (!IsActiveInternal)
		{
			return;
		}
		int[] array = config.GrantedTags ?? Array.Empty<int>();
		foreach (int tagId in array)
		{
			if (GameplayTagUtils.GetGameplayTagById(tagId).HasValue)
			{
				baseTagComponent.TagContainer.UpdateExactTag(ETagChannel.BattleBuff, tagId, newStack - stackCountInternal);
			}
		}
	}

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

	private void SetStateModifier(IBuffModifierData modifier)
	{
		ECalculationPolicyType eCalculationPolicyType = (ECalculationPolicyType)modifier.CalculationPolicy[0];
		if (eCalculationPolicyType == ECalculationPolicyType.AddFromAttr || eCalculationPolicyType == ECalculationPolicyType.OverrideFromAttr || eCalculationPolicyType == ECalculationPolicyType.AddFromAttrRate)
		{
			int attrId = modifier.CalculationPolicy[1];
			EBuffAttributeCaptureSource eBuffAttributeCaptureSource = (EBuffAttributeCaptureSource)modifier.CalculationPolicy[2];
			EAttributeBasedFloatCalculationType attrCalcPolicy = (EAttributeBasedFloatCalculationType)modifier.CalculationPolicy[3];
			bool num = modifier.CalculationPolicy.Length > 4 && modifier.CalculationPolicy[4] != 0;
			BaseAttributeComponent baseAttributeComponent = ((eBuffAttributeCaptureSource == EBuffAttributeCaptureSource.Instigator) ? GetInstigatorAttributeSet() : GetOwnerAttributeSet());
			if (num && baseAttributeComponent == null)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity? owner = GetOwner();
				global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", Handle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("施加者", InstigatorId);
				instance.Warn(CombatLog.EDebugModule.Buff, owner, "buff找不到属性来源，快照将被取值为0", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
			}
			float? item = (num ? new float?((baseAttributeComponent != null) ? AbilityUtils.GetAttrValue(baseAttributeComponent, (EAttributeType)attrId, attrCalcPolicy) : 0f) : null);
			StateModifiers.Add((modifier, item));
		}
		else
		{
			StateModifiers.Add((modifier, null));
		}
	}

	private void SetNonStateModifier(IBuffModifierData modifier)
	{
		int num = ((StackCountInternal == 0) ? 1 : StackCountInternal);
		BaseAttributeComponent baseAttributeComponent = GetOwner()?.GetComponent<BaseAttributeComponent>();
		if (baseAttributeComponent == null)
		{
			return;
		}
		int item = 0;
		EAttributeType attributeId = modifier.AttributeId;
		float levelValue = AbilityUtils.GetLevelValue(modifier.Value1, LevelInternal, 0f);
		float levelValue2 = AbilityUtils.GetLevelValue(modifier.Value2, LevelInternal, 0f);
		switch (modifier.CalculationPolicy[0])
		{
		case 0:
		case 1:
		case 3:
			item = baseAttributeComponent.AddModifier(attributeId, new CharacterAttributeTypes.AttributeModifier
			{
				Type = (ECalculationPolicyType)modifier.CalculationPolicy[0],
				Value1 = levelValue * (float)num
			});
			break;
		case 2:
		case 4:
		case 9:
		{
			ECalculationPolicyType type = (ECalculationPolicyType)modifier.CalculationPolicy[0];
			int num2 = modifier.CalculationPolicy[1];
			int num3 = modifier.CalculationPolicy[2];
			EAttributeBasedFloatCalculationType eAttributeBasedFloatCalculationType = (EAttributeBasedFloatCalculationType)modifier.CalculationPolicy[3];
			bool flag = modifier.CalculationPolicy.Length > 4 && modifier.CalculationPolicy[4] != 0;
			float? min = ((modifier.CalculationPolicy.Length > 5) ? new float?(modifier.CalculationPolicy[5]) : null);
			float? ratio = ((modifier.CalculationPolicy.Length > 6) ? new float?(modifier.CalculationPolicy[6]) : null);
			float? max = ((modifier.CalculationPolicy.Length > 7) ? new float?(modifier.CalculationPolicy[7]) : null);
			bool floorAfterRatio = modifier.CalculationPolicy.Length > 8 && modifier.CalculationPolicy[8] == 1;
			BaseAttributeComponent baseAttributeComponent2 = ((num3 == 1) ? GetInstigatorAttributeSet() : GetOwnerAttributeSet());
			long? num4 = ((num3 == 1) ? InstigatorId : new long?(0L));
			if (baseAttributeComponent2 == null || !num4.HasValue)
			{
				CombatLog instance2 = Singleton<CombatLog>.Instance;
				Entity? owner2 = GetOwner();
				<>y__InlineArray5<(string, object)> buffer = default(<>y__InlineArray5<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", Handle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", OwnerBuffCompInternal?.GetDebugName());
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 3) = ("施加者", InstigatorId);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 4) = ("attrId", attributeId);
				instance2.Error(CombatLog.EDebugModule.Buff, owner2, "持续型buff设置属性modifier时缺少来源", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray5<(string, object)>, (string, object)>(in buffer, 5));
				return;
			}
			item = baseAttributeComponent.AddModifier(attributeId, new CharacterAttributeTypes.AttributeModifier
			{
				Type = type,
				Value1 = levelValue * (float)num,
				Value2 = levelValue2 * (float)num,
				SourceEntity = num4.Value,
				SourceAttributeId = (EAttributeType)num2,
				SourceCalculationType = eAttributeBasedFloatCalculationType,
				SnapshotSource = (flag ? new float?(AbilityUtils.GetAttrValue(baseAttributeComponent2, (EAttributeType)num2, eAttributeBasedFloatCalculationType)) : null),
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
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity? owner = GetOwner();
			(string, object) reference = ("buffId", Id);
			instance.Error(CombatLog.EDebugModule.Buff, owner, "不能对非状态属性使用时间膨胀类属性修改", new ReadOnlySpan<(string, object)>(ref reference));
			break;
		}
		}
		NonStateModifierHandles.Add(((int)attributeId, item));
	}

	public static void ModifyStateAttribute(IAttributeSet? instigator, BaseAttributeComponent owner, IBuffModifierData modifier, int level, float timeDilation, int stackCount, float? snapshot = null)
	{
		EAttributeType attributeId = modifier.AttributeId;
		if (!CharacterAttributeTypes.stateAttributeIds.Contains(attributeId))
		{
			return;
		}
		float levelValue = AbilityUtils.GetLevelValue(modifier.Value1, level, 0f);
		float levelValue2 = AbilityUtils.GetLevelValue(modifier.Value2, level, 0f);
		switch (modifier.CalculationPolicy[0])
		{
		case 0:
			owner.AddBaseValue(attributeId, levelValue * (float)stackCount);
			break;
		case 1:
		{
			float baseValue = owner.GetBaseValue(attributeId);
			float num2 = levelValue * 0.0001f * (float)stackCount + 1f;
			owner.SetBaseValue(attributeId, baseValue * num2);
			break;
		}
		case 2:
		case 4:
		case 9:
		{
			ECalculationPolicyType eCalculationPolicyType = (ECalculationPolicyType)modifier.CalculationPolicy[0];
			int attrId2 = modifier.CalculationPolicy[1];
			int num3 = modifier.CalculationPolicy[2];
			EAttributeBasedFloatCalculationType attrCalcPolicy2 = (EAttributeBasedFloatCalculationType)modifier.CalculationPolicy[3];
			float? num4 = ((modifier.CalculationPolicy.Length > 5) ? new float?(modifier.CalculationPolicy[5]) : null);
			float? num5 = ((modifier.CalculationPolicy.Length > 6) ? new float?(modifier.CalculationPolicy[6]) : null);
			float? num6 = ((modifier.CalculationPolicy.Length > 7) ? new float?(modifier.CalculationPolicy[7]) : null);
			bool flag = modifier.CalculationPolicy.Length > 8 && modifier.CalculationPolicy[8] == 1;
			object obj;
			if (num3 != 1)
			{
				obj = owner;
			}
			else
			{
				obj = instigator;
			}
			IAttributeSet attributeSet = (IAttributeSet)obj;
			if (attributeSet == null)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity entity = owner.Entity;
				(string, object) reference = ("attrId", attributeId);
				instance.Error(CombatLog.EDebugModule.Buff, entity, "瞬间/周期buff属性修改时缺少来源", new ReadOnlySpan<(string, object)>(ref reference));
				break;
			}
			float num7 = snapshot ?? AbilityUtils.GetAttrValue(attributeSet, (EAttributeType)attrId2, attrCalcPolicy2);
			if (num4.HasValue)
			{
				num7 -= num4.Value;
				if (num7 <= 0f)
				{
					break;
				}
			}
			if (num5.HasValue)
			{
				num7 /= num5.Value;
				if (flag)
				{
					num7 = (float)Math.Floor(num7);
				}
			}
			if (eCalculationPolicyType == ECalculationPolicyType.AddFromAttrRate)
			{
				float baseValue2 = owner.GetBaseValue(attributeId);
				owner.AddBaseValue(attributeId, levelValue * 0.0001f * num7 * 0.0001f * baseValue2 * (float)stackCount);
				break;
			}
			float num8 = levelValue * 0.0001f * num7 + levelValue2;
			if (num6.HasValue && num8 > num6.Value)
			{
				num8 = num6.Value;
			}
			if (eCalculationPolicyType == ECalculationPolicyType.AddFromAttr)
			{
				owner.AddBaseValue(attributeId, num8 * (float)stackCount);
			}
			else
			{
				owner.SetBaseValue(attributeId, num8);
			}
			break;
		}
		case 3:
			owner.SetBaseValue(attributeId, levelValue);
			break;
		case 5:
			owner.AddBaseValue(attributeId, levelValue * timeDilation * (float)stackCount);
			break;
		case 6:
		{
			int attrId = modifier.CalculationPolicy[1];
			EAttributeBasedFloatCalculationType attrCalcPolicy = EAttributeBasedFloatCalculationType.BaseValue;
			float num = snapshot ?? AbilityUtils.GetAttrValue(owner, (EAttributeType)attrId, attrCalcPolicy);
			owner.AddBaseValue(attributeId, (levelValue * 0.0001f * num + levelValue2) * timeDilation * (float)stackCount);
			break;
		}
		case 7:
		case 8:
			break;
		}
	}

	public static void CreateStaticDefaultValue()
	{
		_buffPool = new List<ActiveBuffInternal>();
	}

	public static void ResetStaticDefaultValue()
	{
		_buffPool = null;
	}
}
