#[cfg(not(feature = "regenerate"))]
mod core;
#[cfg(feature = "regenerate")]
mod core {
    include!(concat!(env!("OUT_DIR"), "/core.rs"));
}

mod core2;

pub use self::core::*;
