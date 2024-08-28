#[allow(dead_code)]
pub mod durable {
    #[allow(dead_code)]
    pub mod core {
        #[allow(dead_code, clippy::all)]
        pub mod sql {
            #[used]
            #[doc(hidden)]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            /// Information about a SQL type as used by the underlying database driver.
            ///
            /// These map fairly closely to the underlying SQLx implementation in the
            /// runtime.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct TypeInfo {
                handle: _rt::Resource<TypeInfo>,
            }
            impl TypeInfo {
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
            unsafe impl _rt::WasmResource for TypeInfo {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[resource-drop]type-info"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            /// A timestamp recording a point in time without a timezone.
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Timestamp {
                pub seconds: i64,
                pub subsec_nanos: u32,
            }
            impl ::core::fmt::Debug for Timestamp {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Timestamp")
                        .field("seconds", &self.seconds)
                        .field("subsec-nanos", &self.subsec_nanos)
                        .finish()
                }
            }
            /// A timestamp combined with a utc offset representing a timezone.
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Timestamptz {
                pub seconds: i64,
                pub subsec_nanos: u32,
                /// Positive values are on the eastern hemisphere while negative
                /// values are on the western hemisphere.
                pub offset: i32,
            }
            impl ::core::fmt::Debug for Timestamptz {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Timestamptz")
                        .field("seconds", &self.seconds)
                        .field("subsec-nanos", &self.subsec_nanos)
                        .field("offset", &self.offset)
                        .finish()
                }
            }
            /// A UUID.
            ///
            /// Since WIT does not support 128-bit integers the uuid is split into the
            /// hi and lo 64 bits here.
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Uuid {
                pub hi: u64,
                pub lo: u64,
            }
            impl ::core::fmt::Debug for Uuid {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Uuid")
                        .field("hi", &self.hi)
                        .field("lo", &self.lo)
                        .finish()
                }
            }
            /// An IPv4 network range.
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Ipv4Network {
                /// The IP representation in little-endian byte order.
                pub addr: u32,
                /// The network prefix.
                ///
                /// Constructing values with a IPv4 prefix larger than 32 will result in
                /// an error when constructing a value.
                pub prefix: u8,
            }
            impl ::core::fmt::Debug for Ipv4Network {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Ipv4Network")
                        .field("addr", &self.addr)
                        .field("prefix", &self.prefix)
                        .finish()
                }
            }
            /// An IPv6 network range.
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Ipv6Network {
                /// The 128-bit IPv6 address, split into the low-128 bits followed by
                /// high-128 bits.
                ///
                /// This ensures that the representation in memory is in little-endian
                /// byte order.
                pub addr: (u64, u64),
                /// The network prefix.
                ///
                /// Constructing values with a IPv6 prefix larger than 128 will result
                /// in an error when constructing a value.
                pub prefix: u8,
            }
            impl ::core::fmt::Debug for Ipv6Network {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Ipv6Network")
                        .field("addr", &self.addr)
                        .field("prefix", &self.prefix)
                        .finish()
                }
            }
            /// An IP network range, either v4 or v6.
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub enum IpNetwork {
                V4(Ipv4Network),
                V6(Ipv6Network),
            }
            impl ::core::fmt::Debug for IpNetwork {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        IpNetwork::V4(e) => {
                            f.debug_tuple("IpNetwork::V4").field(e).finish()
                        }
                        IpNetwork::V6(e) => {
                            f.debug_tuple("IpNetwork::V6").field(e).finish()
                        }
                    }
                }
            }
            /// A database value.
            ///
            /// This is opaque so that new value types can be added in the future
            /// without having to bump make breaking changes to the API here.
            #[derive(Debug)]
            #[repr(transparent)]
            pub struct Value {
                handle: _rt::Resource<Value>,
            }
            impl Value {
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
            unsafe impl _rt::WasmResource for Value {
                #[inline]
                unsafe fn drop(_handle: u32) {
                    #[cfg(not(target_arch = "wasm32"))]
                    unreachable!();
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[resource-drop]value"]
                            fn drop(_: u32);
                        }
                        drop(_handle);
                    }
                }
            }
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct Column {
                pub name: _rt::String,
                pub value: Value,
            }
            impl ::core::fmt::Debug for Column {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Column")
                        .field("name", &self.name)
                        .field("value", &self.value)
                        .finish()
                }
            }
            #[derive(serde::Deserialize, serde::Serialize)]
            pub struct Row {
                pub columns: _rt::Vec<Column>,
            }
            impl ::core::fmt::Debug for Row {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Row").field("columns", &self.columns).finish()
                }
            }
            #[derive(serde::Deserialize, serde::Serialize)]
            pub enum QueryResult {
                /// A query has completed and here are the number of rows that were
                /// modified.
                Count(u64),
                /// A single row from the query output.
                Row(Row),
            }
            impl ::core::fmt::Debug for QueryResult {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        QueryResult::Count(e) => {
                            f.debug_tuple("QueryResult::Count").field(e).finish()
                        }
                        QueryResult::Row(e) => {
                            f.debug_tuple("QueryResult::Row").field(e).finish()
                        }
                    }
                }
            }
            #[repr(C)]
            #[derive(Clone, Copy, serde::Deserialize, serde::Serialize)]
            pub struct Options {
                /// Allows the runtime to limit the number of rows returned.
                ///
                /// Setting limit > 1 means that all rows will be returned.
                pub limit: u8,
                /// Whether the runtime should keep the state in its query cache.
                ///
                /// This has no observable effects on the execution of the statement,
                /// however it may be more performant if there are lots of instances of
                /// the same workflow running on the same worker.
                pub persistent: bool,
            }
            impl ::core::fmt::Debug for Options {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("Options")
                        .field("limit", &self.limit)
                        .field("persistent", &self.persistent)
                        .finish()
                }
            }
            #[derive(Clone, serde::Deserialize, serde::Serialize)]
            pub struct ColumnDecodeError {
                pub index: _rt::String,
                pub source: _rt::String,
            }
            impl ::core::fmt::Debug for ColumnDecodeError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("ColumnDecodeError")
                        .field("index", &self.index)
                        .field("source", &self.source)
                        .finish()
                }
            }
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
            pub enum DatabaseErrorKind {
                UniqueViolation,
                ForeignKeyViolation,
                NotNullViolation,
                CheckViolation,
                Other,
            }
            impl ::core::fmt::Debug for DatabaseErrorKind {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        DatabaseErrorKind::UniqueViolation => {
                            f.debug_tuple("DatabaseErrorKind::UniqueViolation").finish()
                        }
                        DatabaseErrorKind::ForeignKeyViolation => {
                            f.debug_tuple("DatabaseErrorKind::ForeignKeyViolation")
                                .finish()
                        }
                        DatabaseErrorKind::NotNullViolation => {
                            f.debug_tuple("DatabaseErrorKind::NotNullViolation").finish()
                        }
                        DatabaseErrorKind::CheckViolation => {
                            f.debug_tuple("DatabaseErrorKind::CheckViolation").finish()
                        }
                        DatabaseErrorKind::Other => {
                            f.debug_tuple("DatabaseErrorKind::Other").finish()
                        }
                    }
                }
            }
            impl DatabaseErrorKind {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> DatabaseErrorKind {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => DatabaseErrorKind::UniqueViolation,
                        1 => DatabaseErrorKind::ForeignKeyViolation,
                        2 => DatabaseErrorKind::NotNullViolation,
                        3 => DatabaseErrorKind::CheckViolation,
                        4 => DatabaseErrorKind::Other,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            #[derive(Clone, serde::Deserialize, serde::Serialize)]
            pub struct DatabaseError {
                pub message: _rt::String,
                pub kind: DatabaseErrorKind,
                pub code: Option<_rt::String>,
                pub constraint: Option<_rt::String>,
                pub table: Option<_rt::String>,
            }
            impl ::core::fmt::Debug for DatabaseError {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    f.debug_struct("DatabaseError")
                        .field("message", &self.message)
                        .field("kind", &self.kind)
                        .field("code", &self.code)
                        .field("constraint", &self.constraint)
                        .field("table", &self.table)
                        .finish()
                }
            }
            #[derive(Clone, serde::Deserialize, serde::Serialize)]
            pub enum Error {
                ColumnDecode(ColumnDecodeError),
                TypeNotFound(_rt::String),
                Encode(_rt::String),
                Decode(_rt::String),
                Database(DatabaseError),
                Other(_rt::String),
            }
            impl ::core::fmt::Debug for Error {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        Error::ColumnDecode(e) => {
                            f.debug_tuple("Error::ColumnDecode").field(e).finish()
                        }
                        Error::TypeNotFound(e) => {
                            f.debug_tuple("Error::TypeNotFound").field(e).finish()
                        }
                        Error::Encode(e) => {
                            f.debug_tuple("Error::Encode").field(e).finish()
                        }
                        Error::Decode(e) => {
                            f.debug_tuple("Error::Decode").field(e).finish()
                        }
                        Error::Database(e) => {
                            f.debug_tuple("Error::Database").field(e).finish()
                        }
                        Error::Other(e) => {
                            f.debug_tuple("Error::Other").field(e).finish()
                        }
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// The database system name of this type.
                ///
                /// This will not include length specifiers. The type name returned
                /// is a rough approximation of how it would be written in SQL for the
                /// database.
                pub fn name(&self) -> _rt::String {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]type-info.name"]
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
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether `self` and `other` represent mutually compatible types.
                pub fn compatible(&self, other: &TypeInfo) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]type-info.compatible"]
                            fn wit_import(_: i32, _: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            (self).handle() as i32,
                            (other).handle() as i32,
                        );
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether `self` and `other` represent exactly the same type.
                pub fn equal(&self, other: &TypeInfo) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]type-info.equal"]
                            fn wit_import(_: i32, _: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            (self).handle() as i32,
                            (other).handle() as i32,
                        );
                        _rt::bool_lift(ret as u8)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Create a clone of this type-info.
                pub fn clone(&self) -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]type-info.clone"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Serialize this type-info to json.
                ///
                /// The actual json returned by this function is not meant to be introspected.
                pub fn serialize(&self) -> Result<_rt::String, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]type-info.serialize"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l5 = *ptr0.add(4).cast::<*mut u8>();
                                    let l6 = *ptr0.add(8).cast::<usize>();
                                    let len7 = l6;
                                    let bytes7 = _rt::Vec::from_raw_parts(
                                        l5.cast(),
                                        len7,
                                        len7,
                                    );
                                    _rt::string_lift(bytes7)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Deserialize the type-info from json.
                pub fn deserialize(json: &str) -> Result<TypeInfo, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = json;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.deserialize"]
                            fn wit_import(_: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import(ptr0.cast_mut(), len0, ptr1);
                        let l2 = i32::from(*ptr1.add(0).cast::<u8>());
                        match l2 {
                            0 => {
                                let e = {
                                    let l3 = *ptr1.add(4).cast::<i32>();
                                    TypeInfo::from_handle(l3 as u32)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l4 = *ptr1.add(4).cast::<*mut u8>();
                                    let l5 = *ptr1.add(8).cast::<usize>();
                                    let len6 = l5;
                                    let bytes6 = _rt::Vec::from_raw_parts(
                                        l4.cast(),
                                        len6,
                                        len6,
                                    );
                                    _rt::string_lift(bytes6)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                /// Attempt to create a type directly from a name.
                ///
                /// This provides no guarantee that there is actually a type with that
                /// name within the database. Attempting to use a type that doesn't
                /// exist will result in a failure when making a query.
                ///
                /// This returns an error if there is no type with the provided name
                /// within the database.
                /// with-name: static func(name: string) -> result<type-info>;
                /// Attempt to create an array of the named type.
                ///
                /// This provides no guarantee that there is actually a type with that
                /// name within the database. Attempting to use a type that doesn't
                /// exist will result in a failure when making a query.
                /// with-array-of: static func(name: string) -> result<type-info>;
                pub fn boolean() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.boolean"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float4() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.float4"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float8() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.float8"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int1() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int1"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int2() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int2"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int4() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int4"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int8() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int8"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn text() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.text"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn bytea() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.bytea"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamptz() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.timestamptz"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamp() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.timestamp"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn uuid() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.uuid"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn jsonb() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.jsonb"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn inet() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.inet"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn boolean_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.boolean-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float4_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.float4-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float8_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.float8-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int1_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int1-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int2_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int2-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int4_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int4-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int8_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.int8-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn text_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.text-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn bytea_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.bytea-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamptz_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.timestamptz-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamp_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.timestamp-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn uuid_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.uuid-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn jsonb_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.jsonb-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl TypeInfo {
                #[allow(unused_unsafe, clippy::all)]
                pub fn inet_array() -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]type-info.inet-array"]
                            fn wit_import() -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import() -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import();
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Whether this value is NULL.
                ///
                /// If this is true then all of the `as-*` methods will return none.
                pub fn is_null(&self) -> bool {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.is-null"]
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
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// The type of this value.
                pub fn type_info(&self) -> TypeInfo {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.type-info"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        TypeInfo::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Create a clone of this value.
                pub fn clone(&self) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.clone"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((self).handle() as i32);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Serialize this type-info to json.
                ///
                /// The actual json returned by this function is not meant to be introspected.
                pub fn serialize(&self) -> Result<_rt::String, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.serialize"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l5 = *ptr0.add(4).cast::<*mut u8>();
                                    let l6 = *ptr0.add(8).cast::<usize>();
                                    let len7 = l6;
                                    let bytes7 = _rt::Vec::from_raw_parts(
                                        l5.cast(),
                                        len7,
                                        len7,
                                    );
                                    _rt::string_lift(bytes7)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Deserialize the type-info from json.
                pub fn deserialize(json: &str) -> Result<Value, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec0 = json;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        let ptr1 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.deserialize"]
                            fn wit_import(_: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import(ptr0.cast_mut(), len0, ptr1);
                        let l2 = i32::from(*ptr1.add(0).cast::<u8>());
                        match l2 {
                            0 => {
                                let e = {
                                    let l3 = *ptr1.add(4).cast::<i32>();
                                    Value::from_handle(l3 as u32)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l4 = *ptr1.add(4).cast::<*mut u8>();
                                    let l5 = *ptr1.add(8).cast::<usize>();
                                    let len6 = l5;
                                    let bytes6 = _rt::Vec::from_raw_parts(
                                        l4.cast(),
                                        len6,
                                        len6,
                                    );
                                    _rt::string_lift(bytes6)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_boolean(&self) -> Option<bool> {
                    unsafe {
                        #[repr(align(1))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 2]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 2],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-boolean"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = i32::from(*ptr0.add(1).cast::<u8>());
                                    _rt::bool_lift(l2 as u8)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_float4(&self) -> Option<f32> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-float4"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<f32>();
                                    l2
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_float8(&self) -> Option<f64> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 16],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-float8"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<f64>();
                                    l2
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int1(&self) -> Option<i8> {
                    unsafe {
                        #[repr(align(1))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 2]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 2],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int1"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = i32::from(*ptr0.add(1).cast::<i8>());
                                    l2 as i8
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int2(&self) -> Option<i16> {
                    unsafe {
                        #[repr(align(2))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 4]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 4],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int2"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = i32::from(*ptr0.add(2).cast::<i16>());
                                    l2 as i16
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int4(&self) -> Option<i32> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 8]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 8],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int4"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<i32>();
                                    l2
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int8(&self) -> Option<i64> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 16]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 16],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int8"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<i64>();
                                    l2
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_text(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-text"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_bytea(&self) -> Option<_rt::Vec<u8>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-bytea"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_timestamptz(&self) -> Option<Timestamptz> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 24],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-timestamptz"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<i64>();
                                    let l3 = *ptr0.add(16).cast::<i32>();
                                    let l4 = *ptr0.add(20).cast::<i32>();
                                    Timestamptz {
                                        seconds: l2,
                                        subsec_nanos: l3 as u32,
                                        offset: l4,
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_timestamp(&self) -> Option<Timestamp> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 24],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-timestamp"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<i64>();
                                    let l3 = *ptr0.add(16).cast::<i32>();
                                    Timestamp {
                                        seconds: l2,
                                        subsec_nanos: l3 as u32,
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_uuid(&self) -> Option<Uuid> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 24]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 24],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-uuid"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(8).cast::<i64>();
                                    let l3 = *ptr0.add(16).cast::<i64>();
                                    Uuid {
                                        hi: l2 as u64,
                                        lo: l3 as u64,
                                    }
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Note that this function works for both json and jsonb types.
                pub fn as_json(&self) -> Option<_rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-json"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    let bytes4 = _rt::Vec::from_raw_parts(
                                        l2.cast(),
                                        len4,
                                        len4,
                                    );
                                    _rt::string_lift(bytes4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_inet(&self) -> Option<IpNetwork> {
                    unsafe {
                        #[repr(align(8))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 40]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 40],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-inet"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = i32::from(*ptr0.add(8).cast::<u8>());
                                    let v8 = match l2 {
                                        0 => {
                                            let e8 = {
                                                let l3 = *ptr0.add(16).cast::<i32>();
                                                let l4 = i32::from(*ptr0.add(20).cast::<u8>());
                                                Ipv4Network {
                                                    addr: l3 as u32,
                                                    prefix: l4 as u8,
                                                }
                                            };
                                            IpNetwork::V4(e8)
                                        }
                                        n => {
                                            debug_assert_eq!(n, 1, "invalid enum discriminant");
                                            let e8 = {
                                                let l5 = *ptr0.add(16).cast::<i64>();
                                                let l6 = *ptr0.add(24).cast::<i64>();
                                                let l7 = i32::from(*ptr0.add(32).cast::<u8>());
                                                Ipv6Network {
                                                    addr: (l5 as u64, l6 as u64),
                                                    prefix: l7 as u8,
                                                }
                                            };
                                            IpNetwork::V6(e8)
                                        }
                                    };
                                    v8
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_boolean_array(&self) -> Option<_rt::Vec<bool>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-boolean-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base5 = l2;
                                    let len5 = l3;
                                    let mut result5 = _rt::Vec::with_capacity(len5);
                                    for i in 0..len5 {
                                        let base = base5.add(i * 1);
                                        let e5 = {
                                            let l4 = i32::from(*base.add(0).cast::<u8>());
                                            _rt::bool_lift(l4 as u8)
                                        };
                                        result5.push(e5);
                                    }
                                    _rt::cabi_dealloc(base5, len5 * 1, 1);
                                    result5
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_float4_array(&self) -> Option<_rt::Vec<f32>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-float4-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_float8_array(&self) -> Option<_rt::Vec<f64>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-float8-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int1_array(&self) -> Option<_rt::Vec<i8>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int1-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int2_array(&self) -> Option<_rt::Vec<i16>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int2-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int4_array(&self) -> Option<_rt::Vec<i32>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int4-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_int8_array(&self) -> Option<_rt::Vec<i64>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-int8-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_text_array(&self) -> Option<_rt::Vec<_rt::String>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-text-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base7 = l2;
                                    let len7 = l3;
                                    let mut result7 = _rt::Vec::with_capacity(len7);
                                    for i in 0..len7 {
                                        let base = base7.add(i * 8);
                                        let e7 = {
                                            let l4 = *base.add(0).cast::<*mut u8>();
                                            let l5 = *base.add(4).cast::<usize>();
                                            let len6 = l5;
                                            let bytes6 = _rt::Vec::from_raw_parts(
                                                l4.cast(),
                                                len6,
                                                len6,
                                            );
                                            _rt::string_lift(bytes6)
                                        };
                                        result7.push(e7);
                                    }
                                    _rt::cabi_dealloc(base7, len7 * 8, 4);
                                    result7
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_bytea_array(&self) -> Option<_rt::Vec<_rt::Vec<u8>>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-bytea-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base7 = l2;
                                    let len7 = l3;
                                    let mut result7 = _rt::Vec::with_capacity(len7);
                                    for i in 0..len7 {
                                        let base = base7.add(i * 8);
                                        let e7 = {
                                            let l4 = *base.add(0).cast::<*mut u8>();
                                            let l5 = *base.add(4).cast::<usize>();
                                            let len6 = l5;
                                            _rt::Vec::from_raw_parts(l4.cast(), len6, len6)
                                        };
                                        result7.push(e7);
                                    }
                                    _rt::cabi_dealloc(base7, len7 * 8, 4);
                                    result7
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_timestamptz_array(&self) -> Option<_rt::Vec<Timestamptz>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-timestamptz-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_timestamp_array(&self) -> Option<_rt::Vec<Timestamp>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-timestamp-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_uuid_array(&self) -> Option<_rt::Vec<Uuid>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-uuid-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let len4 = l3;
                                    _rt::Vec::from_raw_parts(l2.cast(), len4, len4)
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_json_array(&self) -> Option<_rt::Vec<_rt::String>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-json-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base7 = l2;
                                    let len7 = l3;
                                    let mut result7 = _rt::Vec::with_capacity(len7);
                                    for i in 0..len7 {
                                        let base = base7.add(i * 8);
                                        let e7 = {
                                            let l4 = *base.add(0).cast::<*mut u8>();
                                            let l5 = *base.add(4).cast::<usize>();
                                            let len6 = l5;
                                            let bytes6 = _rt::Vec::from_raw_parts(
                                                l4.cast(),
                                                len6,
                                                len6,
                                            );
                                            _rt::string_lift(bytes6)
                                        };
                                        result7.push(e7);
                                    }
                                    _rt::cabi_dealloc(base7, len7 * 8, 4);
                                    result7
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn as_inet_array(&self) -> Option<_rt::Vec<IpNetwork>> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[method]value.as-inet-array"]
                            fn wit_import(_: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import((self).handle() as i32, ptr0);
                        let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                        match l1 {
                            0 => None,
                            1 => {
                                let e = {
                                    let l2 = *ptr0.add(4).cast::<*mut u8>();
                                    let l3 = *ptr0.add(8).cast::<usize>();
                                    let base11 = l2;
                                    let len11 = l3;
                                    let mut result11 = _rt::Vec::with_capacity(len11);
                                    for i in 0..len11 {
                                        let base = base11.add(i * 32);
                                        let e11 = {
                                            let l4 = i32::from(*base.add(0).cast::<u8>());
                                            let v10 = match l4 {
                                                0 => {
                                                    let e10 = {
                                                        let l5 = *base.add(8).cast::<i32>();
                                                        let l6 = i32::from(*base.add(12).cast::<u8>());
                                                        Ipv4Network {
                                                            addr: l5 as u32,
                                                            prefix: l6 as u8,
                                                        }
                                                    };
                                                    IpNetwork::V4(e10)
                                                }
                                                n => {
                                                    debug_assert_eq!(n, 1, "invalid enum discriminant");
                                                    let e10 = {
                                                        let l7 = *base.add(8).cast::<i64>();
                                                        let l8 = *base.add(16).cast::<i64>();
                                                        let l9 = i32::from(*base.add(24).cast::<u8>());
                                                        Ipv6Network {
                                                            addr: (l7 as u64, l8 as u64),
                                                            prefix: l9 as u8,
                                                        }
                                                    };
                                                    IpNetwork::V6(e10)
                                                }
                                            };
                                            v10
                                        };
                                        result11.push(e11);
                                    }
                                    _rt::cabi_dealloc(base11, len11 * 32, 8);
                                    result11
                                };
                                Some(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                /// Create a null value with the provided type info.
                pub fn null(tyinfo: TypeInfo) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.null"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import((&tyinfo).take_handle() as i32);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn boolean(value: bool) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.boolean"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            match &value {
                                true => 1,
                                false => 0,
                            },
                        );
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float4(value: f32) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.float4"]
                            fn wit_import(_: f32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f32(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float8(value: f64) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.float8"]
                            fn wit_import(_: f64) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: f64) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_f64(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int1(value: i8) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int1"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_i32(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int2(value: i16) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int2"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_i32(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int4(value: i32) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int4"]
                            fn wit_import(_: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_i32(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int8(value: i64) -> Value {
                    unsafe {
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int8"]
                            fn wit_import(_: i64) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i64) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_i64(&value));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn text(value: &str) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.text"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn bytea(value: &[u8]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.bytea"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamptz(value: Timestamptz) -> Value {
                    unsafe {
                        let Timestamptz {
                            seconds: seconds0,
                            subsec_nanos: subsec_nanos0,
                            offset: offset0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.timestamptz"]
                            fn wit_import(_: i64, _: i32, _: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i64, _: i32, _: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            _rt::as_i64(seconds0),
                            _rt::as_i32(subsec_nanos0),
                            _rt::as_i32(offset0),
                        );
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamp(value: Timestamp) -> Value {
                    unsafe {
                        let Timestamp {
                            seconds: seconds0,
                            subsec_nanos: subsec_nanos0,
                        } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.timestamp"]
                            fn wit_import(_: i64, _: i32) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i64, _: i32) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(
                            _rt::as_i64(seconds0),
                            _rt::as_i32(subsec_nanos0),
                        );
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn uuid(value: Uuid) -> Value {
                    unsafe {
                        let Uuid { hi: hi0, lo: lo0 } = value;
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.uuid"]
                            fn wit_import(_: i64, _: i64) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i64, _: i64) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(_rt::as_i64(hi0), _rt::as_i64(lo0));
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn jsonb(value: &str) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.jsonb"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn inet(value: IpNetwork) -> Result<Value, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let (result3_0, result3_1, result3_2, result3_3) = match value {
                            IpNetwork::V4(e) => {
                                let Ipv4Network { addr: addr0, prefix: prefix0 } = e;
                                (
                                    0i32,
                                    i64::from(_rt::as_i32(addr0)),
                                    i64::from(_rt::as_i32(prefix0)),
                                    0i32,
                                )
                            }
                            IpNetwork::V6(e) => {
                                let Ipv6Network { addr: addr1, prefix: prefix1 } = e;
                                let (t2_0, t2_1) = addr1;
                                (
                                    1i32,
                                    _rt::as_i64(t2_0),
                                    _rt::as_i64(t2_1),
                                    _rt::as_i32(prefix1),
                                )
                            }
                        };
                        let ptr4 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.inet"]
                            fn wit_import(_: i32, _: i64, _: i64, _: i32, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: i32, _: i64, _: i64, _: i32, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import(result3_0, result3_1, result3_2, result3_3, ptr4);
                        let l5 = i32::from(*ptr4.add(0).cast::<u8>());
                        match l5 {
                            0 => {
                                let e = {
                                    let l6 = *ptr4.add(4).cast::<i32>();
                                    Value::from_handle(l6 as u32)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l7 = *ptr4.add(4).cast::<*mut u8>();
                                    let l8 = *ptr4.add(8).cast::<usize>();
                                    let len9 = l8;
                                    let bytes9 = _rt::Vec::from_raw_parts(
                                        l7.cast(),
                                        len9,
                                        len9,
                                    );
                                    _rt::string_lift(bytes9)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn boolean_array(value: &[bool]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let len0 = vec0.len();
                        let layout0 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec0.len() * 1,
                            1,
                        );
                        let result0 = if layout0.size() != 0 {
                            let ptr = _rt::alloc::alloc(layout0).cast::<u8>();
                            if ptr.is_null() {
                                _rt::alloc::handle_alloc_error(layout0);
                            }
                            ptr
                        } else {
                            { ::core::ptr::null_mut() }
                        };
                        for (i, e) in vec0.into_iter().enumerate() {
                            let base = result0.add(i * 1);
                            {
                                *base.add(0).cast::<u8>() = (match e {
                                    true => 1,
                                    false => 0,
                                }) as u8;
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.boolean-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(result0, len0);
                        if layout0.size() != 0 {
                            _rt::alloc::dealloc(result0.cast(), layout0);
                        }
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float4_array(value: &[f32]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.float4-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn float8_array(value: &[f64]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.float8-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int1_array(value: &[i8]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int1-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int2_array(value: &[i16]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int2-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int4_array(value: &[i32]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int4-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn int8_array(value: &[i64]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.int8-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn text_array(value: &[&str]) -> Value {
                    unsafe {
                        let vec1 = value;
                        let len1 = vec1.len();
                        let layout1 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec1.len() * 8,
                            4,
                        );
                        let result1 = if layout1.size() != 0 {
                            let ptr = _rt::alloc::alloc(layout1).cast::<u8>();
                            if ptr.is_null() {
                                _rt::alloc::handle_alloc_error(layout1);
                            }
                            ptr
                        } else {
                            { ::core::ptr::null_mut() }
                        };
                        for (i, e) in vec1.into_iter().enumerate() {
                            let base = result1.add(i * 8);
                            {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                *base.add(4).cast::<usize>() = len0;
                                *base.add(0).cast::<*mut u8>() = ptr0.cast_mut();
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.text-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(result1, len1);
                        if layout1.size() != 0 {
                            _rt::alloc::dealloc(result1.cast(), layout1);
                        }
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn bytea_array(value: &[&[u8]]) -> Value {
                    unsafe {
                        let vec1 = value;
                        let len1 = vec1.len();
                        let layout1 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec1.len() * 8,
                            4,
                        );
                        let result1 = if layout1.size() != 0 {
                            let ptr = _rt::alloc::alloc(layout1).cast::<u8>();
                            if ptr.is_null() {
                                _rt::alloc::handle_alloc_error(layout1);
                            }
                            ptr
                        } else {
                            { ::core::ptr::null_mut() }
                        };
                        for (i, e) in vec1.into_iter().enumerate() {
                            let base = result1.add(i * 8);
                            {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                *base.add(4).cast::<usize>() = len0;
                                *base.add(0).cast::<*mut u8>() = ptr0.cast_mut();
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.bytea-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(result1, len1);
                        if layout1.size() != 0 {
                            _rt::alloc::dealloc(result1.cast(), layout1);
                        }
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamptz_array(value: &[Timestamptz]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.timestamptz-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn timestamp_array(value: &[Timestamp]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.timestamp-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn uuid_array(value: &[Uuid]) -> Value {
                    unsafe {
                        let vec0 = value;
                        let ptr0 = vec0.as_ptr().cast::<u8>();
                        let len0 = vec0.len();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.uuid-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(ptr0.cast_mut(), len0);
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn jsonb_array(value: &[&str]) -> Value {
                    unsafe {
                        let vec1 = value;
                        let len1 = vec1.len();
                        let layout1 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec1.len() * 8,
                            4,
                        );
                        let result1 = if layout1.size() != 0 {
                            let ptr = _rt::alloc::alloc(layout1).cast::<u8>();
                            if ptr.is_null() {
                                _rt::alloc::handle_alloc_error(layout1);
                            }
                            ptr
                        } else {
                            { ::core::ptr::null_mut() }
                        };
                        for (i, e) in vec1.into_iter().enumerate() {
                            let base = result1.add(i * 8);
                            {
                                let vec0 = e;
                                let ptr0 = vec0.as_ptr().cast::<u8>();
                                let len0 = vec0.len();
                                *base.add(4).cast::<usize>() = len0;
                                *base.add(0).cast::<*mut u8>() = ptr0.cast_mut();
                            }
                        }
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.jsonb-array"]
                            fn wit_import(_: *mut u8, _: usize) -> i32;
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize) -> i32 {
                            unreachable!()
                        }
                        let ret = wit_import(result1, len1);
                        if layout1.size() != 0 {
                            _rt::alloc::dealloc(result1.cast(), layout1);
                        }
                        Value::from_handle(ret as u32)
                    }
                }
            }
            impl Value {
                #[allow(unused_unsafe, clippy::all)]
                pub fn inet_array(value: &[IpNetwork]) -> Result<Value, _rt::String> {
                    unsafe {
                        #[repr(align(4))]
                        struct RetArea([::core::mem::MaybeUninit<u8>; 12]);
                        let mut ret_area = RetArea(
                            [::core::mem::MaybeUninit::uninit(); 12],
                        );
                        let vec3 = value;
                        let len3 = vec3.len();
                        let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                            vec3.len() * 32,
                            8,
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
                            let base = result3.add(i * 32);
                            {
                                match e {
                                    IpNetwork::V4(e) => {
                                        *base.add(0).cast::<u8>() = (0i32) as u8;
                                        let Ipv4Network { addr: addr0, prefix: prefix0 } = e;
                                        *base.add(8).cast::<i32>() = _rt::as_i32(addr0);
                                        *base.add(12).cast::<u8>() = (_rt::as_i32(prefix0)) as u8;
                                    }
                                    IpNetwork::V6(e) => {
                                        *base.add(0).cast::<u8>() = (1i32) as u8;
                                        let Ipv6Network { addr: addr1, prefix: prefix1 } = e;
                                        let (t2_0, t2_1) = addr1;
                                        *base.add(8).cast::<i64>() = _rt::as_i64(t2_0);
                                        *base.add(16).cast::<i64>() = _rt::as_i64(t2_1);
                                        *base.add(24).cast::<u8>() = (_rt::as_i32(prefix1)) as u8;
                                    }
                                }
                            }
                        }
                        let ptr4 = ret_area.0.as_mut_ptr().cast::<u8>();
                        #[cfg(target_arch = "wasm32")]
                        #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                        extern "C" {
                            #[link_name = "[static]value.inet-array"]
                            fn wit_import(_: *mut u8, _: usize, _: *mut u8);
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        fn wit_import(_: *mut u8, _: usize, _: *mut u8) {
                            unreachable!()
                        }
                        wit_import(result3, len3, ptr4);
                        let l5 = i32::from(*ptr4.add(0).cast::<u8>());
                        if layout3.size() != 0 {
                            _rt::alloc::dealloc(result3.cast(), layout3);
                        }
                        match l5 {
                            0 => {
                                let e = {
                                    let l6 = *ptr4.add(4).cast::<i32>();
                                    Value::from_handle(l6 as u32)
                                };
                                Ok(e)
                            }
                            1 => {
                                let e = {
                                    let l7 = *ptr4.add(4).cast::<*mut u8>();
                                    let l8 = *ptr4.add(8).cast::<usize>();
                                    let len9 = l8;
                                    let bytes9 = _rt::Vec::from_raw_parts(
                                        l7.cast(),
                                        len9,
                                        len9,
                                    );
                                    _rt::string_lift(bytes9)
                                };
                                Err(e)
                            }
                            _ => _rt::invalid_enum_discriminant(),
                        }
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Make a query to the database.
            ///
            /// This function will start the query. Then rows and results can be
            /// fetched by calling `fetch` until it returns none.
            ///
            /// Calling query again while there are still results from a previous query
            /// results in the remaining unconsumed rows being discarded.
            pub fn query(sql: &str, params: _rt::Vec<Value>, options: Options) {
                unsafe {
                    let vec0 = sql;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    let vec1 = &params;
                    let len1 = vec1.len();
                    let layout1 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec1.len() * 4,
                        4,
                    );
                    let result1 = if layout1.size() != 0 {
                        let ptr = _rt::alloc::alloc(layout1).cast::<u8>();
                        if ptr.is_null() {
                            _rt::alloc::handle_alloc_error(layout1);
                        }
                        ptr
                    } else {
                        { ::core::ptr::null_mut() }
                    };
                    for (i, e) in vec1.into_iter().enumerate() {
                        let base = result1.add(i * 4);
                        {
                            *base.add(0).cast::<i32>() = (e).take_handle() as i32;
                        }
                    }
                    let Options { limit: limit2, persistent: persistent2 } = options;
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                    extern "C" {
                        #[link_name = "query"]
                        fn wit_import(
                            _: *mut u8,
                            _: usize,
                            _: *mut u8,
                            _: usize,
                            _: i32,
                            _: i32,
                        );
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(
                        _: *mut u8,
                        _: usize,
                        _: *mut u8,
                        _: usize,
                        _: i32,
                        _: i32,
                    ) {
                        unreachable!()
                    }
                    wit_import(
                        ptr0.cast_mut(),
                        len0,
                        result1,
                        len1,
                        _rt::as_i32(limit2),
                        match persistent2 {
                            true => 1,
                            false => 0,
                        },
                    );
                    if layout1.size() != 0 {
                        _rt::alloc::dealloc(result1.cast(), layout1);
                    }
                }
            }
            #[allow(unused_unsafe, clippy::all)]
            /// Fetch either a query result or a single row from the query.
            pub fn fetch() -> Option<Result<QueryResult, Error>> {
                unsafe {
                    #[repr(align(8))]
                    struct RetArea([::core::mem::MaybeUninit<u8>; 72]);
                    let mut ret_area = RetArea([::core::mem::MaybeUninit::uninit(); 72]);
                    let ptr0 = ret_area.0.as_mut_ptr().cast::<u8>();
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/sql@2.2.0")]
                    extern "C" {
                        #[link_name = "fetch"]
                        fn wit_import(_: *mut u8);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    fn wit_import(_: *mut u8) {
                        unreachable!()
                    }
                    wit_import(ptr0);
                    let l1 = i32::from(*ptr0.add(0).cast::<u8>());
                    match l1 {
                        0 => None,
                        1 => {
                            let e = {
                                let l2 = i32::from(*ptr0.add(8).cast::<u8>());
                                match l2 {
                                    0 => {
                                        let e = {
                                            let l3 = i32::from(*ptr0.add(16).cast::<u8>());
                                            let v12 = match l3 {
                                                0 => {
                                                    let e12 = {
                                                        let l4 = *ptr0.add(24).cast::<i64>();
                                                        l4 as u64
                                                    };
                                                    QueryResult::Count(e12)
                                                }
                                                n => {
                                                    debug_assert_eq!(n, 1, "invalid enum discriminant");
                                                    let e12 = {
                                                        let l5 = *ptr0.add(24).cast::<*mut u8>();
                                                        let l6 = *ptr0.add(28).cast::<usize>();
                                                        let base11 = l5;
                                                        let len11 = l6;
                                                        let mut result11 = _rt::Vec::with_capacity(len11);
                                                        for i in 0..len11 {
                                                            let base = base11.add(i * 12);
                                                            let e11 = {
                                                                let l7 = *base.add(0).cast::<*mut u8>();
                                                                let l8 = *base.add(4).cast::<usize>();
                                                                let len9 = l8;
                                                                let bytes9 = _rt::Vec::from_raw_parts(
                                                                    l7.cast(),
                                                                    len9,
                                                                    len9,
                                                                );
                                                                let l10 = *base.add(8).cast::<i32>();
                                                                Column {
                                                                    name: _rt::string_lift(bytes9),
                                                                    value: Value::from_handle(l10 as u32),
                                                                }
                                                            };
                                                            result11.push(e11);
                                                        }
                                                        _rt::cabi_dealloc(base11, len11 * 12, 4);
                                                        Row { columns: result11 }
                                                    };
                                                    QueryResult::Row(e12)
                                                }
                                            };
                                            v12
                                        };
                                        Ok(e)
                                    }
                                    1 => {
                                        let e = {
                                            let l13 = i32::from(*ptr0.add(16).cast::<u8>());
                                            let v48 = match l13 {
                                                0 => {
                                                    let e48 = {
                                                        let l14 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l15 = *ptr0.add(24).cast::<usize>();
                                                        let len16 = l15;
                                                        let bytes16 = _rt::Vec::from_raw_parts(
                                                            l14.cast(),
                                                            len16,
                                                            len16,
                                                        );
                                                        let l17 = *ptr0.add(28).cast::<*mut u8>();
                                                        let l18 = *ptr0.add(32).cast::<usize>();
                                                        let len19 = l18;
                                                        let bytes19 = _rt::Vec::from_raw_parts(
                                                            l17.cast(),
                                                            len19,
                                                            len19,
                                                        );
                                                        ColumnDecodeError {
                                                            index: _rt::string_lift(bytes16),
                                                            source: _rt::string_lift(bytes19),
                                                        }
                                                    };
                                                    Error::ColumnDecode(e48)
                                                }
                                                1 => {
                                                    let e48 = {
                                                        let l20 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l21 = *ptr0.add(24).cast::<usize>();
                                                        let len22 = l21;
                                                        let bytes22 = _rt::Vec::from_raw_parts(
                                                            l20.cast(),
                                                            len22,
                                                            len22,
                                                        );
                                                        _rt::string_lift(bytes22)
                                                    };
                                                    Error::TypeNotFound(e48)
                                                }
                                                2 => {
                                                    let e48 = {
                                                        let l23 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l24 = *ptr0.add(24).cast::<usize>();
                                                        let len25 = l24;
                                                        let bytes25 = _rt::Vec::from_raw_parts(
                                                            l23.cast(),
                                                            len25,
                                                            len25,
                                                        );
                                                        _rt::string_lift(bytes25)
                                                    };
                                                    Error::Encode(e48)
                                                }
                                                3 => {
                                                    let e48 = {
                                                        let l26 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l27 = *ptr0.add(24).cast::<usize>();
                                                        let len28 = l27;
                                                        let bytes28 = _rt::Vec::from_raw_parts(
                                                            l26.cast(),
                                                            len28,
                                                            len28,
                                                        );
                                                        _rt::string_lift(bytes28)
                                                    };
                                                    Error::Decode(e48)
                                                }
                                                4 => {
                                                    let e48 = {
                                                        let l29 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l30 = *ptr0.add(24).cast::<usize>();
                                                        let len31 = l30;
                                                        let bytes31 = _rt::Vec::from_raw_parts(
                                                            l29.cast(),
                                                            len31,
                                                            len31,
                                                        );
                                                        let l32 = i32::from(*ptr0.add(28).cast::<u8>());
                                                        let l33 = i32::from(*ptr0.add(32).cast::<u8>());
                                                        let l37 = i32::from(*ptr0.add(44).cast::<u8>());
                                                        let l41 = i32::from(*ptr0.add(56).cast::<u8>());
                                                        DatabaseError {
                                                            message: _rt::string_lift(bytes31),
                                                            kind: DatabaseErrorKind::_lift(l32 as u8),
                                                            code: match l33 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l34 = *ptr0.add(36).cast::<*mut u8>();
                                                                        let l35 = *ptr0.add(40).cast::<usize>();
                                                                        let len36 = l35;
                                                                        let bytes36 = _rt::Vec::from_raw_parts(
                                                                            l34.cast(),
                                                                            len36,
                                                                            len36,
                                                                        );
                                                                        _rt::string_lift(bytes36)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                            constraint: match l37 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l38 = *ptr0.add(48).cast::<*mut u8>();
                                                                        let l39 = *ptr0.add(52).cast::<usize>();
                                                                        let len40 = l39;
                                                                        let bytes40 = _rt::Vec::from_raw_parts(
                                                                            l38.cast(),
                                                                            len40,
                                                                            len40,
                                                                        );
                                                                        _rt::string_lift(bytes40)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                            table: match l41 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l42 = *ptr0.add(60).cast::<*mut u8>();
                                                                        let l43 = *ptr0.add(64).cast::<usize>();
                                                                        let len44 = l43;
                                                                        let bytes44 = _rt::Vec::from_raw_parts(
                                                                            l42.cast(),
                                                                            len44,
                                                                            len44,
                                                                        );
                                                                        _rt::string_lift(bytes44)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                        }
                                                    };
                                                    Error::Database(e48)
                                                }
                                                n => {
                                                    debug_assert_eq!(n, 5, "invalid enum discriminant");
                                                    let e48 = {
                                                        let l45 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l46 = *ptr0.add(24).cast::<usize>();
                                                        let len47 = l46;
                                                        let bytes47 = _rt::Vec::from_raw_parts(
                                                            l45.cast(),
                                                            len47,
                                                            len47,
                                                        );
                                                        _rt::string_lift(bytes47)
                                                    };
                                                    Error::Other(e48)
                                                }
                                            };
                                            v48
                                        };
                                        Err(e)
                                    }
                                    _ => _rt::invalid_enum_discriminant(),
                                }
                            };
                            Some(e)
                        }
                        _ => _rt::invalid_enum_discriminant(),
                    }
                }
            }
        }
    }
}
mod _rt {
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
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
    pub unsafe fn string_lift(bytes: Vec<u8>) -> String {
        if cfg!(debug_assertions) {
            String::from_utf8(bytes).unwrap()
        } else {
            String::from_utf8_unchecked(bytes)
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
    pub unsafe fn invalid_enum_discriminant<T>() -> T {
        if cfg!(debug_assertions) {
            panic!("invalid enum discriminant")
        } else {
            core::hint::unreachable_unchecked()
        }
    }
    pub unsafe fn cabi_dealloc(ptr: *mut u8, size: usize, align: usize) {
        if size == 0 {
            return;
        }
        let layout = alloc::Layout::from_size_align_unchecked(size, align);
        alloc::dealloc(ptr, layout);
    }
    pub fn as_f32<T: AsF32>(t: T) -> f32 {
        t.as_f32()
    }
    pub trait AsF32 {
        fn as_f32(self) -> f32;
    }
    impl<'a, T: Copy + AsF32> AsF32 for &'a T {
        fn as_f32(self) -> f32 {
            (*self).as_f32()
        }
    }
    impl AsF32 for f32 {
        #[inline]
        fn as_f32(self) -> f32 {
            self as f32
        }
    }
    pub fn as_f64<T: AsF64>(t: T) -> f64 {
        t.as_f64()
    }
    pub trait AsF64 {
        fn as_f64(self) -> f64;
    }
    impl<'a, T: Copy + AsF64> AsF64 for &'a T {
        fn as_f64(self) -> f64 {
            (*self).as_f64()
        }
    }
    impl AsF64 for f64 {
        #[inline]
        fn as_f64(self) -> f64 {
            self as f64
        }
    }
    pub fn as_i32<T: AsI32>(t: T) -> i32 {
        t.as_i32()
    }
    pub trait AsI32 {
        fn as_i32(self) -> i32;
    }
    impl<'a, T: Copy + AsI32> AsI32 for &'a T {
        fn as_i32(self) -> i32 {
            (*self).as_i32()
        }
    }
    impl AsI32 for i32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u32 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u16 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for i8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for u8 {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for char {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
        }
    }
    impl AsI32 for usize {
        #[inline]
        fn as_i32(self) -> i32 {
            self as i32
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
    pub use alloc_crate::alloc;
    extern crate alloc as alloc_crate;
}
#[cfg(target_arch = "wasm32")]
#[link_section = "component-type:wit-bindgen:0.30.0:import-sql:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 4731] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xfa#\x01A\x02\x01A\x02\
\x01B\xf6\x01\x04\0\x09type-info\x03\x01\x01r\x02\x07secondsx\x0csubsec-nanosy\x04\
\0\x09timestamp\x03\0\x01\x01r\x03\x07secondsx\x0csubsec-nanosy\x06offsetz\x04\0\
\x0btimestamptz\x03\0\x03\x01r\x02\x02hiw\x02low\x04\0\x04uuid\x03\0\x05\x01r\x02\
\x04addry\x06prefix}\x04\0\x0cipv4-network\x03\0\x07\x01o\x02ww\x01r\x02\x04addr\
\x09\x06prefix}\x04\0\x0cipv6-network\x03\0\x0a\x01q\x02\x02v4\x01\x08\0\x02v6\x01\
\x0b\0\x04\0\x0aip-network\x03\0\x0c\x04\0\x05value\x03\x01\x01i\x0e\x01r\x02\x04\
names\x05value\x0f\x04\0\x06column\x03\0\x10\x01p\x11\x01r\x01\x07columns\x12\x04\
\0\x03row\x03\0\x13\x01q\x02\x05count\x01w\0\x03row\x01\x14\0\x04\0\x0cquery-res\
ult\x03\0\x15\x01r\x02\x05limit}\x0apersistent\x7f\x04\0\x07options\x03\0\x17\x01\
r\x02\x05indexs\x06sources\x04\0\x13column-decode-error\x03\0\x19\x01m\x05\x10un\
ique-violation\x15foreign-key-violation\x12not-null-violation\x0fcheck-violation\
\x05other\x04\0\x13database-error-kind\x03\0\x1b\x01ks\x01r\x05\x07messages\x04k\
ind\x1c\x04code\x1d\x0aconstraint\x1d\x05table\x1d\x04\0\x0edatabase-error\x03\0\
\x1e\x01q\x06\x0dcolumn-decode\x01\x1a\0\x0etype-not-found\x01s\0\x06encode\x01s\
\0\x06decode\x01s\0\x08database\x01\x1f\0\x05other\x01s\0\x04\0\x05error\x03\0\x20\
\x01h\0\x01@\x01\x04self\"\0s\x04\0\x16[method]type-info.name\x01#\x01@\x02\x04s\
elf\"\x05other\"\0\x7f\x04\0\x1c[method]type-info.compatible\x01$\x04\0\x17[meth\
od]type-info.equal\x01$\x01i\0\x01@\x01\x04self\"\0%\x04\0\x17[method]type-info.\
clone\x01&\x01j\x01s\x01s\x01@\x01\x04self\"\0'\x04\0\x1b[method]type-info.seria\
lize\x01(\x01j\x01%\x01s\x01@\x01\x04jsons\0)\x04\0\x1d[static]type-info.deseria\
lize\x01*\x01@\0\0%\x04\0\x19[static]type-info.boolean\x01+\x04\0\x18[static]typ\
e-info.float4\x01+\x04\0\x18[static]type-info.float8\x01+\x04\0\x16[static]type-\
info.int1\x01+\x04\0\x16[static]type-info.int2\x01+\x04\0\x16[static]type-info.i\
nt4\x01+\x04\0\x16[static]type-info.int8\x01+\x04\0\x16[static]type-info.text\x01\
+\x04\0\x17[static]type-info.bytea\x01+\x04\0\x1d[static]type-info.timestamptz\x01\
+\x04\0\x1b[static]type-info.timestamp\x01+\x04\0\x16[static]type-info.uuid\x01+\
\x04\0\x17[static]type-info.jsonb\x01+\x04\0\x16[static]type-info.inet\x01+\x04\0\
\x1f[static]type-info.boolean-array\x01+\x04\0\x1e[static]type-info.float4-array\
\x01+\x04\0\x1e[static]type-info.float8-array\x01+\x04\0\x1c[static]type-info.in\
t1-array\x01+\x04\0\x1c[static]type-info.int2-array\x01+\x04\0\x1c[static]type-i\
nfo.int4-array\x01+\x04\0\x1c[static]type-info.int8-array\x01+\x04\0\x1c[static]\
type-info.text-array\x01+\x04\0\x1d[static]type-info.bytea-array\x01+\x04\0#[sta\
tic]type-info.timestamptz-array\x01+\x04\0![static]type-info.timestamp-array\x01\
+\x04\0\x1c[static]type-info.uuid-array\x01+\x04\0\x1d[static]type-info.jsonb-ar\
ray\x01+\x04\0\x1c[static]type-info.inet-array\x01+\x01h\x0e\x01@\x01\x04self,\0\
\x7f\x04\0\x15[method]value.is-null\x01-\x01@\x01\x04self,\0%\x04\0\x17[method]v\
alue.type-info\x01.\x01@\x01\x04self,\0\x0f\x04\0\x13[method]value.clone\x01/\x01\
@\x01\x04self,\0'\x04\0\x17[method]value.serialize\x010\x01j\x01\x0f\x01s\x01@\x01\
\x04jsons\01\x04\0\x19[static]value.deserialize\x012\x01k\x7f\x01@\x01\x04self,\0\
3\x04\0\x18[method]value.as-boolean\x014\x01kv\x01@\x01\x04self,\05\x04\0\x17[me\
thod]value.as-float4\x016\x01ku\x01@\x01\x04self,\07\x04\0\x17[method]value.as-f\
loat8\x018\x01k~\x01@\x01\x04self,\09\x04\0\x15[method]value.as-int1\x01:\x01k|\x01\
@\x01\x04self,\0;\x04\0\x15[method]value.as-int2\x01<\x01kz\x01@\x01\x04self,\0=\
\x04\0\x15[method]value.as-int4\x01>\x01kx\x01@\x01\x04self,\0?\x04\0\x15[method\
]value.as-int8\x01@\x01@\x01\x04self,\0\x1d\x04\0\x15[method]value.as-text\x01A\x01\
p}\x01k\xc2\0\x01@\x01\x04self,\0\xc3\0\x04\0\x16[method]value.as-bytea\x01D\x01\
k\x04\x01@\x01\x04self,\0\xc5\0\x04\0\x1c[method]value.as-timestamptz\x01F\x01k\x02\
\x01@\x01\x04self,\0\xc7\0\x04\0\x1a[method]value.as-timestamp\x01H\x01k\x06\x01\
@\x01\x04self,\0\xc9\0\x04\0\x15[method]value.as-uuid\x01J\x04\0\x15[method]valu\
e.as-json\x01A\x01k\x0d\x01@\x01\x04self,\0\xcb\0\x04\0\x15[method]value.as-inet\
\x01L\x01p\x7f\x01k\xcd\0\x01@\x01\x04self,\0\xce\0\x04\0\x1e[method]value.as-bo\
olean-array\x01O\x01pv\x01k\xd0\0\x01@\x01\x04self,\0\xd1\0\x04\0\x1d[method]val\
ue.as-float4-array\x01R\x01pu\x01k\xd3\0\x01@\x01\x04self,\0\xd4\0\x04\0\x1d[met\
hod]value.as-float8-array\x01U\x01p~\x01k\xd6\0\x01@\x01\x04self,\0\xd7\0\x04\0\x1b\
[method]value.as-int1-array\x01X\x01p|\x01k\xd9\0\x01@\x01\x04self,\0\xda\0\x04\0\
\x1b[method]value.as-int2-array\x01[\x01pz\x01k\xdc\0\x01@\x01\x04self,\0\xdd\0\x04\
\0\x1b[method]value.as-int4-array\x01^\x01px\x01k\xdf\0\x01@\x01\x04self,\0\xe0\0\
\x04\0\x1b[method]value.as-int8-array\x01a\x01ps\x01k\xe2\0\x01@\x01\x04self,\0\xe3\
\0\x04\0\x1b[method]value.as-text-array\x01d\x01p\xc2\0\x01k\xe5\0\x01@\x01\x04s\
elf,\0\xe6\0\x04\0\x1c[method]value.as-bytea-array\x01g\x01p\x04\x01k\xe8\0\x01@\
\x01\x04self,\0\xe9\0\x04\0\"[method]value.as-timestamptz-array\x01j\x01p\x02\x01\
k\xeb\0\x01@\x01\x04self,\0\xec\0\x04\0\x20[method]value.as-timestamp-array\x01m\
\x01p\x06\x01k\xee\0\x01@\x01\x04self,\0\xef\0\x04\0\x1b[method]value.as-uuid-ar\
ray\x01p\x04\0\x1b[method]value.as-json-array\x01d\x01p\x0d\x01k\xf1\0\x01@\x01\x04\
self,\0\xf2\0\x04\0\x1b[method]value.as-inet-array\x01s\x01@\x01\x06tyinfo%\0\x0f\
\x04\0\x12[static]value.null\x01t\x01@\x01\x05value\x7f\0\x0f\x04\0\x15[static]v\
alue.boolean\x01u\x01@\x01\x05valuev\0\x0f\x04\0\x14[static]value.float4\x01v\x01\
@\x01\x05valueu\0\x0f\x04\0\x14[static]value.float8\x01w\x01@\x01\x05value~\0\x0f\
\x04\0\x12[static]value.int1\x01x\x01@\x01\x05value|\0\x0f\x04\0\x12[static]valu\
e.int2\x01y\x01@\x01\x05valuez\0\x0f\x04\0\x12[static]value.int4\x01z\x01@\x01\x05\
valuex\0\x0f\x04\0\x12[static]value.int8\x01{\x01@\x01\x05values\0\x0f\x04\0\x12\
[static]value.text\x01|\x01@\x01\x05value\xc2\0\0\x0f\x04\0\x13[static]value.byt\
ea\x01}\x01@\x01\x05value\x04\0\x0f\x04\0\x19[static]value.timestamptz\x01~\x01@\
\x01\x05value\x02\0\x0f\x04\0\x17[static]value.timestamp\x01\x7f\x01@\x01\x05val\
ue\x06\0\x0f\x04\0\x12[static]value.uuid\x01\x80\x01\x04\0\x13[static]value.json\
b\x01|\x01@\x01\x05value\x0d\01\x04\0\x12[static]value.inet\x01\x81\x01\x01@\x01\
\x05value\xcd\0\0\x0f\x04\0\x1b[static]value.boolean-array\x01\x82\x01\x01@\x01\x05\
value\xd0\0\0\x0f\x04\0\x1a[static]value.float4-array\x01\x83\x01\x01@\x01\x05va\
lue\xd3\0\0\x0f\x04\0\x1a[static]value.float8-array\x01\x84\x01\x01@\x01\x05valu\
e\xd6\0\0\x0f\x04\0\x18[static]value.int1-array\x01\x85\x01\x01@\x01\x05value\xd9\
\0\0\x0f\x04\0\x18[static]value.int2-array\x01\x86\x01\x01@\x01\x05value\xdc\0\0\
\x0f\x04\0\x18[static]value.int4-array\x01\x87\x01\x01@\x01\x05value\xdf\0\0\x0f\
\x04\0\x18[static]value.int8-array\x01\x88\x01\x01@\x01\x05value\xe2\0\0\x0f\x04\
\0\x18[static]value.text-array\x01\x89\x01\x01@\x01\x05value\xe5\0\0\x0f\x04\0\x19\
[static]value.bytea-array\x01\x8a\x01\x01@\x01\x05value\xe8\0\0\x0f\x04\0\x1f[st\
atic]value.timestamptz-array\x01\x8b\x01\x01@\x01\x05value\xeb\0\0\x0f\x04\0\x1d\
[static]value.timestamp-array\x01\x8c\x01\x01@\x01\x05value\xee\0\0\x0f\x04\0\x18\
[static]value.uuid-array\x01\x8d\x01\x04\0\x19[static]value.jsonb-array\x01\x89\x01\
\x01@\x01\x05value\xf1\0\01\x04\0\x18[static]value.inet-array\x01\x8e\x01\x01p\x0f\
\x01@\x03\x03sqls\x06params\x8f\x01\x07options\x18\x01\0\x04\0\x05query\x01\x90\x01\
\x01j\x01\x16\x01!\x01k\x91\x01\x01@\0\0\x92\x01\x04\0\x05fetch\x01\x93\x01\x03\x01\
\x16durable:core/sql@2.2.0\x05\0\x04\x01\x1ddurable:core/import-sql@2.2.0\x04\0\x0b\
\x10\x01\0\x0aimport-sql\x03\0\0\0G\x09producers\x01\x0cprocessed-by\x02\x0dwit-\
component\x070.215.0\x10wit-bindgen-rust\x060.30.0";
#[inline(never)]
#[doc(hidden)]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
