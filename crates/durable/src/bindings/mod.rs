mod core {
    #[cfg(feature = "regenerate")]
    include!(concat!(env!("OUT_DIR"), "/core.rs"));

    #[cfg(not(feature = "regenerate"))]
    include!("core.rs");
}

pub use self::core::*;
