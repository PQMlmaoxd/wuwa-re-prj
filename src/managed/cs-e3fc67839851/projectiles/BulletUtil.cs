/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002de1 (global::BulletUtil; raw type row 23)
 * Scope: one selected whole source-level method body only.
 * The original type header is `public class BulletUtil`; the selected method
 * is static. Other helpers remain external; see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using System.Linq;
using Aki.Protocol;
using Aki.Protocol.Summon;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Core.Fight;
using CSharpScript.Game;
using CSharpScript.Game.Effect;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.NewWorld.Character.Common.Component.Abilities.Follow;
using CSharpScript.Game.NewWorld.Common.Component;
using CSharpScript.Game.NewWorld.Pawn.Component;
using Google.Protobuf;
using UnrealEngine;

public partial class BulletUtil
{
    // Token 0x06017ab7; original public static, non-virtual; canonical source lines 129-141.
    public static bool DoesEntityContainsTag(Entity? entity, int gameplayTagId)
    {
        if (entity == null)
        {
            return false;
        }
        LevelTagComponent? component = entity.GetComponent<LevelTagComponent>();
        if (component != null && component.HasTag(gameplayTagId))
        {
            return true;
        }
        return entity.GetComponent<BaseTagComponent>()?.HasTag(gameplayTagId) ?? false;
    }
}
