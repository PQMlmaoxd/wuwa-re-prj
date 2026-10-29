// RECONSTRUCTED_NATIVE
// Identity-only reconstruction. Native bodies are UNRESOLVED_NONPLAIN in the
// at-rest image; body unavailable from at-rest image. No .cpp reconstruction exists.
// Observed layout/semantics only.
// Original Kuro source unavailable.
// Build: BLD-WUWA-0001
// Unit: RCN-C4-NATIVE-BOUNDARY-0001
// Material claims: CLM-C4-0001,CLM-C4-0002,CLM-C4-0003,CLM-C4-0004,CLM-C4-0005,CLM-C4-0006
// Classification: CLS-C4-NATIVE-BOUNDARY-0001
// Review: PENDING

// RECON_META_BEGIN
// CLM-C4-0001 [OBSERVED/OBSERVATION] — non-plain-at-rest finding: .text high-entropy, bodies UNRESOLVED_NONPLAIN at rest
// CLM-C4-0002 [OBSERVED/OBSERVATION] — P1 FEventSystem identity anchors: registration table + entry VAs + validation strings
// CLM-C4-0003 [OBSERVED/OBSERVATION] — P2 budget identity anchors: TickOutside/AfterTickOutside + CSharpInterface surface
// CLM-C4-0004 [OBSERVED/OBSERVATION] — P3 KV typed __Internal surface identity (20 methods + Clear)
// CLM-C4-0005 [OBSERVED/OBSERVATION] — C3 mechanism impact: all C4-fed mechanisms UNRESOLVED, C3 conclusions unchanged
// CLM-C4-0006 [OBSERVED/OBSERVATION] — boundary model completeness: 3 rows, semantic fields remain UNKNOWN
// RECON_META_END

// Identity-only reconstruction of the native cross-language KV surface.
// UKuroVariableFunctionLibrary exposes a TYPED key-value API through UFunction
// reflection binding strings (observed in the plain-data name table of the
// 3.6.0 shipping image, VA region 0x1672a2ef0-0x1672a3dd0):
//   types  : String / Int / Float / Bool / Object
//   ops    : Set / Remove / Has / Get  (20 __-Internal methods)
//   plus   : Clear
// JS-side usage (census-observed): InputCSharpProxyController writes handles
// and flags via Has->Remove->Set replace semantics; UiViewBase reads
// CsRootItem widget handles via GetObject(Name+"CSRootItem"). Managed-side
// reads/writes are INFERRED (CSharpScript.dll unavailable).
// The executable's .text is non-plain at rest; per the C4 authorization hard
// safety boundary, all bodies are UNRESOLVED_NONPLAIN. Storage semantics
// (pointer vs copy vs wrapper), lookup implementation, ownership and
// lifetime remain UNKNOWN. No bodies are reconstructed.

#pragma once

// Typed surface identities (observed; bodies unavailable):
//   __Set{String,Int,Float,Bool,Object}Value_Internal
//   __Remove{String,Int,Float,Bool,Object}Value_Internal
//   __Has{String,Int,Float,Bool,Object}Value_Internal
//   __Get{String,Int,Float,Bool,Object}Value_Internal
//   Clear
//
// Body unavailable from at-rest image.
// Native identity and call relationship observed.

// classification: CLM-C4-0099 + CLS-C4-NATIVE-BOUNDARY-0001 [UNKNOWN/OBSERVATION]
