#[allow(dead_code)]
pub mod durable {
    #[allow(dead_code)]
    pub mod core {
        #[allow(dead_code, clippy::all)]
        pub mod sql {
            #[used]
            #[doc(hidden)]
            #[cfg(target_arch = "wasm32")]
            static __FORCE_SECTION_REF: fn() = super::super::super::__link_custom_section_describing_imports;
            use super::super::super::_rt;
            #[repr(u8)]
            #[derive(Clone, Copy, Eq, PartialEq)]
            pub enum PrimitiveType {
                Boolean,
                Float4,
                Float8,
                Int1,
                Int2,
                Int4,
                Int8,
                Text,
                Bytea,
            }
            impl ::core::fmt::Debug for PrimitiveType {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        PrimitiveType::Boolean => {
                            f.debug_tuple("PrimitiveType::Boolean").finish()
                        }
                        PrimitiveType::Float4 => {
                            f.debug_tuple("PrimitiveType::Float4").finish()
                        }
                        PrimitiveType::Float8 => {
                            f.debug_tuple("PrimitiveType::Float8").finish()
                        }
                        PrimitiveType::Int1 => {
                            f.debug_tuple("PrimitiveType::Int1").finish()
                        }
                        PrimitiveType::Int2 => {
                            f.debug_tuple("PrimitiveType::Int2").finish()
                        }
                        PrimitiveType::Int4 => {
                            f.debug_tuple("PrimitiveType::Int4").finish()
                        }
                        PrimitiveType::Int8 => {
                            f.debug_tuple("PrimitiveType::Int8").finish()
                        }
                        PrimitiveType::Text => {
                            f.debug_tuple("PrimitiveType::Text").finish()
                        }
                        PrimitiveType::Bytea => {
                            f.debug_tuple("PrimitiveType::Bytea").finish()
                        }
                    }
                }
            }
            impl PrimitiveType {
                #[doc(hidden)]
                pub unsafe fn _lift(val: u8) -> PrimitiveType {
                    if !cfg!(debug_assertions) {
                        return ::core::mem::transmute(val);
                    }
                    match val {
                        0 => PrimitiveType::Boolean,
                        1 => PrimitiveType::Float4,
                        2 => PrimitiveType::Float8,
                        3 => PrimitiveType::Int1,
                        4 => PrimitiveType::Int2,
                        5 => PrimitiveType::Int4,
                        6 => PrimitiveType::Int8,
                        7 => PrimitiveType::Text,
                        8 => PrimitiveType::Bytea,
                        _ => panic!("invalid enum discriminant"),
                    }
                }
            }
            #[derive(Clone)]
            pub enum Value {
                Null(PrimitiveType),
                Boolean(bool),
                Float4(f32),
                Float8(f64),
                Int1(i8),
                Int2(i16),
                Int4(i32),
                Int8(i64),
                Text(_rt::String),
                Bytea(_rt::Vec<u8>),
            }
            impl ::core::fmt::Debug for Value {
                fn fmt(
                    &self,
                    f: &mut ::core::fmt::Formatter<'_>,
                ) -> ::core::fmt::Result {
                    match self {
                        Value::Null(e) => f.debug_tuple("Value::Null").field(e).finish(),
                        Value::Boolean(e) => {
                            f.debug_tuple("Value::Boolean").field(e).finish()
                        }
                        Value::Float4(e) => {
                            f.debug_tuple("Value::Float4").field(e).finish()
                        }
                        Value::Float8(e) => {
                            f.debug_tuple("Value::Float8").field(e).finish()
                        }
                        Value::Int1(e) => f.debug_tuple("Value::Int1").field(e).finish(),
                        Value::Int2(e) => f.debug_tuple("Value::Int2").field(e).finish(),
                        Value::Int4(e) => f.debug_tuple("Value::Int4").field(e).finish(),
                        Value::Int8(e) => f.debug_tuple("Value::Int8").field(e).finish(),
                        Value::Text(e) => f.debug_tuple("Value::Text").field(e).finish(),
                        Value::Bytea(e) => {
                            f.debug_tuple("Value::Bytea").field(e).finish()
                        }
                    }
                }
            }
            #[derive(Clone)]
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
            #[derive(Clone)]
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
            #[derive(Clone)]
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
            #[derive(Clone, Copy)]
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
            #[derive(Clone)]
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
            #[derive(Clone, Copy, Eq, PartialEq)]
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
            #[derive(Clone)]
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
            #[derive(Clone)]
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
            #[allow(unused_unsafe, clippy::all)]
            /// Make a query to the database.
            ///
            /// This function will start the query. Then rows and results can be
            /// fetched by calling `fetch` until it returns none.
            ///
            /// Calling query again while there are still results from a previous query
            /// result in the remaining unconsumed rows being discarded.
            pub fn query(sql: &str, params: &[Value], options: Options) {
                unsafe {
                    let vec0 = sql;
                    let ptr0 = vec0.as_ptr().cast::<u8>();
                    let len0 = vec0.len();
                    let vec3 = params;
                    let len3 = vec3.len();
                    let layout3 = _rt::alloc::Layout::from_size_align_unchecked(
                        vec3.len() * 16,
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
                        let base = result3.add(i * 16);
                        {
                            match e {
                                Value::Null(e) => {
                                    *base.add(0).cast::<u8>() = (0i32) as u8;
                                    *base.add(8).cast::<u8>() = (e.clone() as i32) as u8;
                                }
                                Value::Boolean(e) => {
                                    *base.add(0).cast::<u8>() = (1i32) as u8;
                                    *base.add(8).cast::<u8>() = (match e {
                                        true => 1,
                                        false => 0,
                                    }) as u8;
                                }
                                Value::Float4(e) => {
                                    *base.add(0).cast::<u8>() = (2i32) as u8;
                                    *base.add(8).cast::<f32>() = _rt::as_f32(e);
                                }
                                Value::Float8(e) => {
                                    *base.add(0).cast::<u8>() = (3i32) as u8;
                                    *base.add(8).cast::<f64>() = _rt::as_f64(e);
                                }
                                Value::Int1(e) => {
                                    *base.add(0).cast::<u8>() = (4i32) as u8;
                                    *base.add(8).cast::<u8>() = (_rt::as_i32(e)) as u8;
                                }
                                Value::Int2(e) => {
                                    *base.add(0).cast::<u8>() = (5i32) as u8;
                                    *base.add(8).cast::<u16>() = (_rt::as_i32(e)) as u16;
                                }
                                Value::Int4(e) => {
                                    *base.add(0).cast::<u8>() = (6i32) as u8;
                                    *base.add(8).cast::<i32>() = _rt::as_i32(e);
                                }
                                Value::Int8(e) => {
                                    *base.add(0).cast::<u8>() = (7i32) as u8;
                                    *base.add(8).cast::<i64>() = _rt::as_i64(e);
                                }
                                Value::Text(e) => {
                                    *base.add(0).cast::<u8>() = (8i32) as u8;
                                    let vec1 = e;
                                    let ptr1 = vec1.as_ptr().cast::<u8>();
                                    let len1 = vec1.len();
                                    *base.add(12).cast::<usize>() = len1;
                                    *base.add(8).cast::<*mut u8>() = ptr1.cast_mut();
                                }
                                Value::Bytea(e) => {
                                    *base.add(0).cast::<u8>() = (9i32) as u8;
                                    let vec2 = e;
                                    let ptr2 = vec2.as_ptr().cast::<u8>();
                                    let len2 = vec2.len();
                                    *base.add(12).cast::<usize>() = len2;
                                    *base.add(8).cast::<*mut u8>() = ptr2.cast_mut();
                                }
                            }
                        }
                    }
                    let Options { limit: limit4, persistent: persistent4 } = options;
                    #[cfg(target_arch = "wasm32")]
                    #[link(wasm_import_module = "durable:core/sql@1.0.0")]
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
                        result3,
                        len3,
                        _rt::as_i32(limit4),
                        match persistent4 {
                            true => 1,
                            false => 0,
                        },
                    );
                    if layout3.size() != 0 {
                        _rt::alloc::dealloc(result3.cast(), layout3);
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
                    #[link(wasm_import_module = "durable:core/sql@1.0.0")]
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
                                            let v27 = match l3 {
                                                0 => {
                                                    let e27 = {
                                                        let l4 = *ptr0.add(24).cast::<i64>();
                                                        l4 as u64
                                                    };
                                                    QueryResult::Count(e27)
                                                }
                                                n => {
                                                    debug_assert_eq!(n, 1, "invalid enum discriminant");
                                                    let e27 = {
                                                        let l5 = *ptr0.add(24).cast::<*mut u8>();
                                                        let l6 = *ptr0.add(28).cast::<usize>();
                                                        let base26 = l5;
                                                        let len26 = l6;
                                                        let mut result26 = _rt::Vec::with_capacity(len26);
                                                        for i in 0..len26 {
                                                            let base = base26.add(i * 24);
                                                            let e26 = {
                                                                let l7 = *base.add(0).cast::<*mut u8>();
                                                                let l8 = *base.add(4).cast::<usize>();
                                                                let len9 = l8;
                                                                let bytes9 = _rt::Vec::from_raw_parts(
                                                                    l7.cast(),
                                                                    len9,
                                                                    len9,
                                                                );
                                                                let l10 = i32::from(*base.add(8).cast::<u8>());
                                                                let v25 = match l10 {
                                                                    0 => {
                                                                        let e25 = {
                                                                            let l11 = i32::from(*base.add(16).cast::<u8>());
                                                                            PrimitiveType::_lift(l11 as u8)
                                                                        };
                                                                        Value::Null(e25)
                                                                    }
                                                                    1 => {
                                                                        let e25 = {
                                                                            let l12 = i32::from(*base.add(16).cast::<u8>());
                                                                            _rt::bool_lift(l12 as u8)
                                                                        };
                                                                        Value::Boolean(e25)
                                                                    }
                                                                    2 => {
                                                                        let e25 = {
                                                                            let l13 = *base.add(16).cast::<f32>();
                                                                            l13
                                                                        };
                                                                        Value::Float4(e25)
                                                                    }
                                                                    3 => {
                                                                        let e25 = {
                                                                            let l14 = *base.add(16).cast::<f64>();
                                                                            l14
                                                                        };
                                                                        Value::Float8(e25)
                                                                    }
                                                                    4 => {
                                                                        let e25 = {
                                                                            let l15 = i32::from(*base.add(16).cast::<i8>());
                                                                            l15 as i8
                                                                        };
                                                                        Value::Int1(e25)
                                                                    }
                                                                    5 => {
                                                                        let e25 = {
                                                                            let l16 = i32::from(*base.add(16).cast::<i16>());
                                                                            l16 as i16
                                                                        };
                                                                        Value::Int2(e25)
                                                                    }
                                                                    6 => {
                                                                        let e25 = {
                                                                            let l17 = *base.add(16).cast::<i32>();
                                                                            l17
                                                                        };
                                                                        Value::Int4(e25)
                                                                    }
                                                                    7 => {
                                                                        let e25 = {
                                                                            let l18 = *base.add(16).cast::<i64>();
                                                                            l18
                                                                        };
                                                                        Value::Int8(e25)
                                                                    }
                                                                    8 => {
                                                                        let e25 = {
                                                                            let l19 = *base.add(16).cast::<*mut u8>();
                                                                            let l20 = *base.add(20).cast::<usize>();
                                                                            let len21 = l20;
                                                                            let bytes21 = _rt::Vec::from_raw_parts(
                                                                                l19.cast(),
                                                                                len21,
                                                                                len21,
                                                                            );
                                                                            _rt::string_lift(bytes21)
                                                                        };
                                                                        Value::Text(e25)
                                                                    }
                                                                    n => {
                                                                        debug_assert_eq!(n, 9, "invalid enum discriminant");
                                                                        let e25 = {
                                                                            let l22 = *base.add(16).cast::<*mut u8>();
                                                                            let l23 = *base.add(20).cast::<usize>();
                                                                            let len24 = l23;
                                                                            _rt::Vec::from_raw_parts(l22.cast(), len24, len24)
                                                                        };
                                                                        Value::Bytea(e25)
                                                                    }
                                                                };
                                                                Column {
                                                                    name: _rt::string_lift(bytes9),
                                                                    value: v25,
                                                                }
                                                            };
                                                            result26.push(e26);
                                                        }
                                                        _rt::cabi_dealloc(base26, len26 * 24, 8);
                                                        Row { columns: result26 }
                                                    };
                                                    QueryResult::Row(e27)
                                                }
                                            };
                                            v27
                                        };
                                        Ok(e)
                                    }
                                    1 => {
                                        let e = {
                                            let l28 = i32::from(*ptr0.add(16).cast::<u8>());
                                            let v63 = match l28 {
                                                0 => {
                                                    let e63 = {
                                                        let l29 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l30 = *ptr0.add(24).cast::<usize>();
                                                        let len31 = l30;
                                                        let bytes31 = _rt::Vec::from_raw_parts(
                                                            l29.cast(),
                                                            len31,
                                                            len31,
                                                        );
                                                        let l32 = *ptr0.add(28).cast::<*mut u8>();
                                                        let l33 = *ptr0.add(32).cast::<usize>();
                                                        let len34 = l33;
                                                        let bytes34 = _rt::Vec::from_raw_parts(
                                                            l32.cast(),
                                                            len34,
                                                            len34,
                                                        );
                                                        ColumnDecodeError {
                                                            index: _rt::string_lift(bytes31),
                                                            source: _rt::string_lift(bytes34),
                                                        }
                                                    };
                                                    Error::ColumnDecode(e63)
                                                }
                                                1 => {
                                                    let e63 = {
                                                        let l35 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l36 = *ptr0.add(24).cast::<usize>();
                                                        let len37 = l36;
                                                        let bytes37 = _rt::Vec::from_raw_parts(
                                                            l35.cast(),
                                                            len37,
                                                            len37,
                                                        );
                                                        _rt::string_lift(bytes37)
                                                    };
                                                    Error::TypeNotFound(e63)
                                                }
                                                2 => {
                                                    let e63 = {
                                                        let l38 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l39 = *ptr0.add(24).cast::<usize>();
                                                        let len40 = l39;
                                                        let bytes40 = _rt::Vec::from_raw_parts(
                                                            l38.cast(),
                                                            len40,
                                                            len40,
                                                        );
                                                        _rt::string_lift(bytes40)
                                                    };
                                                    Error::Encode(e63)
                                                }
                                                3 => {
                                                    let e63 = {
                                                        let l41 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l42 = *ptr0.add(24).cast::<usize>();
                                                        let len43 = l42;
                                                        let bytes43 = _rt::Vec::from_raw_parts(
                                                            l41.cast(),
                                                            len43,
                                                            len43,
                                                        );
                                                        _rt::string_lift(bytes43)
                                                    };
                                                    Error::Decode(e63)
                                                }
                                                4 => {
                                                    let e63 = {
                                                        let l44 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l45 = *ptr0.add(24).cast::<usize>();
                                                        let len46 = l45;
                                                        let bytes46 = _rt::Vec::from_raw_parts(
                                                            l44.cast(),
                                                            len46,
                                                            len46,
                                                        );
                                                        let l47 = i32::from(*ptr0.add(28).cast::<u8>());
                                                        let l48 = i32::from(*ptr0.add(32).cast::<u8>());
                                                        let l52 = i32::from(*ptr0.add(44).cast::<u8>());
                                                        let l56 = i32::from(*ptr0.add(56).cast::<u8>());
                                                        DatabaseError {
                                                            message: _rt::string_lift(bytes46),
                                                            kind: DatabaseErrorKind::_lift(l47 as u8),
                                                            code: match l48 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l49 = *ptr0.add(36).cast::<*mut u8>();
                                                                        let l50 = *ptr0.add(40).cast::<usize>();
                                                                        let len51 = l50;
                                                                        let bytes51 = _rt::Vec::from_raw_parts(
                                                                            l49.cast(),
                                                                            len51,
                                                                            len51,
                                                                        );
                                                                        _rt::string_lift(bytes51)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                            constraint: match l52 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l53 = *ptr0.add(48).cast::<*mut u8>();
                                                                        let l54 = *ptr0.add(52).cast::<usize>();
                                                                        let len55 = l54;
                                                                        let bytes55 = _rt::Vec::from_raw_parts(
                                                                            l53.cast(),
                                                                            len55,
                                                                            len55,
                                                                        );
                                                                        _rt::string_lift(bytes55)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                            table: match l56 {
                                                                0 => None,
                                                                1 => {
                                                                    let e = {
                                                                        let l57 = *ptr0.add(60).cast::<*mut u8>();
                                                                        let l58 = *ptr0.add(64).cast::<usize>();
                                                                        let len59 = l58;
                                                                        let bytes59 = _rt::Vec::from_raw_parts(
                                                                            l57.cast(),
                                                                            len59,
                                                                            len59,
                                                                        );
                                                                        _rt::string_lift(bytes59)
                                                                    };
                                                                    Some(e)
                                                                }
                                                                _ => _rt::invalid_enum_discriminant(),
                                                            },
                                                        }
                                                    };
                                                    Error::Database(e63)
                                                }
                                                n => {
                                                    debug_assert_eq!(n, 5, "invalid enum discriminant");
                                                    let e63 = {
                                                        let l60 = *ptr0.add(20).cast::<*mut u8>();
                                                        let l61 = *ptr0.add(24).cast::<usize>();
                                                        let len62 = l61;
                                                        let bytes62 = _rt::Vec::from_raw_parts(
                                                            l60.cast(),
                                                            len62,
                                                            len62,
                                                        );
                                                        _rt::string_lift(bytes62)
                                                    };
                                                    Error::Other(e63)
                                                }
                                            };
                                            v63
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
    pub use alloc_crate::string::String;
    pub use alloc_crate::vec::Vec;
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
#[link_section = "component-type:wit-bindgen:0.28.0:import-sql:encoded world"]
#[doc(hidden)]
pub static __WIT_BINDGEN_COMPONENT_TYPE: [u8; 875] = *b"\
\0asm\x0d\0\x01\0\0\x19\x16wit-component-encoding\x04\0\x07\xea\x05\x01A\x02\x01\
A\x02\x01B\x1e\x01m\x09\x07boolean\x06float4\x06float8\x04int1\x04int2\x04int4\x04\
int8\x04text\x05bytea\x04\0\x0eprimitive-type\x03\0\0\x01p}\x01q\x0a\x04null\x01\
\x01\0\x07boolean\x01\x7f\0\x06float4\x01v\0\x06float8\x01u\0\x04int1\x01~\0\x04\
int2\x01|\0\x04int4\x01z\0\x04int8\x01x\0\x04text\x01s\0\x05bytea\x01\x02\0\x04\0\
\x05value\x03\0\x03\x01r\x02\x04names\x05value\x04\x04\0\x06column\x03\0\x05\x01\
p\x06\x01r\x01\x07columns\x07\x04\0\x03row\x03\0\x08\x01q\x02\x05count\x01w\0\x03\
row\x01\x09\0\x04\0\x0cquery-result\x03\0\x0a\x01r\x02\x05limit}\x0apersistent\x7f\
\x04\0\x07options\x03\0\x0c\x01r\x02\x05indexs\x06sources\x04\0\x13column-decode\
-error\x03\0\x0e\x01m\x05\x10unique-violation\x15foreign-key-violation\x12not-nu\
ll-violation\x0fcheck-violation\x05other\x04\0\x13database-error-kind\x03\0\x10\x01\
ks\x01r\x05\x07messages\x04kind\x11\x04code\x12\x0aconstraint\x12\x05table\x12\x04\
\0\x0edatabase-error\x03\0\x13\x01q\x06\x0dcolumn-decode\x01\x0f\0\x0etype-not-f\
ound\x01s\0\x06encode\x01s\0\x06decode\x01s\0\x08database\x01\x14\0\x05other\x01\
s\0\x04\0\x05error\x03\0\x15\x01p\x04\x01@\x03\x03sqls\x06params\x17\x07options\x0d\
\x01\0\x04\0\x05query\x01\x18\x01j\x01\x0b\x01\x16\x01k\x19\x01@\0\0\x1a\x04\0\x05\
fetch\x01\x1b\x03\x01\x16durable:core/sql@1.0.0\x05\0\x04\x01\x1ddurable:core/im\
port-sql@1.0.0\x04\0\x0b\x10\x01\0\x0aimport-sql\x03\0\0\0G\x09producers\x01\x0c\
processed-by\x02\x0dwit-component\x070.214.0\x10wit-bindgen-rust\x060.28.0";
#[inline(never)]
#[doc(hidden)]
#[cfg(target_arch = "wasm32")]
pub fn __link_custom_section_describing_imports() {
    wit_bindgen_rt::maybe_link_cabi_realloc();
}
