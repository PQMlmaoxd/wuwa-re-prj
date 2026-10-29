/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; the original type was not partial and this
 * file is not a standalone-compiling whole-class recovery.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020017b2 (global::CooperationModel)
 * Canonical source: .../908b9b895dee1e2af290a434aa1b68647ecc897d300ca08308734af8bfde928f/source/_global/CooperationModel.cs
 * Canonical raw methods: .../908b9b895dee1e2af290a434aa1b68647ecc897d300ca08308734af8bfde928f/evidence/raw/methods.jsonl
 * Canonical raw types: .../908b9b895dee1e2af290a434aa1b68647ecc897d300ca08308734af8bfde928f/evidence/raw/types.jsonl
 * Scope: selected whole source-level body for R22 row 196 only.
 * OnLeaveLevel, GetHandlers, Clear callback bodies, and other members remain
 * external; no empty callback implementation is invented.
 */

[Model(0)]
public partial class CooperationModel : ModelBase<CooperationModel>
{
    // Observed field 0x0400516c; private nullable reference, default null.
    private ICooperationHandler[]? Handlers;

    // MethodDef 0x0600ab24; protected virtual override; source lines 6-15; raw row 29.
    protected override bool OnInit()
    {
        Handlers = new ICooperationHandler[3]
        {
            new LinkCooperationHandler(),
            new QteCooperationHandler(),
            new SceneTeamCooperationHandler()
        };
        return true;
    }
}