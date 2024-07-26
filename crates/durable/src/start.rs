use crate::bindings::{export, Guest};

extern "Rust" {
    #[link_name = "_start"]
    fn _start();
}

pub fn durable_start(main: fn()) {
    if let Err(payload) = std::panic::catch_unwind(|| main()) {
        let message: Option<&str> = if let Some(message) = payload.downcast_ref::<String>() {
            Some(message)
        } else if let Some(message) = payload.downcast_ref::<&str>() {
            Some(message)
        } else {
            None
        };

        let message = match message {
            Some(message) => format!("workflow panicked: {message}"),
            None => format!("workflow panicked"),
        };

        crate::abort(&message);
    }
}

struct DurableGuest;

impl Guest for DurableGuest {
    fn start() {
        unsafe { _start() }
    }
}

export!(DurableGuest with_types_in crate::bindings);

#[macro_export]
macro_rules! durable_main {
    ($main:path) => {
        #[no_mangle]
        fn _start() {
            $crate::export::durable_start($main)
        }
    };
}
