// PERF-02: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; Queue`1 TypeDef 0x02000071.
// This is the custom FIFO queue used by TaskSystem, not System.Collections.Generic.Queue.
// SOURCE_VERIFIED means the corresponding canonical decompiled member was inspected.
// DIRECT_IL_VERIFIED means a complete successful direct-CIL response is stored.

using System;
using System.Collections;
using System.Collections.Generic;
using UnrealEngine.Extension;

public class Queue<T> : IEnumerable<T>, IEnumerable, IClearable
{
    // OBSERVED_FROM_CIL/CANONICAL_SOURCE fields. The projection keeps the observed
    // ring-buffer shape; nullable syntax is source-like metadata.
    private const int MINIMUM_GROW = 4;
    private const float THREE_QUARTER = 0.75f;

    private int Version;
    private T?[] DataArray;
    private readonly int DefaultCapacity;
    private int Head;
    private int Tail;
    private int SizeInternal;

    // MethodDef 0x060002a4; SOURCE_VERIFIED; metadata only (not directly captured).
    public int Size => SizeInternal;

    // MethodDef 0x060002a8; SOURCE_VERIFIED; metadata only (not directly captured).
    public T? Front
    {
        get
        {
            if (SizeInternal == 0)
            {
                Singleton<Log>.Instance.Error(
                    (ELogModule)16,
                    (ELogAuthor)3,
                    "队列为空",
                    default(ReadOnlySpan<(string, object)>));
                return default;
            }

            return DataArray[Head];
        }
    }

    // MethodDef 0x060002ac; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool Empty => SizeInternal == 0;

    // MethodDef 0x060002a3; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public Queue(int capacity = 4)
    {
        DefaultCapacity = capacity;
        DataArray = new T[capacity];
    }

    // MethodDef 0x060002a5; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public void Push(T element)
    {
        if (SizeInternal == DataArray.Length)
        {
            int capacity = DataArray.Length * 2;
            if (capacity < DataArray.Length + MINIMUM_GROW)
            {
                capacity = DataArray.Length + MINIMUM_GROW;
            }

            SetCapacity(capacity);
        }

        DataArray[Tail] = element;
        Tail = (Tail + 1) % DataArray.Length;
        SizeInternal++;
        Version++;
    }

