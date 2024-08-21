use sqlx::postgres::types::Oid;
use sqlx::postgres::PgTypeInfo;

macro_rules! decl_oids {
    {
        $( const $name:ident = $value:expr; )*
    } => {
        $(
            #[allow(dead_code)]
            pub(crate) const $name: PgTypeInfo = PgTypeInfo::with_oid(Oid($value));
        )*
    }
}

// You can verify the OIDs here by running the following query:
//
// SELECT ty.oid, ty.typname, arr.oid
// FROM pg_type as ty
// JOIN pg_type arr ON arr.oid = ty.typarray
// WHERE ty.oid < 10000
// ORDER BY ty.oid ASC;
decl_oids! {
    const BOOL = 16;
    const BOOL_ARRAY = 1000;
    const BYTEA = 17;
    const BYTEA_ARRAY = 1001;
    const CHAR = 18;
    const CHAR_ARRAY = 1002;
    const INT8 = 20;
    const INT8_ARRAY = 1016;
    const INT2 = 20;
    const INT2_ARRAY = 1005;
    const INT4 = 23;
    const INT4_ARRAY = 1007;
    const TEXT = 25;
    const TEXT_ARRAY = 1009;
    const JSON = 114;
    const JSON_ARRAY = 199;
    const FLOAT4 = 700;
    const FLOAT4_ARRAY = 1021;
    const FLOAT8 = 701;
    const FLOAT8_ARRAY = 1022;
    const INET = 869;
    const INET_ARRAY = 1041;
    const TIMESTAMP = 1114;
    const TIMESTAMP_ARRAY = 1115;
    const TIMESTAMPTZ = 1184;
    const TIMESTAMPTZ_ARRAY = 1185;
    const UUID = 2950;
    const UUID_ARRAY = 2951;
    const JSONB = 3802;
    const JSONB_ARRAY = 3807;
}
