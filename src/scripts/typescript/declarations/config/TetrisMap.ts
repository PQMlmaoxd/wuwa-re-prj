// RECONSTRUCTED_CONFIG schema for TetrisMap (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/TetrisMap.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt, TetrisBoardItemConfig); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface TetrisMapRow {
  id: number;  // OBSERVED generated-accessor voffset 4 (i32)
  mapid: number;  // OBSERVED generated-accessor voffset 6 (i32)
  column1: TetrisBoardItemConfig;  // OBSERVED generated-accessor voffset 8 (table:TetrisBoardItemConfig)
  column2: unknown;  // OBSERVED generated-accessor voffset 10 (table:?)
  column3: unknown;  // OBSERVED generated-accessor voffset 12 (table:?)
  column4: unknown;  // OBSERVED generated-accessor voffset 14 (table:?)
  column5: unknown;  // OBSERVED generated-accessor voffset 16 (table:?)
  column6: unknown;  // OBSERVED generated-accessor voffset 18 (table:?)
  column7: unknown;  // OBSERVED generated-accessor voffset 20 (table:?)
  column8: unknown;  // OBSERVED generated-accessor voffset 22 (table:?)
}