    // MethodDef 0x060002a6; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public T? Pop()
    {
        if (SizeInternal == 0)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)3,
                "队列为空",
                default(ReadOnlySpan<(string, object)>));
            return default;
        }

        T? result = DataArray[Head];
        DataArray[Head] = default;
        Head = (Head + 1) % DataArray.Length;
        SizeInternal--;
        Version++;
        int capacity = DataArray.Length;
        if (capacity > 4 && capacity > DefaultCapacity && SizeInternal == capacity / 2)
        {
            int newCapacity = (int)(capacity * THREE_QUARTER);
            if (newCapacity > DefaultCapacity)
            {
                SetCapacity(newCapacity);
            }
        }
        return result;
    }

    // MethodDef 0x060002a7; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool TryPop(out T? result)
    {
        if (SizeInternal == 0)
        {
            result = default;
            return false;
        }

        result = DataArray[Head];
        DataArray[Head] = default;
        Head = (Head + 1) % DataArray.Length;
        SizeInternal--;
        Version++;
        int capacity = DataArray.Length;
        if (capacity > 4 && capacity > DefaultCapacity && SizeInternal == capacity / 2)
        {
            int newCapacity = (int)(capacity * THREE_QUARTER);
            if (newCapacity > DefaultCapacity)
            {
                SetCapacity(newCapacity);
            }
        }
        return true;
    }

    // MethodDef 0x060002a9; SOURCE_VERIFIED; metadata only (not directly captured).
    public bool TryGetFront(out T? result)
    {
        if (SizeInternal == 0)
        {
            result = default;
            return false;
        }

        result = DataArray[Head];
        return true;
    }

    // MethodDef 0x060002aa; SOURCE_VERIFIED; metadata only (not directly captured).
    // RECONSTRUCTED allocation risk: the bounds-error tuple array below is a
    // projection convenience; canonical source shows compiler-generated inline-array
    // span storage instead.
    public T? Get(int index)
    {
        if (SizeInternal == 0)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)14,
                "队列为空",
                default(ReadOnlySpan<(string, object)>));
            return default;
        }

        if (index < 0 || index >= SizeInternal)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)14,
                "下标越界",
                new (string, object)[] { ("index", index), ("size", SizeInternal) });
            return default;
        }

        return DataArray[(Head + index) % DataArray.Length];
    }

    // MethodDef 0x060002ab; SOURCE_VERIFIED; metadata only (not directly captured).
    private void SetCapacity(int capacity)
    {
        if (SizeInternal == 0)
        {
            DataArray = new T[capacity];
            Head = 0;
            Tail = 0;
            return;
        }

        int oldCapacity = DataArray.Length;
        if (capacity < oldCapacity)
        {
            if (Head == 0)
            {
                T[] array = new T[capacity];
                Array.Copy(DataArray, 0, array, 0, capacity);
                DataArray = array;
                Tail %= capacity;
                return;
            }

            int removed = oldCapacity - capacity;
            int boundary = oldCapacity - removed;
            if (Head < Tail)
            {
                int write = 0;
                for (int i = boundary; i < Tail; i++)
                {
                    DataArray[write++] = DataArray[i];
                    DataArray[i] = default;
                }

                if (boundary <= Head)
                {
                    Head = (Head + removed) % capacity;
                }

                int moved = Tail - boundary;
                Tail = moved > 0 ? moved % capacity : Tail % capacity;
                T[] array = new T[capacity];
                Array.Copy(DataArray, 0, array, 0, capacity);
                DataArray = array;
            }
            else if (Head >= Tail)
            {
                int tailSegment = oldCapacity - Head;
                int newHead = capacity - tailSegment;
                for (int i = 0; i < tailSegment; i++)
                {
                    DataArray[newHead + i] = DataArray[Head + i];
                    DataArray[Head + i] = default;
                }

                T[] array = new T[capacity];
                Array.Copy(DataArray, 0, array, 0, capacity);
                DataArray = array;
                Head = newHead;
            }

            return;
        }

        T[] expanded = new T[capacity];
        Array.Copy(DataArray, 0, expanded, 0, DataArray.Length);
        DataArray = expanded;
        if (Tail == 0)
        {
            Tail = SizeInternal;
        }
        else if (Head >= Tail)
        {
            int headSegment = oldCapacity - Head;
            int newHead = DataArray.Length - headSegment;
            for (int i = 0; i < headSegment; i++)
            {
                DataArray[newHead + i] = DataArray[Head + i];
                DataArray[Head + i] = default;
            }

            Head = newHead;
        }
    }

    // MethodDef 0x060002ad; SOURCE_VERIFIED; DIRECT_IL_VERIFIED.
    public void Clear()
    {
        Array.Clear(DataArray, 0, DataArray.Length);
        Head = 0;
        Tail = 0;
        SizeInternal = 0;
        Version++;
    }

    private struct Enumerator : IEnumerator<T>, IEnumerator, IDisposable
    {
        private Queue<T> Queue;
        private readonly int VersionAtCreation;
        private int Index;
        private T? CurrentValue;

        // MethodDef 0x0604670c; SOURCE_VERIFIED; metadata only (not directly captured).
        public Enumerator(Queue<T> queue)
        {
            Queue = queue;
            Index = 0;
            CurrentValue = default;
            VersionAtCreation = queue.Version;
        }

        // MethodDef 0x0604670f; SOURCE_VERIFIED; metadata only (not directly captured).
        readonly T IEnumerator<T>.Current => CurrentValue!;
        // MethodDef 0x06046710; SOURCE_VERIFIED; metadata only (not directly captured).
        readonly object? IEnumerator.Current => CurrentValue;

        // MethodDef 0x0604670d; SOURCE_VERIFIED; metadata only (not directly captured).
        public bool MoveNext()
        {
            if (VersionAtCreation != Queue.Version)
            {
                throw new InvalidOperationException("The queue was modified during foreach");
            }

            if (Index < Queue.SizeInternal)
            {
                CurrentValue = Queue.Get(Index++);
                return true;
            }

            return false;
        }

        // MethodDef 0x0604670e; SOURCE_VERIFIED; metadata only (not directly captured).
        public void Reset() => Index = 0;
        // MethodDef 0x06046711; SOURCE_VERIFIED; metadata only (not directly captured).
        public void Dispose() { }
    }

    // MethodDef 0x060002ae; SOURCE_VERIFIED; metadata only (not directly captured).
    public IEnumerator<T> GetEnumerator() => new Enumerator(this);
    // MethodDef 0x060002af; SOURCE_VERIFIED; metadata only (not directly captured).
    IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();
}
