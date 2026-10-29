// NONCANONICAL / RECONSTRUCTED / STATIC_ONLY.
// OBSERVED: coreclr.dll SHA-256
// 8c968ce9703d37ff046ec4f46a1f3de880f5450f8f7131cb368f1a54e29e5e38.
// The exported names, ordinals, RVAs, VAs, and raw anchors below are retained
// as provenance; this namespace is packaging, not an original SDK signature.
// UNKNOWN: original C++ types, ownership, callers, thread-start routine,
// lifetime, and current-main/game reachability.

#include <Windows.h>
#include <cstdint>

namespace reconstructed::performance::native {

// OBSERVED export mono_native_thread_create, ordinal 870,
// RVA 0x26fe00, VA 0x18026fe00; raw anchor 47 bytes at file offset 0x26f200.
// RECONSTRUCTED from the direct KERNEL32!CreateThread edge.
uint32_t mono_native_thread_create(
    DWORD* thread_id,
    LPTHREAD_START_ROUTINE start,
    void* state) {
    HANDLE handle = CreateThread(nullptr, 0, start, state, 0, thread_id);
    return handle != nullptr ? 1u : 0u;
}

// OBSERVED export mono_native_thread_join, ordinal 873,
// RVA 0x26fe50, VA 0x18026fe50; raw anchor 88 bytes at file offset 0x26f250.
// RECONSTRUCTED from the direct OpenThread/WaitForSingleObject/CloseHandle
// edges. The numeric return is a source-like projection of observed EAX state,
// not a claim about the original BOOL/DWORD/HANDLE declaration.
uint32_t mono_native_thread_join(DWORD thread_id) {
    HANDLE handle = OpenThread(0x00100000u, TRUE, thread_id);
    if (!handle)
        return 0u;

    DWORD result = WaitForSingleObject(handle, 0xffffffffu);
    (void)CloseHandle(handle);
    return result != 0xffffffffu ? 1u : 0u;
}

} // namespace reconstructed::performance::native
