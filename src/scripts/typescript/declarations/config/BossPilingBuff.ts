// RECONSTRUCTED_CONFIG schema for BossPilingBuff (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/BossPilingBuff.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface BossPilingBuffRow {
  id: number;  // OBSERVED generated-accessor voffset 4 (i32)
  buffid: number;  // OBSERVED generated-accessor voffset 6 (i64)
  name: string;  // OBSERVED generated-accessor voffset 8 (string)
  icon: string;  // OBSERVED generated-accessor voffset 10 (string)
  quality: number;  // OBSERVED generated-accessor voffset 12 (i32)
  desc: string;  // OBSERVED generated-accessor voffset 14 (string)
  descargs: string[];  // OBSERVED generated-accessor voffset 16 (vector<string>)
}
