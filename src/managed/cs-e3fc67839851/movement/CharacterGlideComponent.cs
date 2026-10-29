/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003042 (global::CharacterGlideComponent; raw type row 28 located by type_token)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/3a2daa57d1d1534ef6fe5101721a681b8ef054d320cc0c3bed7427401a5a16b0/source/_global/CharacterGlideComponent.cs
 * Raw methods root SHA256: bacb7360a908c17ff3a0221a8b6e480d991eee36273f4dca488040f5bc520fee
 * Raw types root SHA256: e4116181e48370c54da78560c17891329f7583ac1ea38ef4e38644d2276f251e
 * Scope: one selected ordinary whole method body and one observed private
 * field. Other fields, lifecycle/event registration, callbacks, helpers, and
 * movement behavior remain external. The source view is not a complete class
 * and does not promise compilation or runtime verification.
 *
 * Selected field:
 *   0x0400c769 CharacterActorComponent? ActorComp
 */

using System;
using Aki.Config;
using Aki.TDConfigMgr.Action;
using AkiClient.Game.Aki.Character.Input.Blueprints;
using CSharpScript.Core.Common;
using CSharpScript.Game.Common.Event;
using CSharpScript.Game.Module.Gamepad;
using CSharpScript.Game.Module.SeamlessTravel;
using CSharpScript.Game.Module.Teleport;
using CSharpScript.Typing;
using UnrealEngine;

public partial class CharacterGlideComponent : EntityComponent
{
	// Field 0x0400c769; original private CharacterActorComponent?; no initializer.
	private CharacterActorComponent? ActorComp;

	// Token 0x06019485; original public instance non-virtual; canonical source lines 524-532; raw methods row 695; RVA 7611643; 43 bytes; EH0; optional context = "CharacterGlideComponent".
	public void EnterGlideState(string context = "CharacterGlideComponent")
	{
		ActorComp.Actor.KuroSetMovementMode(new SetMovementModeInfo
		{
			Mode = EMovementMode.MOVE_Custom,
			CustomMode = 2,
			Context = context
		});
	}
}
