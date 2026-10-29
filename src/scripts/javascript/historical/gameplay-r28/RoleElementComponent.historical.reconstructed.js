// R28: NONCANONICAL / RECONSTRUCTED historical JavaScript source view.
// Retained snapshot input (Windows): <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_02>/extracted\pakchunk3-WindowsNoEditor.pak\JavaScript\Game\NewWorld\Character\Role\Component\RoleElementComponent.js
// Original logical module path: JavaScript/Game/NewWorld/Character/Role/Component/RoleElementComponent.js
// Retained source: 6123 UTF-8 bytes; SHA-256 e1bd124e7c771e75ff8a2fa17f3aa4ec56756c08111a4c0ef8d90bf329f22e0d
// Manifest pin: protocol/R5A1_JS_CORPUS_SHA256SUMS.txt:15062 (exact path row).
// The retained input has no BOM, LF offsets 0 and 6074, three physical lines,
// code on physical line 2, and an unavailable source-map comment on line 3.
// Archive association: INFERRED. Game version, build compatibility, and
// migration status are UNKNOWN / NOT_VERIFIED. Runtime is PARKED.
//
// Exactly five complete retained function bodies are selected below, in the
// original method order: set kin, get kin, Fin, TriggerEvents, and
// ClearElementEnergy. Minified identifiers, parameters, numeric selectors,
// and external members are retained; no semantic renaming is asserted.
// The observed class opening and closing shell are retained without a
// constructor, imports, exports, registration, fields, helpers, or execution.

let RoleElementComponent=class RoleElementComponent extends EntityComponent_1.EntityComponent{
    // OBSERVED complete body: set kin(t), [4164,4572), 408 UTF-8 bytes,
    // SHA-256 37f76883b963a9db892e0d9859b94847b3b4005f8ce4551d7bd91e5f7c85d265.
    set kin(t) {
        this.kin !== t && this.n$t?.IsAutonomousProxy && (
            t
                ? (
                    t = (0, FormationDataController_1.isBattleMulti)(),
                    this.jAl = t ? CharacterBuffIds_1.buffId.ActivateMultiQte : CharacterBuffIds_1.buffId.ActivateQte,
                    this.m1t.AddBuff(this.jAl, {
                        InstigatorId: this.m1t.CreatureDataId,
                        Reason: "RoleElementComponent获取激活QTE的Tag"
                    }),
                    t && this.JPa()
                )
                : this.m1t.RemoveBuff(this.jAl, -1, "RoleElementComponent移除激活QTE的Tag")
        )
    }

    // OBSERVED complete body: get kin(), [4572,4621), 49 UTF-8 bytes,
    // SHA-256 5c28ee893c2a1cbdaf9fa9742fa879625e8b926ccdd4954f201a0b068a8f4c81.
    get kin() {
        return this.Xte.HasExactTag(166024319)
    }

    // OBSERVED complete body: Fin(t), [4901,5224), 323 UTF-8 bytes,
    // SHA-256 874e3a7ed82fd9d610d304c0dbcefdab983e56d0f57886399949a555b447e18b.
    Fin(t) {
        t >= this.TriggerEnergy - Number.EPSILON
            ? (
                t = ModelManager_1.ModelManager.SceneTeamModel.GetTeamItem(this.Entity.Id, { ParamType: 1 })?.IsControl(),
                !this.kin && t && FormationDataController_1.FormationDataController.GlobalIsInFight && (
                    this.kin = !0,
                    this.m1t.TriggerEvents(9, this.m1t, { ElementType: this.RoleElementType })
                )
            )
            : this.kin = !1
    }

    // OBSERVED complete body: TriggerEvents(t), [5224,5389), 165 UTF-8 bytes,
    // SHA-256 8d378cdb5b2052a2e7dec77fa3d015f664a4e8523ac370168ddc8bfd45deb6d5.
    TriggerEvents(t) {
        var t = t.GetComponent(102),
            e = { ElementType: this.RoleElementType, ElementType2: t };
        this.m1t.TriggerEvents(10, t.m1t, e),
        t.m1t.TriggerEvents(13, this.m1t, e)
    }

    // OBSERVED complete body: ClearElementEnergy(t,e=CharacterBuffIds_1.buffId.ElementClean),
    // [5389,5571), 182 UTF-8 bytes, SHA-256
    // fb09f6e8ce3fb39e93a16ac861d8309f78ddbde9d61faeb46d430c7b7989a385.
    ClearElementEnergy(t, e = CharacterBuffIds_1.buffId.ElementClean) {
        this.m1t.AddBuff(e, {
            InstigatorId: t.GetComponent(0).GetCreatureDataId(),
            Reason: "ClearElementEnergy消耗元素能量"
        })
    }
};
