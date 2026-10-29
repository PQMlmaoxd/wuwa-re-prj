// RECONSTRUCTED_CONFIG schema for SkillTree (R2-A).
// Status: SCHEMA_RECOVERABLE_PENDING_IMPLEMENTATION.
// Evidence: OBSERVED from the generated JS config accessor
//   Core/Define/Config/SkillTree.js (pakchunk3 JS corpus, sha-pinned in C0 census lineage);
//   voffsets map 1:1 to FlatBuffers vtable slots; cross-checked by deterministic FB
//   structural decode of the local ConfigDB BinData (R2AE rows).
// Element classes resolved by generated code where named (DicStringFloat, ConfigPropValue,
// DicIntInt); unresolved element classes remain `unknown` — NO semantic guessing.
// Build: BLD-WUWA-0001 | Unit: RCN-R2A-CONFIG-SPECIAL-0001
// Material claims: CLM-R2A-0001,CLM-R2A-0002,CLM-R2A-0003,CLM-R2A-0004,CLM-R2A-0005,CLM-R2A-0006
// Classification: CLS-R2A-CONFIG-SPECIAL-0001 | Review: PENDING

export interface SkillTreeRow {
  id: number;  // OBSERVED generated-accessor voffset 4 (i32)
  nodeindex: number;  // OBSERVED generated-accessor voffset 6 (i32)
  nodegroup: number;  // OBSERVED generated-accessor voffset 8 (i32)
  parentnodes: number[];  // OBSERVED generated-accessor voffset 10 (vector<i32>)
  nodetype: number;  // OBSERVED generated-accessor voffset 12 (i32)
  coordinate: number;  // OBSERVED generated-accessor voffset 14 (i32)
  condition: number[];  // OBSERVED generated-accessor voffset 16 (vector<i32>)
  skillid: number;  // OBSERVED generated-accessor voffset 18 (i32)
  skillbranchids: number[];  // OBSERVED generated-accessor voffset 20 (vector<i32>)
  skillbranchdesclist: string[];  // OBSERVED generated-accessor voffset 22 (vector<string>)
  propertynodetitle: string;  // OBSERVED generated-accessor voffset 24 (string)
  propertynodedescribe: string;  // OBSERVED generated-accessor voffset 26 (string)
  propertynodeparam: string[];  // OBSERVED generated-accessor voffset 28 (vector<string>)
  propertynodeicon: string;  // OBSERVED generated-accessor voffset 30 (string)
  property: ConfigPropValue>[];  // OBSERVED generated-accessor voffset 32 (vector<table:ConfigPropValue>)
  consume: DicIntInt>[];  // OBSERVED generated-accessor voffset 34 (vector<table:DicIntInt>)
  unlockcondition: number;  // OBSERVED generated-accessor voffset 36 (i32)
}
