// NONCANONICAL / RECONSTRUCTED / STATIC_EVIDENCE_ONLY / NON_STANDALONE.
//
// Provider-local dispatch-adapter projection for libxell.dll RVA 0xc8c0.  This
// is not original source, a sleep implementation, a game implementation, or a
// build target.  The void return type is a projection only; the original
// return type and ABI are unknown.
//
// OBSERVED binary SHA-256:
// 61aeeab7fdcfb6d1742f86c14c68c663d759607d01a6132645c857291eaebb86.
// OBSERVED body: RVA 0xc8c0, VA 0x18000c8c0, raw 0xbcc0..0xbd04, 68 bytes.
// OBSERVED body SHA-256:
// 7c92b95797666e3d933695b420cf31721ba3347f1e4c62ce3bf8878fafec5276.
// Full provenance, CFG, bytes, validation, and unknowns:
// docs/performance/NATIVE_IFACE02_LIBXELL_DESTINATION.md
//
// The notation below is intentionally undeclared and non-buildable.  It is
// register-state source-like notation, not an extern declaration or helper
// stub.  PUSH64/POP64 update RSP; zeroextend32 models a 32-bit write's
// zero-extension into its 64-bit register view; READ32/READ64 are observed
// memory reads; RETURN_MACHINE_RAX denotes the machine return in RAX.
// OPAQUE_* names are comments-documented call boundaries only.

namespace reconstructed::performance::native::libxell_static {

// REGISTER-STATE projection only.  The original signature, types, and ABI are
// unknown.  Address-derived names do not identify original classes or methods.
void projected_RVA_0xc8c0() {
    PUSH64(RBX);  // updates RSP
    RSP -= 0x30;
    RBX = zeroextend32(RDX);

    if (RCX == 0) {
        RAX = zeroextend32(RCX - 8);
        // RCX is zero on this branch: RAX is exactly 0x00000000fffffff8.
        goto epilogue;
    }

    // OPAQUE_DIRECT_CALL_RVA_0x1610 is notation only.  RCX remains live into
    // this call; EDX and other incoming volatile registers are live as
    // observed.  Outputs are unknown except for registers subsequently read.
    OPAQUE_DIRECT_CALL_RVA_0x1610(
        /* liveRCX unchanged, liveEDX/otherincomingvolatile registers;
           outputs unknown except registers subsequently read */);

    // The direct-helper boundary remains opaque; no helper ABI preservation is
    // assumed here.
    R8 = RAX;
    RCX = READ64(RAX);
    RAX = READ64(RCX + 0x28);
    // This EBX read is after the direct helper and before the indirect call.
    RDX = zeroextend32(RBX);
    RCX = R8;

    // Actual machine transfer: CALL QWORD [loadedbase + 0x65020] through the
    // observed, unread RIP-relative memory slot.  RAX is an input register
    // loaded from [[direct-helper-return] + 0x28].  Treating RAX as the
    // candidate intended target of guarded dispatch is an inference from the
    // rendered guard-dispatch context, not proof that the slot forwards RAX or
    // that a callback is delivered.  The helper return is RCX and R8; EDX is
    // the post-helper EBX value; R9 is not set by this body.
    OPAQUE_INDIRECT_CALL_VIA_MEMORY_RVA_0x65020(
        /* actual transfer through unread slot; RAX=input/candidate intended
           target only; guarded-dispatch reading is inferred, not proof the
           slot forwards RAX or callback delivery; RCX=helperreturn,
           EDX=posthelperEBX; R8 also helperreturn; R9 not set by this body */);
    goto epilogue;

handler_entry_c8ee:
    // Disconnected normal-CFG block.  "Handler entry" is a supported
    // inference from unwind flags plus the captured Hex-Rays result, not an
    // independently confirmed EH map.  Exception routing and typed filters
    // were not recovered.
    RAX = zeroextend32(READ32(RSP + 0x40));
    RSP += 0x30;
    // POP64 restores the saved entry RBX; no helper-preservation fact is used.
    RBX = POP64();
    RETURN_MACHINE_RAX;

handler_entry_c8f8:
    // Disconnected normal-CFG block with the same supported handler-entry
    // inference; no independent EH map or typed route was recovered.
    RAX = zeroextend32(0xfffffc18u);
    // Exact machine value: 0x00000000fffffc18, not sign-extended -1000.

epilogue:
    RSP += 0x30;
    // This final POP64 restores the saved entry RBX; it is not a read of an
    // RBX value presumed preserved by the indirect helper.
    RBX = POP64();
    RETURN_MACHINE_RAX;
}

}  // namespace reconstructed::performance::native::libxell_static
