// PERF-01: NONCANONICAL / RECONSTRUCTED source-like projection.
// Evidence: CSharpScript.dll SHA-256 e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db;
// MVID 2d6d37cb-76e5-4075-8220-3eeca41471e3; PriorityQueue`1 TypeDef 0x0200006b.
// This intentionally retains observed duplicate/update/remove behavior.

using System;
using System.Collections.Generic;
using UnrealEngine.Extension;

public class PriorityQueue<T> : IClearable
{
    private readonly List<T> _Heap = new List<T>();
    private readonly Dictionary<T, int> _Hash = new Dictionary<T, int>();
    private readonly Comparison<T> _Compare;

    // TypeDef 0x0200006b; MethodDef 0x0600026d (get_Size); original source _global/PriorityQueue_g1.cs:13.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0600026d.
    public int Size => _Heap.Count;

    // TypeDef 0x0200006b; MethodDef 0x0600026e (get_Empty); original source _global/PriorityQueue_g1.cs:15.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0600026e.
    public bool Empty => _Heap.Count == 0;

    // TypeDef 0x0200006b; MethodDef 0x0600026f (get_Top); original source _global/PriorityQueue_g1.cs:17-27.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0600026f.
    public T? Top
    {
        get
        {
            if (_Heap.Count <= 0)
            {
                Singleton<Log>.Instance.Error(
                    (ELogModule)16,
                    (ELogAuthor)1,
                    "优先队列为空",
                    default(ReadOnlySpan<(string, object)>));
                return default(T);
            }

            return _Heap[0];
        }
    }

    // TypeDef 0x0200006b; MethodDef 0x0600026b; original source _global/PriorityQueue_g1.cs:30-33.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0600026b.
    public PriorityQueue(Comparison<T> compare)
    {
        _Compare = compare;
    }

    // TypeDef 0x0200006b; MethodDef 0x0600026c; original source _global/PriorityQueue_g1.cs:35-46.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x0600026c.
    public void Clone(PriorityQueue<T> origin)
    {
        Clear();
        foreach (T item in origin._Heap)
        {
            _Heap.Add(item);
        }

        foreach (KeyValuePair<T, int> item in origin._Hash)
        {
            _Hash.Add(item.Key, item.Value);
        }
    }

    // TypeDef 0x0200006b; MethodDef 0x06000270; original source _global/PriorityQueue_g1.cs:48-52.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x06000270.
    public void Clear()
    {
        _Heap.Clear();
        _Hash.Clear();
    }

    // TypeDef 0x0200006b; MethodDef 0x06000271; original source _global/PriorityQueue_g1.cs:54-57.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x06000271.
    public bool Has(T item)
    {
        return _Hash.ContainsKey(item);
    }

    // TypeDef 0x0200006b; MethodDef 0x06000272; original source _global/PriorityQueue_g1.cs:59-65.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#PriorityQueue.Push.
    public void Push(T item)
    {
        int index = _Heap.Count;
        _Heap.Add(item);
        _Hash[item] = index;
        Up(index);
    }

    // TypeDef 0x0200006b; MethodDef 0x06000273; original source _global/PriorityQueue_g1.cs:67-85.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#PriorityQueue.Pop.
    public T? Pop()
    {
        if (_Heap.Count <= 0)
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)1,
                "优先队列为空",
                default(ReadOnlySpan<(string, object)>));
            return default(T);
        }

        T value = _Heap[0];
        _Hash.Remove(value);
        T last = _Heap[_Heap.Count - 1];
        _Heap.RemoveAt(_Heap.Count - 1);
        if (_Heap.Count > 0)
        {
            _Heap[0] = last;
            _Hash[last] = 0;
            Down(0);
        }

        return value;
    }

    // TypeDef 0x0200006b; MethodDef 0x06000274; original source _global/PriorityQueue_g1.cs:87-107.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x06000274.
    public bool Remove(T item)
    {
        if (!_Hash.TryGetValue(item, out int index))
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)1,
                "元素不在优先队列中",
                default(ReadOnlySpan<(string, object)>));
            return false;
        }

        if (index == _Heap.Count - 1)
        {
            _Heap.RemoveAt(_Heap.Count - 1);
            _Hash.Remove(item);
            return true;
        }

        _Hash.Remove(item);
        T last = _Heap[_Heap.Count - 1];
        _Heap.RemoveAt(_Heap.Count - 1);
        _Heap[index] = last;
        _Hash[last] = index;
        Down(index);
        return true;
    }

    // TypeDef 0x0200006b; MethodDef 0x06000275; original source _global/PriorityQueue_g1.cs:109-121.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x06000275.
    public bool Update(T item)
    {
        if (!_Hash.TryGetValue(item, out int index))
        {
            Singleton<Log>.Instance.Error(
                (ELogModule)16,
                (ELogAuthor)1,
                "元素不在优先队列中",
                default(ReadOnlySpan<(string, object)>));
            return false;
        }

        if (!Up(index))
        {
            Down(index);
        }

        return true;
    }

    // TypeDef 0x0200006b; MethodDef 0x06000276; original source _global/PriorityQueue_g1.cs:123-129.
    // CIL evidence: evidence/performance/bounded_il_evidence.json method 0x06000276.
    public void Heapify()
    {
        for (int index = (_Heap.Count - 2) / 2; index >= 0; index--)
        {
            Down(index);
        }
    }

    // TypeDef 0x0200006b; MethodDef 0x06000277; original source _global/PriorityQueue_g1.cs:131-147.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#PriorityQueue.Up.
    private bool Up(int index)
    {
        bool changed = false;
        int current = index;
        while (current > 0)
        {
            int parent = (current - 1) / 2;
            if (_Compare(_Heap[current], _Heap[parent]) >= 0)
            {
                break;
            }

            Swap(current, parent);
            current = parent;
            changed = true;
        }

        return changed;
    }

    // TypeDef 0x0200006b; MethodDef 0x06000278; original source _global/PriorityQueue_g1.cs:149-171.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#PriorityQueue.Down.
    private bool Down(int index)
    {
        bool changed = false;
        int current = index;
        while (true)
        {
            int left = 2 * current + 1;
            int right = 2 * current + 2;
            int child = left;
            if (right < _Heap.Count && _Compare(_Heap[right], _Heap[left]) < 0)
            {
                child = right;
            }

            if (child >= _Heap.Count || _Compare(_Heap[current], _Heap[child]) <= 0)
            {
                break;
            }

            Swap(current, child);
            current = child;
            changed = true;
        }

        return changed;
    }

    // TypeDef 0x0200006b; MethodDef 0x06000279; original source _global/PriorityQueue_g1.cs:173-180.
    // CIL evidence: evidence/performance/bounded_il_critical.cil.txt#PriorityQueue.Swap.
    private void Swap(int i, int j)
    {
        T value = _Heap[i];
        _Heap[i] = _Heap[j];
        _Heap[j] = value;
        _Hash[_Heap[i]] = i;
        _Hash[_Heap[j]] = j;
    }
}
