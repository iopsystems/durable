#[allow(dead_code)]
pub mod durable {
    #[allow(dead_code)]
    pub mod core {
        #[allow(dead_code, clippy::all)]
        pub mod http {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[derive(Clone)]
            pub struct HttpHeader {
                pub name: _rt::String,
                pub value: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for HttpHeader {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HttpHeader")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct HttpRequest {
                pub method: _rt::String,
                pub url: _rt::String,
                pub headers: _rt::Vec<HttpHeader>,
                pub body: Option<_rt::Vec<u8>>,
                pub timeout: Option<u64>,
            }
            impl ::core::fmt::Debug for HttpRequest {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HttpRequest")
                        .field("method", &self.method)
                        .field("url", &self.url)
                        .field("headers", &self.headers)
                        .field("body", &self.body)
                        .field("timeout", &self.timeout)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct HttpResponse {
                pub status: u16,
                pub headers: _rt::Vec<HttpHeader>,
                pub body: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for HttpResponse {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HttpResponse")
                        .field("status", &self.status)
                        .field("headers", &self.headers)
                        .field("body", &self.body)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub enum HttpError {
                Timeout,
                InvalidMethod,
                InvalidUrl(_rt::String),
                InvalidHeaderName,
                InvalidHeaderValue,
                Other(_rt::String),
            }
            impl ::core::fmt::Debug for HttpError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        HttpError::Timeout => {
                            f.debug_tuple("HttpError::Timeout").finish()
                        }
                        HttpError::InvalidMethod => {
                            f.debug_tuple("HttpError::InvalidMethod").finish()
                        }
                        HttpError::InvalidUrl(e) => {
                            f.debug_tuple("HttpError::InvalidUrl").field(e).finish()
                        }
                        HttpError::InvalidHeaderName => {
                            f.debug_tuple("HttpError::InvalidHeaderName").finish()
                        }
                        HttpError::InvalidHeaderValue => {
                            f.debug_tuple("HttpError::InvalidHeaderValue").finish()
                        }
                        HttpError::Other(e) => {
                            f.debug_tuple("HttpError::Other").field(e).finish()
                        }
                    }
                }
            }
            impl ::core::fmt::Display for HttpError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
            impl std::error::Error for HttpError {}
            #[allow(unused_unsafe, clippy::all)]
            /// Make an HTTP request.
            ///
            /// # Parameters
            /// - `request` - A description of the HTTP request to make.
            pub fn fetch(request: &HttpRequest) -> Result<HttpResponse, HttpError> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 24]);
                    let HttpRequest {
                        method: method0,
                        url: url0,
                        headers: headers0,
                        body: body0,
                        timeout: timeout0,
                    } = request;
                    let vec1 = method0;
                    let ptr1 = vec1.as_ptr().cast::<u8>();
                    let len1 = vec1.len();
                    let vec2 = url0;
                    let ptr2 = vec2.as_ptr().cast::<u8>();
                    let len2 = vec2.len();
                    let vec6 = headers0;
                    let len6 = vec6.len();
                    let layout6 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec6.len() * 16,
                        4,
                    );
                    let result6 = if layout6.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout6).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout6);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
                    };
                    for (i, e) in vec6.into_iter().enumerate() {
                        let base = result6.add(i * 16);
                        {
                            let HttpHeader { name: name3, value: value3 } = e;
                            let vec4 = name3;
                            let ptr4 = vec4.as_ptr().cast::<u8>();
                            let len4 = vec4.len();
                            *base.add(4).cast::<usize>() = len4;
                            *base.add(0).cast::<*mut u8>() = ptr4.cast_mut();
                            let vec5 = value3;
                            let ptr5 = vec5.as_ptr().cast::<u8>();
                            let len5 = vec5.len();
                            *base.add(12).cast::<usize>() = len5;
                            *base.add(8).cast::<*mut u8>() = ptr5.cast_mut();
                        }
                    }
                    let (result8_0, result8_1, result8_2) = match body0 {
                        Some(e) => {
                            let vec7 = e;
                            let ptr7 = vec7.as_ptr().cast::<u8>();
                            let len7 = vec7.len();
                            (1i32, ptr7.cast_mut(), len7)
                        }
                        None => (0i32, ::core::ptr::null_mut(), 0usize),
                    };
                    let (result9_0, result9_1) = match timeout0 {
                        Some(e) => (1i32, _rt::as_i64(e)),
                        None => (0i32, 0i64),
                    };
                    let ptr10 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/http@2.0.0")]
                    extern "C" {
                        #[link_name = "fetch"]
                        fn wit_import(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: i32,
                            _: *mut u8,
                            _: usize,
                            _: i32,
                            _: i64,
                            _: *mut u8,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: i32,
                        _: *mut u8,
                        _: usize,
                        _: i32,
                        _: i64,
                        _: *mut u8,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        ptr1.cast_mut(),
                        len1,
                        ptr2.cast_mut(),
                        len2,
                        result6,
                        len6,
                        result8_0,
                        result8_1,
                        result8_2,
                        result9_0,
                        result9_1,
                        ptr10,
                    );
                    let l11 = i32::from(*ptr10.add(0).cast::<u8>());
                    if layout6.size() != 0 {
                        _rt::alloc::dealloc(result6.cast(), layout6);
                    }
                    match l11 {
                        0 => {
                            let e = {
                                let l12 = i32::from(*ptr10.add(4).cast::<u16>());
                                let l13 = *ptr10.add(8).cast::<*mut u8>();
                                let l14 = *ptr10.add(12).cast::<usize>();
                                let base21 = l13;
                                let len21 = l14;
                                let mut result21 = _rt::Vec::with_capacity(len21);
                                for i in 0..len21 {
                                    let base = base21.add(i * 16);
                                    let e21 = {
                                        let l15 = *base.add(0).cast::<*mut u8>();
                                        let l16 = *base.add(4).cast::<usize>();
                                        let len17 = l16;
                                        let bytes17 = _rt::Vec::from_raw_parts(
                                            l15.cast(),
                                            len17,
                                            len17,
                                        );
                                        let l18 = *base.add(8).cast::<*mut u8>();
                                        let l19 = *base.add(12).cast::<usize>();
                                        let len20 = l19;
                                        HttpHeader {
                                            name: _rt::string_lift(bytes17),
                                            value: _rt::Vec::from_raw_parts(l18.cast(), len20, len20),
                                        }
                                    };
                                    result21.push(e21);
                                }
                                _rt::cabi_dealloc(base21, len21 * 16, 4);
                                let l22 = *ptr10.add(16).cast::<*mut u8>();
                                let l23 = *ptr10.add(20).cast::<usize>();
                                let len24 = l23;
                                HttpResponse {
                                    status: l12 as u16,
                                    headers: result21,
                                    body: _rt::Vec::from_raw_parts(l22.cast(), len24, len24),
                                }
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l25 = i32::from(*ptr10.add(4).cast::<u8>());
                                let v32 = match l25 {
                                    0 => HttpError::Timeout,
                                    1 => HttpError::InvalidMethod,
                                    2 => {
                                        let e32 = {
                                            let l26 = *ptr10.add(8).cast::<*mut u8>();
                                            let l27 = *ptr10.add(12).cast::<usize>();
                                            let len28 = l27;
                                            let bytes28 = _rt::Vec::from_raw_parts(
                                                l26.cast(),
                                                len28,
                                                len28,
                                            );
                                            _rt::string_lift(bytes28)
                                        };
                                        HttpError::InvalidUrl(e32)
                                    }
                                    3 => HttpError::InvalidHeaderName,
                                    4 => HttpError::InvalidHeaderValue,
                                    n => {
                                        debug_assert_eq!(n, 5, "invalid enum discriminant");
                                        let e32 = {
                                            let l29 = *ptr10.add(8).cast::<*mut u8>();
                                            let l30 = *ptr10.add(12).cast::<usize>();
                                            let len31 = l30;
                                            let bytes31 = _rt::Vec::from_raw_parts(
                                                l29.cast(),
                                                len31,
                                                len31,
                                            );
                                            _rt::string_lift(bytes31)
                                        };
                                        HttpError::Other(e32)
                                    }
                                };
                                v32
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
mod _rt {
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub use alloc_crate::alloc;
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
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
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
#[link_section = "component-type:wit-bindgen:0.30.0:import-http:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 495] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xed\x02\x01A\x02\x01\
A\x02\x01B\x0f\x01p}\x01r\x02\x04names\x05value\0\x04\0\x0bhttp-header\x03\0\x01\
\x01p\x02\x01k\0\x01kw\x01r\x05\x06methods\x03urls\x07headers\x03\x04body\x04\x07\
timeout\x05\x04\0\x0chttp-request\x03\0\x06\x01r\x03\x06status{\x07headers\x03\x04\
body\0\x04\0\x0dhttp-response\x03\0\x08\x01q\x06\x07timeout\0\0\x0einvalid-metho\
d\0\0\x0binvalid-url\x01s\0\x13invalid-header-name\0\0\x14invalid-header-value\0\
\0\x05other\x01s\0\x04\0\x0ahttp-error\x03\0\x0a\x01j\x01\x09\x01\x0b\x01@\x01\x07\
request\x07\0\x0c\x04\0\x05fetch\x01\x0d\x03\x01\x17durable:core/http@2.0.0\x05\0\
\x04\x01\x1edurable:core/import-http@2.0.0\x04\0\x0b\x11\x01\0\x0bimport-http\x03\
\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-component\x070.215.0\x10wit-\
bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
