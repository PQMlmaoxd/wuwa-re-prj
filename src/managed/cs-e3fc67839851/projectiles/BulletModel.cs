/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02002e0a (global::BulletModel; raw type row 169)
 * Scope: one selected whole source-level method body only.
 * Other model members and the network delivery boundary remain external;
 * see METHOD_EVIDENCE_MAP.md.
 */

using System;
using System.Collections.Generic;
using Aki.Protocol;
using AkiClient.Game.Aki.Character.BaseCharacter;
using AkiClient.Game.Aki.Core.Fight;
using AkiClient.Game.Aki.Core.Fight.Bullet;
using AkiClient.Game.Aki.Data.Fight;
using CSharpScript.Core.Common;
using CSharpScript.Game;
using CSharpScript.Game.Camera;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Effect;
using CSharpScript.Game.Module.CombatMessage;
using CSharpScript.Game.Module.Plot;
using CSharpScript.Game.NewWorld.Pawn.Component;
using CSharpScript.Typing;
using Google.Protobuf;
using UnrealEngine;

[Model(0)]
public partial class BulletModel : ModelBase<BulletModel>
{
    // Token 0x06017cf4; original public, non-virtual; canonical source lines 646-655.
    public void SendDodgeInfoPush(Entity dodgeEntity, long bulletOwnerCreatureDataId, long bulletId, long? preMessageId = null)
    {
        if (dodgeEntity.Valid && bulletOwnerCreatureDataId > 0)
        {
            DodgeInfoPush dodgeInfoPush = DodgeInfoPush.Create();
            dodgeInfoPush.BulletOwnerId = Singleton<MathUtils>.Instance.NumberToLong(bulletOwnerCreatureDataId);
            dodgeInfoPush.BulletId = bulletId;
            Singleton<CombatNet>.Instance.Send(EPushMessageId.DodgeInfoPush, dodgeEntity, (IMessage)(object)dodgeInfoPush, preMessageId);
        }
    }
}
