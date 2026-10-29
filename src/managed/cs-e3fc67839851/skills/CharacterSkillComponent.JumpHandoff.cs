/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003123 (global::CharacterSkillComponent; raw type row 38)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/656cbc14e1ab5439580840fbf0dd815dba00ee19b29e003ed9c27ab799d39671/source/_global/CharacterSkillComponent.cs
 * Canonical source bytes/SHA256: 12082 / f3f09122a02dbd54af93911eae96aa30515c841a008af4dd381e2954beb58585
 * Raw methods root SHA256: a0bd0f4426be75bf057de3081236eabcc25b1dabf469eacbeabc072c97f7275a
 * Raw types root SHA256: 98ee16f4a3446746845b2ae9dd40bce3cec7ffb161b176b0edb254888fedaa7b
 * Scope: one selected ordinary whole method body and one observed private
 * static field. Inherited members, omitted fields, helpers, callbacks, and
 * lifecycle behavior remain external. The source view is not a complete class
 * and does not promise compilation or runtime verification.
 *
 * Selected field:
 *   0x0400d0da private System.Int32 _jumpPriority
 *
 * `jump_priority` configuration loading at canonical source lines 58-64 is
 * source context only; its numeric value and lifecycle are UNKNOWN here.
 */

using System;
using Aki.Config;
using AkiClient.Game.Aki.Character.BaseCharacter;
using CSharpScript.Game;
using UnrealEngine;

public partial class CharacterSkillComponent : BaseSkillComponent, IStaticVariableResetter
{
	// Field 0x0400d0da; original private static System.Int32; canonical source line 28; no initializer.
	private static int _jumpPriority;

	// Token 0x0601a0ad; original public instance non-virtual; canonical source lines 337-340; raw methods row 707; RVA 7997055; 16 bytes; EH0.
	public bool CheckJumpCanInterrupt()
	{
		return DoCheckInterrupt(1, _jumpPriority);
	}
}
