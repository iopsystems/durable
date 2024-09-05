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
            pub struct HttpHeaderResult {
                pub name: _rt::String,
                pub value: _rt::Vec<u8>,
            }
            impl ::core::fmt::Debug for HttpHeaderResult {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HttpHeaderResult")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct HttpHeaderParam<'a> {
                pub name: &'a str,
                pub value: &'a [u8],
            }
            impl<'a> ::core::fmt::Debug for HttpHeaderParam<'a> {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("HttpHeaderParam")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(Clone)]
            pub struct HttpRequest<'a> {
                pub method: &'a str,
                pub url: &'a str,
                pub headers: &'a [HttpHeaderParam<'a>],
                pub body: Option<&'a [u8]>,
                pub timeout: Option<u64>,
            }
            impl<'a> ::core::fmt::Debug for HttpRequest<'a> {
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
                pub headers: _rt::Vec<HttpHeaderResult>,
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
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct HttpError2 {
                handle: _rt::Resource<HttpError2>,
            }
            impl HttpError2 {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for HttpError2 {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[resource-drop]http-error2"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            /// A HTTP request.
            ///
            /// In order to actually make the request you will need to call `fetch2`.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct HttpRequest2 {
                handle: _rt::Resource<HttpRequest2>,
            }
            impl HttpRequest2 {
                #[doc(hidden)]
                pub unsafe fn from_handle(handle: u32) -> Self {
                    Self {
                        handle: _rt::Resource::from_handle(handle),
                    }
                }
                #[doc(hidden)]
                pub fn take_handle(&self) -> u32 {
                    _rt::Resource::take_handle(&self.handle)
                }
                #[doc(hidden)]
                pub fn handle(&self) -> u32 {
                    _rt::Resource::handle(&self.handle)
                }
            }
            unsafe impl _rt::WasmResource for HttpRequest2 {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[resource-drop]http-request2"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Make an HTTP request.
            ///
            /// # Parameters
            /// - `request` - A description of the HTTP request to make.
            pub fn fetch(request: HttpRequest<'_>) -> Result<HttpResponse, HttpError> {
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
                            let HttpHeaderParam { name: name3, value: value3 } = e;
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
                    #[link(wasm_import_module = "durable:core/http@2.5.0")]
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
                                        HttpHeaderResult {
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
            impl HttpError2 {
                #[allow(unused_unsafe, clippy::all)]
                /// The error message describing what went wrong.
                pub fn message(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-error2.message"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = *ptr0.add(0).cast::<*mut u8>();
                        let l2 = *ptr0.add(4).cast::<usize>();
                        let len3 = l2;
                        let bytes3 = _rt::Vec::from_raw_parts(l1.cast(), len3, len3);
                        _rt::string_lift(bytes3)
                    }
                }
            }
            impl HttpError2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether this error is related to a timeout.
                pub fn is_timeout(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-error2.is-timeout"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl HttpError2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether this error was created while building the request.
                pub fn is_builder(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-error2.is-builder"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl HttpError2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether this error is related to a request.
                pub fn is_request(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-error2.is-request"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl HttpError2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether this error is related to the attempt to connect while making the
                /// request.
                pub fn is_connect(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-error2.is-connect"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Create a new request from an HTTP method and a URL.
                pub fn new(method: &str, url: &str) -> Result<HttpRequest2, HttpError2> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let vec0 = method;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let vec1 = url;
                        let ptr1 = vec1.as_ptr().cast::<u8>();
                        let len1 = vec1.len();
                        let ptr2 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[static]http-request2.new"]
                            fn wit_import(
                                _: *mut u8,
                                _: usize,
                                _: *mut u8,
                                _: usize,
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
                        ) {
                            unreachable!()
                        }
                        wit_import(ptr0.cast_mut(), len0, ptr1.cast_mut(), len1, ptr2);
                        let l3 = i32::from(*ptr2.add(0).cast::<u8>());
                        match l3 {
                            0 => {
                                let e = {
                                    let l4 = *ptr2.add(4).cast::<i32>();
                                    HttpRequest2::from_handle(l4 as u32)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l5 = *ptr2.add(4).cast::<i32>();
                                    HttpError2::from_handle(l5 as u32)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Set the HTTP method for this request.
                pub fn set_method(&self, method: &str) -> Result<(), HttpError2> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let vec0 = method;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-request2.set-method"]
                            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
                        let l2 = i32::from(*ptr1.add(0).cast::<u8>());
                        match l2 {
                            0 => {
                                let e = ();
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l3 = *ptr1.add(4).cast::<i32>();
                                    HttpError2::from_handle(l3 as u32)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Set the URL for this request.
                pub fn set_url(&self, url: &str) -> Result<(), HttpError2> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let vec0 = url;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-request2.set-url"]
                            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0, ptr1);
                        let l2 = i32::from(*ptr1.add(0).cast::<u8>());
                        match l2 {
                            0 => {
                                let e = ();
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l3 = *ptr1.add(4).cast::<i32>();
                                    HttpError2::from_handle(l3 as u32)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Set all the headers for this request at once.
                ///
                /// This overrides any headers that have been previously set.
                pub fn set_headers(
                    &self,
                    headers: &[HttpHeaderParam<'_>],
                ) -> Result<(), HttpError2> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let vec3 = headers;
                        let len3 = vec3.len();
                        let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec3.len() * 16,
                            4,
                        );
                        let result3 = if layout3.size() != 0 {
                            let ptr = _rt::alloc::alloc(layout3).cast::<u8>();
                            if ptr.is_null() {
                                _rt::alloc::handle_alloc_error(layout3);
                            }
                            ptr
                        } else {
                            { ::core::ptr::null_mut() }
                        };
                        for (i, e) in vec3.into_iter().enumerate() {
                            let base = result3.add(i * 16);
                            {
                                let HttpHeaderParam { name: name0, value: value0 } = e;
                                let vec1 = name0;
                                let ptr1 = vec1.as_ptr().cast::<u8>();
                                let len1 = vec1.len();
                                *base.add(4).cast::<usize>() = len1;
                                *base.add(0).cast::<*mut u8>() = ptr1.cast_mut();
                                let vec2 = value0;
                                let ptr2 = vec2.as_ptr().cast::<u8>();
                                let len2 = vec2.len();
                                *base.add(12).cast::<usize>() = len2;
                                *base.add(8).cast::<*mut u8>() = ptr2.cast_mut();
                            }
                        }
                        let ptr4 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-request2.set-headers"]
                            fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, result3, len3, ptr4);
                        let l5 = i32::from(*ptr4.add(0).cast::<u8>());
                        if layout3.size() != 0 {
                            _rt::alloc::dealloc(result3.cast(), layout3);
                        }
                        match l5 {
                            0 => {
                                let e = ();
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l6 = *ptr4.add(4).cast::<i32>();
                                    HttpError2::from_handle(l6 as u32)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Set the request timeout, in nanoseconds.
                pub fn set_timeout(&self, timeout: u64) {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-request2.set-timeout"]
                            fn wit_import(_: i32, _: i64);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i64) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, _rt::as_i64(&timeout));
                    }
                }
            }
            impl HttpRequest2 {
                #[allow(unused_unsafe, clippy::all)]
                /// Set the body of this request.
                pub fn set_body(&self, body: &[u8]) {
                    unsafe {
                        let vec0 = body;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/http@2.5.0")]
                        extern "C" {
                            #[link_name = "[method]http-request2.set-body"]
                            fn wit_import(_: i32, _: *mut u8, _: usize);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8, _: usize) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0.cast_mut(), len0);
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Make an HTTP request.
            ///
            /// This is similar to `fetch` except it returns an opaque error resource
            /// instead of an error enum.
            ///
            /// # Parameters
            /// - `request` - A description of the HTTP request to make.
            ///
            /// # Traps
            /// This function will trap if called from outside of a durable transaction.
            pub fn fetch2(request: HttpRequest2) -> Result<HttpResponse, HttpError2> {
                unsafe {
                    #[repr(align(4))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 24]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/http@2.5.0")]
                    extern "C" {
                        #[link_name = "fetch2"]
                        fn wit_import(_: i32, _: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: i32, _: *mut u8) {
                        unreachable!()
                    }
                    wit_import((&request).take_handle() as i32, ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => {
                            let e = {
                                let l2 = i32::from(*ptr0.add(4).cast::<u16>());
                                let l3 = *ptr0.add(8).cast::<*mut u8>();
                                let l4 = *ptr0.add(12).cast::<usize>();
                                let base11 = l3;
                                let len11 = l4;
                                let mut result11 = _rt::Vec::with_capacity(len11);
                                for i in 0..len11 {
                                    let base = base11.add(i * 16);
                                    let e11 = {
                                        let l5 = *base.add(0).cast::<*mut u8>();
                                        let l6 = *base.add(4).cast::<usize>();
                                        let len7 = l6;
                                        let bytes7 = _rt::Vec::from_raw_parts(
                                            l5.cast(),
                                            len7,
                                            len7,
                                        );
                                        let l8 = *base.add(8).cast::<*mut u8>();
                                        let l9 = *base.add(12).cast::<usize>();
                                        let len10 = l9;
                                        HttpHeaderResult {
                                            name: _rt::string_lift(bytes7),
                                            value: _rt::Vec::from_raw_parts(l8.cast(), len10, len10),
                                        }
                                    };
                                    result11.push(e11);
                                }
                                _rt::cabi_dealloc(base11, len11 * 16, 4);
                                let l12 = *ptr0.add(16).cast::<*mut u8>();
                                let l13 = *ptr0.add(20).cast::<usize>();
                                let len14 = l13;
                                HttpResponse {
                                    status: l2 as u16,
                                    headers: result11,
                                    body: _rt::Vec::from_raw_parts(l12.cast(), len14, len14),
                                }
                            };
                            Ok(e)
                        }
                        1 => {
                            let e = {
                                let l15 = *ptr0.add(4).cast::<i32>();
                                HttpError2::from_handle(l15 as u32)
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
    use core::fmt;
    use core::marker;
    use core::sync::atomic::{AtomicU32, Ordering::Relaxed};
    /// A type which represents a component model resource, either imported or
    /// exported into this component.
    ///
    /// This is a low-level wrapper which handles the lifetime of the resource
    /// (namely this has a destructor). The `T` provided defines the component model
    /// intrinsics that this wrapper uses.
    ///
    /// One of the chief purposes of this type is to provide `Deref` implementations
    /// to access the underlying data when it is owned.
    ///
    /// This type is primarily used in generated code for exported and imported
    /// resources.
    #[repr(transparent)]
    pub struct Resource<T: WasmResource> {
        handle: AtomicU32,
        _marker: marker::PhantomData<T>,
    }
    /// A trait which all wasm resources implement, namely providing the ability to
    /// drop a resource.
    ///
    /// This generally is implemented by generated code, not user-facing code.
    #[allow(clippy::missing_safety_doc)]
    pub unsafe trait WasmResource {
        /// Invokes the `[resource-drop]...` intrinsic.
        unsafe fn drop(handle: u32);
    }
    impl<T: WasmResource> Resource<T> {
        #[doc(hidden)]
        pub unsafe fn from_handle(handle: u32) -> Self {
            debug_assert!(handle != u32::MAX);
            Self {
                handle: AtomicU32::new(handle),
                _marker: marker::PhantomData,
            }
        }
        /// Takes ownership of the handle owned by `resource`.
        ///
        /// Note that this ideally would be `into_handle` taking `Resource<T>` by
        /// ownership. The code generator does not enable that in all situations,
        /// unfortunately, so this is provided instead.
        ///
        /// Also note that `take_handle` is in theory only ever called on values
        /// owned by a generated function. For example a generated function might
        /// take `Resource<T>` as an argument but then call `take_handle` on a
        /// reference to that argument. In that sense the dynamic nature of
        /// `take_handle` should only be exposed internally to generated code, not
        /// to user code.
        #[doc(hidden)]
        pub fn take_handle(resource: &Resource<T>) -> u32 {
            resource.handle.swap(u32::MAX, Relaxed)
        }
        #[doc(hidden)]
        pub fn handle(resource: &Resource<T>) -> u32 {
            resource.handle.load(Relaxed)
        }
    }
    impl<T: WasmResource> fmt::Debug for Resource<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Resource").field("handle", &self.handle).finish()
        }
    }
    impl<T: WasmResource> Drop for Resource<T> {
        fn drop(&mut self) {
            unsafe {
                match self.handle.load(Relaxed) {
                    u32::MAX => {}
                    other => T::drop(other),
                }
            }
        }
    }
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
    pub unsafe fn bool_lift(val: u8) -> bool {
        if cfg!(debug_assertions) {
            match val {
                0 => false,
                1 => true,
                _ => panic!("invalid bool discriminant"),
            }
        } else {
            val != 0
        }
    }
    extern crate alloc as alloc_crate;
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:import-http:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 1099] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xc9\x07\x01A\x02\x01\
A\x02\x01B-\x01p}\x01r\x02\x04names\x05value\0\x04\0\x0bhttp-header\x03\0\x01\x01\
p\x02\x01k\0\x01kw\x01r\x05\x06methods\x03urls\x07headers\x03\x04body\x04\x07tim\
eout\x05\x04\0\x0chttp-request\x03\0\x06\x01r\x03\x06status{\x07headers\x03\x04b\
ody\0\x04\0\x0dhttp-response\x03\0\x08\x01q\x06\x07timeout\0\0\x0einvalid-method\
\0\0\x0binvalid-url\x01s\0\x13invalid-header-name\0\0\x14invalid-header-value\0\0\
\x05other\x01s\0\x04\0\x0ahttp-error\x03\0\x0a\x04\0\x0bhttp-error2\x03\x01\x04\0\
\x0dhttp-request2\x03\x01\x01h\x0c\x01@\x01\x04self\x0e\0s\x04\0\x1b[method]http\
-error2.message\x01\x0f\x01@\x01\x04self\x0e\0\x7f\x04\0\x1e[method]http-error2.\
is-timeout\x01\x10\x04\0\x1e[method]http-error2.is-builder\x01\x10\x04\0\x1e[met\
hod]http-error2.is-request\x01\x10\x04\0\x1e[method]http-error2.is-connect\x01\x10\
\x01i\x0d\x01i\x0c\x01j\x01\x11\x01\x12\x01@\x02\x06methods\x03urls\0\x13\x04\0\x19\
[static]http-request2.new\x01\x14\x01h\x0d\x01j\0\x01\x12\x01@\x02\x04self\x15\x06\
methods\0\x16\x04\0\x20[method]http-request2.set-method\x01\x17\x01@\x02\x04self\
\x15\x03urls\0\x16\x04\0\x1d[method]http-request2.set-url\x01\x18\x01@\x02\x04se\
lf\x15\x07headers\x03\0\x16\x04\0![method]http-request2.set-headers\x01\x19\x01@\
\x02\x04self\x15\x07timeoutw\x01\0\x04\0![method]http-request2.set-timeout\x01\x1a\
\x01@\x02\x04self\x15\x04body\0\x01\0\x04\0\x1e[method]http-request2.set-body\x01\
\x1b\x01j\x01\x09\x01\x0b\x01@\x01\x07request\x07\0\x1c\x04\0\x05fetch\x01\x1d\x01\
j\x01\x09\x01\x12\x01@\x01\x07request\x11\0\x1e\x04\0\x06fetch2\x01\x1f\x03\x01\x17\
durable:core/http@2.5.0\x05\0\x04\x01\x1edurable:core/import-http@2.5.0\x04\0\x0b\
\x11\x01\0\x0bimport-http\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit\
-component\x070.215.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
