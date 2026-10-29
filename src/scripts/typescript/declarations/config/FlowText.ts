// RECONSTRUCTED_CONFIG schema for FlowText (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/FlowText.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface FlowTextRow {
  key: number;  // OBSERVED generated-accessor voffset 4 (i64)
  flowlistid: string;  // OBSERVED generated-accessor voffset 6 (string)
  plotlineid: string;  // OBSERVED generated-accessor voffset 8 (string)
  id: number;  // OBSERVED generated-accessor voffset 10 (i32)
  text: number;  // OBSERVED generated-accessor voffset 12 (i32)
  sound: string;  // OBSERVED generated-accessor voffset 14 (string)
  eskey: string;  // OBSERVED generated-accessor voffset 16 (string)
}
