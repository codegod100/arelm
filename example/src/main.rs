//! Example consumer of the `root` (arelm) cell: a desktop binary that links
//! `root//:arelm-lib` and calls its public `run()` entrypoint.
//!
//! From the repo root: `buck2 run example//:app`.
fn main() {
    arelm::run();
}
