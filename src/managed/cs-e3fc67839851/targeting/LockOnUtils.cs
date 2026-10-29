/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public, non-static, and
 * not partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020030be (global::LockOnUtils; raw type row 82)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/a4a34f0354e168579aced8f8c731f50538f14a45fe1320a580d76f67d9fa0b55/source/_global/LockOnUtils.cs
 * Canonical source bytes/SHA256: 5891 / fc0d504a0ca8a0ce4eaee64d131f5571defa28194a6215cca1cd2d16ba80538a
 * Raw methods root SHA256: 81a78ad6e0dbcd454793d701af32cd01313275adc44ce81f526d557fa4835f07
 * Raw types root SHA256: c42278d1be9685c4840c6d89fda7e9edccae3215194eabdda1691a8e1c9a00b1
 * Scope: one selected ordinary whole static method. Static fields, static
 * initialization/reset infrastructure, omitted helpers, and lifecycle behavior
 * remain external. The source view is not a complete class and does not promise
 * compilation or runtime verification.
 */

using System;
using CSharpScript.Game.Utils;
using UnrealEngine;

public partial class LockOnUtils : IStaticVariableResetter
{
	// Token 0x06019b73; original public static; canonical source lines 86-153; raw methods row 638; RVA 7843820; 379 bytes; EH0.
	public static bool IsValidLockOnTarget(EntityHandle? handle, FGameplayTagContainer? lockOnGameplayTags = null, FGameplayTagContainer? ignoreLockOnGameplayTagContainer = null, bool forceCheckTag = false)
	{
		if (handle == null || !handle.Valid || handle == null || !handle.IsInit)
		{
			return false;
		}
		WorldEntity? entity = handle.Entity;
		if (entity == null || !entity.Active)
		{
			return false;
		}
		CreatureDataComponent component = handle.Entity.GetComponent<CreatureDataComponent>();
		if (component != null && component.GetRemoveState())
		{
			return false;
		}
		if (component == null || !component.GetVisible())
		{
			return false;
		}
		BaseTagComponent component2 = handle.Entity.GetComponent<BaseTagComponent>();
		if (component2 == null && forceCheckTag)
		{
			return false;
		}
		if (component2 != null)
		{
			int num = ((ignoreLockOnGameplayTagContainer != null) ? ignoreLockOnGameplayTagContainer.GameplayTags.Num() : 0);
			if (num > 0)
			{
				for (int i = 0; i < num; i++)
				{
					FGameplayTag tag = ignoreLockOnGameplayTagContainer.GameplayTags.Get(i);
					if (component2.HasTag(tag.TagId()))
					{
						return false;
					}
				}
			}
			else
			{
				global::<>y__InlineArray2<int> buffer = default(global::<>y__InlineArray2<int>);
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<int>, int>(ref buffer, 0) = GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.濒死"];
				global::<PrivateImplementationDetails>.InlineArrayElementRef<global::<>y__InlineArray2<int>, int>(ref buffer, 1) = GameplayTagDefine.EGameplayTagId["行为状态.逻辑状态.隐身.不可锁定"];
				ReadOnlySpan<int> tagIds = global::<PrivateImplementationDetails>.InlineArrayAsReadOnlySpan<global::<>y__InlineArray2<int>, int>(in buffer, 2);
				if (component2.HasAnyTag(in tagIds))
				{
					return false;
				}
			}
			int num2 = ((lockOnGameplayTags != null) ? lockOnGameplayTags.GameplayTags.Num() : 0);
			if (num2 > 0)
			{
				bool result = false;
				for (int j = 0; j < num2; j++)
				{
					FGameplayTag tag2 = lockOnGameplayTags.GameplayTags.Get(j);
					if (component2.HasTag(tag2.TagId()))
					{
						result = true;
						break;
					}
				}
				return result;
			}
		}
		return true;
	}
}
