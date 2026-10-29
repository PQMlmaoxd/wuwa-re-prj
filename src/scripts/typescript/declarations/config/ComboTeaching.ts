// RECONSTRUCTED_CONFIG schema for ComboTeaching (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/ComboTeaching.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface ComboTeachingRow {
  id: number;  // OBSERVED generated-accessor voffset 4 (i32)
  nextroleguideid: number;  // OBSERVED generated-accessor voffset 6 (i32)
  keyid: string[];  // OBSERVED generated-accessor voffset 8 (vector<string>)
  icontext: string[];  // OBSERVED generated-accessor voffset 10 (vector<string>)
  icontagtext: string[];  // OBSERVED generated-accessor voffset 12 (vector<string>)
  descriptiontitle: string;  // OBSERVED generated-accessor voffset 14 (string)
  descriptioncontent: string;  // OBSERVED generated-accessor voffset 16 (string)
  commandid: number[];  // OBSERVED generated-accessor voffset 18 (vector<i32>)
  guideid: number[];  // OBSERVED generated-accessor voffset 20 (vector<i32>)
  inputenums: string[];  // OBSERVED generated-accessor voffset 22 (vector<string>)
  addbuffid: number[];  // OBSERVED generated-accessor voffset 24 (vector<f64>)
}
