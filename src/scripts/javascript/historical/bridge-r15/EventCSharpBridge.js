// R15: NONCANONICAL / RECONSTRUCTED JavaScript source view.
// Retained snapshot input (Windows): <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_02>/extracted\pakchunk3-WindowsNoEditor.pak\JavaScript\Game\Common\Event\EventCSharpBridge.js
// Original logical module path: JavaScript/Game/Common/Event/EventCSharpBridge.js
// Retained snapshot SHA-256: 6c91779f6f0f5c25c72ecfaa61aac679a809100e78aecfc5dd4bc61ef325d9ab
// The compact module body occupied physical source line 2. The observed line
// 3 sourceMappingURL comment is omitted because EventCSharpBridge.js.map is
// absent from the retained JavaScript root; no source map is fabricated.
// Identifiers, branch limits, evaluation order, return expressions, and
// native exception propagation below are preserved from the retained bytes.

"use strict";

Object.defineProperty(exports, "__esModule", { value: !0 }),
exports.EventCSharpBridge = void 0;

const cpp_1 = require("cpp"),
    Entity_1 = require("../../../Core/Entity/Entity"),
    ModelManager_1 = require("../../Manager/ModelManager"),
    EventSystem_1 = require("./EventSystem");

class EventCSharpBridge {
    static InitializeEnvironment() {
        cpp_1.FEventSystem.Init(this.y0m)
    }

    static DestroyEnvironment() {
        cpp_1.FEventSystem.Clear()
    }

    static Emit(t, ...e) {
        return 0 === e.length
            ? cpp_1.FEventSystem.EmitFromJavaScript(t, 0)
            : 1 === e.length
              ? cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0])
              : 2 === e.length
                ? cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1])
                : 3 === e.length
                  ? cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2])
                  : 4 === e.length
                    ? cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2], e[3])
                    : 5 === e.length &&
                      cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2], e[3], e[4]),
            !0
    }

    static EmitWithTarget(t, e, ...p) {
        return t instanceof Entity_1.Entity &&
            (0 === p.length
                ? cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id)
                : 1 === p.length
                  ? cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0])
                  : 2 === p.length
                    ? cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1])
                    : 3 === p.length
                      ? cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2])
                      : 4 === p.length
                        ? cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2], p[3])
                        : 5 === p.length &&
                          cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2], p[3], p[4])),
            !0
    }

    static y0m(t, e, ...p) {
        0 === e
            ? EventSystem_1.EventSystem.Emit(t, ...p)
            : (e = ModelManager_1.ModelManager.CreatureModel?.GetEntityById(e))?.Entity &&
              EventSystem_1.EventSystem.EmitWithTarget(e.Entity, t, ...p)
    }
}

exports.EventCSharpBridge = EventCSharpBridge;
