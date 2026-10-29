/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002da3 (global::BulletDataLogic; raw type row 74)
 * Scope: one selected property getter body only.
 * The getter is one selected MethodDef; no constructor or other accessor is
 * supplied. Missing members remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using AkiClient.Game.Aki.Core.Fight;
using CSharpScript.Core.Common;
using UnrealEngine;

public partial class BulletDataLogic
{
    // Field 0x0400b504; canonical source line 9.
    public readonly BulletLogicType_C? Data;

    // Field 0x0400b515; canonical source line 43.
    private bool? CanDodgeInternal;

    // Token 0x06017947; original public property getter; canonical source lines 225-235.
    // Getter identity count: one selected MethodDef.
    public bool CanDodge
    {
        get
        {
            if (!CanDodgeInternal.HasValue)
            {
                CanDodgeInternal = Data.是否可以触发极限闪避;
            }
            return CanDodgeInternal.Value;
        }
    }

    // R20 extension: three observed cache fields and their selected getters
    // are appended; all other members remain external to this source view.

    // Field 0x0400b510; canonical source line 33.
    private bool? CanCounterAttackInternal;

    // Field 0x0400b511; canonical source line 35.
    private bool? CounterAttackIgnoreAngleInternal;

    // Field 0x0400b512; canonical source line 37.
    private bool? CounterAttackIgnoreDistanceInternal;

    // Token 0x06017942; original public property getter; canonical source lines 165-175.
    public bool CanCounterAttack
    {
        get
        {
            if (!CanCounterAttackInternal.HasValue)
            {
                CanCounterAttackInternal = Data.是否可以触发拼刀;
            }
            return CanCounterAttackInternal.Value;
        }
    }

    // Token 0x06017943; original public property getter; canonical source lines 177-187.
    public bool CounterAttackIgnoreAngle
    {
        get
        {
            if (!CounterAttackIgnoreAngleInternal.HasValue)
            {
                CounterAttackIgnoreAngleInternal = Data.拼刀忽略角度;
            }
            return CounterAttackIgnoreAngleInternal.Value;
        }
    }

    // Token 0x06017944; original public property getter; canonical source lines 189-199.
    public bool CounterAttackIgnoreDist
    {
        get
        {
            if (!CounterAttackIgnoreDistanceInternal.HasValue)
            {
                CounterAttackIgnoreDistanceInternal = Data.拼刀忽略距离;
            }
            return CounterAttackIgnoreDistanceInternal.Value;
        }
    }
}
