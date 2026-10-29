// RECONSTRUCTED_NATIVE
// Observed layout/semantics only.
// Original Kuro source unavailable.
// Build: BLD-WUWA-0001
// Unit: RCN-C4-NATIVE-BOUNDARY-0001
// Material claims: CLM-C4-0001,CLM-C4-0002,CLM-C4-0003,CLM-C4-0004,CLM-C4-0005,CLM-C4-0006
// Classification: CLS-C4-NATIVE-BOUNDARY-0001
// Review: PENDING

// Identity-only reconstruction. The registered native entry points of the
// RECON_META_BEGIN
// CLM-C4-0001 [OBSERVED/OBSERVATION] — non-plain-at-rest finding: .text high-entropy, bodies UNRESOLVED_NONPLAIN at rest
// CLM-C4-0002 [OBSERVED/OBSERVATION] — P1 FEventSystem identity anchors: registration table + entry VAs + validation strings
// CLM-C4-0003 [OBSERVED/OBSERVATION] — P2 budget identity anchors: TickOutside/AfterTickOutside + CSharpInterface surface
// CLM-C4-0004 [OBSERVED/OBSERVATION] — P3 KV typed __Internal surface identity (20 methods + Clear)
// CLM-C4-0005 [OBSERVED/OBSERVATION] — C3 mechanism impact: all C4-fed mechanisms UNRESOLVED, C3 conclusions unchanged
// CLM-C4-0006 [OBSERVED/OBSERVATION] — boundary model completeness: 3 rows, semantic fields remain UNKNOWN
// RECON_META_END
// Puerts cpp-module class FEventSystem were located in the 3.6.0 shipping
// image (sha256 a6cdb83f6f2909b5a13ca19b1d5ff4f0455f743e8fabf40bb07546500968cdfa)
// via the module registration table at VA 0x177877f30-0x177877f70.
// The executable's .text section is non-plain at rest (entropy
// 7.99+/8.0); per the C4 authorization hard safety boundary (static/offline
// only, no runtime unpacking), every body below is UNRESOLVED_NONPLAIN.
// No function bodies are reconstructed. Per-dispatch copy-vs-forward
// semantics remain UNKNOWN (C3 P1 question unresolved at the native side).

#pragma once

// Registered cpp-module entry points (identity observed, body unavailable):
//   FEventSystem::EmitFromJavaScript -> 0x16146e7f0
//   FEventSystem::Init                -> 0x16146e810
//   FEventSystem::Clear               -> 0x16146e780
// JS-side call shape (census-observed, EventCSharpBridge.js):
//   cpp.FEventSystem.EmitFromJavaScript(name, 0, a1..a5)
//   cpp.FEventSystem.Init(y0m)   // native->JS callback registration
//   cpp.FEventSystem.Clear()
// Identity-level boundary facts (plain-data strings):
//   '[FEventSystem::EmitFromJavaScript] Invalid argument!' @ VA 0x167156538
//   '[FEventSystem::Init] Invalid argument!'                @ VA 0x167156600
//   '[FEventSystem::Clear] Invalid argument!'                @ VA 0x167156628
//   param-type validation strings 'string'/'boolean'/'void'/'ArrayBuffer'
//   clustered @ VA ~0x1671565f0 (typed argument validation at the boundary,
//   identity level only).
//
// Body unavailable from at-rest image.
// Native identity and call relationship observed.

// classification: CLM-C4-0099 + CLS-C4-NATIVE-BOUNDARY-0001 [UNKNOWN/OBSERVATION]
