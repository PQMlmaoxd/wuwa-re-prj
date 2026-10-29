using System;
using System.Collections;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using Aki.Protocol;
using CSharpScript.Core.Common;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities.Follow;
using CSharpScript.Typing;
using Google.Protobuf;

public class BaseBuffComponent : EntityComponent, IBuffComponent, IStaticVariableResetter
{
	private struct PendingInstantCueItem
	{
		public long CueId;

		public GameplayCueParam Param;
	}

	[CompilerGenerated]
	private sealed class <GetAllBuffById>d__121 : IEnumerable<ActiveBuffInternal>, IEnumerable, IEnumerator<ActiveBuffInternal>, IEnumerator, IDisposable
	{
		private int <>1__state;

		private ActiveBuffInternal <>2__current;

		private int <>l__initialThreadId;

		public BaseBuffComponent <>4__this;

		private long buffId;

		public long <>3__buffId;

		private HashSet<int> <handleSet>5__2;

		private List<int> <toRemove>5__3;

		private HashSet<int>.Enumerator <>7__wrap3;

		ActiveBuffInternal IEnumerator<ActiveBuffInternal>.Current
		{
			[DebuggerHidden]
			get
			{
				return <>2__current;
			}
		}

		object IEnumerator.Current
		{
			[DebuggerHidden]
			get
			{
				return <>2__current;
			}
		}

		[DebuggerHidden]
		public <GetAllBuffById>d__121(int <>1__state)
		{
			this.<>1__state = <>1__state;
			<>l__initialThreadId = Environment.CurrentManagedThreadId;
		}

		[DebuggerHidden]
		void IDisposable.Dispose()
		{
			int num = <>1__state;
			if (num == -3 || num == 1)
			{
				try
				{
				}
				finally
				{
					<>m__Finally1();
				}
			}
			<handleSet>5__2 = null;
			<toRemove>5__3 = null;
			<>7__wrap3 = default(HashSet<int>.Enumerator);
			<>1__state = -2;
		}

		private bool MoveNext()
		{
			try
			{
				int num = <>1__state;
				BaseBuffComponent baseBuffComponent = <>4__this;
				switch (num)
				{
				default:
					return false;
				case 0:
					<>1__state = -1;
					if (!baseBuffComponent.BuffIdToHandleMap.TryGetValue(buffId, out <handleSet>5__2))
					{
						return false;
					}
					if (<handleSet>5__2.Count == 0)
					{
						baseBuffComponent.BuffIdToHandleMap.Remove(buffId);
						return false;
					}
					<toRemove>5__3 = new List<int>();
					<>7__wrap3 = <handleSet>5__2.GetEnumerator();
					<>1__state = -3;
					break;
				case 1:
					<>1__state = -3;
					break;
				}
				while (<>7__wrap3.MoveNext())
				{
					int current = <>7__wrap3.Current;
					ActiveBuffInternal valueOrDefault = baseBuffComponent.BuffContainer.GetValueOrDefault(current);
					if (valueOrDefault == null)
					{
						<toRemove>5__3.Add(current);
					}
					else if (!baseBuffComponent.BuffGarbageSet.Contains(valueOrDefault.Handle))
					{
						<>2__current = valueOrDefault;
						<>1__state = 1;
						return true;
					}
				}
				<>m__Finally1();
				<>7__wrap3 = default(HashSet<int>.Enumerator);
				foreach (int item in <toRemove>5__3)
				{
					<handleSet>5__2.Remove(item);
				}
				return false;
			}
			catch
			{
				//try-fault
				((IDisposable)this).Dispose();
				throw;
			}
		}

		bool IEnumerator.MoveNext()
		{
			//ILSpy generated this explicit interface implementation from .override directive in MoveNext
			return this.MoveNext();
		}

		private void <>m__Finally1()
		{
			<>1__state = -1;
			((IDisposable)<>7__wrap3).Dispose();
		}

		[DebuggerHidden]
		void IEnumerator.Reset()
		{
			throw new NotSupportedException();
		}

		[DebuggerHidden]
		IEnumerator<ActiveBuffInternal> IEnumerable<ActiveBuffInternal>.GetEnumerator()
		{
			<GetAllBuffById>d__121 <GetAllBuffById>d__;
			if (<>1__state == -2 && <>l__initialThreadId == Environment.CurrentManagedThreadId)
			{
				<>1__state = 0;
				<GetAllBuffById>d__ = this;
			}
			else
			{
				<GetAllBuffById>d__ = new <GetAllBuffById>d__121(0)
				{
					<>4__this = <>4__this
				};
			}
			<GetAllBuffById>d__.buffId = <>3__buffId;
			return <GetAllBuffById>d__;
		}

		[DebuggerHidden]
		IEnumerator IEnumerable.GetEnumerator()
		{
			return ((IEnumerable<ActiveBuffInternal>)this).GetEnumerator();
		}
	}

	protected CreatureDataComponent? CreatureDataComponent;

	protected BaseDeathComponent? DeathComponent;

	protected readonly Dictionary<int, ActiveBuffInternal> BuffContainer = new Dictionary<int, ActiveBuffInternal>();

	protected readonly Dictionary<long, HashSet<int>> BuffIdToHandleMap = new Dictionary<long, HashSet<int>>();

	protected readonly HashSet<int> BuffGarbageSet = new HashSet<int>();

	protected readonly Dictionary<long, int> WaitRemoveBuffResponse = new Dictionary<long, int>();

	protected readonly Dictionary<long, double> PendingAddBuff = new Dictionary<long, double>();

	[StaticVariableRuleIgnore]
	protected static readonly HashSet<long> NoLogBuffSet;

	protected readonly Dictionary<int, HashSet<int>> TagListenerDict = new Dictionary<int, HashSet<int>>();

	protected readonly Dictionary<long, HashSet<int>> BuffListenerDict = new Dictionary<long, HashSet<int>>();

	public readonly Dictionary<int, HashSet<int>> TagImmuneListenerDict = new Dictionary<int, HashSet<int>>();

	public readonly Dictionary<int, Dictionary<long, HashSet<int>>> VictimTagListenerDict = new Dictionary<int, Dictionary<long, HashSet<int>>>();

	protected readonly ConfigOverrideConditionListener ConfigOverrideConditionListener = new ConfigOverrideConditionListener();

	public readonly Dictionary<long, Dictionary<long, HashSet<int>>> VictimBuffListenerDict = new Dictionary<long, Dictionary<long, HashSet<int>>>();

	protected readonly Dictionary<int, Dictionary<int, List<int>>> InstigatorEventListenDict = new Dictionary<int, Dictionary<int, List<int>>>();

	[StaticVariableRuleIgnore]
	private static readonly Stat CheckWhenTagChangedStat;

	[StaticVariableRuleIgnore]
	private static readonly Stat CheckWhenBuffChangedStat;

	public readonly Dictionary<long, int> BuffRoutineExpirationLock = new Dictionary<long, int>();

	protected Dictionary<long, Dictionary<int, double>> EffectTimeoutMap = new Dictionary<long, Dictionary<int, double>>();

	protected Dictionary<long, Dictionary<int, Dictionary<int, double>>> EffectTargetTimeoutMap = new Dictionary<long, Dictionary<int, Dictionary<int, double>>>();

	private static readonly int MaxTargetCdCount;

	private static Dictionary<long, Stat?>? _buffStatMap;

	[StaticVariableRuleIgnore]
	private static readonly Stat GetStackableBuffStat;

	[StaticVariableRuleIgnore]
	private static readonly Stat OnBuffStackDecreasedStat;

	[StaticVariableRuleIgnore]
	private static readonly Stat OnBuffStackIncreasedStat;

	private readonly HashSet<int> TagInChanged = new HashSet<int>();

	private int BuffLockInternal;

	protected readonly Dictionary<EBuffTriggerType, List<IBuffTrigger>> TriggerMap = new Dictionary<EBuffTriggerType, List<IBuffTrigger>>();

	[StaticVariableRuleIgnore]
	private static readonly Stat AddGameplayCueStat;

	[StaticVariableRuleIgnore]
	private static readonly Stat AddBuffBroadcastNotifyStat;

	public Dictionary<long, Dictionary<int, (float period, float duration)>> OwnerBuffTimeModifiers = new Dictionary<long, Dictionary<int, (float, float)>>();

	public Dictionary<long, Dictionary<int, (float period, float duration)>> InstigatorBuffTimeModifiers = new Dictionary<long, Dictionary<int, (float, float)>>();

	public Dictionary<long, Dictionary<int, BuffStackModifierValue>> BuffStackModifiers = new Dictionary<long, Dictionary<int, BuffStackModifierValue>>();

	private bool EnableDeferredCueCreation;

	protected readonly Dictionary<int, List<int>> Buff2CueMap = new Dictionary<int, List<int>>();

	private readonly HashSet<int> PendingCreateCueBuffs = new HashSet<int>();

	private readonly List<PendingInstantCueItem> PendingInstantCues = new List<PendingInstantCueItem>();

	private readonly HashSet<int> PendingDestroyCueBuffs = new HashSet<int>();

	[StaticVariableRuleIgnore]
	private static readonly Stat CreateGameplayCueByBuffStat;

	[StaticVariableRuleIgnore]
	private static readonly Stat DestroyGameplayCueByBuffStat;

	protected Dictionary<long, List<List<int>>>? SeamlessTravelBuffCue;

	private EBuffComponentType BuffComponentType;

	protected readonly Dictionary<long, BuffReplace> ReplaceBuffMap = new Dictionary<long, BuffReplace>();

	public long CreatureDataId => CreatureDataComponent.GetCreatureDataId();

	public ExtraEffectManager? BuffEffectManager { get; set; }

	private static Dictionary<long, Stat?> BuffStatMap => _buffStatMap;

	protected int BuffLock
	{
		get
		{
			return BuffLockInternal;
		}
		set
		{
			BuffLockInternal = value;
			if (value > 0 || BuffGarbageSet.Count <= 0)
			{
				return;
			}
			foreach (int item in BuffGarbageSet)
			{
				ActiveBuffInternal valueOrDefault = BuffContainer.GetValueOrDefault(item);
				BuffContainer.Remove(item);
				if (valueOrDefault == null)
				{
					continue;
				}
				if (BuffIdToHandleMap.TryGetValue(valueOrDefault.Id, out HashSet<int> value2))
				{
					value2.Remove(item);
					if (value2.Count == 0)
					{
						BuffIdToHandleMap.Remove(valueOrDefault.Id);
					}
				}
				UnregisterConfigOverrideListener(valueOrDefault);
				ActiveBuffInternal.ReleaseBuff(valueOrDefault);
			}
			BuffGarbageSet.Clear();
		}
	}

	static BaseBuffComponent()
	{
		NoLogBuffSet = new HashSet<long> { -3L, 1101003012L, 800080191L, 800100003081L, 1302121034L, 1304700001L, 640007016L, 1202803014L, 1202002003L };
		CheckWhenTagChangedStat = Stat.Create("BaseBuffComponent.CheckWhenTagChanged");
		CheckWhenBuffChangedStat = Stat.Create("BaseBuffComponent.CheckWhenBuffChanged");
		MaxTargetCdCount = 50;
		GetStackableBuffStat = Stat.CreateNoFlameGraph("BaseBuffComponent.GetStackableBuff");
		OnBuffStackDecreasedStat = Stat.Create("BaseBuffComponent.OnBuffStackDecreased");
		OnBuffStackIncreasedStat = Stat.Create("BaseBuffComponent.OnBuffStackIncreased");
		AddGameplayCueStat = Stat.Create("AddBuff_Cue");
		AddBuffBroadcastNotifyStat = Stat.Create("BaseBuffComponent.BroadcastAddBuffNotify");
		CreateGameplayCueByBuffStat = Stat.Create("CreateGameplayCueByBuff");
		DestroyGameplayCueByBuffStat = Stat.Create("DestroyGameplayCueByBuff");
		StaticVariableRegister.RegisterAndExecute((Action)CreateStaticDefaultValue, (Action)ResetStaticDefaultValue);
	}

	public virtual string GetDebugName()
	{
		return "";
	}

	protected override bool OnInit()
	{
		base.OnInit();
		CreatureDataComponent = base.Entity.GetComponent<CreatureDataComponent>();
		DeathComponent = base.Entity.GetComponent<BaseDeathComponent>();
		Singleton<EventSystem>.Instance.Add<int>(EEventName.CharOnRoleDead, OnAnyInstigatorDead);
		Singleton<EventSystem>.Instance.Add<ERemoveEntityType, EntityHandle>(EEventName.RemoveEntity, OnAnyInstigatorRemoved);
		return true;
	}

	protected override bool OnClear()
	{
		ControllerBase<BuffController>.Instance.UnregisterBuffComponent(this);
		ExtraEffectManager buffEffectManager = BuffEffectManager;
		if (buffEffectManager != null)
		{
			foreach (BuffEffect allEffect in buffEffectManager.GetAllEffects())
			{
				allEffect?.OnRemoved(bPremature: true);
			}
			buffEffectManager.Clear();
		}
		foreach (ActiveBuffInternal value in BuffContainer.Values)
		{
			foreach (ExtraEffectParameters effectInfo in value.Config.EffectInfos)
			{
				effectInfo.ExecutionEffect?.OnBuffRemovedCallback(value);
			}
		}
		BuffContainer.Clear();
		BuffGarbageSet.Clear();
		PendingAddBuff.Clear();
		EffectTimeoutMap.Clear();
		EffectTargetTimeoutMap.Clear();
		TagInChanged.Clear();
		ConfigOverrideConditionListener.Clear();
		SeamlessTravelBuffCue = null;
		if (Singleton<EventSystem>.Instance.Has<int>(EEventName.CharOnRoleDead, OnAnyInstigatorDead))
		{
			Singleton<EventSystem>.Instance.Remove<int>(EEventName.CharOnRoleDead, OnAnyInstigatorDead);
		}
		if (Singleton<EventSystem>.Instance.Has<ERemoveEntityType, EntityHandle>(EEventName.RemoveEntity, OnAnyInstigatorRemoved))
		{
			Singleton<EventSystem>.Instance.Remove<ERemoveEntityType, EntityHandle>(EEventName.RemoveEntity, OnAnyInstigatorRemoved);
		}
		InstigatorEventListenDict.Clear();
		return true;
	}

	protected override void OnTick(float delta)
	{
		BuffLock = 0;
		TagInChanged.Clear();
	}

	protected virtual void InitBornBuff()
	{
	}

	protected virtual bool NeedBroadcastBuff(ActiveBuffInternal? buff, bool fromServer = false)
	{
		if (fromServer)
		{
			return false;
		}
		if (buff != null && buff.Config.OnlyLocalAdd)
		{
			return false;
		}
		return HasBuffAuthority();
	}

	public virtual bool HasBuffAuthority()
	{
		return false;
	}

	protected virtual bool NeedAddBuffOrder(long buffId)
	{
		return true;
	}

	public virtual Entity? GetEntity()
	{
		return null;
	}

	public virtual Entity? GetExactEntity()
	{
		return base.Entity;
	}

	public virtual BaseAttributeComponent? GetAttributeComponent()
	{
		return null;
	}

	public virtual BaseTagComponent? GetTagComponent()
	{
		return null;
	}

	public virtual BaseSkillComponent? GetSkillComponent()
	{
		return null;
	}

	public virtual CharacterPassiveSkillComponent? GetPassiveSkillComponent()
	{
		return GetEntity()?.GetComponent<CharacterPassiveSkillComponent>();
	}

	public virtual BaseActorComponent? GetActorComponent()
	{
		return null;
	}

	public virtual BaseGameplayCueComponent? GetCueComponent()
	{
		return null;
	}

	public virtual float GetTimeScale()
	{
		return 0f;
	}

	public virtual float GetLogicTimeScale()
	{
		return 1f;
	}

