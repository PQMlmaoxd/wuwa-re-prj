// R27: NONCANONICAL / RECONSTRUCTED JavaScript source view.
// Retained snapshot input (Windows): <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_02>/extracted\pakchunk3-WindowsNoEditor.pak\JavaScript\Game\NewWorld\Character\Common\Component\CharacterHitComponent.js
// Original logical module path: JavaScript/Game/NewWorld/Character/Common/Component/CharacterHitComponent.js
// Retained snapshot SHA-256: 03ab7a42cf41c2357fef862ae64ca25915ba843d1fcabb3ec07212db2905adb2
// Manifest pin: protocol/R5A1_JS_CORPUS_SHA256SUMS.txt:14792 (exact path row).
// Physical source: 46337 UTF-8 bytes, no BOM, LF offsets 0 and 46287; code is
// physical source line 2. The observed line-3 sourceMappingURL comment is
// omitted because CharacterHitComponent.js.map is unavailable; no map is made.
// Archive association: INFERRED. Game version, build compatibility, and
// migration status are UNKNOWN / NOT_VERIFIED. Runtime is PARKED.
//
// Exactly five complete retained function bodies are selected below, in the
// original method order g6r, f6r, p6r, v6r, a6r. Minified identifiers and
// parameters are retained; no semantic renaming is asserted. The class shell
// is the observed assignment form. Constructor, fields, helpers, static
// probes, root variables, export, and registration remain EXTERNAL.

