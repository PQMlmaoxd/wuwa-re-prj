// RECONSTRUCTED_CONFIG schema for InteractData (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/InteractData.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface InteractDataRow {
  guid: string;  // OBSERVED generated-accessor voffset 4 (string)
  type: string;  // OBSERVED generated-accessor voffset 6 (string)
  icon: string;  // OBSERVED generated-accessor voffset 8 (string)
  tidcontent: string;  // OBSERVED generated-accessor voffset 10 (string)
  condition: string;  // OBSERVED generated-accessor voffset 12 (string)
  uniquenesstest: string;  // OBSERVED generated-accessor voffset 14 (string)
  dointacttype: string;  // OBSERVED generated-accessor voffset 16 (string)
  range: number;  // OBSERVED generated-accessor voffset 18 (i32)
  duration: string;  // OBSERVED generated-accessor voffset 20 (string)
  optionlocktypelist: string;  // OBSERVED generated-accessor voffset 22 (string)
}