	protected Entity? ResolveBuffStackInstigator(long? instigatorId)
	{
		if (!instigatorId.HasValue || instigatorId == 0)
		{
			return null;
		}
		WorldEntity worldEntity = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId.Value)?.Entity;
		if (worldEntity == null || !worldEntity.Valid || !worldEntity.IsInit)
		{
			return null;
		}
		return worldEntity;
	}

	protected void EmitBuffStackInstigatorChanged(long buffId, int oldStack, int newStack, long? instigatorId)
	{
		Entity entity = ResolveBuffStackInstigator(instigatorId);
		if (entity != null)
		{
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(entity, EAbilityEventName.BuffStackInstigatorChanged, buffId, buffId, oldStack, newStack, base.Entity, entity);
		}
	}

	private void AddHandleMark<TKey>(Dictionary<TKey, HashSet<int>> dict, TKey key, int handle) where TKey : notnull
	{
		if (!dict.TryGetValue(key, out HashSet<int> value))
		{
			value = (dict[key] = new HashSet<int>());
		}
		value.Add(handle);
	}

	public void AddEntityMark<TKey>(Dictionary<TKey, Dictionary<long, HashSet<int>>> dict, TKey key, long creatureDataId, int handle) where TKey : notnull
	{
		if (!dict.TryGetValue(key, out Dictionary<long, HashSet<int>> value))
		{
			value = (dict[key] = new Dictionary<long, HashSet<int>>());
		}
		if (!value.TryGetValue(creatureDataId, out var value2))
		{
			value2 = (value[creatureDataId] = new HashSet<int>());
		}
		value2.Add(handle);
	}

	private void RemoveHandleMark<TKey>(Dictionary<TKey, HashSet<int>> dict, TKey key, int handle) where TKey : notnull
	{
		if (dict.TryGetValue(key, out HashSet<int> value))
		{
			value.Remove(handle);
			if (value.Count <= 0)
			{
				dict.Remove(key);
			}
		}
	}

	public void RemoveEntityMark<TKey>(Dictionary<TKey, Dictionary<long, HashSet<int>>> dict, TKey key, long creatureDataId, int handle) where TKey : notnull
	{
		if (!dict.TryGetValue(key, out Dictionary<long, HashSet<int>> value) || !value.TryGetValue(creatureDataId, out var value2))
		{
			return;
		}
		value2.Remove(handle);
		if (value2.Count <= 0)
		{
			value.Remove(creatureDataId);
			if (value.Count <= 0)
			{
				dict.Remove(key);
			}
		}
	}

	protected int[] GetInvolvedTags(BuffDefinition config)
	{
		List<int> list = new List<int>();
		if (config.ActivateTagRequirements != null)
		{
			list.AddRange(config.ActivateTagRequirements);
		}
		if (config.ActivateTagIgnores != null)
		{
			list.AddRange(config.ActivateTagIgnores);
		}
		if (config.RemoveTagExistAll != null)
		{
			list.AddRange(config.RemoveTagExistAll);
		}
		if (config.RemoveTagExistAny != null)
		{
			list.AddRange(config.RemoveTagExistAny);
		}
		if (config.RemoveTagIgnores != null)
		{
			list.AddRange(config.RemoveTagIgnores);
		}
		if (config.BuffAction != null)
		{
			TBuffAction[] buffAction = config.BuffAction;
			foreach (TBuffAction tBuffAction in buffAction)
			{
				EBuffActionType type = tBuffAction.Type;
				if ((uint)(type - 101) <= 7u)
				{
					list.AddRange(tBuffAction.Tags);
				}
			}
		}
		return list.ToArray();
	}

	protected long[] GetInvolvedBuffIds(BuffDefinition config)
	{
		List<long> list = new List<long>();
		if (config.BuffAction != null)
		{
			TBuffAction[] buffAction = config.BuffAction;
			foreach (TBuffAction tBuffAction in buffAction)
			{
				EBuffActionType type = tBuffAction.Type;
				if ((uint)(type - 109) <= 7u)
				{
					list.AddRange(tBuffAction.Buffs);
				}
			}
		}
		return list.ToArray();
	}

	protected int[] GetInvolvedInstigatorTags(BuffDefinition config)
	{
		List<int> list = new List<int>();
		if (config.BuffAction != null)
		{
			TBuffAction[] buffAction = config.BuffAction;
			foreach (TBuffAction tBuffAction in buffAction)
			{
				EBuffActionType type = tBuffAction.Type;
				if ((uint)(type - 1) <= 7u)
				{
					list.AddRange(tBuffAction.Tags);
				}
			}
		}
		return list.ToArray();
	}

	protected long[] GetInvolvedInstigatorBuffIds(BuffDefinition config)
	{
		List<long> list = new List<long>();
		if (config.BuffAction != null)
		{
			TBuffAction[] buffAction = config.BuffAction;
			foreach (TBuffAction tBuffAction in buffAction)
			{
				EBuffActionType type = tBuffAction.Type;
				if ((uint)(type - 9) <= 7u)
				{
					list.AddRange(tBuffAction.Buffs);
				}
			}
		}
		return list.ToArray();
	}

	protected List<TBuffAction> GetInstigatorEventActions(BuffDefinition config)
	{
		List<TBuffAction> list = new List<TBuffAction>();
		if (config.BuffAction != null)
		{
			TBuffAction[] buffAction = config.BuffAction;
			foreach (TBuffAction tBuffAction in buffAction)
			{
				if (tBuffAction.Type == EBuffActionType.RemoveBuffWhenInstigatorDeadOrRemoved)
				{
					list.Add(tBuffAction);
				}
			}
		}
		return list;
	}

	protected void RegisterInstigatorEventListener(ActiveBuffInternal buff)
	{
		List<TBuffAction> instigatorEventActions = GetInstigatorEventActions(buff.Config);
		if (instigatorEventActions.Count == 0)
		{
			return;
		}
		Entity instigator = buff.GetInstigator();
		if (instigator == null)
		{
			return;
		}
		int id = instigator.Id;
		int handle = buff.Handle;
		foreach (TBuffAction item in instigatorEventActions)
		{
			if (item.CustomParams == null)
			{
				continue;
			}
			int[] customParams = item.CustomParams;
			foreach (int key in customParams)
			{
				if (!InstigatorEventListenDict.TryGetValue(id, out Dictionary<int, List<int>> value))
				{
					value = new Dictionary<int, List<int>>();
					InstigatorEventListenDict[id] = value;
				}
				if (!value.TryGetValue(key, out var value2))
				{
					value2 = (value[key] = new List<int>());
				}
				value2.Add(handle);
			}
		}
	}

	protected void UnregisterInstigatorEventListener(ActiveBuffInternal buff)
	{
		Entity instigator = buff.GetInstigator();
		if (instigator == null)
		{
			return;
		}
		int id = instigator.Id;
		if (!InstigatorEventListenDict.TryGetValue(id, out Dictionary<int, List<int>> value))
		{
			return;
		}
		int handle = buff.Handle;
		foreach (int item in value.Keys.ToList())
		{
			List<int> list = value[item];
			list.Remove(handle);
			if (list.Count == 0)
			{
				value.Remove(item);
			}
		}
		if (value.Count == 0)
		{
			InstigatorEventListenDict.Remove(id);
		}
	}

	private void OnAnyInstigatorDead(int charId)
	{
		if (!InstigatorEventListenDict.TryGetValue(charId, out Dictionary<int, List<int>> value) || value.Count == 0 || !value.TryGetValue(0, out var value2))
		{
			return;
		}
		foreach (int item in value2.ToList())
		{
			if (GetBuffByHandle(item) != null && HasBuffAuthority())
			{
				RemoveBuffByHandle(item, -1, "因为创建者死亡触发BuffAction");
			}
		}
	}

	private void OnAnyInstigatorRemoved(ERemoveEntityType removeType, EntityHandle handle)
	{
		WorldEntity worldEntity = ModelBase<CreatureModel>.Instance.GetEntityById(handle.Id)?.Entity;
		if (worldEntity == null)
		{
			return;
		}
		int id = worldEntity.Id;
		if (!InstigatorEventListenDict.TryGetValue(id, out Dictionary<int, List<int>> value) || value.Count == 0 || !value.TryGetValue(1, out var value2))
		{
			return;
		}
		foreach (int item in value2.ToList())
		{
			if (GetBuffByHandle(item) != null && HasBuffAuthority())
			{
				RemoveBuffByHandle(item, -1, "因为创建者移出队伍触发BuffAction");
			}
		}
	}

	protected void MarkListenerBuff(ActiveBuffInternal buff)
	{
		if (!NeedCheck(buff.Config))
		{
			return;
		}
		BuffDefinition config = buff.Config;
		if (config == null)
		{
			return;
		}
		int handle = buff.Handle;
		int[] involvedTags = GetInvolvedTags(config);
		foreach (int key in involvedTags)
		{
			AddHandleMark(TagListenerDict, key, handle);
		}
		long[] involvedBuffIds = GetInvolvedBuffIds(config);
		foreach (long key2 in involvedBuffIds)
		{
			AddHandleMark(BuffListenerDict, key2, handle);
		}
		if (config.ImmuneTags != null)
		{
			involvedTags = config.ImmuneTags;
			foreach (int key3 in involvedTags)
			{
				AddHandleMark(TagImmuneListenerDict, key3, handle);
			}
		}
		CharacterBuffComponent instigatorBuffComponent = buff.GetInstigatorBuffComponent();
		long? num = CreatureDataComponent?.GetCreatureDataId();
		if (instigatorBuffComponent != null && num.HasValue)
		{
			involvedTags = GetInvolvedInstigatorTags(config);
			foreach (int key4 in involvedTags)
			{
				instigatorBuffComponent.AddEntityMark(instigatorBuffComponent.VictimTagListenerDict, key4, num.Value, handle);
			}
			involvedBuffIds = GetInvolvedInstigatorBuffIds(config);
			foreach (long key5 in involvedBuffIds)
			{
				instigatorBuffComponent.AddEntityMark(instigatorBuffComponent.VictimBuffListenerDict, key5, num.Value, handle);
			}
		}
		RegisterInstigatorEventListener(buff);
	}

	protected void RemoveListenerBuff(ActiveBuffInternal buff)
	{
		if (!NeedCheck(buff.Config))
		{
			return;
		}
		BuffDefinition config = buff.Config;
		if (config == null)
		{
			return;
		}
		int handle = buff.Handle;
		int[] involvedTags = GetInvolvedTags(config);
		foreach (int key in involvedTags)
		{
			RemoveHandleMark(TagListenerDict, key, handle);
		}
		long[] involvedBuffIds = GetInvolvedBuffIds(config);
		foreach (long key2 in involvedBuffIds)
		{
			RemoveHandleMark(BuffListenerDict, key2, handle);
		}
		if (config.ImmuneTags != null)
		{
			involvedTags = config.ImmuneTags;
			foreach (int key3 in involvedTags)
			{
				RemoveHandleMark(TagImmuneListenerDict, key3, handle);
			}
		}
		CharacterBuffComponent instigatorBuffComponent = buff.GetInstigatorBuffComponent();
		long? num = CreatureDataComponent?.GetCreatureDataId();
		if (instigatorBuffComponent != null && num.HasValue)
		{
			involvedTags = GetInvolvedInstigatorTags(config);
			foreach (int key4 in involvedTags)
			{
				instigatorBuffComponent.RemoveEntityMark(instigatorBuffComponent.VictimTagListenerDict, key4, num.Value, handle);
			}
			involvedBuffIds = GetInvolvedInstigatorBuffIds(config);
			foreach (long key5 in involvedBuffIds)
			{
				instigatorBuffComponent.RemoveEntityMark(instigatorBuffComponent.VictimBuffListenerDict, key5, num.Value, handle);
			}
		}
		UnregisterInstigatorEventListener(buff);
	}

	protected void CheckWhenTagChanged(int tagId)
	{
		BuffLock++;
		if (TagListenerDict.TryGetValue(tagId, out HashSet<int> value))
		{
			string nameByTagId = GameplayTagUtils.GetNameByTagId(tagId);
			int[] array = value.ToArray();
			foreach (int handle in array)
			{
				IActiveBuff buffByHandle = GetBuffByHandle(handle);
				if (buffByHandle != null && CheckRemove(buffByHandle.Config, buffByHandle.GetInstigator()))
				{
					RemoveBuffInner(handle, -1, isPrematureRemoval: true, "因为tag " + nameByTagId + "的变化触发");
				}
			}
			array = value.ToArray();
			foreach (int handle2 in array)
			{
				IActiveBuff buffByHandle2 = GetBuffByHandle(handle2);
				if (buffByHandle2 == null)
				{
					RemoveHandleMark(TagListenerDict, tagId, handle2);
					continue;
				}
				bool flag = CheckActivate(buffByHandle2.Config, buffByHandle2.GetInstigator());
				if (flag != buffByHandle2.IsActive())
				{
					OnBuffActiveChanged(buffByHandle2 as ActiveBuffInternal, flag);
				}
			}
		}
		if (VictimTagListenerDict.TryGetValue(tagId, out Dictionary<long, HashSet<int>> value2))
		{
			string nameByTagId2 = GameplayTagUtils.GetNameByTagId(tagId);
			KeyValuePair<long, HashSet<int>>[] array2 = value2.ToArray();
			long key;
			HashSet<int> value3;
			foreach (KeyValuePair<long, HashSet<int>> keyValuePair in array2)
			{
				keyValuePair.Deconstruct(out key, out value3);
				long creatureDataId = key;
				int[] array = value3.ToArray();
				foreach (int handle3 in array)
				{
					BaseBuffComponent baseBuffComponent = ModelBase<CreatureModel>.Instance.GetEntity(creatureDataId)?.Entity?.GetComponent<BaseBuffComponent>();
					IActiveBuff activeBuff = baseBuffComponent?.GetBuffByHandle(handle3);
					if (baseBuffComponent != null && activeBuff != null && baseBuffComponent.CheckRemove(activeBuff.Config, activeBuff.GetInstigator()))
					{
						baseBuffComponent.RemoveBuffInner(handle3, -1, isPrematureRemoval: true, "因为施加者的tag " + nameByTagId2 + "的变化触发");
					}
				}
			}
			array2 = value2.ToArray();
			foreach (KeyValuePair<long, HashSet<int>> keyValuePair in array2)
			{
				keyValuePair.Deconstruct(out key, out value3);
				long creatureDataId2 = key;
				int[] array = value3.ToArray();
				foreach (int handle4 in array)
				{
					BaseBuffComponent baseBuffComponent2 = ModelBase<CreatureModel>.Instance.GetEntity(creatureDataId2)?.Entity?.GetComponent<BaseBuffComponent>();
					IActiveBuff activeBuff2 = baseBuffComponent2?.GetBuffByHandle(handle4);
					if (baseBuffComponent2 == null || activeBuff2 == null)
					{
						RemoveEntityMark(VictimTagListenerDict, tagId, creatureDataId2, handle4);
						continue;
					}
					bool flag2 = baseBuffComponent2.CheckActivate(activeBuff2.Config, activeBuff2.GetInstigator());
					if (flag2 != activeBuff2.IsActive())
					{
						baseBuffComponent2?.OnBuffActiveChanged(activeBuff2 as ActiveBuffInternal, flag2);
					}
				}
			}
		}
		BuffLock--;
	}

	protected void CheckWhenBuffChanged(long buffId)
	{
		if (BuffListenerDict.TryGetValue(buffId, out HashSet<int> value))
		{
			BuffLock++;
			int[] array = value.ToArray();
			foreach (int handle in array)
			{
				IActiveBuff buffByHandle = GetBuffByHandle(handle);
				if (buffByHandle != null && CheckRemove(buffByHandle.Config, buffByHandle.GetInstigator()))
				{
					RemoveBuffInner(handle, -1, isPrematureRemoval: true, $"因为buff {buffId}的变化触发");
				}
			}
			array = value.ToArray();
			foreach (int handle2 in array)
			{
				IActiveBuff buffByHandle2 = GetBuffByHandle(handle2);
				if (buffByHandle2 == null)
				{
					RemoveHandleMark(BuffListenerDict, buffId, handle2);
					continue;
				}
				bool flag = CheckActivate(buffByHandle2.Config, buffByHandle2.GetInstigator());
				if (flag != buffByHandle2.IsActive())
				{
					OnBuffActiveChanged(buffByHandle2 as ActiveBuffInternal, flag);
				}
			}
			BuffLock--;
		}
		if (!VictimBuffListenerDict.TryGetValue(buffId, out Dictionary<long, HashSet<int>> value2))
		{
			return;
		}
		BuffLock++;
		KeyValuePair<long, HashSet<int>>[] array2 = value2.ToArray();
		long key;
		HashSet<int> value3;
		foreach (KeyValuePair<long, HashSet<int>> keyValuePair in array2)
		{
			keyValuePair.Deconstruct(out key, out value3);
			long creatureDataId = key;
			int[] array = value3.ToArray();
			foreach (int handle3 in array)
			{
				BaseBuffComponent baseBuffComponent = ModelBase<CreatureModel>.Instance.GetEntity(creatureDataId)?.Entity?.GetComponent<BaseBuffComponent>();
				IActiveBuff activeBuff = baseBuffComponent?.GetBuffByHandle(handle3);
				if (baseBuffComponent != null && activeBuff != null && baseBuffComponent.CheckRemove(activeBuff.Config, activeBuff.GetInstigator()))
				{
					baseBuffComponent.RemoveBuffInner(handle3, -1, isPrematureRemoval: true, $"因为施加者的buff {buffId}的变化触发");
				}
			}
		}
		array2 = value2.ToArray();
		foreach (KeyValuePair<long, HashSet<int>> keyValuePair in array2)
		{
			keyValuePair.Deconstruct(out key, out value3);
			long creatureDataId2 = key;
			int[] array = value3.ToArray();
			foreach (int handle4 in array)
			{
				BaseBuffComponent baseBuffComponent2 = ModelBase<CreatureModel>.Instance.GetEntity(creatureDataId2)?.Entity?.GetComponent<BaseBuffComponent>();
				IActiveBuff activeBuff2 = baseBuffComponent2?.GetBuffByHandle(handle4);
				if (baseBuffComponent2 == null || activeBuff2 == null)
				{
					RemoveEntityMark(VictimBuffListenerDict, buffId, creatureDataId2, handle4);
					continue;
				}
				bool flag2 = baseBuffComponent2.CheckActivate(activeBuff2.Config, activeBuff2.GetInstigator());
				if (flag2 != activeBuff2.IsActive())
				{
					baseBuffComponent2?.OnBuffActiveChanged(activeBuff2 as ActiveBuffInternal, flag2);
				}
			}
		}
		BuffLock--;
	}

	protected void UnregisterConfigOverrideListener(ActiveBuffInternal buff)
	{
		int? configOverrideListenerId = buff.GetConfigOverrideListenerId();
		if (configOverrideListenerId.HasValue)
		{
			ConfigOverrideConditionListener.RemoveListener(configOverrideListenerId.Value);
		}
	}

	public void AddBuffRoutineExpirationLock(long buffId)
	{
		int valueOrDefault = BuffRoutineExpirationLock.GetValueOrDefault(buffId, 0);
		BuffRoutineExpirationLock[buffId] = valueOrDefault + 1;
	}

	public void RemoveBuffRoutineExpirationLock(long buffId)
	{
		int valueOrDefault = BuffRoutineExpirationLock.GetValueOrDefault(buffId, 0);
		if (valueOrDefault <= 1)
		{
			BuffRoutineExpirationLock.Remove(buffId);
		}
		else
		{
			BuffRoutineExpirationLock[buffId] = valueOrDefault - 1;
		}
	}

	public virtual bool HasBuffRoutineExpirationLock(long buffId)
	{
		return BuffRoutineExpirationLock.GetValueOrDefault(buffId, 0) > 0;
	}

	public virtual bool IsPaused()
	{
		return false;
	}

	public void RefreshTimeScale()
	{
		float timeScale = GetTimeScale();
		bool forcePaused = IsPaused();
		foreach (ActiveBuffInternal value in BuffContainer.Values)
		{
			value.OnTimeScaleChanged(timeScale, forcePaused);
		}
	}

	protected override void OnChangeTimeDilation(float timeDilation)
	{
		RefreshTimeScale();
	}

	public bool NeedCheck(BuffDefinition config)
	{
		if (config == null)
		{
			return false;
		}
		if (config.Id.GetValueOrDefault() == -3)
		{
			return true;
		}
		return HasBuffAuthority();
	}

	protected virtual bool CheckAdd(BuffDefinition config, long instigatorId, bool fromServer)
	{
		if (fromServer)
		{
			return true;
		}
		if (!fromServer && NeedCheck(config))
		{
			BaseDeathComponent? deathComponent = DeathComponent;
			if (deathComponent != null && deathComponent.IsDead() && config.EffectInfos != null)
			{
				foreach (ExtraEffectParameters effectInfo in config.EffectInfos)
				{
					if (effectInfo.ExtraEffectId == EExtraEffectId.Frozen)
					{
						return false;
					}
				}
			}
			BaseTagComponent tagComponent = GetTagComponent();
			WorldEntity worldEntity = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId)?.Entity;
			BaseTagComponent baseTagComponent = worldEntity?.GetComponent<BaseTagComponent>();
			if (config.Probability < 10000 && RandomSystem.GetRandomPercent() > (float)config.Probability)
			{
				return false;
			}
			if (tagComponent == null)
			{
				return false;
			}
			if (config.AddTagIgnores != null)
			{
				int[] addTagIgnores = config.AddTagIgnores;
				foreach (int tagId in addTagIgnores)
				{
					if (tagComponent.HasTag(tagId))
					{
						return false;
					}
				}
			}
			if (config.AddTagRequirements != null)
			{
				int[] addTagIgnores = config.AddTagRequirements;
				foreach (int tagId2 in addTagIgnores)
				{
					if (!tagComponent.HasTag(tagId2))
					{
						return false;
					}
				}
			}
			if (CheckRemove(config, worldEntity))
			{
				return false;
			}
			if (config.RemoveTagIgnores != null && config.RemoveTagIgnores.Length != 0)
			{
				bool flag = false;
				int[] addTagIgnores = config.RemoveTagIgnores;
				foreach (int tagId3 in addTagIgnores)
				{
					if (tagComponent.HasTag(tagId3))
					{
						flag = true;
						break;
					}
				}
				if (!flag)
				{
					return false;
				}
			}
			int[] array = config.RemoveTagExistAll ?? Array.Empty<int>();
			if (array.Length != 0)
			{
				bool flag2 = true;
				int[] addTagIgnores = array;
				foreach (int tagId4 in addTagIgnores)
				{
					if (!tagComponent.HasTag(tagId4))
					{
						flag2 = false;
						break;
					}
				}
				if (flag2)
				{
					return false;
				}
			}
			int[] array2 = config.RemoveTagExistAny ?? Array.Empty<int>();
			if (array2.Length != 0)
			{
				int[] addTagIgnores = array2;
				foreach (int tagId5 in addTagIgnores)
				{
					if (tagComponent.HasTag(tagId5))
					{
						return false;
					}
				}
			}
			if (baseTagComponent != null)
			{
				if (config.AddInstigatorTagIgnores != null)
				{
					int[] addTagIgnores = config.AddInstigatorTagIgnores;
					foreach (int tagId6 in addTagIgnores)
					{
						if (baseTagComponent.HasTag(tagId6))
						{
							return false;
						}
					}
				}
				if (config.AddInstigatorTagRequirements != null)
				{
					int[] addTagIgnores = config.AddInstigatorTagRequirements;
					foreach (int tagId7 in addTagIgnores)
					{
						if (!baseTagComponent.HasTag(tagId7))
						{
							return false;
						}
					}
				}
			}
		}
		return !CheckImmune(config);
	}

	public virtual bool CheckImmune(BuffDefinition config)
	{
		foreach (int key in TagImmuneListenerDict.Keys)
		{
			bool flag = false;
			if (config.GrantedTags != null)
			{
				int[] grantedTags = config.GrantedTags;
				for (int i = 0; i < grantedTags.Length; i++)
				{
					if (GameplayTagUtils.IsChildTag(grantedTags[i], key))
					{
						flag = true;
						break;
					}
				}
			}
			if (!flag)
			{
				continue;
			}
			foreach (int item in TagImmuneListenerDict[key])
			{
				ActiveBuffInternal valueOrDefault = BuffContainer.GetValueOrDefault(item);
				BuffDefinition buffDefinition = valueOrDefault?.Config;
				if (buffDefinition != null && valueOrDefault != null && valueOrDefault.IsValid() && valueOrDefault.IsActive() && config.GrantedTags != null)
				{
					bool flag2 = GameplayTagUtils.HasAll(config.GrantedTags, buffDefinition.ImmuneTags) && !GameplayTagUtils.HasAny(config.GrantedTags, buffDefinition.ImmuneTagIgnores);
					if ((valueOrDefault?.IsActive() ?? false) && flag2)
					{
						return true;
					}
				}
			}
		}
		return false;
	}

	protected bool HasTagRemoveCheck(BuffDefinition config)
	{
		int[]? removeTagIgnores = config.RemoveTagIgnores;
		if (((removeTagIgnores != null && removeTagIgnores.Length != 0) ? 1 : 0) <= (false ? 1 : 0))
		{
			int[]? removeTagExistAll = config.RemoveTagExistAll;
			if (((removeTagExistAll != null && removeTagExistAll.Length != 0) ? 1 : 0) <= (false ? 1 : 0))
			{
				int[]? removeTagExistAny = config.RemoveTagExistAny;
				if (((removeTagExistAny != null && removeTagExistAny.Length != 0) ? 1 : 0) <= (false ? 1 : 0))
				{
					if (config.BuffAction != null)
					{
						TBuffAction[] buffAction = config.BuffAction;
						foreach (TBuffAction tBuffAction in buffAction)
						{
							if (BuffTypesHelper.actionTagRemove.Contains(tBuffAction.Type))
							{
								return true;
							}
						}
					}
					return false;
				}
			}
		}
		return true;
	}

	protected bool CheckRemoveAction(BuffDefinition config, Entity? instigator)
	{
		if (config.BuffAction == null)
		{
			return false;
		}
		BaseTagComponent baseTagComponent = instigator?.GetComponent<BaseTagComponent>();
		BaseBuffComponent baseBuffComponent = instigator?.GetComponent<BaseBuffComponent>();
		BaseTagComponent tagComponent = GetTagComponent();
		TBuffAction[] buffAction = config.BuffAction;
		foreach (TBuffAction tBuffAction in buffAction)
		{
			switch (tBuffAction.Type)
			{
			case EBuffActionType.RemoveBuffWhenInstigatorHasEveryTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				bool flag3 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId4 in tags)
				{
					if (!baseTagComponent.HasTag(tagId4))
					{
						flag3 = false;
						break;
					}
				}
				if (flag3)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorHasSomeTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId2 in tags)
				{
					if (baseTagComponent.HasTag(tagId2))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorNotHasEveryTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				bool flag6 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId7 in tags)
				{
					if (baseTagComponent.HasTag(tagId7))
					{
						flag6 = false;
						break;
					}
				}
				if (flag6)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorNotHasSomeTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId in tags)
				{
					if (!baseTagComponent.HasTag(tagId))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorHasEveryBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				bool flag7 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId8 in buffs)
				{
					if (!baseBuffComponent.HasActiveBuff(buffId8))
					{
						flag7 = false;
						break;
					}
				}
				if (flag7)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorHasSomeBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId4 in buffs)
				{
					if (baseBuffComponent.HasActiveBuff(buffId4))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorNotHasEveryBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				bool flag4 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId7 in buffs)
				{
					if (baseBuffComponent.HasActiveBuff(buffId7))
					{
						flag4 = false;
						break;
					}
				}
				if (flag4)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenInstigatorNotHasSomeBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId3 in buffs)
				{
					if (!baseBuffComponent.HasActiveBuff(buffId3))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimHasEveryTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				bool flag5 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId5 in tags)
				{
					if (!tagComponent.HasTag(tagId5))
					{
						flag5 = false;
						break;
					}
				}
				if (flag5)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimHasSomeTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId3 in tags)
				{
					if (tagComponent.HasTag(tagId3))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimNotHasEveryTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				bool flag8 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId8 in tags)
				{
					if (tagComponent.HasTag(tagId8))
					{
						flag8 = false;
						break;
					}
				}
				if (flag8)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimNotHasSomeTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId6 in tags)
				{
					if (!tagComponent.HasTag(tagId6))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimHasEveryBuff:
			{
				bool flag2 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId6 in buffs)
				{
					if (!HasActiveBuff(buffId6))
					{
						flag2 = false;
						break;
					}
				}
				if (flag2)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimHasSomeBuff:
			{
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId5 in buffs)
				{
					if (HasActiveBuff(buffId5))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimNotHasEveryBuff:
			{
				bool flag = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId2 in buffs)
				{
					if (HasActiveBuff(buffId2))
					{
						flag = false;
						break;
					}
				}
				if (flag)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.RemoveBuffWhenVictimNotHasSomeBuff:
			{
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId in buffs)
				{
					if (!HasActiveBuff(buffId))
					{
						return true;
					}
				}
				break;
			}
			}
		}
		return false;
	}

	protected bool CheckInactivateAction(BuffDefinition config, Entity? instigator)
	{
		if (config.BuffAction == null)
		{
			return false;
		}
		BaseTagComponent baseTagComponent = instigator?.GetComponent<BaseTagComponent>();
		BaseBuffComponent baseBuffComponent = instigator?.GetComponent<BaseBuffComponent>();
		BaseTagComponent tagComponent = GetTagComponent();
		TBuffAction[] buffAction = config.BuffAction;
		foreach (TBuffAction tBuffAction in buffAction)
		{
			switch (tBuffAction.Type)
			{
			case EBuffActionType.InactiveBuffWhenInstigatorHasEveryTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				bool flag3 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId4 in tags)
				{
					if (!baseTagComponent.HasTag(tagId4))
					{
						flag3 = false;
						break;
					}
				}
				if (flag3)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorHasSomeTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId2 in tags)
				{
					if (baseTagComponent.HasTag(tagId2))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorNotHasEveryTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				bool flag6 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId7 in tags)
				{
					if (baseTagComponent.HasTag(tagId7))
					{
						flag6 = false;
						break;
					}
				}
				if (flag6)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorNotHasSomeTag:
			{
				if (baseTagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId in tags)
				{
					if (!baseTagComponent.HasTag(tagId))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorHasEveryBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				bool flag7 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId8 in buffs)
				{
					if (!baseBuffComponent.HasActiveBuff(buffId8))
					{
						flag7 = false;
						break;
					}
				}
				if (flag7)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorHasSomeBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId4 in buffs)
				{
					if (baseBuffComponent.HasActiveBuff(buffId4))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorNotHasEveryBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				bool flag4 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId7 in buffs)
				{
					if (baseBuffComponent.HasActiveBuff(buffId7))
					{
						flag4 = false;
						break;
					}
				}
				if (flag4)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenInstigatorNotHasSomeBuff:
			{
				if (baseBuffComponent == null)
				{
					break;
				}
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId3 in buffs)
				{
					if (!baseBuffComponent.HasActiveBuff(buffId3))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimHasEveryTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				bool flag5 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId5 in tags)
				{
					if (!tagComponent.HasTag(tagId5))
					{
						flag5 = false;
						break;
					}
				}
				if (flag5)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimHasSomeTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId3 in tags)
				{
					if (tagComponent.HasTag(tagId3))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimNotHasEveryTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				bool flag8 = true;
				int[] tags = tBuffAction.Tags;
				foreach (int tagId8 in tags)
				{
					if (tagComponent.HasTag(tagId8))
					{
						flag8 = false;
						break;
					}
				}
				if (flag8)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimNotHasSomeTag:
			{
				if (tagComponent == null)
				{
					break;
				}
				int[] tags = tBuffAction.Tags;
				foreach (int tagId6 in tags)
				{
					if (!tagComponent.HasTag(tagId6))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimHasEveryBuff:
			{
				bool flag2 = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId6 in buffs)
				{
					if (!HasActiveBuff(buffId6))
					{
						flag2 = false;
						break;
					}
				}
				if (flag2)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimHasSomeBuff:
			{
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId5 in buffs)
				{
					if (HasActiveBuff(buffId5))
					{
						return true;
					}
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimNotHasEveryBuff:
			{
				bool flag = true;
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId2 in buffs)
				{
					if (HasActiveBuff(buffId2))
					{
						flag = false;
						break;
					}
				}
				if (flag)
				{
					return true;
				}
				break;
			}
			case EBuffActionType.InactiveBuffWhenVictimNotHasSomeBuff:
			{
				long[] buffs = tBuffAction.Buffs;
				foreach (long buffId in buffs)
				{
					if (!HasActiveBuff(buffId))
					{
						return true;
					}
				}
				break;
			}
			}
		}
		return false;
	}

	protected virtual bool CheckRemove(BuffDefinition config, Entity? instigator)
	{
		BaseTagComponent tagComponent = GetTagComponent();
		if (tagComponent == null)
		{
			if (!HasTagRemoveCheck(config))
			{
				return false;
			}
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			(string, object) reference = ("buffId", config.Id);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "检查buff移除条件时找不到Tag组件，默认移除", new ReadOnlySpan<(string, object)>(ref reference));
			return true;
		}
		if (CheckRemoveAction(config, instigator))
		{
			return true;
		}
		bool flag = false;
		int[]? removeTagIgnores = config.RemoveTagIgnores;
		if (removeTagIgnores != null && removeTagIgnores.Length != 0)
		{
			flag = true;
			int[] removeTagIgnores2 = config.RemoveTagIgnores;
			foreach (int tagId in removeTagIgnores2)
			{
				if (tagComponent.HasTag(tagId))
				{
					flag = false;
					break;
				}
			}
		}
		bool flag2 = false;
		int[]? removeTagExistAll = config.RemoveTagExistAll;
		if (removeTagExistAll != null && removeTagExistAll.Length != 0)
		{
			flag2 = true;
			int[] removeTagIgnores2 = config.RemoveTagExistAll;
			foreach (int tagId2 in removeTagIgnores2)
			{
				if (!tagComponent.HasTag(tagId2))
				{
					flag2 = false;
					break;
				}
			}
		}
		bool flag3 = false;
		int[]? removeTagExistAny = config.RemoveTagExistAny;
		if (removeTagExistAny != null && removeTagExistAny.Length != 0)
		{
			int[] removeTagIgnores2 = config.RemoveTagExistAny;
			foreach (int tagId3 in removeTagIgnores2)
			{
				if (tagComponent.HasTag(tagId3))
				{
					flag3 = true;
					break;
				}
			}
		}
		return flag || flag2 || flag3;
	}

	protected virtual bool CheckActivate(BuffDefinition config, Entity? instigator)
	{
		BaseTagComponent tagComponent = GetTagComponent();
		if (tagComponent == null)
		{
			return false;
		}
		if (config.ActivateTagIgnores != null)
		{
			int[] activateTagIgnores = config.ActivateTagIgnores;
			foreach (int tagId in activateTagIgnores)
			{
				if (tagComponent.HasTag(tagId))
				{
					return false;
				}
			}
		}
		if (config.ActivateTagRequirements != null)
		{
			int[] activateTagIgnores = config.ActivateTagRequirements;
			foreach (int tagId2 in activateTagIgnores)
			{
				if (!tagComponent.HasTag(tagId2))
				{
					return false;
				}
			}
		}
		if (CheckInactivateAction(config, instigator))
		{
			return false;
		}
		return true;
	}

	public virtual void SetBuffEffectCd(long buffId, int index, float remainCd)
	{
		if (!EffectTimeoutMap.TryGetValue(buffId, out Dictionary<int, double> value))
		{
			value = new Dictionary<int, double>();
			EffectTimeoutMap[buffId] = value;
		}
		if (remainCd <= 0f)
		{
			value.Remove(index);
		}
		else
		{
			value[index] = Singleton<Time>.Instance.ServerCombatStopTime + (double)remainCd;
		}
	}

	public double GetBuffEffectCd(long buffId, int index)
	{
		if (!EffectTimeoutMap.TryGetValue(buffId, out Dictionary<int, double> value))
		{
			return 0.0;
		}
		if (!value.TryGetValue(index, out var value2))
		{
			return 0.0;
		}
		if (value2 <= Singleton<Time>.Instance.ServerCombatStopTime)
		{
			return 0.0;
		}
		return value2 - Singleton<Time>.Instance.ServerCombatStopTime;
	}

	private void RemoveInvalidTargetCd(Dictionary<int, double> targetMap)
	{
		List<int> list = new List<int>();
		foreach (KeyValuePair<int, double> item in targetMap)
		{
			if (item.Value <= Singleton<Time>.Instance.ServerCombatStopTime)
			{
				list.Add(item.Key);
			}
		}
		foreach (int item2 in list)
		{
			targetMap.Remove(item2);
		}
		if (targetMap.Count < MaxTargetCdCount)
		{
			return;
		}
		int? num = null;
		using (Dictionary<int, double>.KeyCollection.Enumerator enumerator3 = targetMap.Keys.GetEnumerator())
		{
			if (enumerator3.MoveNext())
			{
				int current3 = enumerator3.Current;
				num = current3;
			}
		}
		CombatLog instance = Singleton<CombatLog>.Instance;
		Entity entity = base.Entity;
		global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("当前数量", targetMap.Count);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("上限", MaxTargetCdCount);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("移除的entityId", num);
		instance.Warn(CombatLog.EDebugModule.Buff, entity, "目标CD条目数量超上限，移除最早插入的目标CD", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
		if (num.HasValue)
		{
			targetMap.Remove(num.Value);
		}
	}

	public virtual void SetBuffEffectCdForTarget(long buffId, int index, int targetEntityId, float remainCd)
	{
		if (targetEntityId == 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("effectIndex", index);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "设置目标CD时目标实体无效(entityId=0)，跳过", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
			return;
		}
		if (!EffectTargetTimeoutMap.TryGetValue(buffId, out Dictionary<int, Dictionary<int, double>> value))
		{
			value = new Dictionary<int, Dictionary<int, double>>();
			EffectTargetTimeoutMap[buffId] = value;
		}
		if (!value.TryGetValue(index, out var value2))
		{
			value2 = (value[index] = new Dictionary<int, double>());
		}
		if (remainCd <= 0f)
		{
			value2.Remove(targetEntityId);
			return;
		}
		RemoveInvalidTargetCd(value2);
		value2.ContainsKey(targetEntityId);
		value2.Remove(targetEntityId);
		value2[targetEntityId] = Singleton<Time>.Instance.ServerCombatStopTime + (double)remainCd;
	}

	public string GetTargetCdDebugStr(long buffId, int index)
	{
		if (!EffectTargetTimeoutMap.TryGetValue(buffId, out Dictionary<int, Dictionary<int, double>> value))
		{
			return "";
		}
		if (!value.TryGetValue(index, out var value2) || value2.Count == 0)
		{
			return "";
		}
		List<string> list = new List<string>();
		foreach (KeyValuePair<int, double> item in value2)
		{
			double num = item.Value - Singleton<Time>.Instance.ServerCombatStopTime;
			if (num > 0.0)
			{
				list.Add($"{item.Key}:{num / 1000.0:F1}s");
			}
		}
		if (list.Count == 0)
		{
			return "";
		}
		return $", targetCd({list.Count}): [{string.Join(", ", list)}]";
	}

	public double GetBuffEffectCdForTarget(long buffId, int index, int targetEntityId)
	{
		if (targetEntityId == 0)
		{
			return 0.0;
		}
		if (!EffectTargetTimeoutMap.TryGetValue(buffId, out Dictionary<int, Dictionary<int, double>> value))
		{
			return 0.0;
		}
		if (!value.TryGetValue(index, out var value2))
		{
			return 0.0;
		}
		if (!value2.TryGetValue(targetEntityId, out var value3))
		{
			return 0.0;
		}
		if (value3 <= Singleton<Time>.Instance.ServerCombatStopTime)
		{
			return 0.0;
		}
		return value3 - Singleton<Time>.Instance.ServerCombatStopTime;
	}

	public long? CreateAnimNotifyContentWithAnimBp(int anIndex, string animName)
	{
		Skill skill = GetSkillComponent()?.CurrentSkill;
		long? num = skill?.CombatMessageId;
		if (anIndex == -1)
		{
			return null;
		}
		if (!num.HasValue || num == 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("anIndex", anIndex);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("animName", animName);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("SkillId", skill?.SkillId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("contextId", num);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "CreateANCWithAnimBp Error", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
			return null;
		}
		long value = ModelBase<CombatMessageModel>.Instance.GenMessageId();
		ControllerBase<SkillMessageController>.Instance.AnimNotifyRequest(base.Entity, skill?.SkillId ?? 0, -1, anIndex, num.Value, value);
		return value;
	}

	public long? CreateAnimNotifyContentWithoutSkill(MontageInfo montageInfo, int anIndex)
	{
		if (!montageInfo.MontageTaskMessageId.HasValue || montageInfo.MontageTaskMessageId == 0 || anIndex == -1)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("ANIndex", anIndex);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("montageInfo", montageInfo);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "CreateANCWithoutSkill Error", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
			return null;
		}
		long value = ModelBase<CombatMessageModel>.Instance.GenMessageId();
		ControllerBase<SkillMessageController>.Instance.AnimNotifyRequest(base.Entity, -1, -1, anIndex, montageInfo.MontageTaskMessageId.Value, value);
		return value;
	}

	public long? CreateAnimNotifyContentWithSkill(MontageInfo montageInfo, int anIndex)
	{
		Skill skill = GetSkillComponent()?.GetSkill(montageInfo.SkillId.GetValueOrDefault());
		if (skill == null || !montageInfo.MontageIndex.HasValue || !montageInfo.MontageTaskMessageId.HasValue || montageInfo.MontageTaskMessageId == 0 || anIndex == -1)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("ANIndex", anIndex);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("montageInfo", montageInfo);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "CreateANCWithSkill Error", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
			return null;
		}
		int skillId = skill.SkillId;
		long? montageTaskMessageId = montageInfo.MontageTaskMessageId;
		long value = ModelBase<CombatMessageModel>.Instance.GenMessageId();
		ControllerBase<SkillMessageController>.Instance.AnimNotifyRequest(base.Entity, skillId, montageInfo.MontageIndex.Value, anIndex, montageTaskMessageId.Value, value);
		return value;
	}

	public long? CreateAnimNotifyContent(string montageName, int anIndex)
	{
		MontageInfo montageInfo = base.Entity.GetComponent<BaseMontageComponent>()?.GetMontageInfo(montageName);
		if (montageInfo == null)
		{
			return CreateAnimNotifyContentWithAnimBp(anIndex, montageName);
		}
		if (GetSkillComponent()?.GetSkill(montageInfo.SkillId.GetValueOrDefault()) == null)
		{
			return CreateAnimNotifyContentWithoutSkill(montageInfo, anIndex);
		}
		return CreateAnimNotifyContentWithSkill(montageInfo, anIndex);
	}

	public void AddBuffFromAi(long? contextId, long buffId, AddBuffParam buffParams)
	{
		_ = contextId.HasValue;
		buffParams.PreMessageId = contextId;
		AddBuff(buffId, buffParams);
	}

	public void AddBuffForDebug(long buffId, AddBuffParam buffParams)
	{
		buffParams.PreMessageId = -1L;
		AddBuff(buffId, buffParams);
	}

	public virtual void AddBuff(long buffId, AddBuffParam buffParams)
	{
		if (buffId <= 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("创建者", buffParams.InstigatorId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", GetDebugName());
			instance.Error(CombatLog.EDebugModule.Buff, entity, "尝试添加buff时传入了不合法的buffId", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return;
		}
		BaseBuffComponent baseBuffComponent = ModelBase<CreatureModel>.Instance.GetEntity(buffParams.InstigatorId)?.Entity?.GetComponent<BaseBuffComponent>();
		buffId = (baseBuffComponent?.GetReplaceBuffId(buffId, buffParams)).GetValueOrDefault(buffId);
		int[] array = ModelBase<GameModeModel>.Instance.InstanceDungeon?.GetCustomTypesArray();
		bool flag = array != null && Array.IndexOf(array, 5) >= 0;
		if ((!buffParams.PreMessageId.HasValue || buffParams.PreMessageId == 0) && !BuffIdUtils.CheckBuffInSpecialList(buffId) && !flag)
		{
			CombatLog instance2 = Singleton<CombatLog>.Instance;
			Entity entity2 = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer2 = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer2, 0) = ("原因", buffParams.Reason);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer2, 1) = ("buffId", buffId);
			instance2.Error(CombatLog.EDebugModule.Buff, entity2, "加Buff上文无效", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer2, 2));
		}
		else
		{
			buffParams.Level = buffParams.Level ?? baseBuffComponent?.GetBuffLevel(buffId);
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

	public virtual int AddBuffLocal(long buffId, AddBuffParam buffParams)
	{
		long instigatorId = buffParams.InstigatorId;
		int? num = buffParams.Level;
		int? outerStackCount = buffParams.OuterStackCount;
		ApplyGEType valueOrDefault = buffParams.ApplyType.GetValueOrDefault();
		long? preMessageId = buffParams.PreMessageId;
		long? messageId = buffParams.MessageId;
		float? duration = buffParams.Duration ?? ActiveBuffConfigs.USE_INTERNAL_DURATION;
		int valueOrDefault2 = buffParams.ServerId.GetValueOrDefault(-1);
		bool valueOrDefault3 = buffParams.IsIterable.GetValueOrDefault(true);
		bool valueOrDefault4 = buffParams.IsServerOrder.GetValueOrDefault();
		string reason = buffParams.Reason;
		long? bulletMessageId = buffParams.BulletMessageId;
		bool valueOrDefault5 = buffParams.BornBuff.GetValueOrDefault();
		if (!HasBuffAuthority())
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			<>y__InlineArray5<(string, object)> buffer = default(<>y__InlineArray5<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 1) = ("施加者", instigatorId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", GetDebugName());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 3) = ("前置行为id", preMessageId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 4) = ("原因", reason);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "[local] 模拟端不本地添加buff", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray5<(string, object)>, (string, object)>(in buffer, 5));
			return -1;
		}
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(buffId, reason);
		if (buffDefinition == null)
		{
			return -1;
		}
		BaseBuffComponent baseBuffComponent = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId)?.Entity?.GetComponent<BaseBuffComponent>();
		num = num ?? (baseBuffComponent?.GetBuffLevel(buffId)).GetValueOrDefault(1);
		return AddBuffInner(buffId, buffDefinition, instigatorId, num.Value, outerStackCount, valueOrDefault, preMessageId, messageId, duration, null, valueOrDefault2, reason, fromServer: false, valueOrDefault3, valueOrDefault4, null, bulletMessageId, valueOrDefault5);
	}

	protected virtual void AddBuffOrder(long buffId, AddBuffParam buffParams)
	{
	}

	protected void AddBuffRemote(long buffId, int handle, long overrideConfigId, AddBuffParam buffParams)
	{
		long instigatorId = buffParams.InstigatorId;
		int valueOrDefault = buffParams.Level.GetValueOrDefault(1);
		int? outerStackCount = buffParams.OuterStackCount;
		ApplyGEType valueOrDefault2 = buffParams.ApplyType.GetValueOrDefault();
		long? preMessageId = buffParams.PreMessageId;
		long? messageId = buffParams.MessageId;
		float? duration = buffParams.Duration ?? ActiveBuffConfigs.USE_INTERNAL_DURATION;
		float? remainDuration = buffParams.RemainDuration;
		int valueOrDefault3 = buffParams.ServerId.GetValueOrDefault(-1);
		bool? isActive = buffParams.IsActive;
		string reason = buffParams.Reason;
		bool valueOrDefault4 = buffParams.BornBuff.GetValueOrDefault();
		BuffDefinition config = ControllerBase<BuffController>.Instance.GetBuffDefinition(buffId, reason);
		if (overrideConfigId != 0L)
		{
			BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(overrideConfigId);
			if (buffDefinition != null)
			{
				config = buffDefinition;
			}
		}
		int key = AddBuffInner(buffId, config, instigatorId, valueOrDefault, outerStackCount, valueOrDefault2, preMessageId, messageId, duration, isActive, valueOrDefault3, reason, fromServer: true, isIterable: true, isServerOrder: false, handle, null, valueOrDefault4);
		ActiveBuffInternal valueOrDefault5 = BuffContainer.GetValueOrDefault(key);
		if (valueOrDefault5 != null && remainDuration.HasValue)
		{
			valueOrDefault5.SetRemainDuration(remainDuration.Value);
		}
	}

	public void AddIterativeBuff(long buffId, IActiveBuff? preBuff, int? stackCount, bool isIterable, string reason, long? bulletMessageId = null, IBuffComponent? instigator = null)
	{
		if (preBuff == null)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("持有者", GetDebugName());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("原因", reason);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "尝试添加迭代buff失败，未找到前置buff", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return;
		}
		long num = preBuff.InstigatorId.GetValueOrDefault();
		if (instigator != null)
		{
			num = (instigator as BaseBuffComponent)?.CreatureDataId ?? num;
		}
		AddBuff(buffId, new AddBuffParam
		{
			InstigatorId = num,
			Level = preBuff.Level,
			PreMessageId = preBuff.MessageId,
			ServerId = preBuff.ServerId,
			OuterStackCount = stackCount,
			IsIterable = isIterable,
			Reason = reason,
			BulletMessageId = bulletMessageId
		});
	}

	protected static Stat? GetBuffStat(long buffId)
	{
		if (!Stat.Enable)
		{
			return null;
		}
		if (!BuffStatMap.ContainsKey(buffId))
		{
			BuffStatMap[buffId] = Stat.CreateNoFlameGraph(buffId.ToString(), "", "STATGROUP_KuroBattle");
		}
		return BuffStatMap[buffId];
	}

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

	public virtual void RemoveBuff(long buffId, int stackCount, string reason, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
	{
		if (HasBuffAuthority())
		{
			RemoveBuffLocal(buffId, stackCount, reason, preMessageId, isServerRequest, instigatorId);
		}
		else
		{
			RemoveBuffOrder(buffId, stackCount, reason, instigatorId);
		}
	}

	protected virtual void RemoveBuffOrder(long buffId, int stackCount, string reason, long? instigatorId = null)
	{
	}

	public virtual int RemoveBuffLocal(long buffId, int stackCount, string reason, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
	{
		if (buffId <= 0)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("持有者", GetDebugName());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("原因", reason);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "尝试本地移除buff时传入了不合法的buffId", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return 0;
		}
		if (!HasBuffAuthority())
		{
			CombatLog instance2 = Singleton<CombatLog>.Instance;
			Entity entity2 = base.Entity;
			global::<>y__InlineArray3<(string, object)> buffer2 = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 0) = ("buffId", buffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 1) = ("持有者", GetDebugName());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer2, 2) = ("原因", reason);
			instance2.Error(CombatLog.EDebugModule.Buff, entity2, "无法直接移除非本端控制实体持有的buff，请换用RemoveBuff接口", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer2, 3));
			return 0;
		}
		int result = 0;
		ActiveBuffInternal[] array = GetAllBuffById(buffId).ToArray();
		foreach (ActiveBuffInternal activeBuffInternal in array)
		{
			result = RemoveBuffInner(activeBuffInternal.Handle, stackCount, isPrematureRemoval: true, reason, preMessageId, isServerRequest, instigatorId);
		}
		return result;
	}

	public virtual void RemoveBuffByServerId(int serverId, int stackCount, long preMessageId, string reason, long? instigatorId = null)
	{
		IActiveBuff[] allBuffs = GetAllBuffs();
		foreach (IActiveBuff activeBuff in allBuffs)
		{
			if (activeBuff.ServerId == serverId)
			{
				RemoveBuffInner(activeBuff.Handle, stackCount, isPrematureRemoval: true, reason, preMessageId, true, instigatorId);
			}
		}
	}

	public virtual void RemoveBuffByTagLocal(int tagId, string? reason, long? instigatorId = null)
	{
		if (!HasBuffAuthority())
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("tagId", tagId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("tagName", GameplayTagUtils.GetNameByTagId(tagId));
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", GetDebugName());
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("原因", reason);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "尝试根据tag移除非本地控制的实体的buff，移除操作将不被执行", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
			return;
		}
		string nameByTagId = GameplayTagUtils.GetNameByTagId(tagId);
		List<int> list = new List<int>();
		foreach (ActiveBuffInternal value in BuffContainer.Values)
		{
			if (value.Config.GrantedTags == null)
			{
				continue;
			}
			int[] grantedTags = value.Config.GrantedTags;
			for (int i = 0; i < grantedTags.Length; i++)
			{
				if (GameplayTagUtils.IsChildTag(grantedTags[i], tagId))
				{
					list.Add(value.Handle);
					break;
				}
			}
		}
		foreach (int item in list)
		{
			RemoveBuffInner(item, -1, isPrematureRemoval: true, reason ?? ("移除tag " + nameByTagId), null, null, instigatorId);
		}
	}

	public void RemoveBuffByEffectType(EExtraEffectId effectId, string reason)
	{
		HashSet<long> hashSet = new HashSet<long>();
		Type buffEffectClass = ExtraEffectDefine.GetBuffEffectClass(effectId);
		if (buffEffectClass == null)
		{
			return;
		}
		foreach (BuffEffect allEffect in BuffEffectManager.GetAllEffects())
		{
			if (buffEffectClass.IsInstanceOfType(allEffect))
			{
				hashSet.Add(allEffect.BuffId);
			}
		}
		foreach (long item in hashSet)
		{
			RemoveBuff(item, -1, reason);
		}
	}

	public virtual int RemoveBuffByHandle(int handle, int removeStackCount = -1, string? reason = null, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
	{
		if (!HasBuffAuthority())
		{
			IActiveBuff buffByHandle = GetBuffByHandle(handle);
			if (buffByHandle != null && buffByHandle.Id > 0)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity entity = base.Entity;
				<>y__InlineArray5<(string, object)> buffer = default(<>y__InlineArray5<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buffByHandle?.Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", handle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", GetDebugName());
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 3) = ("说明", buffByHandle?.Config?.Desc);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 4) = ("原因", reason);
				instance.Warn(CombatLog.EDebugModule.Buff, entity, "尝试直接通过handle移除非本端控制buff，后续需要新增协议", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray5<(string, object)>, (string, object)>(in buffer, 5));
			}
		}
		return RemoveBuffByHandleLocal(handle, removeStackCount, reason, preMessageId, isServerRequest, instigatorId);
	}

	protected int RemoveBuffByHandleLocal(int handle, int removeStackCount = -1, string? reason = null, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
	{
		return RemoveBuffInner(handle, removeStackCount, isPrematureRemoval: true, reason, preMessageId, isServerRequest, instigatorId);
	}

	public void RemoveBuffWhenTimeout(ActiveBuffInternal buff)
	{
		int stackExpirationRemoveNumber = buff.Config.StackExpirationRemoveNumber;
		RemoveBuffInner(buff.Handle, stackExpirationRemoveNumber, isPrematureRemoval: false, "时间结束自然移除");
		if (stackExpirationRemoveNumber > 0 && buff.IsValid())
		{
			buff.SetDuration();
		}
	}

	public virtual int RemoveBuffInner(int handle, int removeStackCount, bool isPrematureRemoval, string? reason, long? preMessageId = null, bool? isServerRequest = null, long? instigatorId = null)
	{
		if (!(GetBuffByHandle(handle) is ActiveBuffInternal activeBuffInternal))
		{
			return 0;
		}
		BuffLock++;
		<>y__InlineArray5<(string, object)> buffer = default(<>y__InlineArray5<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", activeBuffInternal.Id);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 1) = ("持有者", GetDebugName());
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 2) = ("handle", activeBuffInternal?.Handle);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 3) = ("说明", activeBuffInternal?.Config.Desc);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<<>y__InlineArray5<(string, object)>, (string, object)>(ref buffer, 4) = ("原因", reason);
		ReadOnlySpan<(string, object)> pairs = global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<<>y__InlineArray5<(string, object)>, (string, object)>(in buffer, 5);
		if (!NoLogBuffSet.Contains(activeBuffInternal.Id))
		{
			Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.Buff, base.Entity, "本地移除buff", pairs);
		}
		int stackCount = activeBuffInternal.StackCount;
		int num = ((removeStackCount > 0) ? Math.Max(0, stackCount - removeStackCount) : 0);
		if (num <= 0)
		{
			try
			{
				OnBuffRemoved(activeBuffInternal, isPrematureRemoval, reason, isServerRequest, preMessageId, instigatorId);
			}
			catch (Exception e)
			{
				Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff移除时发生异常", e, pairs);
			}
		}
		else
		{
			if (!isPrematureRemoval && HasBuffRoutineExpirationLock(activeBuffInternal.Id))
			{
				num = stackCount;
			}
			try
			{
				OnBuffStackDecreased(activeBuffInternal, stackCount, num, isPrematureRemoval, reason, EBuffStackPeriodResetOverride.Default, instigatorId);
			}
			catch (Exception e2)
			{
				Singleton<CombatLog>.Instance.ErrorWithStack(CombatLog.EDebugModule.Buff, base.Entity, "Buff层数降低时发生异常", e2, pairs);
			}
		}
		BuffLock--;
		return stackCount - num;
	}

	public IActiveBuff[] GetAllBuffs()
	{
		List<IActiveBuff> list = new List<IActiveBuff>();
		foreach (ActiveBuffInternal value in BuffContainer.Values)
		{
			if (!BuffGarbageSet.Contains(value.Handle))
			{
				list.Add(value);
			}
		}
		return list.ToArray();
	}

	public IActiveBuff? GetBuffByHandle(int handle)
	{
		if (BuffGarbageSet.Contains(handle))
		{
			return null;
		}
		return BuffContainer.GetValueOrDefault(handle);
	}

	public IActiveBuff? GetPendingBuffByHandle(int handleId)
	{
		return BuffContainer.GetValueOrDefault(handleId);
	}

	public bool HasActiveBuff(long buffId)
	{
		return GetActiveBuffById(buffId) != null;
	}

	public bool HasBuff(long buffId, bool excludeRequestRemove = false)
	{
		if (excludeRequestRemove && WaitRemoveBuffResponse.ContainsKey(buffId))
		{
			return false;
		}
		return GetBuffById(buffId) != null;
	}

	public bool IsPendingAddBuff(long buffId, int checkInterval = 200)
	{
		if (!PendingAddBuff.TryGetValue(buffId, out var value))
		{
			return false;
		}
		return Singleton<Time>.Instance.Now - value < (double)checkInterval;
	}

	public virtual BaseBuffComponent? GetBuffApplyTarget(long buffId, long instigatorId)
	{
		return this;
	}

	public ActiveBuffInternal? GetActiveBuffById(long buffId)
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
			else if (valueOrDefault.IsActive() && !BuffGarbageSet.Contains(valueOrDefault.Handle))
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

	public ActiveBuffInternal? GetBuffById(long buffId)
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
			else if (!BuffGarbageSet.Contains(valueOrDefault.Handle))
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

	[IteratorStateMachine(typeof(<GetAllBuffById>d__121))]
	public IEnumerable<ActiveBuffInternal> GetAllBuffById(long buffId)
	{
		//yield-return decompiler failed: Unexpected instruction in Iterator.Dispose()
		return new <GetAllBuffById>d__121(-2)
		{
			<>4__this = this,
			<>3__buffId = buffId
		};
	}

	public HashSet<int> GetBuffHandleByEffectId(int effectId)
	{
		HashSet<int> hashSet = new HashSet<int>();
		foreach (var (item, activeBuffInternal2) in BuffContainer)
		{
			if (BuffGarbageSet.Contains(item))
			{
				continue;
			}
			foreach (ExtraEffectParameters effectInfo in activeBuffInternal2.Config.EffectInfos)
			{
				if (effectInfo.ExtraEffectId == (EExtraEffectId)effectId)
				{
					hashSet.Add(item);
					break;
				}
			}
		}
		return hashSet;
	}

	public virtual int? GetBuffLevel(long buffId)
	{
		return null;
	}

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

	public virtual int GetBuffTotalStackById(long buffId, bool onlyActiveBuff = false)
	{
		if (!BuffIdToHandleMap.TryGetValue(buffId, out HashSet<int> value))
		{
			return 0;
		}
		if (value.Count == 0)
		{
			BuffIdToHandleMap.Remove(buffId);
			return 0;
		}
		int num = 0;
		List<int> list = new List<int>();
		foreach (int item in value)
		{
			ActiveBuffInternal valueOrDefault = BuffContainer.GetValueOrDefault(item);
			if (valueOrDefault == null)
			{
				list.Add(item);
			}
			else if (!BuffGarbageSet.Contains(valueOrDefault.Handle) && (!onlyActiveBuff || valueOrDefault.IsActive()))
			{
				num += valueOrDefault.StackCount;
			}
		}
		foreach (int item2 in list)
		{
			value.Remove(item2);
		}
		return num;
	}

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
			if (!BuffIdToHandleMap.TryGetValue(buff.Id, out HashSet<int> value))
			{
				value = new HashSet<int>();
				BuffIdToHandleMap[buff.Id] = value;
			}
			value.Add(handle);
			MarkListenerBuff(buff);
			BuffEffectManager?.OnBuffAdded(buff);
		}
		if (!NeedCheck(buff.Config))
		{
			if (!isActive.HasValue)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity entity = base.Entity;
				global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buff.Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", buff.Handle);
				instance.Error(CombatLog.EDebugModule.Buff, entity, "buff激活状态未知", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
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

	public void ApplyPeriodExecution(ActiveBuffInternal buff)
	{
		BuffDefinition config = buff.Config;
		BaseAttributeComponent instigatorAttributeSet = buff.GetInstigatorAttributeSet();
		if (config.Modifiers != null && config.Modifiers.Length != 0)
		{
			BaseAttributeComponent attributeComponent = GetAttributeComponent();
			if (attributeComponent == null)
			{
				CombatLog instance = Singleton<CombatLog>.Instance;
				Entity entity = base.Entity;
				global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buff.Id);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", buff.Handle);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("持有者", buff.GetOwnerBuffComponent()?.GetDebugName());
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("施加者", buff.InstigatorId);
				instance.Warn(CombatLog.EDebugModule.Buff, entity, "周期buff尝试修改属性，但owner不存在", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
			}
			else
			{
				float timeScale = GetTimeScale();
				for (int i = 0; i < buff.StateModifiers.Count; i++)
				{
					(IBuffModifierData, float?) tuple = buff.StateModifiers[i];
					if (HasBuffAuthority())
					{
						ActiveBuffInternal.ModifyStateAttribute(instigatorAttributeSet, attributeComponent, tuple.Item1, buff.Level, timeScale, buff.StackCount, tuple.Item2);
					}
				}
			}
		}
		foreach (ExtraEffectParameters effectInfo in config.EffectInfos)
		{
			effectInfo.ExecutionEffect?.OnPeriodCallback(buff);
		}
		foreach (BuffEffect item in BuffEffectManager?.GetEffectsByHandle(buff.Handle) ?? Array.Empty<BuffEffect>())
		{
			item.OnPeriodCallback();
		}
	}

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
			long[] array = ((!isPrematureRemoval) ? buff.Config?.RoutineExpirationEffects : buff.Config?.PrematureExpirationEffects);
			if (array != null)
			{
				long[] array2 = array;
				foreach (long buffId in array2)
				{
					AddIterativeBuff(buffId, buff, null, isIterable: true, $"因为Buff{buff.Id}移除导致的添加");
				}
			}
			long[] buffsAddedByStackCountOnRemoved = buff.Config.BuffsAddedByStackCountOnRemoved;
			if (isPrematureRemoval && buffsAddedByStackCountOnRemoved != null && stackCount - 1 >= 0 && stackCount - 1 < buffsAddedByStackCountOnRemoved.Length)
			{
				long num = buffsAddedByStackCountOnRemoved[stackCount - 1];
				if (num != 0L)
				{
					AddIterativeBuff(num, buff, null, isIterable: true, $"buff{buff.Id}移除时根据buff层数获取指定buff");
				}
			}
		}
		ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.BuffStackChanged, buff.Id, buff.Id, stackCount, 0, base.Entity);
		EmitBuffStackInstigatorChanged(buff.Id, stackCount, 0, instigatorId);
	}

	protected virtual void OnBuffStackDecreased(ActiveBuffInternal buff, int oldStack, int newStack, bool isPrematureRemoval, string? reason, EBuffStackPeriodResetOverride stackPeriodResetPolicy = EBuffStackPeriodResetOverride.Default, long? instigatorId = null)
	{
		if (buff != null)
		{
			buff.SetStackCount(newStack, stackPeriodResetPolicy);
			CheckWhenBuffChanged(buff.Id);
			BuffEffectManager.OnStackDecreased(buff, newStack, oldStack, isPrematureRemoval);
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.BuffStackChanged, buff.Id, buff.Id, oldStack, newStack, base.Entity);
			EmitBuffStackInstigatorChanged(buff.Id, oldStack, newStack, instigatorId);
		}
	}

	protected virtual void OnBuffStackIncreased(ActiveBuffInternal buff, int oldStack, int newStack, long? instigatorId, int level, int? outerStackCount, ApplyGEType applyType, long? preMessageId, float? duration, int serverId, bool isIterable, bool isServerOrder, string? reason, EBuffStackDurationOverride stackDurationRefreshPolicy = EBuffStackDurationOverride.Default, EBuffStackPeriodResetOverride stackPeriodResetPolicy = EBuffStackPeriodResetOverride.Default)
	{
		if (buff == null)
		{
			return;
		}
		buff.SetStackCount(newStack, stackPeriodResetPolicy);
		if (stackDurationRefreshPolicy == EBuffStackDurationOverride.Default && buff.Config.StackDurationRefreshPolicy == EBuffStackDurationRefreshPolicy.Refresh && (NeedCheck(buff.Config) || applyType != 0))
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
			int num = ((buff.StackLimitCount > 0) ? buff.StackLimitCount : int.MaxValue);
			if (oldStack >= num && newStack >= num)
			{
				long[] overflowEffects = config.OverflowEffects;
				if (overflowEffects != null && overflowEffects.Length != 0)
				{
					long[] array = overflowEffects;
					foreach (long buffId in array)
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

	protected virtual void OnBuffActiveChanged(ActiveBuffInternal? buff, bool isActive)
	{
		if (buff == null)
		{
			return;
		}
		buff.SetActivate(isActive);
		foreach (ExtraEffectParameters effectInfo in buff.Config.EffectInfos)
		{
			effectInfo.ExecutionEffect?.OnBuffActiveChangedCallback(buff, isActive);
		}
		CheckWhenBuffChanged(buff.Id);
		BuffEffectManager.OnBuffInhibitedChanged(buff, !isActive);
		OnAnyBuffInhibitionChanged(buff);
	}

	public void OnTagChanged(int tagId)
	{
		if (TagInChanged.Contains(tagId))
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			(string, object) reference = ("tagId", tagId);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "Tag变更循环派发", new ReadOnlySpan<(string, object)>(ref reference));
		}
		else
		{
			TagInChanged.Add(tagId);
			CheckWhenTagChanged(tagId);
			TagInChanged.Remove(tagId);
		}
	}

	public void AddTrigger(int handleId, EBuffTriggerType buffEffectEventType, IBuffTrigger passiveEffects)
	{
		if (!TriggerMap.TryGetValue(buffEffectEventType, out List<IBuffTrigger> value))
		{
			value = new List<IBuffTrigger>();
			TriggerMap[buffEffectEventType] = value;
		}
		value.Add(passiveEffects);
	}

	public void RemoveTrigger(int handleId, EBuffTriggerType extraEffectType)
	{
		if (!TriggerMap.TryGetValue(extraEffectType, out List<IBuffTrigger> value))
		{
			return;
		}
		int num = -1;
		for (int i = 0; i < value.Count; i++)
		{
			if (value[i].ActiveHandleId == handleId)
			{
				num = i;
				break;
			}
		}
		if (num != -1)
		{
			value.RemoveAt(num);
		}
	}

	public virtual void TriggerEvents(EBuffTriggerType passiveEffectType, BaseBuffComponent? opponentBuffComp, Partial_RequirementPayload payload)
	{
		if (opponentBuffComp == null)
		{
			return;
		}
		Singleton<EventSystem>.Instance.EmitWithTarget(base.Entity, EEventName.OnBuffTriggerEvent, passiveEffectType, opponentBuffComp, payload);
		if (TriggerMap.TryGetValue(passiveEffectType, out List<IBuffTrigger> value))
		{
			IBuffTrigger[] array = value.ToArray();
			for (int i = 0; i < array.Length; i++)
			{
				array[i].TryExecute(payload, opponentBuffComp);
			}
		}
	}

	public static Entity? ShouldForwardToFrontChar(Entity entity)
	{
		FollowShooterComponent component = entity.GetComponent<FollowShooterComponent>();
		if (!component.IsAutonomousProxy)
		{
			return null;
		}
		SceneTeamItem getCurrentTeamItem = ModelBase<SceneTeamModel>.Instance.GetCurrentTeamItem;
		if (getCurrentTeamItem?.EntityHandle?.Entity == null)
		{
			return null;
		}
		WorldEntity entity2 = getCurrentTeamItem.EntityHandle.Entity;
		CreatureDataComponent component2 = entity2.GetComponent<CreatureDataComponent>();
		if (component2 == null)
		{
			return null;
		}
		int roleId = component2.GetRoleId();
		if (component.GetApplyTriggerRoles().Contains(roleId))
		{
			return entity2;
		}
		return null;
	}

	public static bool IsFollowShooterByEntity(Entity entity)
	{
		if (entity.GetComponent<FollowShooterComponent>() == null)
		{
			return false;
		}
		return true;
	}

	public bool HasBuffTrigger(EBuffTriggerType effectType)
	{
		return TriggerMap.ContainsKey(effectType);
	}

	public int AddGameplayCue(long[] cueIds, float duration, string reason)
	{
		if (cueIds == null || cueIds.Length == 0)
		{
			return -1;
		}
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.CreateDynamicBuffRef();
		buffDefinition.GameplayCueIds = cueIds;
		buffDefinition.Desc = reason;
		buffDefinition.DurationPolicy = ((duration != 0f) ? ((duration < 0f) ? EBuffDurationType.Infinite : EBuffDurationType.HasDuration) : EBuffDurationType.Instant);
		if (duration > 0f)
		{
			buffDefinition.DurationCalculationPolicy = new int[1];
			buffDefinition.DurationMagnitude = new float[1] { duration };
		}
		return AddBuffInner(-3L, buffDefinition, null, 1, null, ApplyGEType.Common, null, null, duration, null, -1, reason, fromServer: false, isIterable: true, isServerOrder: false, null);
	}

	[CombatListen(ENotifyMessageId.ApplyGameplayEffectNotify, false, false)]
	public static void BroadcastAddBuffNotify(Entity? entity, ApplyGameplayEffectNotify data, CombatCommon? combatCommon = null)
	{
		int handle = data.Handle;
		long id = data.Id;
		long instigatorId = data.InstigatorId;
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		long value = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.PreMessageId ?? (-1));
		long value2 = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.MessageId ?? (-1));
		if (baseBuffComponent == null || !baseBuffComponent.Valid)
		{
			return;
		}
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(id);
		long num = 0L;
		if (data.ConfBuffId != 0L)
		{
			num = data.ConfBuffId;
			BuffDefinition buffDefinition2 = ControllerBase<BuffController>.Instance.GetBuffDefinition(num);
			if (buffDefinition2 != null)
			{
				buffDefinition = buffDefinition2;
			}
		}
		if (buffDefinition == null || handle == -1)
		{
			return;
		}
		BaseTagComponent tagComponent = baseBuffComponent.GetTagComponent();
		if (tagComponent != null && buffDefinition.RemoveBuffWithTags != null)
		{
			int[] removeBuffWithTags = buffDefinition.RemoveBuffWithTags;
			foreach (int value3 in removeBuffWithTags)
			{
				tagComponent.RemoveTag(value3);
			}
		}
		baseBuffComponent.AddBuffRemote(id, handle, num, new AddBuffParam
		{
			IsActive = data.IsActive,
			InstigatorId = instigatorId,
			Level = data.Level,
			ApplyType = data.ApplyType,
			PreMessageId = value,
			MessageId = value2,
			ServerId = data.ServerId,
			Duration = (data.HasDuration ? new float?(data.Duration) : null),
			OuterStackCount = data.StackCount,
			RemainDuration = (data.HasLeftDuration ? new float?(data.LeftDuration) : null),
			Reason = "远端通知添加buff"
		});
	}

	[CombatListen(ENotifyMessageId.ApplyBuffFailedNotify, false, false)]
	public static void BroadcastAddBuffFailedNotify(Entity? entity, ApplyBuffFailedNotify data, CombatCommon? combatCommon = null)
	{
		long instigatorId = data.InstigatorId;
		WorldEntity worldEntity = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId)?.Entity;
		long buffId = data.BuffId;
		if (entity != null && worldEntity != null)
		{
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent((Entity?)worldEntity, EAbilityEventName.AddBuffFailure, buffId, buffId, entity, (Entity)worldEntity, data.StackCount, (long?)Singleton<MathUtils>.Instance.LongToBigInt(data.TransferContextId?.BulletContextId ?? 0));
		}
	}

	[CombatListen(ENotifyMessageId.BuffStackCountNotify, false, false)]
	public static void BroadcastBuffStackChangedNotify(Entity? entity, BuffStackCountNotify data, CombatCommon? combatCommon = null)
	{
		int handleId = data.HandleId;
		int newStackCount = data.NewStackCount;
		long instigatorId = data.InstigatorId;
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		ActiveBuffInternal activeBuffInternal = baseBuffComponent?.GetBuffByHandle(handleId) as ActiveBuffInternal;
		if (baseBuffComponent == null || activeBuffInternal == null)
		{
			return;
		}
		if (activeBuffInternal.StackCount > newStackCount)
		{
			baseBuffComponent.OnBuffStackDecreased(activeBuffInternal, activeBuffInternal.StackCount, newStackCount, isPrematureRemoval: true, "远端Buff层数变化通知", data.NotRefreshPeriod ? EBuffStackPeriodResetOverride.ForceNoRefresh : EBuffStackPeriodResetOverride.Default, instigatorId);
		}
		else if (activeBuffInternal.StackCount <= newStackCount)
		{
			WorldEntity worldEntity = ModelBase<CreatureModel>.Instance.GetEntity(instigatorId)?.Entity;
			if (worldEntity != null)
			{
				ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent<long, Entity, Entity, bool?>(worldEntity, EAbilityEventName.AddBuffToOthers, activeBuffInternal.Id, activeBuffInternal.Id, baseBuffComponent.Entity, worldEntity, null);
			}
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(entity, EAbilityEventName.OnBuffAdd, activeBuffInternal.Id, activeBuffInternal.Id, handleId);
			baseBuffComponent.OnBuffStackIncreased(activeBuffInternal, activeBuffInternal.StackCount, newStackCount, instigatorId, activeBuffInternal.Level, null, ApplyGEType.Common, null, null, activeBuffInternal.ServerId, isIterable: false, isServerOrder: false, "远端Buff层数变化通知", data.NotRefreshDuration ? EBuffStackDurationOverride.ForceNoRefresh : EBuffStackDurationOverride.Default, data.NotRefreshPeriod ? EBuffStackPeriodResetOverride.ForceNoRefresh : EBuffStackPeriodResetOverride.Default);
		}
		if (data.HasDuration)
		{
			activeBuffInternal.SetDuration(data.Duration);
		}
		if (data.HasLeftDuration)
		{
			activeBuffInternal.SetRemainDuration(data.LeftDuration);
		}
	}

	[CombatListen(ENotifyMessageId.RemoveGameplayEffectNotify, false, false)]
	public static void BroadcastRemoveBuffNotify(Entity? entity, RemoveGameplayEffectNotify data, CombatCommon? combatCommon = null)
	{
		int handle = data.Handle;
		long instigatorId = data.InstigatorId;
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		if (baseBuffComponent == null || !baseBuffComponent.Valid)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("entity id", data.EntityId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("handle", data.Handle);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "Invalid entity when processing RemoveGameplayEffectNotify", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
		}
		else
		{
			baseBuffComponent.RemoveBuffInner(handle, -1, isPrematureRemoval: true, "远端通知移除buff", null, null, instigatorId);
		}
	}

	[CombatListen(ENotifyMessageId.BuffDurationNotify, false, false)]
	public static void BuffDurationNotify(Entity? entity, BuffDurationNotify data, CombatCommon? combatCommon = null)
	{
		int handleId = data.HandleId;
		BaseBuffComponent? obj = entity?.GetComponent<BaseBuffComponent>();
		ActiveBuffInternal activeBuffInternal = obj?.GetBuffByHandle(handleId) as ActiveBuffInternal;
		if (obj != null && activeBuffInternal != null)
		{
			if (data.HasDuration)
			{
				activeBuffInternal.SetDuration(data.Duration);
			}
			if (data.HasLeftDuration)
			{
				activeBuffInternal.SetRemainDuration(data.LeftDuration);
			}
		}
	}

	[CombatListen(ENotifyMessageId.ApplyBuffS2cRequestNotify, false, true)]
	public static void OrderAddBuffS2cNotify(Entity? entity, ApplyBuffS2cRequestNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		long id = data.Id;
		long instigatorId = data.InstigatorId;
		long value = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.MessageId ?? (-1));
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(id);
		ApplyBuffS2cResponsePush applyBuffS2cResponsePush = new ApplyBuffS2cResponsePush();
		if (entity == null || baseBuffComponent == null || !baseBuffComponent.Valid || buffDefinition == null)
		{
			applyBuffS2cResponsePush.ErrorCode = ErrorCode.UnKnownError;
			Singleton<CombatNet>.Instance.Send(EPushMessageId.ApplyBuffS2cResponsePush, entity, (IMessage)(object)applyBuffS2cResponsePush, value, null, true);
			return;
		}
		BaseTagComponent tagComponent = baseBuffComponent.GetTagComponent();
		if (tagComponent != null && buffDefinition.RemoveBuffWithTags != null)
		{
			int[] removeBuffWithTags = buffDefinition.RemoveBuffWithTags;
			foreach (int value2 in removeBuffWithTags)
			{
				tagComponent.RemoveTag(value2);
			}
		}
		int num = baseBuffComponent.AddBuffInner(id, ControllerBase<BuffController>.Instance.GetBuffDefinition(id), instigatorId, data.Level, data.StackCount, data.ApplyType, value, null, data.HasDuration ? new float?(data.Duration) : null, null, data.ServerId, "远端请求添加玩家Buff(s2c)", fromServer: false, data.IsIterable, isServerOrder: true, null);
		ActiveBuffInternal valueOrDefault = baseBuffComponent.BuffContainer.GetValueOrDefault(num);
		applyBuffS2cResponsePush.Handle = num;
		applyBuffS2cResponsePush.IsActive = valueOrDefault?.IsActive() ?? false;
		applyBuffS2cResponsePush.ErrorCode = ErrorCode.Success;
		Singleton<CombatNet>.Instance.Send(EPushMessageId.ApplyBuffS2cResponsePush, entity, (IMessage)(object)applyBuffS2cResponsePush, value, null, true);
	}

	[CombatListen(ENotifyMessageId.RemoveBuffS2cRequestNotify, false, false)]
	public static void OrderRemoveBuffS2cNotify(Entity? entity, RemoveBuffS2cRequestNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent? obj = entity?.GetComponent<BaseBuffComponent>();
		long value = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.MessageId ?? (-1));
		long instigatorId = data.InstigatorId;
		obj?.RemoveBuffByHandle(data.Handle, data.StackCount, "远端请求移除buff(s2c)", value, true, instigatorId);
		RemoveBuffS2cResponsePush removeBuffS2cResponsePush = new RemoveBuffS2cResponsePush();
		removeBuffS2cResponsePush.ErrorCode = ErrorCode.Success;
		Singleton<CombatNet>.Instance.Send(EPushMessageId.RemoveBuffS2cResponsePush, entity, (IMessage)(object)removeBuffS2cResponsePush, value, null, true);
	}

	[CombatListen(ENotifyMessageId.RemoveBuffByIdS2cRequestNotify, false, false)]
	public static void OrderRemoveBuffByIdS2cNotify(Entity? entity, RemoveBuffByIdS2cRequestNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		long buffId = data.BuffId;
		long value = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.MessageId ?? (-1));
		RemoveBuffByIdS2cResponsePush removeBuffByIdS2cResponsePush = new RemoveBuffByIdS2cResponsePush();
		long instigatorId = data.InstigatorId;
		if (baseBuffComponent == null || !baseBuffComponent.Valid)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("entity id", entity?.Id);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("buffId", buffId);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "Invalid entity when processing RemoveBuffByIdS2cRequestNotify", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
			removeBuffByIdS2cResponsePush.ErrorCode = ErrorCode.UnKnownError;
			Singleton<CombatNet>.Instance.Send(EPushMessageId.RemoveBuffByIdS2cResponsePush, entity, (IMessage)(object)removeBuffByIdS2cResponsePush, value, null, true);
		}
		else
		{
			baseBuffComponent.RemoveBuff(buffId, data.StackCount, $"远端移除buff(s2c) reason={data.Reason}", value, true, instigatorId);
			removeBuffByIdS2cResponsePush.ErrorCode = ErrorCode.Success;
			Singleton<CombatNet>.Instance.Send(EPushMessageId.RemoveBuffByIdS2cResponsePush, entity, (IMessage)(object)removeBuffByIdS2cResponsePush, value, null, true);
		}
	}

	[CombatListen(ENotifyMessageId.RemoveBuffByServerIdS2cRequestNotify, false, false)]
	public static void OrderRemoveBuffByServerIdNotify(Entity? entity, RemoveBuffByServerIdS2cRequestNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		if (baseBuffComponent == null || !baseBuffComponent.HasBuffAuthority())
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("serverId", data.ServerId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("reason", data.Reason);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "[buffComp] 服务端通知移除非主控buff", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
		}
		else
		{
			int serverId = data.ServerId;
			int reason = data.Reason;
			int stackCount = data.StackCount;
			long preMessageId = Singleton<MathUtils>.Instance.LongToBigInt(combatCommon?.MessageId ?? (-1));
			long instigatorId = data.InstigatorId;
			baseBuffComponent.RemoveBuffByServerId(serverId, stackCount, preMessageId, $"远端移除buff(s2c):{reason}", instigatorId);
		}
	}

	[CombatListen(ENotifyMessageId.OrderApplyBuffNotify, false, false)]
	public static void OrderAddBuffNotify(Entity? entity, OrderApplyBuffNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.CheckGetComponent<BaseBuffComponent>();
		if (baseBuffComponent == null)
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("entity", entity);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("buffId", data.Id);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("InstigatorId", data.InstigatorId);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "收到服务端请求添加buff，但找不到对应的entity", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
			return;
		}
		if (baseBuffComponent == null)
		{
			return;
		}
		long id = data.Id;
		AddBuffParam obj = new AddBuffParam
		{
			InstigatorId = data.InstigatorId,
			Level = data.Level,
			ApplyType = data.ApplyType
		};
		long? preMessageId;
		if (combatCommon != null)
		{
			_ = combatCommon.MessageId;
			if (combatCommon.MessageId != 0L)
			{
				preMessageId = combatCommon.MessageId;
				goto IL_00f9;
			}
		}
		preMessageId = null;
		goto IL_00f9;
		IL_00f9:
		obj.PreMessageId = preMessageId;
		obj.Duration = (data.HasDuration ? new float?(data.Duration) : null);
		obj.IsIterable = data.IsIterable;
		obj.OuterStackCount = data.StackCount;
		obj.ServerId = data.ServerId;
		obj.IsServerOrder = true;
		obj.Reason = "服务端或其它客户端请求添加Buff";
		baseBuffComponent.AddBuffLocal(id, obj);
	}

	[CombatListen(ENotifyMessageId.OrderRemoveBuffNotify, false, false)]
	public static void OrderRemoveBuffNotify(Entity? entity, OrderRemoveBuffNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		long id = data.Id;
		if (baseBuffComponent == null)
		{
			BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(id);
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", id);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("持有者", entity?.Id ?? 0);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("说明", buffDefinition?.Desc ?? "");
			instance.Error(CombatLog.EDebugModule.Buff, entity, "[order] 移除Buff请求找不到对应实体", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
		}
		else
		{
			baseBuffComponent.RemoveBuffLocal(id, data.StackCount, "服务端或其它客户端请求本端移除buff");
		}
	}

	[CombatListen(ENotifyMessageId.OrderRemoveBuffByTagsNotify, false, false)]
	public static void OrderRemoveBuffByTagsNotify(Entity? entity, OrderRemoveBuffByTagsNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		long instigatorId = data.InstigatorId;
		if (baseBuffComponent == null)
		{
			StringBuilder stringBuilder = new StringBuilder();
			foreach (int tagId in data.TagIds)
			{
				if (stringBuilder.Length > 0)
				{
					stringBuilder.Append(", ");
				}
				stringBuilder.Append(GameplayTagUtils.GetNameByTagId(tagId) ?? "");
			}
			CombatLog instance = Singleton<CombatLog>.Instance;
			(string, object) reference = ("tagId", stringBuilder);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "[order] 根据tag移除Buff请求找不到对应实体", new ReadOnlySpan<(string, object)>(ref reference));
			return;
		}
		foreach (int tagId2 in data.TagIds)
		{
			string nameByTagId = GameplayTagUtils.GetNameByTagId(tagId2);
			baseBuffComponent.RemoveBuffByTagLocal(tagId2, "远端请求根据tag " + nameByTagId + " 移除buff", instigatorId);
		}
	}

	[CombatListen(ENotifyMessageId.ActivateBuffNotify, false, false)]
	public static void BroadcastActivateBuffNotify(Entity? entity, ActivateBuffNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent baseBuffComponent = entity?.GetComponent<BaseBuffComponent>();
		int handle = data.Handle;
		bool on = data.On;
		ActiveBuffInternal activeBuffInternal = baseBuffComponent?.GetBuffByHandle(handle) as ActiveBuffInternal;
		if (baseBuffComponent == null || baseBuffComponent.HasBuffAuthority())
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			global::<>y__InlineArray4<(string, object)> buffer = default(global::<>y__InlineArray4<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 0) = ("handle", handle);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 1) = ("buffId", activeBuffInternal?.Id ?? 0);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 2) = ("本地激活状态", on);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray4<(string, object)>, (string, object)>(ref buffer, 3) = ("远端激活状态", data.On);
			instance.Warn(CombatLog.EDebugModule.Buff, entity, "主端收到了非主控端发来的buff激活状态变更请求，或找不到Buff持有者，将不做处理", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray4<(string, object)>, (string, object)>(in buffer, 4));
		}
		else
		{
			baseBuffComponent.OnBuffActiveChanged(activeBuffInternal, on);
		}
	}

	[CombatListen(ENotifyMessageId.TransformBuffStackNotify, false, false)]
	public static void BroadcastTransformBuffStackNotify(Entity? entity, TransformBuffStackNotify data, CombatCommon? combatCommon = null)
	{
		BaseBuffComponent? obj = entity?.GetComponent<BaseBuffComponent>();
		long buffId = data.BuffId;
		int handle = (int)data.BuffHandle;
		obj?.AddBuffStackModifier(buffId, handle, data.StackLimit);
	}

	public void AddBuffTimeModifier(long targetBuffId, int handle, float period, float duration, bool asInstigator)
	{
		if (period != 0f || duration != 0f)
		{
			Dictionary<long, Dictionary<int, (float, float)>> dictionary = (asInstigator ? InstigatorBuffTimeModifiers : OwnerBuffTimeModifiers);
			if (!dictionary.TryGetValue(targetBuffId, out var value))
			{
				value = (dictionary[targetBuffId] = new Dictionary<int, (float, float)>());
			}
			value[handle] = (period, duration);
		}
	}

	public void RemoveBuffTimeModifier(long targetBuffId, int handle, bool asInstigator)
	{
		Dictionary<long, Dictionary<int, (float, float)>> dictionary = (asInstigator ? InstigatorBuffTimeModifiers : OwnerBuffTimeModifiers);
		if (dictionary.TryGetValue(targetBuffId, out var value))
		{
			value.Remove(handle);
			if (value.Count <= 0)
			{
				dictionary.Remove(targetBuffId);
			}
		}
	}

	public float CalculateDurationRate(long buffId, BaseBuffComponent? instigator)
	{
		float num = CalculateDurationExtraRate(buffId, asInstigator: false);
		if (instigator != null)
		{
			num += instigator.CalculateDurationExtraRate(buffId, asInstigator: true);
		}
		return Math.Max(1E-08f, 1f + num * 0.0001f);
	}

	public virtual float CalculateDurationExtraRate(long buffId, bool asInstigator)
	{
		if (!(asInstigator ? InstigatorBuffTimeModifiers : OwnerBuffTimeModifiers).TryGetValue(buffId, out Dictionary<int, (float, float)> value) || value.Count <= 0)
		{
			return 0f;
		}
		float num = 0f;
		foreach (var value2 in value.Values)
		{
			float item = value2.Item2;
			num += item;
		}
		return num;
	}

	public float CalculatePeriodRate(long buffId, BaseBuffComponent? instigator)
	{
		float num = CalculatePeriodExtraRate(buffId, asInstigator: false);
		if (instigator != null)
		{
			num += instigator.CalculatePeriodExtraRate(buffId, asInstigator: true);
		}
		return Math.Max(1E-08f, 1f + num * 0.0001f);
	}

	public virtual float CalculatePeriodExtraRate(long buffId, bool asInstigator)
	{
		if (!(asInstigator ? InstigatorBuffTimeModifiers : OwnerBuffTimeModifiers).TryGetValue(buffId, out Dictionary<int, (float, float)> value) || value.Count <= 0)
		{
			return 1f;
		}
		float num = 0f;
		foreach (var value2 in value.Values)
		{
			float item = value2.Item1;
			num += item;
		}
		return num;
	}

	protected void OnRefreshBuffDuration(long[] buffIds, string? reason)
	{
		RefreshBuffDurationPush refreshBuffDurationPush = RefreshBuffDurationPush.Create();
		refreshBuffDurationPush.BuffIds.AddRange((IEnumerable<long>)buffIds);
		Singleton<CombatNet>.Instance.Send(EPushMessageId.RefreshBuffDurationPush, base.Entity, (IMessage)(object)refreshBuffDurationPush);
		foreach (long buffId in buffIds)
		{
			foreach (ActiveBuffInternal item in GetAllBuffById(buffId))
			{
				item.SetDuration();
			}
		}
	}

	protected virtual void RefreshBuffDurationOrder(long[] buffIds, string? reason)
	{
	}

	public void RefreshBuffDuration(long[] buffIds, string? reason)
	{
		if (HasBuffAuthority())
		{
			OnRefreshBuffDuration(buffIds, reason);
		}
		else
		{
			RefreshBuffDurationOrder(buffIds, reason);
		}
	}

	public void AddBuffStackModifier(long targetBuffId, int handle, int value, EBuffStackModifierType modifierType = EBuffStackModifierType.Incremental)
	{
		if (CheckBuffStackChangeable(targetBuffId) && value != 0)
		{
			if (!BuffStackModifiers.TryGetValue(targetBuffId, out Dictionary<int, BuffStackModifierValue> value2))
			{
				value2 = new Dictionary<int, BuffStackModifierValue>();
				BuffStackModifiers[targetBuffId] = value2;
			}
			value2[handle] = new BuffStackModifierValue
			{
				Value = value,
				Type = modifierType
			};
			RefreshBuffStack(targetBuffId);
			ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.OnBuffMaxStackModifier, targetBuffId, targetBuffId);
		}
	}

	public void RemoveBuffStackModifier(long targetBuffId, int handle)
	{
		if (!CheckBuffStackChangeable(targetBuffId))
		{
			return;
		}
		if (BuffStackModifiers.TryGetValue(targetBuffId, out Dictionary<int, BuffStackModifierValue> value))
		{
			value.Remove(handle);
			if (value.Count <= 0)
			{
				BuffStackModifiers.Remove(targetBuffId);
			}
		}
		RefreshBuffStack(targetBuffId);
		ControllerBase<SceneTeamController>.Instance.EmitAbilityEvent(base.Entity, EAbilityEventName.OnBuffMaxStackModifier, targetBuffId, targetBuffId);
	}

	public int? CalculateBuffStackMaxByConfig(BuffDefinition config, long buffId)
	{
		if (config.StackLimitCount <= 0)
		{
			return int.MaxValue;
		}
		int num = 0;
		if (BuffStackModifiers.TryGetValue(buffId, out Dictionary<int, BuffStackModifierValue> value))
		{
			foreach (BuffStackModifierValue value2 in value.Values)
			{
				if (value2.Type == EBuffStackModifierType.Incremental)
				{
					num += value2.Value;
				}
				else if (value2.Type == EBuffStackModifierType.Override)
				{
					return value2.Value;
				}
			}
		}
		return Math.Max(1, config.StackLimitCount + num);
	}

	public int? CalculateBuffStackMax(long buffId)
	{
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(buffId);
		if (buffDefinition == null)
		{
			return 0;
		}
		if (buffDefinition.StackLimitCount <= 0)
		{
			return int.MaxValue;
		}
		int num = 0;
		if (BuffStackModifiers.TryGetValue(buffId, out Dictionary<int, BuffStackModifierValue> value))
		{
			foreach (BuffStackModifierValue value2 in value.Values)
			{
				if (value2.Type == EBuffStackModifierType.Incremental)
				{
					num += value2.Value;
				}
				else if (value2.Type == EBuffStackModifierType.Override)
				{
					return value2.Value;
				}
			}
		}
		return Math.Max(1, buffDefinition.StackLimitCount + num);
	}

	public int RefreshBuffStackMax(long buffId)
	{
		ActiveBuffInternal buffById = GetBuffById(buffId);
		if (buffById == null)
		{
			return 0;
		}
		int? num = CalculateBuffStackMax(buffId);
		if (!num.HasValue)
		{
			return 0;
		}
		if (num.GetValueOrDefault() == int.MaxValue)
		{
			return int.MaxValue;
		}
		buffById.StackLimitCount = num.Value;
		return buffById.StackLimitCount;
	}

	public void RefreshBuffStack(long buffId)
	{
		ActiveBuffInternal buffById = GetBuffById(buffId);
		if (buffById == null)
		{
			return;
		}
		int stackCount = buffById.StackCount;
		int num = RefreshBuffStackMax(buffId);
		if (num < stackCount)
		{
			if (HasBuffAuthority())
			{
				BuffEffectManager?.OnBuffStackOverflow(buffById, stackCount, stackCount, num);
			}
			OnBuffStackDecreased(buffById, stackCount, num, isPrematureRemoval: true, "buff层数修改器导致");
		}
	}

	public bool CheckBuffStackChangeable(long buffId)
	{
		BuffDefinition buffDefinition = ControllerBase<BuffController>.Instance.GetBuffDefinition(buffId);
		if (buffDefinition != null)
		{
			return buffDefinition.StackLimitCount > 0;
		}
		return false;
	}

	public void ChangeBuffStack(long buffId, EBuffStackDurationOverride stackDurationRefreshPolicy, EBuffStackPeriodResetOverride stackPeriodResetPolicy, int stackChange, string reason, long? instigatorId = null)
	{
		if (!HasBuffAuthority())
		{
			return;
		}
		foreach (ActiveBuffInternal item in GetAllBuffById(buffId))
		{
			int stackCount = item.StackCount;
			int stackLimitCount = item.StackLimitCount;
			int num = Math.Min(stackCount + stackChange, stackLimitCount);
			if (num <= 0)
			{
				RemoveBuffInner(item.Handle, -1, isPrematureRemoval: true, reason, null, null, instigatorId);
			}
			else if (num < stackCount)
			{
				OnBuffStackDecreased(item, stackCount, num, isPrematureRemoval: true, reason, stackPeriodResetPolicy, instigatorId);
			}
			else if (num > stackCount)
			{
				OnBuffStackIncreased(item, stackCount, num, instigatorId, item.Level, null, ApplyGEType.Common, null, null, item.ServerId, isIterable: false, isServerOrder: false, reason, stackDurationRefreshPolicy, stackPeriodResetPolicy);
			}
		}
	}

	public void SetDeferredCueCreationEnable(bool enable)
	{
		EnableDeferredCueCreation = enable;
	}

	public void ProcessPendingCues()
	{
		while (PendingInstantCues.Count > 0 && !ControllerBase<BuffController>.Instance.CueTimeLimit.IsTimeLimitExceeded())
		{
			double microseconds = KuroTime.GetMicroseconds64();
			PendingInstantCueItem pendingInstantCueItem = PendingInstantCues[0];
			PendingInstantCues.RemoveAt(0);
			GetCueComponent()?.AddCue(pendingInstantCueItem.CueId, pendingInstantCueItem.Param);
			double microseconds2 = KuroTime.GetMicroseconds64();
			ControllerBase<BuffController>.Instance.CueTimeLimit.AddCost(microseconds2 - microseconds);
		}
		while (PendingDestroyCueBuffs.Count > 0 && !ControllerBase<BuffController>.Instance.CueTimeLimit.IsTimeLimitExceeded())
		{
			double microseconds3 = KuroTime.GetMicroseconds64();
			int? num = null;
			using (HashSet<int>.Enumerator enumerator = PendingDestroyCueBuffs.GetEnumerator())
			{
				if (enumerator.MoveNext())
				{
					int current = enumerator.Current;
					num = current;
				}
			}
			if (num.HasValue)
			{
				PendingDestroyCueBuffs.Remove(num.Value);
				DestroyGameplayCueByBuffImmediate(num.Value);
			}
			double microseconds4 = KuroTime.GetMicroseconds64();
			ControllerBase<BuffController>.Instance.CueTimeLimit.AddCost(microseconds4 - microseconds3);
		}
		while (PendingCreateCueBuffs.Count > 0 && !ControllerBase<BuffController>.Instance.CueTimeLimit.IsTimeLimitExceeded())
		{
			double microseconds5 = KuroTime.GetMicroseconds64();
			int? num2 = null;
			using (HashSet<int>.Enumerator enumerator = PendingCreateCueBuffs.GetEnumerator())
			{
				if (enumerator.MoveNext())
				{
					int current2 = enumerator.Current;
					num2 = current2;
				}
			}
			if (num2.HasValue)
			{
				PendingCreateCueBuffs.Remove(num2.Value);
				IActiveBuff buffByHandle = GetBuffByHandle(num2.Value);
				if (buffByHandle != null && buffByHandle.IsActive() && !Buff2CueMap.ContainsKey(num2.Value))
				{
					CreateGameplayCueByBuffImmediate(buffByHandle);
				}
			}
			double microseconds6 = KuroTime.GetMicroseconds64();
			ControllerBase<BuffController>.Instance.CueTimeLimit.AddCost(microseconds6 - microseconds5);
		}
		if (PendingCreateCueBuffs.Count == 0 && PendingDestroyCueBuffs.Count == 0 && PendingInstantCues.Count == 0)
		{
			ControllerBase<BuffController>.Instance.UnregisterBuffComponent(this);
		}
	}

	protected void CreateGameplayCueByBuff(IActiveBuff buff)
	{
		if (buff.Config.GameplayCueIds == null || buff.Config.GameplayCueIds.Length == 0)
		{
			return;
		}
		if (PendingDestroyCueBuffs.Contains(buff.Handle))
		{
			PendingDestroyCueBuffs.Remove(buff.Handle);
			return;
		}
		if (!EnableDeferredCueCreation)
		{
			CreateGameplayCueByBuffImmediate(buff);
			return;
		}
		if (buff.IsInstantBuff())
		{
			Entity instigator = buff.GetInstigator();
			long[] gameplayCueIds = buff.Config.GameplayCueIds;
			foreach (long cueId in gameplayCueIds)
			{
				long cueId2 = ReplaceAbnormalCueEffect.ApplyEffects(base.Entity, cueId);
				PendingInstantCues.Add(new PendingInstantCueItem
				{
					CueId = cueId2,
					Param = new GameplayCueParam
					{
						Instant = true,
						Instigator = ModelBase<CharacterModel>.Instance.GetHandleByEntity(instigator)
					}
				});
			}
		}
		else
		{
			PendingCreateCueBuffs.Add(buff.Handle);
		}
		ControllerBase<BuffController>.Instance.RegisterBuffComponent(this);
	}

	protected void CreateGameplayCueByBuffImmediate(IActiveBuff buff)
	{
		BaseGameplayCueComponent cueComponent = GetCueComponent();
		if (cueComponent == null || !buff.IsActive())
		{
			return;
		}
		int handle = buff.Handle;
		if (Buff2CueMap.ContainsKey(handle) || CreateSeamlessTravelCue(buff, cueComponent) || buff.Config.GameplayCueIds == null)
		{
			return;
		}
		long[] gameplayCueIds = buff.Config.GameplayCueIds;
		foreach (long cueId in gameplayCueIds)
		{
			long cueId2 = ReplaceAbnormalCueEffect.ApplyEffects(base.Entity, cueId);
			int num = cueComponent.AddCue(cueId2, new GameplayCueParam
			{
				Buff = buff,
				Instant = buff.IsInstantBuff(),
				Instigator = ModelBase<CharacterModel>.Instance.GetHandleByEntity(buff.GetInstigator())
			});
			if (num != -1 && num != 0)
			{
				if (!Buff2CueMap.TryGetValue(handle, out List<int> value))
				{
					value = new List<int>();
					Buff2CueMap[handle] = value;
				}
				value.Add(num);
			}
		}
	}

	protected void DestroyGameplayCueByBuff(IActiveBuff buff)
	{
		int handle = buff.Handle;
		if (PendingCreateCueBuffs.Contains(handle))
		{
			PendingCreateCueBuffs.Remove(handle);
		}
		else if (!EnableDeferredCueCreation)
		{
			DestroyGameplayCueByBuffImmediate(handle);
		}
		else if (Buff2CueMap.ContainsKey(handle))
		{
			PendingDestroyCueBuffs.Add(handle);
			ControllerBase<BuffController>.Instance.RegisterBuffComponent(this);
		}
	}

	protected void DestroyGameplayCueByBuffImmediate(int buffHandle)
	{
		BaseGameplayCueComponent cueComponent = GetCueComponent();
		if (cueComponent == null || !Buff2CueMap.TryGetValue(buffHandle, out List<int> value))
		{
			return;
		}
		foreach (int item in value)
		{
			cueComponent.RemoveCueByHandle(item);
		}
		Buff2CueMap.Remove(buffHandle);
	}

	protected virtual void OnAnyBuffInhibitionChanged(IActiveBuff buff)
	{
		if (buff.IsActive())
		{
			CreateGameplayCueByBuff(buff);
		}
		else
		{
			DestroyGameplayCueByBuff(buff);
		}
	}

	protected bool CreateSeamlessTravelCue(IActiveBuff buff, BaseGameplayCueComponent cueComp)
	{
		if (SeamlessTravelBuffCue == null)
		{
			return false;
		}
		if (!SeamlessTravelBuffCue.TryGetValue(buff.Id, out List<List<int>> value) || value.Count == 0)
		{
			return false;
		}
		List<int> list = value[value.Count - 1];
		value.RemoveAt(value.Count - 1);
		Buff2CueMap[buff.Handle] = list;
		CombatLog instance = Singleton<CombatLog>.Instance;
		Entity entity = base.Entity;
		global::<>y__InlineArray3<(string, object)> buffer = default(global::<>y__InlineArray3<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 0) = ("buffId", buff.Id);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 1) = ("buffHandle", buff.Handle);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray3<(string, object)>, (string, object)>(ref buffer, 2) = ("cueHandle", list);
		instance.Info(CombatLog.EDebugModule.Buff, entity, "[无缝加载]buff特效复用", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray3<(string, object)>, (string, object)>(in buffer, 3));
		foreach (int item in list)
		{
			cueComp.ChangeBuffHandle(item, buff);
		}
		return true;
	}

	protected bool SeamlessTravelBuffRetain(IActiveBuff buff)
	{
		return buff.Id < 0;
	}

	public void SeamlessTravelingRefresh()
	{
		Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.Buff, base.Entity, "[无缝加载]buff刷新", default(ReadOnlySpan<(string, object)>));
		HashSet<IActiveBuff> hashSet = new HashSet<IActiveBuff>();
		IActiveBuff[] allBuffs = GetAllBuffs();
		foreach (IActiveBuff activeBuff in allBuffs)
		{
			if (!SeamlessTravelBuffRetain(activeBuff))
			{
				hashSet.Add(activeBuff);
			}
		}
		if (SeamlessTravelBuffCue == null)
		{
			SeamlessTravelBuffCue = new Dictionary<long, List<List<int>>>();
		}
		foreach (IActiveBuff item in hashSet)
		{
			if (Buff2CueMap.TryGetValue(item.Handle, out List<int> value))
			{
				if (!SeamlessTravelBuffCue.TryGetValue(item.Id, out List<List<int>> value2))
				{
					value2 = new List<List<int>>();
					SeamlessTravelBuffCue[item.Id] = value2;
				}
				value2.Add(value);
				Buff2CueMap.Remove(item.Handle);
			}
			RemoveBuffByHandle(item.Handle, -1, "无缝加载移除旧buff");
		}
		InitBornBuff();
		BaseGameplayCueComponent cueComponent = GetCueComponent();
		if (cueComponent != null)
		{
			foreach (KeyValuePair<long, List<List<int>>> item2 in SeamlessTravelBuffCue)
			{
				item2.Deconstruct(out var _, out var value3);
				foreach (List<int> item3 in value3)
				{
					foreach (int item4 in item3)
					{
						cueComponent.RemoveCueByHandle(item4);
					}
				}
			}
		}
		SeamlessTravelBuffCue.Clear();
		OnSeamlessTravelingRefreshEnd();
	}

	protected virtual void OnSeamlessTravelingRefreshEnd()
	{
	}

	public static void CreateStaticDefaultValue()
	{
		_buffStatMap = new Dictionary<long, Stat>();
	}

	public static void ResetStaticDefaultValue()
	{
		_buffStatMap = null;
	}

	protected void SetBuffComponentType(EBuffComponentType value)
	{
		BuffComponentType = value;
	}

	public EBuffComponentType GetBuffComponentType()
	{
		return BuffComponentType;
	}

	public bool IsTeamBuffComponent()
	{
		return BuffComponentType == EBuffComponentType.TeamBuffComponent;
	}

	public bool IsRoleBuffComponent()
	{
		return BuffComponentType == EBuffComponentType.RoleBuffComponent;
	}

	public static List<int> GetCurrentEntityIds()
	{
		return (from handle in ModelBase<SceneTeamModel>.Instance.GetTeamEntities(onlyMyRole: true)
			select handle.Id).ToList();
	}

	public static List<int> GetNewOrRemoveTeamEntityIds(List<int> oldTeamEntityIds, bool newEntity = true)
	{
		List<int> oldTeamEntityIds2 = oldTeamEntityIds;
		List<int> currentEntityIds = GetCurrentEntityIds();
		if (newEntity)
		{
			return currentEntityIds.Where((int id) => !oldTeamEntityIds2.Contains(id)).ToList();
		}
		return oldTeamEntityIds2.Where((int id) => !currentEntityIds.Contains(id)).ToList();
	}

	public List<T> GetTargetComponents<T>(EComponent componentType, long? buffId = null) where T : EntityComponent
	{
		List<T> list = new List<T>();
		if (IsTeamBuffComponent())
		{
			foreach (EntityHandle teamEntity in ModelBase<SceneTeamModel>.Instance.GetTeamEntities(onlyMyRole: true))
			{
				WorldEntity? entity = teamEntity.Entity;
				T val = ((entity != null) ? entity.GetComponent<T>() : null);
				if (val != null)
				{
					list.Add(val);
				}
				else if (buffId.HasValue)
				{
					CombatLog instance = Singleton<CombatLog>.Instance;
					Entity entity2 = base.Entity;
					(string, object) reference = ("buffId", buffId);
					instance.Error(CombatLog.EDebugModule.Buff, entity2, "编队额外效果获取组件失败", new ReadOnlySpan<(string, object)>(ref reference));
				}
			}
		}
		else
		{
			Entity? entity3 = GetEntity();
			T val2 = ((entity3 != null) ? entity3.GetComponent<T>() : null);
			if (val2 != null)
			{
				list.Add(val2);
			}
			else if (buffId.HasValue)
			{
				CombatLog instance2 = Singleton<CombatLog>.Instance;
				Entity entity4 = base.Entity;
				(string, object) reference2 = ("buffId", buffId);
				instance2.Error(CombatLog.EDebugModule.Buff, entity4, "非编队额外效果获取组件失败", new ReadOnlySpan<(string, object)>(ref reference2));
			}
		}
		return list;
	}

	public bool AddReplaceBuff(long originBuffId, BuffReplace replaceBuffInfo)
	{
		if (ReplaceBuffMap.ContainsKey(originBuffId))
		{
			CombatLog instance = Singleton<CombatLog>.Instance;
			Entity entity = base.Entity;
			global::<>y__InlineArray2<(string, object)> buffer = default(global::<>y__InlineArray2<(string, object)>);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 0) = ("originBuffId", originBuffId);
			global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer, 1) = ("replaceBuffInfo", replaceBuffInfo);
			instance.Error(CombatLog.EDebugModule.Buff, entity, "添加重复的buff替换关系", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer, 2));
			return false;
		}
		CombatLog instance2 = Singleton<CombatLog>.Instance;
		Entity entity2 = base.Entity;
		global::<>y__InlineArray2<(string, object)> buffer2 = default(global::<>y__InlineArray2<(string, object)>);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer2, 0) = ("originBuffId", originBuffId);
		global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<(string, object)>, (string, object)>(ref buffer2, 1) = ("replaceBuffInfo", replaceBuffInfo);
		instance2.Info(CombatLog.EDebugModule.Buff, entity2, "添加buff替换", global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<(string, object)>, (string, object)>(in buffer2, 2));
		ReplaceBuffMap[originBuffId] = replaceBuffInfo;
		return true;
	}

	public bool RemoveReplaceBuff(long originBuffId)
	{
		CombatLog instance = Singleton<CombatLog>.Instance;
		Entity entity = base.Entity;
		(string, object) reference = ("originBuffId", originBuffId);
		instance.Info(CombatLog.EDebugModule.Buff, entity, "移除buff替换", new ReadOnlySpan<(string, object)>(ref reference));
		ReplaceBuffMap.Remove(originBuffId);
		return true;
	}

	public long? GetReplaceBuffId(long originBuffId, AddBuffParam buffParams)
	{
		if (!ReplaceBuffMap.ContainsKey(originBuffId))
		{
			return null;
		}
		BuffReplace buffReplace = ReplaceBuffMap[originBuffId];
		buffParams.PreMessageId = buffReplace.PreMessageId;
		return buffReplace.ReplaceBuffId;
	}

	public virtual void AddBuffRefEntityId(long buffId, int entityId)
	{
	}

	public virtual void RemoveBuffRefEntityId(long buffId, int entityId)
	{
	}

	public virtual bool HasBuffRefEntityId(long buffId)
	{
		return false;
	}

	public override bool ClearComponent(EntityComponent componentTemplate)
	{
		if (!base.ClearComponent(componentTemplate))
		{
			return false;
		}
		BaseBuffComponent baseBuffComponent = (BaseBuffComponent)componentTemplate;
		if (CanResetComponentProperty("CreatureDataComponent"))
		{
			if (baseBuffComponent.CreatureDataComponent == null)
			{
				CreatureDataComponent = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(CreatureDataComponent), "CreatureDataComponent"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("DeathComponent"))
		{
			if (baseBuffComponent.DeathComponent == null)
			{
				DeathComponent = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(DeathComponent), "DeathComponent"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("BuffContainer") && baseBuffComponent.BuffContainer != null && !CheckClearObject(EntityComponentSystem.ClearObject(BuffContainer), "BuffContainer"))
		{
			return false;
		}
		if (CanResetComponentProperty("BuffIdToHandleMap") && baseBuffComponent.BuffIdToHandleMap != null && !CheckClearObject(EntityComponentSystem.ClearObject(BuffIdToHandleMap), "BuffIdToHandleMap"))
		{
			return false;
		}
		if (CanResetComponentProperty("BuffGarbageSet") && baseBuffComponent.BuffGarbageSet != null && !CheckClearObject(EntityComponentSystem.ClearObject(BuffGarbageSet), "BuffGarbageSet"))
		{
			return false;
		}
		if (CanResetComponentProperty("WaitRemoveBuffResponse") && baseBuffComponent.WaitRemoveBuffResponse != null && !CheckClearObject(EntityComponentSystem.ClearObject(WaitRemoveBuffResponse), "WaitRemoveBuffResponse"))
		{
			return false;
		}
		if (CanResetComponentProperty("PendingAddBuff") && baseBuffComponent.PendingAddBuff != null && !CheckClearObject(EntityComponentSystem.ClearObject(PendingAddBuff), "PendingAddBuff"))
		{
			return false;
		}
		if (CanResetComponentProperty("<BuffEffectManager>k__BackingField"))
		{
			if (baseBuffComponent.BuffEffectManager == null)
			{
				BuffEffectManager = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(BuffEffectManager), "<BuffEffectManager>k__BackingField"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("TagListenerDict") && baseBuffComponent.TagListenerDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(TagListenerDict), "TagListenerDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("BuffListenerDict") && baseBuffComponent.BuffListenerDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(BuffListenerDict), "BuffListenerDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("TagImmuneListenerDict") && baseBuffComponent.TagImmuneListenerDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(TagImmuneListenerDict), "TagImmuneListenerDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("VictimTagListenerDict") && baseBuffComponent.VictimTagListenerDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(VictimTagListenerDict), "VictimTagListenerDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("ConfigOverrideConditionListener") && baseBuffComponent.ConfigOverrideConditionListener != null && !CheckClearObject(EntityComponentSystem.ClearObject(ConfigOverrideConditionListener), "ConfigOverrideConditionListener"))
		{
			return false;
		}
		if (CanResetComponentProperty("VictimBuffListenerDict") && baseBuffComponent.VictimBuffListenerDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(VictimBuffListenerDict), "VictimBuffListenerDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("InstigatorEventListenDict") && baseBuffComponent.InstigatorEventListenDict != null && !CheckClearObject(EntityComponentSystem.ClearObject(InstigatorEventListenDict), "InstigatorEventListenDict"))
		{
			return false;
		}
		if (CanResetComponentProperty("BuffRoutineExpirationLock") && baseBuffComponent.BuffRoutineExpirationLock != null && !CheckClearObject(EntityComponentSystem.ClearObject(BuffRoutineExpirationLock), "BuffRoutineExpirationLock"))
		{
			return false;
		}
		if (CanResetComponentProperty("EffectTimeoutMap"))
		{
			if (baseBuffComponent.EffectTimeoutMap == null)
			{
				EffectTimeoutMap = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(EffectTimeoutMap), "EffectTimeoutMap"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("EffectTargetTimeoutMap"))
		{
			if (baseBuffComponent.EffectTargetTimeoutMap == null)
			{
				EffectTargetTimeoutMap = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(EffectTargetTimeoutMap), "EffectTargetTimeoutMap"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("TagInChanged") && baseBuffComponent.TagInChanged != null && !CheckClearObject(EntityComponentSystem.ClearObject(TagInChanged), "TagInChanged"))
		{
			return false;
		}
		if (CanResetComponentProperty("BuffLockInternal"))
		{
			BuffLockInternal = baseBuffComponent.BuffLockInternal;
		}
		if (CanResetComponentProperty("TriggerMap") && baseBuffComponent.TriggerMap != null && !CheckClearObject(EntityComponentSystem.ClearObject(TriggerMap), "TriggerMap"))
		{
			return false;
		}
		if (CanResetComponentProperty("OwnerBuffTimeModifiers"))
		{
			if (baseBuffComponent.OwnerBuffTimeModifiers == null)
			{
				OwnerBuffTimeModifiers = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(OwnerBuffTimeModifiers), "OwnerBuffTimeModifiers"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("InstigatorBuffTimeModifiers"))
		{
			if (baseBuffComponent.InstigatorBuffTimeModifiers == null)
			{
				InstigatorBuffTimeModifiers = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(InstigatorBuffTimeModifiers), "InstigatorBuffTimeModifiers"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("BuffStackModifiers"))
		{
			if (baseBuffComponent.BuffStackModifiers == null)
			{
				BuffStackModifiers = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(BuffStackModifiers), "BuffStackModifiers"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("EnableDeferredCueCreation"))
		{
			EnableDeferredCueCreation = baseBuffComponent.EnableDeferredCueCreation;
		}
		if (CanResetComponentProperty("Buff2CueMap") && baseBuffComponent.Buff2CueMap != null && !CheckClearObject(EntityComponentSystem.ClearObject(Buff2CueMap), "Buff2CueMap"))
		{
			return false;
		}
		if (CanResetComponentProperty("PendingCreateCueBuffs") && baseBuffComponent.PendingCreateCueBuffs != null && !CheckClearObject(EntityComponentSystem.ClearObject(PendingCreateCueBuffs), "PendingCreateCueBuffs"))
		{
			return false;
		}
		if (CanResetComponentProperty("PendingInstantCues") && baseBuffComponent.PendingInstantCues != null && !CheckClearObject(EntityComponentSystem.ClearObject(PendingInstantCues), "PendingInstantCues"))
		{
			return false;
		}
		if (CanResetComponentProperty("PendingDestroyCueBuffs") && baseBuffComponent.PendingDestroyCueBuffs != null && !CheckClearObject(EntityComponentSystem.ClearObject(PendingDestroyCueBuffs), "PendingDestroyCueBuffs"))
		{
			return false;
		}
		if (CanResetComponentProperty("SeamlessTravelBuffCue"))
		{
			if (baseBuffComponent.SeamlessTravelBuffCue == null)
			{
				SeamlessTravelBuffCue = null;
			}
			else if (!CheckClearObject(EntityComponentSystem.ClearObject(SeamlessTravelBuffCue), "SeamlessTravelBuffCue"))
			{
				return false;
			}
		}
		if (CanResetComponentProperty("BuffComponentType"))
		{
			BuffComponentType = baseBuffComponent.BuffComponentType;
		}
		if (CanResetComponentProperty("ReplaceBuffMap") && baseBuffComponent.ReplaceBuffMap != null && !CheckClearObject(EntityComponentSystem.ClearObject(ReplaceBuffMap), "ReplaceBuffMap"))
		{
			return false;
		}
		return true;
	}
}
