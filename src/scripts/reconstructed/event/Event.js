// PERF03B: NONCANONICAL / RECONSTRUCTED JavaScript projection.
// Original shipped input (Windows): <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_02>/extracted\pakchunk3-WindowsNoEditor.pak\JavaScript\Core\Event\Event.js
// Original logical module path: JavaScript/Core/Event/Event.js
// Original shipped SHA-256: 816d4deb1ca3c0f80b8cd837b22a946164fbaeabd4f0c5d6ad31f30d41707f1f
// Original bytes: 7834 UTF-8 bytes, 6890 UTF-16 code units; source line 2.
// Lineage: LIN-LOCAL-JS-0001; BUILD_IDENTITY_UNRESOLVED; CONTINUITY_UNKNOWN.
// This is RECONSTRUCTED_JS, not original authored TypeScript or source-map output.
//
// Class boundary: the 19 selected ordinary-listener root members/accessors are
// reconstructed below, and RCu is the constructor's nested arrow callable.
// Public hold-key methods and conditional registration/removal methods are omitted;
// their class-level omission is intentional. Emit retains the observed calls to
// the omitted conditional surface and does not replace them with fake stubs.

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
exports.Event = void 0;

const Log_1 = require("../Common/Log");
const Stats_1 = require("../Common/Stats");
const Macro_1 = require("../Preprocessor/Macro");
const EventConditionListener_1 = require("./EventConditionListener");
const DEFAULT_SMALL_NAME_THRESHOLD = 4096;

class Event {
    constructor(t, e = DEFAULT_SMALL_NAME_THRESHOLD) {
        this.rK = t;
        this.RF_ = e;
        this.nK = new Map();
        this.IHl = new Map();
        this.sK = void 0;
        this.aK = void 0;
        this.AF_ = void 0;
        this.PF_ = new Set();
        this.unh = new Map();
        this.RCu = (t, e) => {
            t = this._K.get(t);
            return void 0 !== t && t.has(e);
        };
        this.LCu = new Map();
        this.AF_ = new Int8Array(Math.ceil(this.RF_));
    }

    Has(t, e) {
        var i;
        e = Event.lK.get(e);
        return !!e && ((i = this.nK.get(t)) && i.has(e)
            ? !this.RCu(t, e)
            : void 0 !== (i = this.uK.get(t)) && i.has(e));
    }

    Add(t, e) {
        return this.YW(t, e, 0);
    }

    Once(t, e) {
        return this.YW(t, e, 1);
    }

    Remove(t, e) {
        e = Event.lK.get(e);
        return !!e && this.O7(t, e);
    }

    ClearObject(t) {
        var e = this.nK.get(t);
        if (e)
            for (const i of e.keys())
                this.O7(t, i);
        return !0;
    }

    Emit(t, ...e) {
        if (this.IsEmitting(t))
            return (Log_1.Log.CheckError() && Log_1.Log.Error(
                "Event",
                1,
                "事件重复派发，请检查事件链是否产生循环调用",
                ["name", this.rK[t]],
                ["emittingEventInArray", [...this.AF_.entries()].filter(t => 0 !== t[1]).map(t => t[0])],
                ["emittingEventInSet", this.PF_]
            ), !1);

        this.mK(t, !0);
        var i;
        var r;
        i = this.LCu.get(t);
        i && (r = i.GetHandlesByParam(e[0])) &&
            (this.wCu(t, r, i, ...e), 0 === r.size) &&
            i.DeleteHandlesByParam(e[0]);

        const n = this.nK.get(t);
        n && (this.wCu(t, n, void 0, ...e), 0 === n.size) &&
            this.nK.delete(t);
        this.mK(t, !1);

        r = this._K.get(t);
        if (r) {
            for (const s of r)
                this.gK(t, s);
            r.clear();
            this._K.delete(t);
        }

        e = this.uK.get(t);
        if (e) {
            for (const o of e)
                this.fK(t, o[0], o[1]);
            e.clear();
            this.uK.delete(t);
        }

        // xCu/ACu/PCu belong to the omitted conditional registration/removal
        // surface. These calls are retained exactly at the observed flush point.
        if (i) {
            if (i.PendingRemoveHandles) {
                for (const [h, n] of i.PendingRemoveHandles)
                    for (const a of n)
                        this.ACu(t, h, a);
                i.PendingRemoveHandles.clear();
            }
            if (i.PendingAddHandles) {
                for (const [v, n] of i.PendingAddHandles)
                    for (const _ of n)
                        this.PCu(t, i, _[0], _[1], v);
                i.PendingAddHandles.clear();
            }
            i.IsHandlesEmpty() && this.LCu.delete(t);
        }
        return !0;
    }

