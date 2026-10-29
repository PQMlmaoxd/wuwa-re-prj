/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. The original type was
 * public `CharacterFightStateComponent : EntityComponent` and was not partial;
 * `partial` here is a source-view convenience only. Standalone compilation is
 * not promised.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x0200303c (global::CharacterFightStateComponent)
 * Canonical source:
 *   src/reconstructed/managed/full/CSharpScript/shards/b0db6280431e5daa24136f1dde3cc88ad04b84c71f9bca45fec0150f8c33c6cf/source/_global/CharacterFightStateComponent.cs
 * Canonical raw methods:
 *   src/reconstructed/managed/full/CSharpScript/shards/b0db6280431e5daa24136f1dde3cc88ad04b84c71f9bca45fec0150f8c33c6cf/evidence/raw/methods.jsonl
 * Canonical raw types:
 *   src/reconstructed/managed/full/CSharpScript/shards/b0db6280431e5daa24136f1dde3cc88ad04b84c71f9bca45fec0150f8c33c6cf/evidence/raw/types.jsonl
 * Scope: five selected whole source-level method bodies only. OnStart,
 * PreSwitchRemoteFightState, TrySwitchHitState, TrySwitchSkillState,
 * TrySwitchState, ConfirmState, ResetState, ExitState, GetFightState,
 * CurrentStateToString, TargetStateToString, FightStateToString, ClearComponent,
 * the constructor, and other lifecycle/callback behavior remain external
 * without stubs or helper declarations.
 * Two explicit unchecked operations preserve the observed CIL arithmetic at
 * the unsigned state comparison and incrementing handle sites; see the R30
 * addendum for the only source-vs-canonical mechanical differences.
 */

using System;
using AkiClient.Game.Aki.Character.BaseCharacter;

public partial class CharacterFightStateComponent : EntityComponent
{
	// Observed fields only; no initialization is supplied by this source view.
	private int IncId;

	private CharacterUnifiedStateComponent? UnifiedComp;

	public ECharacterFightState CurrentState;

	public int SubStatePriority;

	public bool IsLocal;

	public bool WaitConfirm;

	public int CurrentHandle;

	// MethodDef 0x06019423; public instance non-virtual; raw row 798; RVA 7592664; 98 bytes.
	public bool CheckSwitchHitState(EHitAnim hitAnim, bool isLocal = false)
	{
		switch (hitAnim)
		{
		case EHitAnim.被破弱:
			return CheckSwitchState(ECharacterFightState.BeBrokeWeakness, 0, isLocal);
		case EHitAnim.被弹反:
			return CheckSwitchState(ECharacterFightState.CounterattackBeHit, 0, isLocal);
		default:
			if (isLocal && UnifiedComp.PositionState == ECharPositionState.Air)
			{
				return CheckSwitchState(ECharacterFightState.BeHit, 2, isLocal);
			}
			switch (hitAnim)
			{
			case EHitAnim.击飞:
			case EHitAnim.击倒:
				return CheckSwitchState(ECharacterFightState.BeHit, 2, isLocal);
			case EHitAnim.压制:
				return CheckSwitchState(ECharacterFightState.BeHit, 1, isLocal);
			default:
				return CheckSwitchState(ECharacterFightState.BeHit, 0, isLocal);
			}
		}
	}

	// MethodDef 0x06019424; public instance non-virtual; raw row 799; RVA 7592776; 98 bytes.
	public int SwitchHitState(EHitAnim hitAnim, bool isLocal = false)
	{
		switch (hitAnim)
		{
		case EHitAnim.被破弱:
			return SwitchState(ECharacterFightState.BeBrokeWeakness, 0, isLocal);
		case EHitAnim.被弹反:
			return SwitchState(ECharacterFightState.CounterattackBeHit, 0, isLocal);
		default:
			if (isLocal && UnifiedComp.PositionState == ECharPositionState.Air)
			{
				return SwitchState(ECharacterFightState.BeHit, 2, isLocal);
			}
			switch (hitAnim)
			{
			case EHitAnim.击飞:
			case EHitAnim.击倒:
				return SwitchState(ECharacterFightState.BeHit, 2, isLocal);
			case EHitAnim.压制:
				return SwitchState(ECharacterFightState.BeHit, 1, isLocal);
			default:
				return SwitchState(ECharacterFightState.BeHit, 0, isLocal);
			}
		}
	}

