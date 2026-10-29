use re_corpus::provenance::ErrataDocument;
use std::path::{Path, PathBuf};

#[test]
fn actual_a01_proposal_is_strictly_valid_and_not_implicitly_activatable() {
    let curated = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let research = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("repository root")
        .to_path_buf();
    let path = curated
        .join("tools/re/local-recovery/bulk-provenance-fix01/corrections/A01/PROPOSED_ERRATA.json");
    let bytes = std::fs::read(&path).expect("read A01 proposal");
    let document = ErrataDocument::parse(&bytes).expect("parse A01 proposal");
    document
        .validate_against_roots(&curated, &research)
        .expect("A01 proposal must validate against actual bytes");
    let error = document
        .validate_for_activation(&curated, &research)
        .expect_err("pending proposal must not be activatable");
    assert_eq!(error.code, "ACTIVATION_REVIEW_NOT_APPROVED");
}
