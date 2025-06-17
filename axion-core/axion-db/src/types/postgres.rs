// axion-db/src/types/postgres.rs
use crate::metadata::AxionDataType;
use crate::types::TypeMapper;

#[derive(Debug, Default, Clone, Copy)]
pub struct PostgresTypeMapper;

// A declarative macro to simplify type mapping.
macro_rules! map_sql_to_axion_type {
    (
        $lower_sql_type:expr,
        $udt_name:expr,
        { $( $sql:pat => $axion:expr ),* $(,)? }
    ) => {
        match $lower_sql_type {
            $( $sql => $axion, )*
            // Handle array types by stripping '[]' and recursing
            s if s.ends_with("[]") => {
                let element_type = s.trim_end_matches("[]");
                let element_udt_name = $udt_name
                    .map(|u| u.strip_prefix('_').unwrap_or(u));
                AxionDataType::Array(Box::new(PostgresTypeMapper.sql_to_axion(element_type, element_udt_name)))
            }
            // Fallback for other UDTs (composites, domains) or unmapped built-ins
            _ => AxionDataType::Unsupported($lower_sql_type.to_string()),
        }
    };
}

impl TypeMapper for PostgresTypeMapper {
    fn sql_to_axion(&self, sql_type: &str, udt_name: Option<&str>) -> AxionDataType {
        // Prioritize UDT name for custom enums, etc.
        if let Some(udt) = udt_name {
            if !udt.starts_with('_') {
                // Internal pg types often start with an underscore
                match udt {
                    "hstore" => return AxionDataType::Json, // Treat hstore like json
                    _ => {
                        // Assume it's a custom enum type
                        return AxionDataType::Enum(udt.to_string());
                    }
                }
            }
        }

        let lower_sql_type = sql_type.to_lowercase();

        map_sql_to_axion_type!(lower_sql_type.as_str(), udt_name, {
            "integer" | "int" | "int4" => AxionDataType::Integer(32),
            "bigint" | "int8" => AxionDataType::Integer(64),
            "smallint" | "int2" => AxionDataType::Integer(16),
            "character varying" | "varchar" | "text" | "name" | "citext" | "char" | "bpchar" => AxionDataType::Text,
            "boolean" | "bool" => AxionDataType::Boolean,
            "date" => AxionDataType::Date,
            "timestamp without time zone" | "timestamp" => AxionDataType::Timestamp,
            "timestamp with time zone" | "timestamptz" => AxionDataType::TimestampTz,
            "time without time zone" | "time" => AxionDataType::Time,
            "numeric" | "decimal" => AxionDataType::Numeric,
            "real" | "float4" => AxionDataType::Float(32),
            "double precision" | "float8" => AxionDataType::Float(64),
            "bytea" => AxionDataType::Bytes,
            "uuid" => AxionDataType::Uuid,
            "json" => AxionDataType::Json,
            "jsonb" => AxionDataType::JsonB,
            "inet" | "cidr" => AxionDataType::Inet,
        })
    }
}