	// MethodDef 0x06019425; public instance non-virtual; raw row 800; RVA 7592886; 58 bytes.
	public bool CheckSwitchState(ECharacterFightState state, int subStatePriority, bool isLocal = false)
	{
		if (isLocal)
		{
			return CheckSwitchStateInternal(CurrentState, SubStatePriority, state, subStatePriority);
		}
		if (WaitConfirm)
		{
			return !CheckSwitchStateInternal(state, subStatePriority, CurrentState, SubStatePriority);
		}
		return true;
	}

	// MethodDef 0x06019426; private instance non-virtual; raw row 801; RVA 7592945; 43 bytes.
	private bool CheckSwitchStateInternal(ECharacterFightState curState, int curSubState, ECharacterFightState newState, int newSubState)
	{
		if (newState == curState)
		{
			if (newSubState == curSubState && (unchecked((uint)(newState - 1)) <= 1u || newState == ECharacterFightState.BeBrokeWeakness || newState == ECharacterFightState.SuperSkill))
			{
				return true;
			}
			if (newSubState > curSubState)
			{
				return true;
			}
			return false;
		}
		if (newState > curState)
		{
			return true;
		}
		return false;
	}

	// MethodDef 0x06019428; private instance non-virtual; raw row 803; RVA 7593164; 121 bytes.
	private int SwitchState(ECharacterFightState state, int subStatePriority, bool isLocal = false)
	{
		CurrentState = state;
		SubStatePriority = subStatePriority;
		IsLocal = isLocal;
		WaitConfirm = isLocal;
		CurrentHandle = unchecked(++IncId);
		Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.FightState, base.Entity, "切换" + (isLocal ? "本地" : "远端") + "主状态成功，" + CurrentStateToString(), default(ReadOnlySpan<(string, object)>));
		return CurrentHandle;
	}
}
/*
 * R31 staged append: six ordinary source-level bodies are proposed as rows
 * 253-258 overall. This second partial declaration is a source-view
 * convenience; the original public CharacterFightStateComponent :
 * EntityComponent type was not partial. No field is declared in this append;
 * omitted lifecycle members and external callers remain external. This is
 * STAGED_RECORD_ONLY / PENDING_INDEPENDENT_READONLY_REVIEW.
 */
public partial class CharacterFightStateComponent : EntityComponent
{
	// R31 row 253; canonical whole body FIGHT_SOURCE:198-206; raw methods:804; RVA 7593300; 119 B; EH0.
	public void ConfirmState(int handle)
	{
		if (CurrentHandle == handle)
		{
			WaitConfirm = false;
			return;
		}
		Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.FightState, base.Entity, $"确认状态失败[handle:{handle}]，当前[handle:{CurrentHandle}]", default(ReadOnlySpan<(string, object)>));
	}

	// R31 row 254; canonical whole body FIGHT_SOURCE:208-216; raw methods:805; RVA 7593432; 117 B; EH0.
	public void ResetState()
	{
		Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.FightState, base.Entity, $"重置主状态[handle:{CurrentHandle}]", default(ReadOnlySpan<(string, object)>));
		CurrentState = ECharacterFightState.Default;
		SubStatePriority = 0;
		IsLocal = false;
		WaitConfirm = false;
		CurrentHandle = 0;
	}

	// R31 row 255; canonical whole body FIGHT_SOURCE:218-233; raw methods:806; RVA 7593564; 178 B; EH0.
	public void ExitState(int handle)
	{
		if (CurrentHandle == handle)
		{
			Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.FightState, base.Entity, "退出主状态，" + CurrentStateToString(), default(ReadOnlySpan<(string, object)>));
			CurrentState = ECharacterFightState.Default;
			SubStatePriority = 0;
			IsLocal = false;
			WaitConfirm = false;
			CurrentHandle = 0;
		}
		else
		{
			Singleton<CombatLog>.Instance.Info(CombatLog.EDebugModule.FightState, base.Entity, $"退出主状态失败，[handle:{handle}]，{CurrentStateToString()}", default(ReadOnlySpan<(string, object)>));
		}
	}

	// R31 row 256; canonical whole body FIGHT_SOURCE:235-242; raw methods:807; RVA 7593754; 26 B; EH0.
	public int GetFightState()
	{
		if (CurrentHandle == 0)
		{
			return 0;
		}
		return ((int)CurrentState << 8) | SubStatePriority;
	}

}
