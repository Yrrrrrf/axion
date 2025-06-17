// axion-db/src/types/mod.rs
use crate::metadata::AxionDataType;

pub mod postgres;

/// A trait for mapping database-specific type names to Axion's normalized data types.
pub trait TypeMapper: Send + Sync {
    /// Maps a SQL type name and an optional UDT name to an `AxionDataType`.
    fn sql_to_axion(&self, sql_type: &str, udt_name: Option<&str>) -> AxionDataType;
}
