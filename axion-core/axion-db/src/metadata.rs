// axion-db/src/metadata.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap; // Added this

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ColumnMetadata {
    pub name: String,
    pub ordinal_position: i32,
    pub sql_type_name: String, // Raw SQL type name (e.g., "VARCHAR", "INTEGER", "character varying")
    pub udt_name: Option<String>, // User-defined type name from `information_schema.columns.udt_name`
    // This is often more specific for enums, domains, composites in Postgres.
    pub internal_axion_type: String, // Axion's standardized internal type name
    pub rust_type_string: String,    // Suggested Rust type as a string
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
    pub foreign_key_reference: Option<ForeignKeyReference>,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForeignKeyReference {
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
    // Could add local_column_name if needed, but often implied by ColumnMetadata
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TableMetadata {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key_columns: Vec<String>,
    pub comment: Option<String>,
    // pub foreign_keys: Vec<ForeignKeyConstraintMeta>, // More detailed later
    // pub unique_constraints: Vec<UniqueConstraintMeta>,
    // pub indexes: Vec<IndexMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ViewMetadata {
    pub schema: String,
    pub name: String,
    pub definition: Option<String>,
    pub columns: Vec<ColumnMetadata>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterMode {
    In,
    Out,
    InOut,
    Variadic,
}

impl Default for ParameterMode {
    fn default() -> Self {
        ParameterMode::In
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ParameterMetadata {
    pub name: String,
    pub sql_type_name: String,
    pub udt_name: Option<String>,
    pub internal_axion_type: String,
    pub rust_type_string: String,
    pub mode: ParameterMode,
    pub ordinal_position: i32,
    pub has_default: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FunctionKind {
    Scalar,
    Table,
    Aggregate,
    Window,
    Procedure, // Added to distinguish
    Trigger,   // Added to distinguish
}

impl Default for FunctionKind {
    fn default() -> Self {
        FunctionKind::Scalar
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FunctionMetadata {
    pub schema: String,
    pub name: String,
    pub parameters: Vec<ParameterMetadata>,
    pub return_sql_type_name: String,
    pub return_udt_name: Option<String>,
    pub return_internal_axion_type: String,
    pub return_rust_type_string: String,
    pub kind: FunctionKind,
    pub definition: Option<String>,
    pub comment: Option<String>,
    pub is_strict: Option<bool>,    // Specific to Postgres
    pub volatility: Option<String>, // Specific to Postgres (VOLATILE, STABLE, IMMUTABLE)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumValueMetadata {
    pub value: String,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EnumMetadata {
    pub schema: String,
    pub name: String,
    pub values: Vec<EnumValueMetadata>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaMetadata {
    pub name: String,
    pub tables: HashMap<String, TableMetadata>,
    pub views: HashMap<String, ViewMetadata>,
    pub functions: HashMap<String, FunctionMetadata>, // Includes procedures and triggers for now
    pub enums: HashMap<String, EnumMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseMetadata {
    pub schemas: HashMap<String, SchemaMetadata>,
}
