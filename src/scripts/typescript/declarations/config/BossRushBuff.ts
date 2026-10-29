// RECONSTRUCTED_CONFIG schema for BossRushBuff (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/BossRushBuff.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface BossRushBuffRow {
  id: number;  // OBSERVED generated-accessor voffset 4 (i32)
  buffid: number[];  // OBSERVED generated-accessor voffset 6 (vector<i64>)
  passiveskill: number[];  // OBSERVED generated-accessor voffset 8 (vector<i64>)
  texture: string;  // OBSERVED generated-accessor voffset 10 (string)
  name: string;  // OBSERVED generated-accessor voffset 12 (string)
  popdescidlist: number[];  // OBSERVED generated-accessor voffset 14 (vector<i32>)
  bufftitle: string;  // OBSERVED generated-accessor voffset 16 (string)
  buffdesc: string;  // OBSERVED generated-accessor voffset 18 (string)
  buffdescparam: string[];  // OBSERVED generated-accessor voffset 20 (vector<string>)
  description: string;  // OBSERVED generated-accessor voffset 22 (string)
  descriptionparam: string[];  // OBSERVED generated-accessor voffset 24 (vector<string>)
}
