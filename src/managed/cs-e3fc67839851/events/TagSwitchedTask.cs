/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a source-only overlay, not original source. `partial` is an
 * overlay convenience and was not observed in the original metadata.
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x02003224 (global::TagSwitchedTask)
 * Scope: two selected whole source-level method bodies only.
 * The implicit constructor and tag delivery remain external; see METHOD_EVIDENCE_MAP.md.
 */

public partial class TagSwitchedTask : ITagTask
{
    private int TagId;

    private BaseTagComponent.TTagSwitchedCallback? Callback;

    private BaseTagComponent? TagComponent;

    // Token 0x0601ab1d; original ordinary public, non-virtual method; not the metadata interface implementation; canonical source lines 9-15.
    public void StartTask(int tagId, BaseTagComponent.TTagSwitchedCallback callback, BaseTagComponent tagComponent, Stat? callbackStat = null)
    {
        TagId = tagId;
        Callback = callback;
        TagComponent = tagComponent;
        TagComponent?.AddTagAddOrRemoveListener(TagId, callback, callbackStat);
    }

    // Token 0x0601ab1e; original public virtual final interface implementation; canonical source lines 17-20.
    public void EndTask()
    {
        TagComponent?.RemoveTagAddOrRemoveListener(TagId, Callback);
    }
}
