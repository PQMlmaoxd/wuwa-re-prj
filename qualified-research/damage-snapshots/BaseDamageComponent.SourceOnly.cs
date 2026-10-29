/*
 * RECONSTRUCTED SOURCE_ONLY / NONCOMPILABLE_FRAGMENT. Not original developer source.
 * Five exact method-source spans from an existing deterministic managed capture.
 * No containing type, dependency stubs, invented signatures or behavior are added.
 * Source spans remain SOURCE_ONLY: selected original-CIL call sites were checked,
 * NOT full source/CIL equivalence, compilation, runtime behavior or legal rights.
 * Assembly SHA-256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3 | TypeDef: 0x02002e65
 * Version: UNKNOWN. Exact origins and byte/body digests: evidence.json.
 */

// AllocateSnapshot | MethodDef 0x0601864f | capture lines 506-543
// BEGIN PRESERVED SOURCE SPAN
	private SnapshotPayload AllocateSnapshot(BaseDamageComponent attacker, TDamageParam inputParam)
	{
		AttributeSnapshot attackerSnapshot = (attacker.FollowComponent?.GetAttributeHolder() ?? attacker.Entity).GetComponent<BaseAttributeComponent>().TakeSnapshot();
		AttributeSnapshot targetSnapshot = (FollowComponent?.GetAttributeHolderExceptVisionSummon() ?? base.Entity).GetComponent<BaseAttributeComponent>().TakeSnapshot() ?? AttributeComponent.TakeSnapshot();
		HashSet<Entity> hashSet = DamageTransferRecipients.ApplyEffects(inputParam.DirectTarget);
		List<DamageTransfer> list = new List<DamageTransfer>();
		foreach (Entity item in hashSet)
		{
			if (!item.Valid)
			{
				continue;
			}
			BaseDeathComponent? component = item.GetComponent<BaseDeathComponent>();
			if (component == null || !component.IsDead())
			{
				BaseDamageComponent component2 = item.GetComponent<BaseDamageComponent>();
				BaseAttributeComponent component3 = item.GetComponent<BaseAttributeComponent>();
				if (component2 != null && component3 != null)
				{
					list.Add(new DamageTransfer
					{
						TransferTarget = component2,
						ToughRecoverDelayTime = component3.GetCurrentValue(EAttributeType.ToughRecoverDelayTime),
						WeakTime = component3.GetCurrentValue(EAttributeType.WeakTime)
					});
				}
			}
		}
		return new SnapshotPayload
		{
			Attacker = ReplaceAttacker(attacker),
			AttackerSnapshot = attackerSnapshot,
			Target = this,
			TargetSnapshot = targetSnapshot,
			DamageTransfers = list,
			HasDamageTransfer = (hashSet.Count > 0)
		};
	}
// END PRESERVED SOURCE SPAN

// ReplaceAttacker | MethodDef 0x06018650 | capture lines 545-552
// BEGIN PRESERVED SOURCE SPAN
	private BaseDamageComponent? ReplaceAttacker(BaseDamageComponent attacker)
	{
		if (GameplayAbilityVisionControl.VisionControlHandle != null && attacker.CreatureDataComponent.SummonType == ESummonType.ConcomitantPhantomRole)
		{
			return ModelBase<CreatureModel>.Instance.GetEntity(attacker.CreatureDataComponent.GetSummonerId()).Entity?.GetComponent<BaseDamageComponent>();
		}
		return attacker;
	}
// END PRESERVED SOURCE SPAN

// ProcessWhenCounter | MethodDef 0x06018651 | capture lines 554-561
// BEGIN PRESERVED SOURCE SPAN
	private void ProcessWhenCounter(RequirementPayload partialRequirements, TDamageParam inputParam, SnapshotPayload payload)
	{
		if (inputParam.IsCounterAttack)
		{
			payload.Attacker.BuffComponent.TriggerEvents(EBuffTriggerType.ForWhenCounter, payload.Target.BuffComponent, partialRequirements);
			BuffComponent.TriggerEvents(EBuffTriggerType.ForWhenBeCountered, payload.Attacker.BuffComponent, partialRequirements);
		}
	}
// END PRESERVED SOURCE SPAN

// ExecDamageEffect | MethodDef 0x06018654 | capture lines 677-681
// BEGIN PRESERVED SOURCE SPAN
	private void ExecDamageEffect(DamageResult result, TDamageParam inputParam, RequirementPayload requirements)
	{
		ExecEnergyCharge(inputParam, requirements);
		ExecPostDamageExtraEffect(result, inputParam.Attacker, requirements);
	}
// END PRESERVED SOURCE SPAN

// ProcessDamageOptimize | MethodDef 0x06018662 | capture lines 972-977
// BEGIN PRESERVED SOURCE SPAN
	private void ProcessDamageOptimize(RequirementPayload partialRequirements, TDamageParam inputParam, SnapshotPayload snapshots)
	{
		ProcessWhenCounter(partialRequirements, inputParam, snapshots);
		SnapModifier.PreCriticalModify(partialRequirements, snapshots);
		SnapModifier.PostCriticalModify(partialRequirements, snapshots);
	}
// END PRESERVED SOURCE SPAN

