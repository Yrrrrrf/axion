// axion-db/src/types/postgres.rs
use crate::metadata::AxionDataType;
use crate::types::TypeMapper;

#[derive(Debug, Default, Clone, Copy)]
pub struct PostgresTypeMapper;

// A declarative macro to simplify the large match statement.
macro_rules! map_sql_to_axion {
    (
        $lower_sql_type:expr,
        { $( $sql:pat => $axion:expr ),* $(,)? }
    ) => {
        match $lower_sql_type {
            $( $sql => $axion, )*
            // Fallback for unmapped built-ins
            _ => AxionDataType::Unsupported($lower_sql_type.to_string()),
        }
    };
}

impl TypeMapper for PostgresTypeMapper {
    fn sql_to_axion(&self, sql_type: &str, udt_name: Option<&str>) -> AxionDataType {
        // Handle Array types first, as they are a special case of `data_type`
        if sql_type == "ARRAY" {
            // For arrays, the element type is in the UDT name (e.g., "_int4", "_varchar").
            // We strip the leading underscore to get the base type.
            if let Some(udt) = udt_name.and_then(|u| u.strip_prefix('_')) {
                return AxionDataType::Array(Box::new(self.sql_to_axion(udt, Some(udt))));
            }
        }

        // Handle User-Defined types next (this is for custom enums)
        if sql_type == "USER-DEFINED" {
            if let Some(udt) = udt_name {
                return AxionDataType::Enum(udt.to_string());
            }
        }

        // Handle all other standard types
        let lower_sql_type = sql_type.to_lowercase();

        map_sql_to_axion!(lower_sql_type.as_str(), {
            "uuid" => AxionDataType::Uuid,
            "integer" | "int" | "int4" => AxionDataType::Integer(32),
            "bigint" | "int8" => AxionDataType::Integer(64),
            "smallint" | "int2" => AxionDataType::Integer(16),
            "character varying" | "varchar" | "text" | "name" | "citext" | "char" | "bpchar" => AxionDataType::Text,
            "boolean" | "bool" => AxionDataType::Boolean,
            "date" => AxionDataType::Date,
            "time without time zone" | "time" => AxionDataType::Time,
            "timestamp without time zone" | "timestamp" => AxionDataType::Timestamp,
            "timestamp with time zone" | "timestamptz" => AxionDataType::TimestampTz,
            "numeric" | "decimal" => AxionDataType::Numeric,
            "real" | "float4" => AxionDataType::Float(32),
            "double precision" | "float8" => AxionDataType::Float(64),
            "bytea" => AxionDataType::Bytes,
            "json" => AxionDataType::Json,
            "jsonb" => AxionDataType::JsonB,
            "inet" | "cidr" => AxionDataType::Inet,
        })
    }
}
