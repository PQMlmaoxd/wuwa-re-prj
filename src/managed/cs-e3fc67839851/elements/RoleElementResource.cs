/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay; it is not original source and is not a
 * standalone-compiling whole-class recovery. `partial` is an overlay
 * convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020031ee (global::RoleElementComponent; raw type row 58)
 * Scope: the two selected R23 accessor bodies only (rows 217-218).
 * The existing AttributeComponent field and all other RoleElementComponent
 * members remain external or in the earlier semantic view.
 */

using Aki.Protocol;

public partial class RoleElementComponent : EntityComponent
{
    // Reuses the observed field 0x0400d653 declared in the existing
    // RoleElementComponent semantic view; it is intentionally not redeclared.

    // Token 0x0601a854; original public, non-virtual accessor;
    // canonical source line 66; raw methods row 863.
    public float RoleElementEnergy => AttributeComponent?.GetCurrentValue(EAttributeType.ElementEnergy) ?? 0f;

    // Token 0x0601a855; original public, non-virtual accessor;
    // canonical source line 68; raw methods row 864.
    public float RoleElementEnergyMax => AttributeComponent?.GetCurrentValue(EAttributeType.ElementEnergyMax) ?? 0f;
}
