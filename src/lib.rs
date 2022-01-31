// [[file:../remote.note::*header][header:1]]
//! Remote Execution and Embarrassingly Parallel Computation for gosh
// header:1 ends here

// [[file:../remote.note::963f5eb8][963f5eb8]]
use gosh_core::*;
use gut::prelude::*;
// 963f5eb8 ends here

// [[file:../remote.note::b21b77b4][b21b77b4]]
mod base;
mod client;
mod interactive;
mod restful;
mod scheduler;
mod server;

pub mod cli;
// b21b77b4 ends here

// [[file:../remote.note::56d334b5][56d334b5]]
#[cfg(feature = "adhoc")]
/// Docs for local mods
pub mod docs {
    macro_rules! export_doc {
        ($l:ident) => {
            pub mod $l {
                pub use crate::$l::*;
            }
        };
    }

    export_doc!(base);
    export_doc!(interactive);
    export_doc!(restful);
}
// 56d334b5 ends here
