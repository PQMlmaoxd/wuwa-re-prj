/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This partial source-only view is not original source and is not a
 * standalone-compilation claim. The selected body is retained as a whole
 * source body; members and the Blueprint graph outside this view remain
 * external or UNKNOWN.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02000e9f (global::InputBlueprintFunctionLibrary; raw type row 3)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/d4b5c27510ddb042ec85ae5693e714c0fe441073c2da9f5a32eaccafca7ade8e/source/_global/InputBlueprintFunctionLibrary.cs
 * Raw methods root SHA256: 1dcce9ca88cf812d0d040d184c770c8baee3c0ec90fd0dc9edb1444d6a2580ff
 * Raw types root SHA256: 6fe9e642c1c6ef5ae690b0e3976fade15f150b4497373086663081f5fb224134
 * Scope: one selected R24 whole body, row 231; no helper or native-wrapper
 * body is invented. The original UClass/object-path/base/interface header is
 * preserved; Blueprint graph behavior and native ABI are UNKNOWN.
 */

using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using AkiClient.Game.Aki.Character.Input.Blueprints;
using AkiClient.Game.Aki.Character.Input.Enum;
using AkiClient.Game.Aki.Character.Input.Structures;
using CSharpScript.Core.Common;
using CSharpScript.Game;
using CSharpScript.Game.Input;
using CSharpScript.Game.Module.BattleUi;
using CSharpScript.Game.Ui;
using UnrealEngine;
using UnrealEngine.Interface;
using UnrealEngine.Service;
using UnrealEngine.Service.Flags;
using UnrealEngine.Utils;

[UClass("/Game/Aki/TypeScript/Game/Input/", EClassFlags.CLASS_None)]
[UnrealObjectPath("/Game/Aki/TypeScript/Game/Input/InputBlueprintFunctionLibrary.InputBlueprintFunctionLibrary_C")]
public partial class InputBlueprintFunctionLibrary : UBlueprintFunctionLibrary, IUnrealUObject, IUnrealObject
{
	// Token 0x06005c6f; original public static; canonical source lines 547-556;
	// raw methods row 77; RVA 1522476; 20 bytes; EH0.
	[UFunction(EFunctionFlags.FUNC_None)]
	public static SInputCommand CharacterVisionSkill2OnPress(float time, SInputCommand parentCommand)
	{
		SInputCommand sInputCommand = InputFunctionVisionSkill2.VisionSkill2OnPress(time);
		if (!(sInputCommand != null))
		{
			return parentCommand;
		}
		return sInputCommand;
	}
}
