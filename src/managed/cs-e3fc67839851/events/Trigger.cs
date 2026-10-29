/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002fc7 (global::Trigger)
 * Scope: five selected whole method bodies only.
 * Unknown dependencies remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using CSharpScript.Utils;

public abstract partial class Trigger
{
    // Observed members referenced by the selected methods. Formula, callback,
    // owner, and lifecycle hooks are otherwise outside this source view.
    private readonly Formula Formula;
    private bool IsActive;
    protected EExecuteType ExecuteType;
    private bool LastFormulaResult;
    public int IterationLock = -1;
    public CharacterTriggerComponent? OwnerTriggerComp { get; set; }
    public TTriggerCallback? Callback { get; set; }

    // Token 0x06018ef6; original public, non-virtual.
    public bool CanContinuousTrigger()
    {
        return ExecuteType != EExecuteType.DisableContinuousTrigger;
    }

    // Token 0x06018ef7; original public, non-virtual.
    public void EvaluateAndExecute(Dictionary<string, TFormulaValue>? extraParams = null)
    {
        // The explicit null is loaded at IL_0007; the Formula.Evaluate call is at IL_0008.
        if (Formula.Evaluate(extraParams, null).TryGetBool(out bool formulaResult) && formulaResult)
        {
            if (!CanContinuousTrigger() && LastFormulaResult)
            {
                return;
            }

            // The latch is set before the frame lock and is not rolled back by a
            // denied lock, null callback, or callback exception.
            LastFormulaResult = true;
            try
            {
                if (TryLock())
                {
                    Callback?.Invoke(Formula.Params, extraParams);
                    Unlock();
                    return;
                }

                CombatLog logger = Singleton<CombatLog>.Instance;
                Entity owner = OwnerTriggerComp?.Entity;
                ReadOnlySpan<(string, object)> details =
                [
                    ("触发器", GetDebugName()),
                    ("Formula", Formula.GetTotalFormulaString())
                ];
                logger.Error(CombatLog.EDebugModule.PassiveSkill, owner, "被动技能不能在同一个调用栈中递归触发", details);
                return;
            }
            catch (Exception exception)
            {
                Unlock();
                CombatLog logger = Singleton<CombatLog>.Instance;
                Entity owner = OwnerTriggerComp?.Entity;
                ReadOnlySpan<(string, object)> details =
                [
                    ("触发器", GetDebugName()),
                    ("Formula", Formula.GetTotalFormulaString()),
                    ("错误信息", exception.Message)
                ];
                logger.ErrorWithStack(CombatLog.EDebugModule.PassiveSkill, owner, "触发器回调函数执行错误", exception, details);
                return;
            }
        }

        LastFormulaResult = false;
    }

    // Token 0x06018ef8; original protected, non-virtual.
    protected bool TryLock()
    {
        // Keep both Time.Frame reads: the value is not introduced as a snapshot.
        if (IterationLock == Singleton<Time>.Instance.Frame)
        {
            return false;
        }
        IterationLock = Singleton<Time>.Instance.Frame;
        return true;
    }

    // Token 0x06018ef9; original protected, non-virtual.
    protected void Unlock()
    {
        IterationLock = -1;
    }

    // Token 0x06018efb; original public, non-virtual.
    public void SetActive(bool isActive)
    {
        if (IsActive != isActive)
        {
            IsActive = isActive;
            if (isActive)
            {
                OnActive();
            }
            else
            {
                OnInactive();
            }
        }
    }
}
