/*
 * NONCANONICAL / RECONSTRUCTED semantic source view.
 * This is a selected source-only overlay, not original source and not a
 * standalone-compilation claim. The original type was public and was not
 * partial; `partial` is an overlay convenience only.
 *
 * Assembly SHA256: e3fc67839851d97f2f7cf749f49329bfbe44d059189383ef93ff24f7d22c02db
 * MVID: 2d6d37cb-76e5-4075-8220-3eeca41471e3
 * Original TypeDef: 0x020030b3 (global::LockOnInfo; raw type row 73)
 * Canonical source root:
 *   src/reconstructed/managed/full/CSharpScript/shards/853bd5820b5c83d778639a1f19a6ede2498b27b7806d99e3ff7cd758116f219f/source/_global/LockOnInfo.cs
 * Canonical source bytes/SHA256: 771 / 2737a71501ac9aff127352b9dc3134fa8334b2a8acbc2dc43cf183a4a0df0efe
 * Raw methods root SHA256: 5e54bdadc64fae2eb9de7fcddb03687775e7e7175484e26abb095ad9deedaba0
 * Raw types root SHA256: c7868dfe246b8b03ac5b59d6e9dab226e887595ceaa47faee633c12f9b3d03e6
 * Scope: two observed public fields and one selected ordinary whole method
 * body. The constructor, Copy, Different, and other behavior remain external.
 * The source view is not a complete class and does not promise compilation or
 * runtime verification.
 *
 * Selected fields:
 *   0x0400cc7c public EntityHandle? EntityHandle
 *   0x0400cc7d public System.String SocketName = ""
 */

public partial class LockOnInfo
{
	// Field 0x0400cc7c; original public EntityHandle?; canonical source line 3; no initializer.
	public EntityHandle? EntityHandle;

	// Field 0x0400cc7d; original public System.String; canonical source line 5; initializer = "".
	public string SocketName = "";

	// Token 0x06019aed; original public instance non-virtual; canonical source lines 19-26; raw methods row 282; RVA 7825548; 119 bytes; EH0.
	public bool Equal(LockOnInfo info)
	{
		if (EntityHandle?.Id == info.EntityHandle?.Id)
		{
			return SocketName == info.SocketName;
		}
		return false;
	}
}