    wCu(e, t, i, ...n) {
        let r = void 0;
        var s;
        !Stats_1.Stat.Enable || (s = this.rK[e], r = Event.dK.get(s)) ||
            (r = Stats_1.Stat.CreateNoFlameGraph("Event." + this.rK[e]), Event.dK.set(s, r));
        r?.Start();
        for (const v of t) {
            var o = v[0];
            var h = o.deref();
            if (h) {
                if (i ? !i.IsInPendingRemove(n[0], o) : !this.RCu(e, o)) {
                    1 === v[1] && (i ? this.xCu(e, o, n[0]) : this.O7(e, o));
                    var a = Event.CK.get(h);
                    a?.Start();
                    try {
                        h(...n);
                    }
                    catch (t) {
                        t instanceof Error
                            ? Log_1.Log.CheckError() && Log_1.Log.ErrorWithStack(
                                "Event",
                                1,
                                "事件处理方法执行异常",
                                t,
                                ["name", this.rK[e]],
                                ["error", t.message]
                            )
                            : Log_1.Log.CheckError() && Log_1.Log.Error(
                                "Event",
                                1,
                                "事件处理方法执行异常",
                                ["name", this.rK[e]],
                                ["error", t]
                            );
                    }
                    a?.Stop();
                }
            }
            else
                (Log_1.Log.CheckError() && Log_1.Log.Error(
                    "Event",
                    1,
                    "事件处理方法已被回收",
                    ["eventName", this.rK[e]],
                    ["stack", void 0]
                ), t.delete(o));
        }
        r?.Stop();
    }

    YW(t, e, i) {
        if (void 0 === this.rK[t])
            return Log_1.Log.CheckError() && Log_1.Log.Error(
                "Event",
                1,
                "事件名不存在，请检查事件名是否正确",
                ["name", t]
            ), !1;

        e = this.UCu(t, e);
        if (!this.IsEmitting(t))
            return this.fK(t, e, i);

        var n = this.nK.get(t);
        var r = this._K.get(t);
        if (n && n.has(e))
            return r && r.has(e)
                ? (r.delete(e), !0)
                : (Log_1.Log.CheckError() && Log_1.Log.Error(
                    "Event",
                    1,
                    "事件已存在，请检查同一个事件名同一个处理函数的注册逻辑",
                    ["name", this.rK[t]]
                ), !1);

        let s = this.uK.get(t);
        return s && s.has(e)
            ? (Log_1.Log.CheckError() && Log_1.Log.Error(
                "Event",
                1,
                "事件重复注册在待修改列表，请检查同一个事件名同一个处理函数的注册逻辑",
                ["name", this.rK[t]]
            ), !1)
            : (s || (s = new Map(), this.uK.set(t, s)), s.set(e, i), !0);
    }

    UCu(t, e) {
        let i = Event.lK.get(e);
        var n;
        return i || (i = new WeakRef(e), Event.lK.set(e, i)),
            Stats_1.Stat.Enable && !Event.CK.has(e) &&
                (n = e.name, Event.CK.set(
                    e,
                    n && 0 < n.length
                        ? Stats_1.Stat.CreateNoFlameGraph("EventHandle." + n)
                        : void 0
                )),
            i;
    }

    fK(t, e, i) {
        let n = this.nK.get(t);
        if (n) {
            if (n.has(e))
                return Log_1.Log.CheckError() && Log_1.Log.Error(
                    "Event",
                    1,
                    "事件重复注册，请检查同一个事件名同一个处理函数的注册逻辑",
                    ["name", this.rK[t]]
                ), !1;
        }
        else
            n = new Map(), this.nK.set(t, n);
        return n.set(e, i), !0;
    }

    O7(t, e) {
        if (!this.IsEmitting(t))
            return this.gK(t, e);

        var i = this.nK.get(t);
        var n = this.uK.get(t);
        if (!i || !i.has(e))
            return n && n.has(e)
                ? (n.delete(e), !0)
                : (Log_1.Log.CheckError() && Log_1.Log.Error(
                    "Event",
                    1,
                    "事件不存在，请检查同一个事件名同一个处理函数的移除逻辑",
                    ["name", this.rK[t]]
                ), !1);

        let r = this._K.get(t);
        return r && r.has(e)
            ? (Log_1.Log.CheckError() && Log_1.Log.Error(
                "Event",
                1,
                "事件重复移除在待移除列表，请检查同一个事件名同一个处理函数的移除逻辑",
                ["name", this.rK[t]]
            ), !1)
            : (r || (r = new Set(), this._K.set(t, r)), r.add(e), !0);
    }

    gK(t, e) {
        var i = this.nK.get(t);
        return i && i.delete(e)
            ? (0 === i.size && this.nK.delete(t), !0)
            : (Log_1.Log.CheckError() && Log_1.Log.Error(
                "Event",
                1,
                "事件不存在，请检查同一个事件名同一个处理函数的移除逻辑",
                ["name", this.rK[t]]
            ), !1);
    }

    IsEmitting(t) {
        return t >= this.RF_ || t < 0 ? this.xF_(t) : 0 !== this.AF_[t];
    }

    xF_(t) {
        return this.PF_.has(t);
    }

    mK(t, e) {
        t >= this.RF_ || t < 0 ? this.UF_(t, e) : this.AF_[t] = e ? 1 : 0;
    }

    UF_(t, e) {
        e ? this.PF_.add(t) : this.PF_.delete(t);
    }

    get uK() {
        return this.sK || (this.sK = new Map()), this.sK;
    }

    get _K() {
        return this.aK || (this.aK = new Map()), this.aK;
    }
}

Event.lK = new WeakMap();
Event.dK = new Map();
Event.CK = new WeakMap();

exports.Event = Event;
