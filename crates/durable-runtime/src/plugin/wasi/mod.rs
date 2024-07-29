use std::time::SystemTime;

use slab::Slab;

mod cli;
mod clocks;
mod filesystem;
mod io;
mod random;

#[derive(Default)]
pub(super) struct WasiResources {
    errors: Slab<anyhow::Error>,
    pollables: Slab<Pollable>,
}

/// A pollable in WASI.
///
/// In our case, the only pollables we support are timeouts and a special case
/// value of u32::MAX for any streams that is _always_ ready to be polled.
#[derive(Copy, Clone, Debug)]
struct Pollable {
    txn: Option<i32>,
    timeout: SystemTime,
}

impl WasiResources {
    pub fn new() -> Self {
        Self::default()
    }
}
