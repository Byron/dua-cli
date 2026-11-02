[private]
default:
    @just --list

# Preview a release; pass --execute to publish.
[positional-arguments]
smart-release *args:
    RUSTDOCFLAGS="-A rustdoc::private_intra_doc_links" cargo smart-release --auto-publish-of-stable-crates "$@"
