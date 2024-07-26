// Generated by `wit-bindgen` 0.28.0. DO NOT EDIT!
// Options used:
#[allow(unused_unsafe, clippy::all)]
/// Get access to the task name.
pub fn task_name() -> _rt::String {
    unsafe {
        #[repr(align(4))]
        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "task-name"]
            fn wit_import(_: *mut u8);
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn wit_import(_: *mut u8) {
            unreachable!()
        }
        wit_import(ptr0);
        let l1 = *ptr0.add(0).cast::<*mut u8>();
        let l2 = *ptr0.add(4).cast::<usize>();
        let len3 = l2;
        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
        _rt::string_lift(bytes3)
    }
}
#[allow(unused_unsafe, clippy::all)]
/// Get access to the encoded json data
pub fn task_data() -> _rt::String {
    unsafe {
        #[repr(align(4))]
        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "task-data"]
            fn wit_import(_: *mut u8);
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn wit_import(_: *mut u8) {
            unreachable!()
        }
        wit_import(ptr0);
        let l1 = *ptr0.add(0).cast::<*mut u8>();
        let l2 = *ptr0.add(4).cast::<usize>();
        let len3 = l2;
        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
        _rt::string_lift(bytes3)
    }
}
#[allow(unused_unsafe, clippy::all)]
/// Start a transaction. If this transaction has already executed to completion
/// then return the data from the last time it was executed.
///
/// Parameters:
/// - `label` - A text label that gets recorded in the event. This is used to
/// validate that events are in fact executing in the same order
/// when the workflow is restarted.
/// - `is-db` - Whether this transaction is a database transaction and should
/// reserve a database connection so that sql can be used within.
pub fn transaction_enter(label: &str, is_db: bool) -> Option<_rt::String> {
    unsafe {
        #[repr(align(4))]
        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
        let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 12]);
        let vec0 = label;
        let ptr0 = vec0.as_ptr().cast::<u8>();
        let len0 = vec0.len();
        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "transaction-enter"]
            fn wit_import(_: *mut u8, _: usize, _: i32, _: *mut u8);
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn wit_import(_: *mut u8, _: usize, _: i32, _: *mut u8) {
            unreachable!()
        }
        wit_import(
            ptr0.cast_mut(),
            len0,
            match &is_db {
                true => 1,
                false => 0,
            },
            ptr1,
        );
        let l2 = i32::from(*ptr1.add(0).cast::<u8>());
        match l2 {
            0 => None,
            1 => {
                let e = {
                    let l3 = *ptr1.add(4).cast::<*mut u8>();
                    let l4 = *ptr1.add(8).cast::<usize>();
                    let len5 = l4;
                    let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);

                    _rt::string_lift(bytes5)
                };
                Some(e)
            }
            _ => _rt::invalid_enum_discriminant(),
        }
    }
}
#[allow(unused_unsafe, clippy::all)]
/// Complete a transaction, saving the result of this transaction for future
/// use.
pub fn transaction_exit(data: &str) {
    unsafe {
        let vec0 = data;
        let ptr0 = vec0.as_ptr().cast::<u8>();
        let len0 = vec0.len();

        #[cfg(target_arch = "wasm32")]
        #[link(wasm_import_module = "$root")]
        extern "C" {
            #[link_name = "transaction-exit"]
            fn wit_import(_: *mut u8, _: usize);
        }

        #[cfg(not(target_arch = "wasm32"))]
        fn wit_import(_: *mut u8, _: usize) {
            unreachable!()
        }
        wit_import(ptr0.cast_mut(), len0);
    }
}
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    extern crate alloc as alloc_crate;
}

#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.28.0:core:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 266] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x8f\x01\x01A\x02\x01\
A\x08\x01@\0\0s\x03\0\x09task-name\x01\0\x03\0\x09task-data\x01\0\x01ks\x01@\x02\
\x05labels\x05is-db\x7f\0\x01\x03\0\x11transaction-enter\x01\x02\x01@\x01\x04dat\
as\x01\0\x03\0\x10transaction-exit\x01\x03\x04\x01\x17durable:core/core@1.0.0\x04\
\0\x0b\x0a\x01\0\x04core\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-\
component\x070.214.0\x10wit-bindgen-rust\x060.28.0";

#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen::rt::maybe_link_cabi_realloc();
}
