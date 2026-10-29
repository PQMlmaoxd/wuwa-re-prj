// NONCANONICAL / RECONSTRUCTED / STATIC_SOURCE_LIKE / NON_STANDALONE.
//
// This is a source-like projection of exactly two observed bodies.  It is not
// original source and is intentionally not a build target.  The CALL_VA_* and
// INDIRECT_CALLBACK_* spellings below are undeclared opaque boundaries, not
// guessed extern declarations or helper stubs.  Address-derived names are used
// so that no original class, method, field, callback, or ABI name is invented.
//
// OBSERVED input: coreclr.dll SHA-256
// 8c968ce9703d37ff046ec4f46a1f3de880f5450f8f7131cb368f1a54e29e5e38.
// OBSERVED image base: 0x180000000.
// UNKNOWN: original signatures/return types, record layouts, class identity,
// callback types/targets, ownership, lifetime, wait policy, and game reachability.

#include <Windows.h>
#include <cstdint>

namespace reconstructed::performance::native::thread_origin_static {

// RVA 0x34FA30 / VA 0x18034FA30 / 188 bytes.
// RECONSTRUCTED: the signature is only a source-like projection.  The original
// return type and ABI declaration were not recovered.
void caller_0x18034FA30() {
    // These are width-preserving absolute-address projections, not recovered
    // fields or a recovered enclosing object.
    auto& g32_0x1804AA064 =
        *reinterpret_cast<volatile std::int32_t*>(0x1804AA064ull);
    auto& g32_0x1804AA060 =
        *reinterpret_cast<volatile std::int32_t*>(0x1804AA060ull);
    auto& g32_0x1804AA048 =
        *reinterpret_cast<volatile std::int32_t*>(0x1804AA048ull);

    // Signed positive count.  The original loop uses the observed signed
    // compare for the threshold maximum; no concurrent-stability assumption is
    // added here.
    if (g32_0x1804AA064 > 0) {
        const std::int32_t source_count = g32_0x1804AA064;
        std::int32_t maximum = g32_0x1804AA060;

        for (std::int32_t k = 0; k < source_count; ++k) {
            const auto threshold = *reinterpret_cast<volatile std::int32_t*>(
                0x1804AA108ull + 0x60ull * static_cast<std::uintptr_t>(k));
            if (maximum < threshold)
                maximum = threshold;
        }

        g32_0x1804AA060 = maximum;
    }

    // Nonzero includes a negative value.  The initialization is therefore
    // preserved even when the later signed creation loop has zero iterations.
    if (g32_0x1804AA060 != 0) {
        g32_0x1804AA048 = 0;

        InitializeSRWLock(reinterpret_cast<PSRWLOCK>(0x1804AA020ull));
        InitializeConditionVariable(
            reinterpret_cast<PCONDITION_VARIABLE>(0x1804AA050ull));
        InitializeConditionVariable(
            reinterpret_cast<PCONDITION_VARIABLE>(0x1804AA058ull));

        // The shared bound is reread by the signed comparison on every
        // iteration.  This is not a cached/fixed population guarantee.
        for (std::int32_t index = 0; index < g32_0x1804AA060; ++index) {
            // OPAQUE CALL NOTATION.  At the observed callsite:
            //   RCX = LEA(0x1804AA068 + 4*index), an ID output address;
            //   RDX = LEA(0x18034FBD0), the concrete start address;
            //   R8  = sign-extended index, passed as numeric LPVOID state;
            //   EAX = returned wrapper value, ignored by this caller.
            // No wrapper body is copied here and no signature is inferred.
            CALL_VA_0x18026FE00(
                RCX = 0x1804AA068ull + 4ull * static_cast<std::uintptr_t>(index),
                RDX = 0x18034FBD0ull,
                R8 = static_cast<std::intptr_t>(index),
                EAX = ignored);
        }
    }
}

// RVA 0x34FBD0 / VA 0x18034FBD0 / 1062 bytes / 60 observed CFG blocks.
// IDA supplied a no-return annotation, but static evidence does not establish
// universal runtime termination.  No normal return path was recovered.
void thread_start_0x18034FBD0(std::intptr_t incoming_rcx) {
    // The incoming value is narrowed exactly as observed.  R15 is the
    // sign-extended form used for index addressing; it is not a heap pointer.
    const std::int32_t index = static_cast<std::int32_t>(incoming_rcx);
    const std::int64_t index_for_addressing = static_cast<std::int64_t>(index);

    // Opaque registration/helper call before the first source-table scan.
    CALL_VA_0x18032FEE0(RCX = incoming_rcx, EAX = ignored);

    // Initial per-index callback scan.  The count is reread by the observed
    // signed loop compare; threshold/callback/context tables are only address
    // projections with 0x60-byte spacing.
    std::int32_t k = 0;
    if (*reinterpret_cast<volatile std::int32_t*>(0x1804AA064ull) > 0) {
        while (k < *reinterpret_cast<volatile std::int32_t*>(0x1804AA064ull)) {
            const auto threshold = *reinterpret_cast<volatile std::int32_t*>(
                0x1804AA108ull + 0x60ull * static_cast<std::uintptr_t>(k));
            if (index >= threshold)
                break;

            const std::uintptr_t callback_slot = *reinterpret_cast<
                volatile std::uintptr_t*>(
                    0x1804AA0E0ull +
                    0x60ull * static_cast<std::uintptr_t>(k));
            if (callback_slot == 0)
                break;

            const std::uintptr_t context_table = *reinterpret_cast<
                volatile std::uintptr_t*>(
                    0x1804AA100ull +
                    0x60ull * static_cast<std::uintptr_t>(k));
            const std::uintptr_t callback_context =
                context_table == 0
                    ? 0
                    : *reinterpret_cast<volatile std::uintptr_t*>(
                          context_table + 8ull *
                                              static_cast<std::uintptr_t>(index_for_addressing));

            // The guard-dispatch call has RAX=callback_slot and
            // RCX=callback_context.  RDX remains the callback qword loaded by
            // the preceding observed MOV; its semantic role is unknown.  EAX
            // is not consumed by this first scan.
            INDIRECT_CALLBACK_EAX_0x1803C1410(
                RAX = callback_slot,
                RCX = callback_context,
                RDX = callback_slot,
                EAX = ignored);

            ++k;
        }
    }

    // This output is initialized once, before the first lock acquisition, not
    // once per job-selection pass.
    std::int32_t out_record = 0;
    std::int32_t out_flag = 0;
    std::uintptr_t out_job = 0;
    std::int64_t selected_record = 0;
    std::uintptr_t selected_offset = 0;
    std::uintptr_t selected_base = 0;
    std::uintptr_t selected_context = 0;
    std::uintptr_t current_context = 0;
    std::int32_t idle_result = 0;

    // Lock choice is reread at every observed acquire/release site.  The same
    // absolute address is used with the two external Win32 lock APIs; no
    // enclosing native type is asserted.
    if (*reinterpret_cast<volatile std::int32_t*>(0x1804AA048ull) != 0)
        EnterCriticalSection(reinterpret_cast<PCRITICAL_SECTION>(0x1804AA020ull));
    else
        AcquireSRWLockExclusive(reinterpret_cast<PSRWLOCK>(0x1804AA020ull));

    for (;;) {
        for (;;) {
            out_flag = 0;
            out_job = 0;

            *reinterpret_cast<volatile std::int32_t*>(
                0x1804AA088ull +
                4ull * static_cast<std::uintptr_t>(index_for_addressing)) = -1;

            // Lock is held.  The four register roles are observed at the
            // callsite; helper return EAX is not consumed directly.
            CALL_VA_0x18034F130(
                RCX = static_cast<std::uint32_t>(index),
                RDX = &out_record,
                R8 = &out_flag,
                R9 = &out_job,
                EAX = ignored);

            *reinterpret_cast<volatile std::int32_t*>(
                0x1804AA088ull +
                4ull * static_cast<std::uintptr_t>(index_for_addressing)) = out_record;

            selected_record = static_cast<std::int64_t>(out_record);
            selected_offset =
                static_cast<std::uintptr_t>(selected_record * 0x60);
            selected_base = 0x1804AA0B0ull + selected_offset;
            const std::uintptr_t selected_context_table = *reinterpret_cast<
                volatile std::uintptr_t*>(selected_base + 0x50ull);
            selected_context =
                selected_context_table == 0
                    ? 0
                    : *reinterpret_cast<volatile std::uintptr_t*>(
                          selected_context_table +
                          8ull * static_cast<std::uintptr_t>(index_for_addressing));
            current_context = selected_context;

            // Unlock before inspecting/calling the selected job.  No RAII or
            // synthetic finally path is introduced.
            if (*reinterpret_cast<volatile std::int32_t*>(0x1804AA048ull) != 0)
                LeaveCriticalSection(reinterpret_cast<PCRITICAL_SECTION>(0x1804AA020ull));
            else
                ReleaseSRWLockExclusive(reinterpret_cast<PSRWLOCK>(0x1804AA020ull));

            const std::uintptr_t job = out_job;
            if (job == 0)
                break;

            const std::uintptr_t job_callback = *reinterpret_cast<
                volatile std::uintptr_t*>(job + 0x8ull);
            // Observed job callback roles: RAX=job+8 target, RCX=selected
            // context, RDX=job.  Target and signature remain unknown.
            INDIRECT_CALLBACK_VOID_0x1803C1410(
                RAX = job_callback,
                RCX = current_context,
                RDX = job,
                EAX = ignored);

            if (*reinterpret_cast<volatile std::int32_t*>(0x1804AA048ull) != 0)
                EnterCriticalSection(reinterpret_cast<PCRITICAL_SECTION>(0x1804AA020ull));
            else
                AcquireSRWLockExclusive(reinterpret_cast<PSRWLOCK>(0x1804AA020ull));

            if (*reinterpret_cast<volatile std::int32_t*>(job + 0x18ull) != 1) {
                CALL_VA_0x18025A690(
                    RCX = 0, RDX = 4, R8 = 0x180434C58ull, EAX = ignored);
                CALL_VA_0x18034F120(
                    /* register state after the opaque logger is unknown */);
                __debugbreak();
            }

            *reinterpret_cast<volatile std::int32_t*>(job + 0x18ull) = 2;
            if (*reinterpret_cast<volatile std::int32_t*>(job + 0x18ull) != 2) {
                CALL_VA_0x18025A690(
                    RCX = 0, RDX = 4, R8 = 0x180434BF0ull, EAX = ignored);
                CALL_VA_0x18034F120(
                    /* register state after the opaque logger is unknown */);
                __debugbreak();
            }

            const std::uintptr_t queue = *reinterpret_cast<
                volatile std::uintptr_t*>(selected_base + 0x0ull);
            const std::uintptr_t queue_length = *reinterpret_cast<
                volatile std::uintptr_t*>(selected_base + 0x10ull);
            // RAW compare is unsigned; the post-match TEST/JS is a separate
            // sign-bit check, not a signed negative-index comparison.
            std::uint64_t queue_index = 0;

            if (queue_length == 0)
                goto job_missing_from_queue;

            while (*reinterpret_cast<volatile std::uintptr_t*>(
                       queue + 8ull * static_cast<std::uintptr_t>(queue_index)) != job) {
                ++queue_index;
                if (queue_index >= static_cast<std::uint64_t>(queue_length))
                    goto job_missing_from_queue;
            }

            if ((queue_index & (std::uint64_t{1} << 63)) != 0)
                goto job_missing_from_queue;

            *reinterpret_cast<volatile std::uintptr_t*>(
                queue + 8ull * static_cast<std::uintptr_t>(queue_index)) = 0;
            CALL_VA_0x180351890(RCX = selected_base, EAX = ignored);
            CALL_VA_0x18034C1C0(
                RCX = job,
                RDX = *reinterpret_cast<volatile std::uintptr_t*>(job + 0x10ull),
                R8 = 0x14,
                EAX = ignored);
            WakeConditionVariable(
                reinterpret_cast<PCONDITION_VARIABLE>(0x1804AA058ull));

            // The lock is still held here; select the next job.
        }

        if (out_flag == 0) {
            CALL_VA_0x18025A600(
                RCX = 0x180434CC0ull, RDX = 0xE4, EAX = ignored);
            __debugbreak();
        }

        if (*reinterpret_cast<volatile std::uintptr_t*>(
                selected_base + 0x38ull) == 0) {
            CALL_VA_0x18025A690(
                RCX = 0, RDX = 4, R8 = 0x180434C80ull, EAX = ignored);
            CALL_VA_0x18034F120(
                /* register state after the opaque logger is unknown */);
            __debugbreak();
        }

    idle_repeat:
        // OBSERVED re-entry target: after the one-time flag and +0x38 null
        // assertions, reload +0x38 and invoke it again without reacquiring.
        const auto selected_idle_callback = *reinterpret_cast<
            volatile std::uintptr_t*>(selected_base + 0x38ull);

        // Idle callback: RAX=record+0x38 target, RCX=current context;
        // other register state is not assigned a semantic role.
        INDIRECT_CALLBACK_VOID_0x1803C1410(
            RAX = selected_idle_callback,
            RCX = current_context,
            RDX = unchanged_register_state,
            EAX = ignored);

        const auto selected_idle_gate = *reinterpret_cast<
            volatile std::uintptr_t*>(selected_base + 0x40ull);
        idle_result = 0;
        if (selected_idle_gate == 0)
            goto relock;

        // The optional idle callback returns an observed EAX value.  Its
        // second observed argument is the selected record index.
        idle_result = INDIRECT_CALLBACK_EAX_0x1803C1410(
            RAX = selected_idle_gate,
            RCX = current_context,
            RDX = selected_record);

        if (idle_result != 0) {
            std::int64_t earlier_record = 0;
            while (earlier_record < selected_record) {
                const auto earlier_offset = static_cast<std::uintptr_t>(
                    earlier_record * 0x60);
                const auto earlier_threshold = *reinterpret_cast<
                    volatile std::int32_t*>(
                        0x1804AA108ull + earlier_offset);

                if (index < earlier_threshold) {
                    const auto earlier_context_table = *reinterpret_cast<
                        volatile std::uintptr_t*>(
                            0x1804AA100ull + earlier_offset);
                    const auto earlier_context =
                        earlier_context_table == 0
                            ? 0
                            : *reinterpret_cast<volatile std::uintptr_t*>(
                                  earlier_context_table +
                                  8ull * static_cast<std::uintptr_t>(index_for_addressing));
                    const auto earlier_predicate = *reinterpret_cast<
                        volatile std::uintptr_t*>(
                            0x1804AA0F8ull + earlier_offset);

                    if (earlier_predicate == 0 ||
                        INDIRECT_CALLBACK_EAX_0x1803C1410(
                            RAX = earlier_predicate,
                            RCX = earlier_context,
                            RDX = unchanged_register_state) != 0) {
                        if (*reinterpret_cast<volatile std::uintptr_t*>(
                                0x1804AA0C0ull + earlier_offset) != 0)
                            goto relock;

                        const auto earlier_gate = *reinterpret_cast<
                            volatile std::uintptr_t*>(
                                0x1804AA0F0ull + earlier_offset);
                        if (earlier_gate != 0 &&
                            INDIRECT_CALLBACK_EAX_0x1803C1410(
                                RAX = earlier_gate,
                                RCX = earlier_context,
                                RDX = earlier_record) != 0)
                            goto relock;
                    }
                }

                ++earlier_record;
            }

            // The observed body restores the selected context after the
            // earlier-record scan before testing the selected gate.
            current_context = selected_context;

            if (*reinterpret_cast<volatile std::uintptr_t*>(
                    0x1804AA0C0ull + selected_offset) == 0)
                goto idle_repeat;
        }

    relock:
        if (*reinterpret_cast<volatile std::int32_t*>(0x1804AA048ull) != 0)
            EnterCriticalSection(reinterpret_cast<PCRITICAL_SECTION>(0x1804AA020ull));
        else
            AcquireSRWLockExclusive(reinterpret_cast<PSRWLOCK>(0x1804AA020ull));

        if (idle_result == 0)
            WakeConditionVariable(
                reinterpret_cast<PCONDITION_VARIABLE>(0x1804AA058ull));
        // Repeat while the selected lock remains held.
    }

job_missing_from_queue:
    CALL_VA_0x18025A690(
        RCX = 0, RDX = 4, R8 = 0x180434C20ull, EAX = ignored);
    CALL_VA_0x18034F120(
        /* register state after the opaque logger is unknown */);
    __debugbreak();
    // No recovered normal continuation, unlock, free, join, or return is added.
}

} // namespace reconstructed::performance::native::thread_origin_static
