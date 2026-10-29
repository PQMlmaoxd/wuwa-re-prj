/*
 * CANDIDATE_ONLY / RECONSTRUCTED source-only view.
 * Method bodies are exact source-byte spans from the selected static evidence.
 * Activation: NOT_APPLIED; equivalence: NOT_ESTABLISHED; publication: DISABLED.
 */
public partial class BaseBuffComponent
{
	// MethodDef 0x06018533; exact source lines 2408-2583.
	public virtual int AddBuffInner(long buffId, BuffDefinition? config, long? instigatorId, int level, int? outerStackCount, ApplyGEType? applyType, long? preMessageId, long? messageId, float? duration, bool? isActive, int serverId, string? reason, bool fromServer, bool isIterable, bool isServerOrder, int? handle, long? bulletMessageId = null, bool? bornBuff = null)
	{
		GetBuffStat(buffId);
		BuffLock++;
		<>y__InlineArray5<(string, object)> buffer = default(<>y__InlineArray5<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 1) = ("创建者id", instigatorId);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", GetDebugName());
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 3) = ("初始激活", isActive);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 4) = ("原因", reason);
		ReadOnlySpan<(string, object)> pairs = global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray5<(string, object)>, (string, object)>(in buffer, 5);
		if (config == null)
		{
			Singleton<CombatLog>.Instance.Error(CombatLog.EDebugModule.Buff, base.Entity, "[local] 添加buff时找不到配置", pairs);
			BuffLock--;
			return -1;
		}
		BuffDefinition buffDefinition = config;
		IConfigOverrideRule configOverrideRule = null;
		if (HasBuffAuthority() && !fromServer && buffDefinition.ConfigOverrides != null && buffDefinition.ConfigOverrides.Length != 0)
		{
			configOverrideRule = ConfigOverrideConditionListener.ResolveOverrideConfigRule(buffDefinition.ConfigOverrides.ToList());
			if (configOverrideRule != null)
			{
				BuffDefinition buffDefinition2 = ControllerBase<BuffController>.Instance.GetBuffDefinition(configOverrideRule.OverrideConfigId, reason);
				if (buffDefinition2 != null)
				{
					buffDefinition = buffDefinition2;
				}
			}
		}
		applyType = applyType.GetValueOrDefault();
		WorldEntity worldEntity = null;
		if (instigatorId.HasValue && instigatorId != 0)
		{
			worldEntity = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId.Value)?.Entity;
			if (worldEntity == null)
			{
				Singleton<CombatLog>.Instance.Warn(CombatLog.EDebugModule.Buff, base.Entity, "[local] 添加buff时找不到施加者，将被丢弃", pairs);
				BuffLock--;
				return -1;
			}
		}
		int num = ((outerStackCount.HasValue && outerStackCount > 0) ? outerStackCount.Value : buffDefinition.DefaultStackCount);
		if (!CheckAdd(buffDefinition, instigatorId.GetValueOrDefault(), fromServer))
		{
			if (worldEntity != null)
			{
				ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent((Entity?)worldEntity, EAbilityEventName.AddBuffFailure, buffId, buffId, base.Entity, (Entity)worldEntity, num, bulletMessageId);
			}
			BuffLock--;
			return -1;
		}
		if (worldEntity != null)
		{
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent((Entity?)worldEntity, EAbilityEventName.AddBuffToOthers, buffId, buffId, base.Entity, (Entity)worldEntity, bornBuff);
		}
		ActiveBuffInternal stackableBuff = GetStackableBuff(instigatorId.GetValueOrDefault(), buffId, buffDefinition.StackingType);
		int num2 = RefreshBuffStackMax(buffId);
		if (stackableBuff == null)
		{
			int num3 = num;
			int? num4 = CalculateBuffStackMaxByConfig(buffDefinition, buffId);
			if (num4.HasValue && num4.GetValueOrDefault() != int.MaxValue)
			{
				num = Math.Min(num, num4.Value);
			}
			ActiveBuffInternal newBuff = null;
			if (buffDefinition.DurationPolicy == EBuffDurationType.Instant)
			{
				handle = -2;
				duration = -1f;
			}
			else
			{
				handle = handle ?? ControllerBase<BuffController>.Instance.GenerateHandle();
			}
			try
			{
				newBuff = ActiveBuffInternal.AllocBuff(buffDefinition, handle.Value, instigatorId, this, serverId, preMessageId, messageId, level, num, duration, applyType.Value, buffId);
			}
			catch (Exception e)
			{
				Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff初始过程中发生异常", e, pairs);
			}
			if (newBuff == null)
			{
				BuffLock--;
				return -1;
			}
			if (num3 > num && num4.HasValue && num4.GetValueOrDefault() != int.MaxValue)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity entity = base.Entity;
				global::<>y__InlineArray3<(string, object)> buffer2 = default(global::<>y__InlineArray3<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 0) = ("原始层数", num3);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 1) = ("最大层数", num4);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 2) = ("截断后层数", num);
				instance.Warn(CombatLog.EDebugModule.Buff, entity, "[local] 新增buff层数超过最大层数限制，已自动截断", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer2, 3));
			}
			if (HasBuffAuthority() && !fromServer && configOverrideRule != null && config.ConfigOverrides != null)
			{
				ConfigOverrideConditionListener.ConfigOverrideListenerRegistration configOverrideListenerRegistration = ConfigOverrideConditionListener.AddListener(config.ConfigOverrides.ToList(), delegate
				{
					newBuff?.OnConfigOverrideChanged();
				}, triggerOnRegister: false, configOverrideRule.OverrideConfigId);
				if (configOverrideListenerRegistration != null)
				{
					newBuff.SetConfigOverrideListenerId(configOverrideListenerRegistration.ListenerId);
				}
			}
			if (!NoLogBuffSet.Contains(buffId))
			{
				CombatLog instance2 = Singleton<CombatLog>.Instance;
				Entity entity2 = base.Entity;
				<>y__InlineArray11<(string, object)> buffer3 = default(<>y__InlineArray11<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 0) = ("buffId", buffId);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 1) = ("创建者id", instigatorId);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 2) = ("持有者", GetDebugName());
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 3) = ("初始激活", isActive);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 4) = ("原因", reason);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 5) = ("handle", handle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 6) = ("前置行为id", preMessageId);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 7) = ("说明", buffDefinition.Desc);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 8) = ("是否迭代", isIterable);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 9) = ("层数", num);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray11<(string, object)>, (string, object)>(ref buffer3, 10) = ("bulletMessage", bulletMessageId);
				instance2.Info(CombatLog.EDebugModule.Buff, entity2, "本地添加buff", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray11<(string, object)>, (string, object)>(in buffer3, 11));
			}
			try
			{
				OnBuffAdded(newBuff, outerStackCount, applyType.Value, preMessageId, duration, isActive, serverId, fromServer, isIterable, isServerOrder, reason);
			}
			catch (Exception e2)
			{
				Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff添加中发生异常", e2, pairs);
			}
			if (buffDefinition.DurationPolicy == EBuffDurationType.Instant)
			{
				UnregisterConfigOverrideListener(newBuff);
				ActiveBuffInternal.ReleaseBuff(newBuff);
			}
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.OnBuffAdd, buffId, buffId, handle.Value);
			BuffLock--;
			return handle.Value;
		}
		if (stackableBuff.Config != null && (num2 <= 0 || !stackableBuff.Config.DenyOverflowAdd || stackableBuff.StackCount < num2))
		{
			if (buffDefinition.StackAppendCount > 0)
			{
				num = buffDefinition.StackAppendCount;
			}
			int stackCount = stackableBuff.StackCount;
			int num5 = stackCount + num;
			int newStack = Math.Min(num5, num2);
			try
			{
				OnBuffStackIncreased(stackableBuff, stackCount, newStack, instigatorId.GetValueOrDefault(), level, outerStackCount, applyType.Value, preMessageId, duration, serverId, isIterable, isServerOrder, reason);
				foreach (ExtraEffectParameters effectInfo in stackableBuff.Config.EffectInfos)
				{
					effectInfo.ExecutionEffect?.OnBuffAddedCallback(stackableBuff, isIterable);
				}
				if (num5 > num2)
				{
					BuffEffectManager?.OnBuffStackOverflow(stackableBuff, stackCount, num5, num2);
				}
			}
			catch (Exception e3)
			{
				Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff层数改变中发生异常", e3, pairs);
			}
		}
		ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.OnBuffAdd, buffId, buffId, stackableBuff.Handle);
		BuffLock--;
		return stackableBuff.Handle;
	}
	// MethodDef 0x0601854a; exact source lines 2945-2984.
	protected ActiveBuffInternal? GetStackableBuff(long instigatorId, long buffId, EBuffStackingType stackingType)
	{
		switch (stackingType)
		{
		case EBuffStackingType.ByOwner:
			return GetBuffById(buffId);
		case EBuffStackingType.ByInstigator:
		{
			if (!BuffIdToHandleMap.TryGetValue(buffId, out HashSet<int> value))
			{
				return null;
			}
			if (value.Count == 0)
			{
				BuffIdToHandleMap.Remove(buffId);
				return null;
			}
			List<int> list = new List<int>();
			foreach (int item in value)
			{
				ActiveBuffInternal valueOrDefault = BuffContainer.GetValueOrDefault(item);
				if (valueOrDefault == null)
				{
					list.Add(item);
				}
				else if (!BuffGarbageSet.Contains(valueOrDefault.Handle) && valueOrDefault.InstigatorId == instigatorId)
				{
					return valueOrDefault;
				}
			}
			foreach (int item2 in list)
			{
				value.Remove(item2);
			}
			return null;
		}
		default:
			return null;
		}
	}
}
