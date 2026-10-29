/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only partial view, not original source or standalone
 * compilation. The original type is a public class, not partial; `partial`
 * is an overlay convenience. Native-backed UFunction dispatch, constructors,
 * static pointers, and external task members remain omitted.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02000ce4 (global::TsTaskUseSkill; raw type row 4)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/1ad1d570cebbd305d6ef29e9ce64d905e6f5c2e68ae5254ffd73664485e29986/source/_global/TsTaskUseSkill.cs
 * Raw methods root SHA256: a4c6c57717efad80bec1d35f3c108b69b887776893aa9b2fca9f8f1032b1a7e5
 * Raw types root SHA256: 9c75f076e5df5014f7500c0f84a5e5480ad875c95bdd103c99874e6434a13464
 * Scope: two selected ordinary whole method bodies and the one observed
 * private field. The inline callback/display-class association is retained
 * inside the selected body and is not declared as a separate source helper.
 */

using System;
using Cysharp.Threading.Tasks;
using UnrealEngine;
using UnrealEngine.Interface;
using UnrealEngine.Service;
using UnrealEngine.Service.Flags;
using UnrealEngine.Utils;

[UClass("/Game/Aki/TypeScript/Game/AI/BehaviorTree/Task/", EClassFlags.CLASS_None)]
[UnrealObjectPath("/Game/Aki/TypeScript/Game/AI/BehaviorTree/Task/TsTaskUseSkill.TsTaskUseSkill_C")]
public partial class TsTaskUseSkill : TsTaskAbortImmediatelyBase, IUnrealUObject, IUnrealObject
{
	// Field 0x04000fc9; original private System.Boolean; no initializer.
	private bool WaitingSkill;

	// Token 0x06004155; original protected virtual newslot; canonical source lines 42-45;
	// raw methods row 42; RVA 439173; 8 bytes; EH0.
	protected virtual void ReceiveExecuteAI_Implementation(AAIController? ownerController, APawn? controlledPawn)
	{
		WaitingSkill = false;
	}

	// Token 0x06004157; original protected virtual newslot; canonical source lines 69-113;
	// raw methods row 44; RVA 439344; 303 bytes; EH0.
	protected virtual void ReceiveTickAI_Implementation(AAIController? ownerController, APawn? controlledPawn, float deltaSeconds)
	{
		//IL_0117: Unknown result type (might be due to invalid IL or missing references)
		//IL_0128: Unknown result type (might be due to invalid IL or missing references)
		AiController aiController = (ownerController as TsAiController)?.AiController;
		if (aiController == null)
		{
			Log instance = Singleton<Log>.Instance;
			(string, object) reference = ("Type", ownerController.GetClass().GetName());
			instance.Error((ELogModule)27, (ELogAuthor)6, "错误的Controller类型", new ReadOnlySpan<(string, object)>(ref reference));
			FinishExecute(bSuccess: false);
			return;
		}
		int entityId = aiController.CharAiDesignComp.Entity.Id;
		CharacterSkillComponent component = aiController.CharAiDesignComp.Entity.GetComponent<CharacterSkillComponent>();
		if (!component.Valid)
		{
			FinishExecute(bSuccess: false);
			return;
		}
		string text = ControllerBase<BlackboardController>.Instance.GetStringValueByEntity(entityId, "SkillId");
		if (string.IsNullOrEmpty(text))
		{
			text = "0";
		}
		if (WaitingSkill)
		{
			return;
		}
		WaitingSkill = true;
		UniTaskExtensions.ContinueWith<bool>(component.BeginSkillAsync(int.Parse(text), new SkillParam
		{
			Target = aiController.AiHateList.GetCurrentTarget()?.Entity,
			Reason = "TsTaskUseSkill.ReceiveTickAI"
		}), (Action<bool>)delegate(bool result)
		{
			FinishExecute(result);
			if (result && aiController.AiSkill != null)
			{
				aiController.AiSkill.SetSkillCdFromNow(ControllerBase<BlackboardController>.Instance.GetIntValueByEntity(entityId, "SkillInfoId"));
			}
			ControllerBase<BlackboardController>.Instance.RemoveValueByEntity(entityId, "SkillId");
			ControllerBase<BlackboardController>.Instance.RemoveValueByEntity(entityId, "SkillInfoId");
		});
	}
}
