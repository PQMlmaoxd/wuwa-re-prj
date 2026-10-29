/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only partial view, not original source or standalone
 * compilation. The original type is a public class, not partial; `partial`
 * is an overlay convenience. Native-backed properties, generated UFunction
 * dispatch, constructors, static pointers, and FinishExecute remain external.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02000cd1 (global::TsTaskSelectSkill; raw type row 17)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/6c0721a72118f3efdfc9498c30aa37b6c3ccf1954cac2209fffbd8fa18caf78e/source/_global/TsTaskSelectSkill.cs
 * Raw methods root SHA256: 512548803c7d08690422bcb68464e4069c2b4234a96b445d6aa9b1241f52a3b8
 * Raw types root SHA256: 0b490f3a5decc0a1df62f2da748d8f3247f0cd33ddc61352ce29ba44eb9ef107
 * Scope: two selected ordinary whole method bodies and the three observed
 * private cached fields used by them. The original UClass/object-path/base/
 * interface header is retained exactly; omitted ABI/native members remain
 * external.
 */

using System;
using UnrealEngine;
using UnrealEngine.Interface;
using UnrealEngine.Service;
using UnrealEngine.Service.Flags;
using UnrealEngine.Utils;

[UClass("/Game/Aki/TypeScript/Game/AI/BehaviorTree/Task/", EClassFlags.CLASS_None)]
[UnrealObjectPath("/Game/Aki/TypeScript/Game/AI/BehaviorTree/Task/TsTaskSelectSkill.TsTaskSelectSkill_C")]
public partial class TsTaskSelectSkill : TsTaskAbortImmediatelyBase, IUnrealUObject, IUnrealObject
{
	// Field 0x04000eb4; original private System.Boolean; no initializer.
	private bool IsInitTsVariables;

	// Field 0x04000eb5; original private System.Int32; no initializer.
	private int TsSkillType;

	// Field 0x04000eb6; original private System.Boolean; no initializer.
	private bool TsDebugLog;

	// Token 0x06004023; original private non-virtual; canonical source lines 54-62;
	// raw methods row 197; RVA 413339; 47 bytes; EH0.
	private void InitTsVariables()
	{
		if (!IsInitTsVariables || GlobalData.IsPlayInEditor)
		{
			IsInitTsVariables = true;
			TsSkillType = SkillType;
			TsDebugLog = DebugLog;
		}
	}

	// Token 0x06004025; original protected virtual newslot; canonical source lines 86-132;
	// raw methods row 199; RVA 413548; 301 bytes; EH0.
	protected virtual void ReceiveTickAI_Implementation(AAIController? ownerController, APawn? controlledPawn, float deltaSeconds)
	{
		AiController aiController = (ownerController as TsAiController)?.AiController;
		if (aiController == null)
		{
			Log instance = Singleton<Log>.Instance;
			(string, object) reference = ("Type", ownerController.GetClass().GetName());
			instance.Error((ELogModule)27, (ELogAuthor)6, "错误的Controller类型", new ReadOnlySpan<(string, object)>(ref reference));
			FinishExecute(bSuccess: false);
			return;
		}
		InitTsVariables();
		if (aiController.AiSkill == null)
		{
			Log instance2 = Singleton<Log>.Instance;
			(string, object) reference2 = ("AiBaseId", aiController.AiBase.Value.Id);
			instance2.Error((ELogModule)27, (ELogAuthor)6, "没有配置技能", new ReadOnlySpan<(string, object)>(ref reference2));
			FinishExecute(bSuccess: false);
			return;
		}
		CharacterSkillComponent component = aiController.CharAiDesignComp.Entity.GetComponent<CharacterSkillComponent>();
		if (!component.Valid)
		{
			FinishExecute(bSuccess: false);
			return;
		}
		EntityHandle currentTarget = aiController.AiHateList.GetCurrentTarget();
		if (currentTarget != null && currentTarget.Valid)
		{
			if (AiLibrary.SelectSkillWithTarget(aiController, component, currentTarget.Entity.GetComponent<CharacterActorComponent>(), TsSkillType, TsDebugLog))
			{
				FinishExecute(bSuccess: true);
			}
			else
			{
				FinishExecute(bSuccess: false);
			}
		}
		else if (AiLibrary.SelectSkillWithoutTarget(aiController, component, TsSkillType))
		{
			FinishExecute(bSuccess: true);
		}
		else
		{
			FinishExecute(bSuccess: false);
		}
	}
}
