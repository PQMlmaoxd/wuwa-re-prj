// PERF-03: NONCANONICAL / RECONSTRUCTED JavaScript projection.
// Original shipped input (Windows): <REDACTED_PUBLICATION_VIEW_PRIVATE_INPUT_ROOT_02>/extracted\pakchunk3-WindowsNoEditor.pak\JavaScript\Game\Common\Event\EventCSharpBridge.js
// Original logical module path: JavaScript/Game/Common/Event/EventCSharpBridge.js
// Original shipped SHA-256: 6c91779f6f0f5c25c72ecfaa61aac679a809100e78aecfc5dd4bc61ef325d9ab
// Original compact implementation was on line 2; line 3 contained a
// sourceMappingURL comment for EventCSharpBridge.js.map. That source-map file is
// absent here, so this projection does not retain or invent a source-map ref.
// Names, arities, branch limits, return values, and native exception behavior
// below are reconstructed from the retained shipped bytes.

"use strict";

Object.defineProperty(exports, "__esModule", { value: true });
exports.EventCSharpBridge = void 0;

const cpp_1 = require("cpp");
const Entity_1 = require("../../../Core/Entity/Entity");
const ModelManager_1 = require("../../Manager/ModelManager");
const EventSystem_1 = require("./EventSystem");

class EventCSharpBridge {
    static InitializeEnvironment() {
        cpp_1.FEventSystem.Init(this.y0m);
    }

    static DestroyEnvironment() {
        cpp_1.FEventSystem.Clear();
    }

    static Emit(t, ...e) {
        if (e.length === 0) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0);
        } else if (e.length === 1) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0]);
        } else if (e.length === 2) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1]);
        } else if (e.length === 3) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2]);
        } else if (e.length === 4) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2], e[3]);
        } else if (e.length === 5) {
            cpp_1.FEventSystem.EmitFromJavaScript(t, 0, e[0], e[1], e[2], e[3], e[4]);
        }
        return true;
    }

    static EmitWithTarget(t, e, ...p) {
        if (t instanceof Entity_1.Entity) {
            if (p.length === 0) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id);
            } else if (p.length === 1) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0]);
            } else if (p.length === 2) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1]);
            } else if (p.length === 3) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2]);
            } else if (p.length === 4) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2], p[3]);
            } else if (p.length === 5) {
                cpp_1.FEventSystem.EmitFromJavaScript(e, t.Id, p[0], p[1], p[2], p[3], p[4]);
            }
        }
        return true;
    }

    static y0m(t, e, ...p) {
        if (e === 0) {
            EventSystem_1.EventSystem.Emit(t, ...p);
        } else {
            e = ModelManager_1.ModelManager.CreatureModel?.GetEntityById(e);
            e?.Entity && EventSystem_1.EventSystem.EmitWithTarget(e.Entity, t, ...p);
        }
    }
}

exports.EventCSharpBridge = EventCSharpBridge;
