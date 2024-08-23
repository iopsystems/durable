use std::io::Write;

// PanicInfo has been deprecated and renamed to PanicHookInfo but only in 1.82
// or newer.
//
// Use a type alias here to avoid the deprecation warning.
#[allow(deprecated)]
type PanicInfo<'a> = std::panic::PanicInfo<'a>;

extern "C" {
    #[allow(dead_code)]
    fn durable_ctor_wrapper();
}

#[no_mangle]
extern "C" fn durable_ctor() {
    if cfg!(target_arch = "wasm32") {
        std::panic::set_hook(Box::new(durable_panic_hook))
    }
}

fn durable_panic_hook(info: &PanicInfo) {
    #[used]
    static __UNUSED_LINK_HACK: unsafe extern "C" fn() = durable_ctor_wrapper;

    let payload = info.payload();
    let msg: &str = if let Some(msg) = payload.downcast_ref::<String>() {
        msg
    } else if let Some(msg) = payload.downcast_ref::<&str>() {
        msg
    } else {
        "Box<dyn Any>"
    };

    crate::transaction::maybe_txn("durable::panic", || {
        use std::fmt::Write;

        let name = crate::task_name();
        let mut message = String::new();

        let _ = write!(&mut message, "task '{name}' panicked");
        if let Some(location) = info.location() {
            let _ = write!(&mut message, " at {location}");
        }

        let _ = write!(&mut message, "\n{msg}\n");

        let mut err = std::io::stderr();
        let _ = err.write_all(message.as_bytes());
    });

    std::process::abort()
}
