use crate::bindings::{export, Guest};

extern "Rust" {
    #[link_name = "_durable_main"]
    fn _durable_main();
}

struct DurableGuest;

impl Guest for DurableGuest {
    fn start() {
        if let Err(payload) = std::panic::catch_unwind(|| unsafe { _durable_main() }) {
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
}

export!(DurableGuest with_types_in crate::bindings);

#[macro_export]
macro_rules! durable_main {
    ($main:path) => {
        #[no_mangle]
        fn _durable_main() {
            $main()
        }
    };
}
