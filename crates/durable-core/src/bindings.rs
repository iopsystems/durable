#[allow(dead_code)]
pub mod durable {
    #[allow(dead_code)]
    pub mod core {
        #[allow(dead_code, clippy::all)]
        pub mod core {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[allow(unused_unsafe, clippy::all)]
            /// Get the task id for the current workflow.
            pub fn task_id() -> i64 {
                unsafe {
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/core@2.4.0")]
                    extern "C" {
                        #[link_name = "task-id"]
                        fn wit_import() -> i64;
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import() -> i64 {
                        unreachable!()
                    }
                    let ret = wit_import();
                    ret
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Get the task name for the current workflow.
            pub fn task_name() -> _rt::String {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/core@2.4.0")]
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
            /// Get the json-encoded task data for the current workflow.
            pub fn task_data() -> _rt::String {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 8]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/core@2.4.0")]
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
            /// # Parameters
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
                    #[link(wasm_import_module = "durable:core/core@2.4.0")]
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
                                let bytes5 = _rt::Vec::from_raw_parts(
                                    l3.cast(),
                                    len5,
                                    len5,
                                );
                                _rt::string_lift(bytes5)
                            };
                            Some(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Complete a transaction, saving the result of this transaction for future use.
            ///
            /// Parameters:
            /// - `data` - JSON-encoded state to save.
            pub fn transaction_exit(data: &str) {
                unsafe {
                    let vec0 = data;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/core@2.4.0")]
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
        }
        #[allow(dead_code, clippy::all)]
        pub mod notify {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            pub type Datetime = super::super::super::wasi::clocks::wall_clock::Datetime;
            /// A notification event.
            #[derive(Clone)]
            pub struct Event {
                /// The wall-clock time at which this notification was created.
                pub created_at: Datetime,
                /// The name of the event itself.
                pub event: _rt::String,
                /// JSON-encoded data associated with the event.
                pub data: _rt::String,
            }
            impl ::core::fmt::Debug for Event {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Event")
                        .field("created-at", &self.created_at)
                        .field("event", &self.event)
                        .field("data", &self.data)
                        .finish()
                }
            }
            /// Errors that can occur as when attempting to notify another task.
            #[derive(Clone)]
            pub enum NotifyError {
                /// There is no task with the requested task id.
                TaskNotFound,
                /// There is a task with the requested id, but it is no longer running.
                TaskDead,
                /// Other unspecified errors that may occur, such as data not being valid JSON.
                Other(_rt::String),
            }
            impl ::core::fmt::Debug for NotifyError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        NotifyError::TaskNotFound => {
                            f.debug_tuple("NotifyError::TaskNotFound").finish()
                        }
                        NotifyError::TaskDead => {
                            f.debug_tuple("NotifyError::TaskDead").finish()
                        }
                        NotifyError::Other(e) => {
                            f.debug_tuple("NotifyError::Other").field(e).finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for NotifyError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for NotifyError {}
            #[allow(unused_unsafe, clippy::all)]
            /// Attempt to read the next available notification, if there is one.
            /// notification: func() -> option<event>;
            /// Read the next available notification, blocking until one is available.
            pub fn notification_blocking() -> Event {
                unsafe {
                    #[repr(align(8))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 32]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 32]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/notify@2.4.0")]
                    extern "C" {
                        #[link_name = "notification-blocking"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<i64>();
                    let l2 = *ptr0.add(8).cast::<i32>();
                    let l3 = *ptr0.add(16).cast::<*mut u8>();
                    let l4 = *ptr0.add(20).cast::<usize>();
                    let len5 = l4;
                    let bytes5 = _rt::Vec::from_raw_parts(l3.cast(), len5, len5);
                    let l6 = *ptr0.add(24).cast::<*mut u8>();
                    let l7 = *ptr0.add(28).cast::<usize>();
                    let len8 = l7;
                    let bytes8 = _rt::Vec::from_raw_parts(l6.cast(), len8, len8);
                    Event {
                        created_at: super::super::super::wasi::clocks::wall_clock::Datetime {
                            seconds: l1 as u64,
                            nanoseconds: l2 as u32,
                        },
                        event: _rt::string_lift(bytes5),
                        data: _rt::string_lift(bytes8),
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Emit a notification for a task.
            pub fn notify(
                task: i64,
                event: &str,
                data: &str,
            ) -> Result<(), NotifyError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let vec0 = event;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    let vec1 = data;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let ptr2 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/notify@2.4.0")]
                    extern "C" {
                        #[link_name = "notify"]
                        fn wit_import(
                            _: i64,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: i64,
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        _rt::as_i64(&task),
                        ptr0.cast_mut(),
                        len0,
                        ptr1.cast_mut(),
                        len1,
                        ptr2,
                    );
                    let l3 = i32::from(*ptr2.add(0).cast::<u8>());
                    match l3 {
                        0 => {
                            let e = ();
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l4 = i32::from(*ptr2.add(4).cast::<u8>());
                                let v8 = match l4 {
                                    0 => NotifyError::TaskNotFound,
                                    1 => NotifyError::TaskDead,
                                    n => {
                                        debug_assert_eq!(n, 2, "invalid enum discriminant");
                                        let e8 = {
                                            let l5 = *ptr2.add(8).cast::<*mut u8>();
                                            let l6 = *ptr2.add(12).cast::<usize>();
                                            let len7 = l6;
                                            let bytes7 = _rt::Vec::from_raw_parts(
                                                l5.cast(),
                                                len7,
                                                len7,
                                            );
                                            _rt::string_lift(bytes7)
                                        };
                                        NotifyError::Other(e8)
                                    }
                                };
                                v8
                            };
                            Err(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
        }
    }
}
#[allow(dead_code)]
pub mod wasi {
    #[allow(dead_code)]
    pub mod clocks {
        #[allow(dead_code, clippy::all)]
        pub mod wall_clock {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            /// A time and date in seconds plus nanoseconds.
            #[repr(C)]
            #[derive(Clone, Copy)]
            pub struct Datetime {
                pub seconds: u64,
                pub nanoseconds: u32,
            }
            impl ::core::fmt::Debug for Datetime {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Datetime")
                        .field("seconds", &self.seconds)
                        .field("nanoseconds", &self.nanoseconds)
                        .finish()
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Read the current value of the clock.
            ///
            /// This clock is not monotonic, therefore calling this function repeatedly
            /// will not necessarily produce a sequence of non-decreasing values.
            ///
            /// The returned timestamps represent the number of seconds since
            /// 1970-01-01T00:00:00Z, also known as [POSIX's Seconds Since the Epoch],
            /// also known as [Unix Time].
            ///
            /// The nanoseconds field of the output is always less than 1000000000.
            ///
            /// [POSIX's Seconds Since the Epoch]: https://pubs.opengroup.org/onlinepubs/9699919799/xrat/V4_xbd_chap04.html#tag_21_04_16
            /// [Unix Time]: https://en.wikipedia.org/wiki/Unix_time
            pub fn now() -> Datetime {
                unsafe {
                    #[repr(align(8))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wasi:clocks/wall-clock@0.2.0")]
                    extern "C" {
                        #[link_name = "now"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<i64>();
                    let l2 = *ptr0.add(8).cast::<i32>();
                    Datetime {
                        seconds: l1 as u64,
                        nanoseconds: l2 as u32,
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Query the resolution of the clock.
            ///
            /// The nanoseconds field of the output is always less than 1000000000.
            pub fn resolution() -> Datetime {
                unsafe {
                    #[repr(align(8))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 16]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "wasi:clocks/wall-clock@0.2.0")]
                    extern "C" {
                        #[link_name = "resolution"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = *ptr0.add(0).cast::<i64>();
                    let l2 = *ptr0.add(8).cast::<i32>();
                    Datetime {
                        seconds: l1 as u64,
                        nanoseconds: l2 as u32,
                    }
                }
            }
        }
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
    pub fn as_i64<T: AsI64>(t: T) -> i64 {
        t.as_i64()
    }
    pub trait AsI64 {
        fn as_i64(self) -> i64;
    }
    impl<'a, T: Copy + AsI64> AsI64 for &'a T {
        fn as_i64(self) -> i64 {
            (*self).as_i64()
        }
    }
    impl AsI64 for i64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    impl AsI64 for u64 {
        #[inline]
        fn as_i64(self) -> i64 {
            self as i64
        }
    }
    extern crate alloc as alloc_crate;
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:import-core:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 665] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\x97\x04\x01A\x02\x01\
A\x07\x01B\x0a\x01@\0\0x\x04\0\x07task-id\x01\0\x01@\0\0s\x04\0\x09task-name\x01\
\x01\x04\0\x09task-data\x01\x01\x01ks\x01@\x02\x05labels\x05is-db\x7f\0\x02\x04\0\
\x11transaction-enter\x01\x03\x01@\x01\x04datas\x01\0\x04\0\x10transaction-exit\x01\
\x04\x03\x01\x17durable:core/core@2.4.0\x05\0\x01B\x05\x01r\x02\x07secondsw\x0bn\
anosecondsy\x04\0\x08datetime\x03\0\0\x01@\0\0\x01\x04\0\x03now\x01\x02\x04\0\x0a\
resolution\x01\x02\x03\x01\x1cwasi:clocks/wall-clock@0.2.0\x05\x01\x02\x03\0\x01\
\x08datetime\x01B\x0b\x02\x03\x02\x01\x02\x04\0\x08datetime\x03\0\0\x01r\x03\x0a\
created-at\x01\x05events\x04datas\x04\0\x05event\x03\0\x02\x01q\x03\x0etask-not-\
found\0\0\x09task-dead\0\0\x05other\x01s\0\x04\0\x0cnotify-error\x03\0\x04\x01@\0\
\0\x03\x04\0\x15notification-blocking\x01\x06\x01j\0\x01\x05\x01@\x03\x04taskx\x05\
events\x04datas\0\x07\x04\0\x06notify\x01\x08\x03\x01\x19durable:core/notify@2.4\
.0\x05\x03\x04\x01\x1edurable:core/import-core@2.4.0\x04\0\x0b\x11\x01\0\x0bimpo\
rt-core\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.21\
5.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