let CharacterHitComponent = CharacterHitComponent_1 = class CharacterHitComponent extends BaseHitComponent_1.BaseHitComponent {
    // OBSERVED source slice [23944,24169), SHA-256 31b989dc801b362f686da55098ebc9216244724da83b944cd93bd6bfdb3c7f37.
    g6r(t, i) {
        let e = !1;
        for (const r of t)
            e ||= r.IsTransferDamage;
        for (const o of t) {
            var s = !(o.SeparateDamage && e),
                h = this.v6r(i, !1, s, o.Index);
            s || (this.ToughDecreaseValue = h)
        }
        e && (
            t = this.v6r(i, !1, !e, t[0].Index),
            this.ToughDecreaseValue = t
        )
    }

    // OBSERVED source slice [24169,24249), SHA-256 cd492ff92725cb25043b0d9ee3c6151b8c7e5332b50c44d779041ff0dbaddd86.
    f6r(t, i) {
        if (t)
            for (const s of t) {
                var e = s.IsWeaknessHit;
                this.v6r(i, e, !1, s.Index)
            }
    }

    // OBSERVED source slice [24249,24395), SHA-256 1d632b0417cfb35bb22f2036326190e7b344071d1adece98474548135f9ccb34.
    p6r(t, i) {
        t && 0 < t.length
            ? (
                t = t[0],
                t = this.v6r(i, this.sVr, !1, t.Index),
                this.ToughDecreaseValue = t
            )
            : (
                t = this.v6r(i, this.sVr, !1),
                this.ToughDecreaseValue = t
            )
    }

    // OBSERVED source slice [24395,25170), SHA-256 055238d0f60488dcd7c347729f6e1c194a40b7267aa1ca98e0935cf25410c283.
    v6r(t, i, e, s = -1, h = 1) {
        var r, o, a, n = t.DamageId, _ = t.Target;
        return !(n < 1) && _ && (
            a = t.Target.GetComponent(22),
            o = t.DirectTarget.GetComponent(45),
            a
        ) && o
            ? (
                r = EntitySystem_1.EntitySystem.Get(t.BulletEntityId)?.GetBulletInfo().ContextId,
                o = o.CurrentSkill,
                a = a?.ExecuteBulletDamage(t.BulletEntityId, {
                    DamageDataId: n,
                    SkillLevel: t.SkillLevel,
                    Attacker: t.Attacker,
                    DirectTarget: t.DirectTarget ?? _,
                    HitPosition: t.HitPosition.ToUeVector(),
                    IsAddEnergy: this.aVr,
                    IsCounterAttack: this.IsTriggerCounterAttack,
                    ForceCritical: i,
                    IsBlocked: e,
                    PartId: s,
                    ExtraRate: h,
                    CounterSkillMessageId: this.IsTriggerCounterAttack ? o?.MNc : void 0,
                    BulletId: t.BulletId,
                    CounterSkillId: this.IsTriggerCounterAttack ? Number(o?.SkillId) : void 0,
                    CounterType: this.cVr
                }, r),
                this.rVr.IsAddEnergy = this.rVr.IsAddEnergy || a.IsAddEnergy,
                a.ToughResult
            )
            : 0
    }

    // OBSERVED source slice [27254,29177), SHA-256 183aa707e6ebefd70b267a9028338e1be124b7979344518b0104623fc85b1c70.
    a6r(t) {
        if (this.IsTriggerCounterAttack)
            CharacterHitComponent_1.E6r.Start(),
            this.KVr(this.rVr),
            CharacterHitComponent_1.E6r.Stop();
        else if (this.AY1)
            return;
        let i = 0;
        var e, s, h, r, o, a = this.rVr.HitEffect;
        a && (i = this.fVr ? 7 : a.被击动作),
        i = this.y6r(i),
        this.kVr(forbidHitTagIds)
            ? this.kVr(enterFkForbidHitTagIds) && this.HVr(t, i)
            : (
                e = t.GetBulletInfo(),
                a && (
                    this.BeHitTime = UE.GameplayStatics.GetTimeSeconds(this.Hte.Actor),
                    s = this.EVr?.GetCurrentValue(EAttributeId.Proto_Tough) ?? 0,
                    this.BeHitLocation.DeepCopy(this.rVr.HitPosition),
                    this.NeedCalculateFallInjure = !0,
                    s = 0 < s || this.ToughDecreaseValue <= 0 || this.OVr(1447214865),
                    h = this.IsTriggerCounterAttack && this.fVr,
                    r = 12 === a.被击动作,
                    o = this.OVr(1922078392),
                    s && !h && !r || o || r && this.OVr(-1525421570)
                        ? (
                            this.HVr(t, i),
                            this.cMc()
                        )
                        : (
                            this.cBe?.LogEndSkillReason(t, s, h),
                            CharacterHitComponent_1.S6r.Start(),
                            this.oVr && !this.oVr.CheckSwitchHitState(i, !0)
                                ? (
                                    this.HVr(t, i),
                                    CharacterHitComponent_1.S6r.Stop(),
                                    this.cMc()
                                )
                                : (
                                    CombatLog_1.CombatLog.Info("Hit", this.Entity, "受击", ["BeHitAnim", i]),
                                    this.ehl = !0,
                                    this.gVr = this.oVr?.SwitchHitState(i, !0) ?? 0,
                                    this.cMc(),
                                    RoleAudioController_1.RoleAudioController.OnPlayerIsHit(this.Entity),
                                    ModelManager_1.ModelManager.GameModeModel.IsMulti && this.Hte.SetMoveControlled(!0, 2, "受击"),
                                    this.BeHitAnim = i,
                                    this.EnterFk = !1,
                                    this.VVr()
                                        ? (
                                            BulletUtil_1.BulletUtil.GetHitRotator(e, this.Hte, this.Gue),
                                            this.mVr = this.Gue.ToUeRotator()
                                        )
                                        : (
                                            EventSystem_1.EventSystem.EmitWithTarget(this.Entity, EventDefine_1.EEventName.CharBeforeSetHitRotator),
                                            this.mVr = BulletUtil_1.BulletUtil.SetHitRotator(e, this.Hte, a.受击朝向Z轴偏转)
                                        ),
                                    this.I6r(),
                                    CharacterHitComponent_1.S6r.Stop(),
                                    CharacterHitComponent_1.T6r.Start(),
                                    this.VVr() && (this.BeHitAnim = BulletUtil_1.BulletUtil.GetOverrideHitAnimByAngle(this.Hte, this.BeHitAnim, this.mVr.Yaw)),
                                    7 !== this.BeHitAnim && 12 !== this.BeHitAnim && this.L6r(a),
                                    this.wY1?.UpdateLastHitTime(),
                                    this.M6r(lightHits.has(this.BeHitAnim) ? 1 : 2),
                                    CharacterHitComponent_1.T6r.Stop()
                                )
                        )
                )
            )
    }
};
