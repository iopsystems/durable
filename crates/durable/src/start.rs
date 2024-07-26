use std::panic::AssertUnwindSafe;

pub fn durable_entry(main: fn()) {
    if let Err(payload) = std::panic::catch_unwind(AssertUnwindSafe(|| main())) {
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

#[macro_export]
macro_rules! durable_main {
    ($main:path) => {
        #[no_mangle]
        extern "C" fn start() {
            $crate::export::durable_entry($main)
        }
    };
}
