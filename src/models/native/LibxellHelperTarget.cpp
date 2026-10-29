// NONCANONICAL / RECONSTRUCTED / STATIC_EVIDENCE_ONLY / NON_STANDALONE.
//
// Conservative register-state projection for libxell.dll RVA 0xc110.  This is
// not original source or a standalone helper/game implementation.  The
// original signature, types, ABI, and ownership semantics are unknown.
//
// OBSERVED binary SHA-256:
// 61aeeab7fdcfb6d1742f86c14c68c663d759607d01a6132645c857291eaebb86.
// OBSERVED body: RVA 0xc110, VA 0x18000c110, raw 0xb510..0xb589, 121 bytes.
// OBSERVED body SHA-256:
// b4068153c5ac226920ddfb0d65a206a12b8cbae3628f026e9388cad5ed4c1123.
// Full provenance, capture, CFG, bytes, validation, and unknowns:
// docs/performance/NATIVE_IFACE04_HELPER_TARGET.md
//
// The notation below is intentionally undeclared and non-buildable.  It is a
// register-state projection, not an extern declaration or helper stub.
// READ32/READ64 and WRITE32/WRITE64 are observed memory accesses.  RSP is the
// machine stack pointer; zeroextend32 models a 32-bit write's register view.
// OPAQUE_* names are call boundaries only.  No callee ABI or preservation is
// assumed.

namespace reconstructed::performance::native::libxell_static {

// NON_STANDALONE register-state projection only.  RCX is the incoming machine
// register; no source-level parameter type or original return type is claimed.
void projected_RVA_0xc110() {
    RSP -= 0x98;

    // VERIFIED: an eight-byte global read resolves to IMAGE_BASE + 0x59028,
    // then is XORed with the current RSP and stored at RSP+0x80.  The rendered
    // security-cookie label and prologue pattern support a cookie-purpose
    // inference only; no check or restoration by the opaque callee is verified.
    RAX = READ64(IMAGE_BASE + 0x59028);
    RAX ^= RSP;
    WRITE64(RSP + 0x80, RAX);

    WRITE32(RSP + 0x20, 0x47466558u);
    WRITE64(RSP + 0x28, 0);

    R8 = zeroextend32(READ32(RCX));
    if (low32(R8) != 0x47466558u) {
        goto mismatch;
    }

    // This field value is live in RAX on entry to the opaque call.  There is no
    // local RAX write after that call, so RET returns the post-call RAX value,
    // not a proven unconditional copy of RCX+0x08.
    RAX = READ64(RCX + 0x08);
    RCX = READ64(RSP + 0x80);
    RCX ^= RSP;
    OPAQUE_DIRECT_CALL_RVA_0x2790(
        /* RAX=field value on call entry; post-call RAX unknown; no assumed
           preservation or ABI contract */);
    // A normal, frame-compatible continuation reaches the captured add/RET;
    // this continuation does not assume callee preservation or an ABI.
    RSP += 0x98;
    RETURN_MACHINE_RAX;

mismatch:
    RDX = zeroextend32(0x47466558u);
    R9 = zeroextend32(1u);
    RCX = RSP + 0x30;
    OPAQUE_DIRECT_CALL_RVA_0x33f0(
        /* R8=mismatching DWORD, R9=1; no argument prototype or callee ABI
           recovered */);

    RDX = IMAGE_BASE + 0x57438;
    RCX = RSP + 0x30;
    OPAQUE_DIRECT_CALL_RVA_0x3c62b(
        /* RDX=IDA-labelled throw-information address (address computation,
           not a memory read); pointed-to content, type, and ABI unverified;
           no throw ABI or noreturn contract assumed */);

    // The captured bytes contain no normal cleanup/return here.  If the opaque
    // call returns, the observed next instruction is an INT3 trap.
    INT3_TRAP;
}

}  // namespace reconstructed::performance::native::libxell_static
