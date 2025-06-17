// in axion-db/src/metadata.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// The root of our introspected database model.
// This will be wrapped in an Arc for cheap, thread-safe sharing.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseMetadata {
    /// A map of schema names to their detailed metadata.
    pub schemas: HashMap<String, SchemaMetadata>,
}

/// Represents a single database schema (like `public` or `app`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaMetadata {
    pub name: String,
    pub tables: HashMap<String, TableMetadata>,
    pub views: HashMap<String, ViewMetadata>,
    pub enums: HashMap<String, EnumMetadata>,
    pub functions: HashMap<String, FunctionMetadata>,
    // Procedures can be represented by FunctionMetadata where kind is Procedure
}

/// A normalized, database-agnostic representation of a data type.
/// This is the core of our robust type system, abstracting away DB-specific names.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AxionDataType {
    // Text types
    Text,
    // Integer types
    Integer(i32), // Using bit-size for precision: 16, 32, 64
    // Floating-point types
    Float(i32), // 32, 64
    // Other numeric types
    Numeric,
    // Boolean
    Boolean,
    // Date and Time
    Timestamp,
    TimestampTz,
    Date,
    Time,
    // Binary
    Bytes,
    // UUID
    Uuid,
    // JSON
    Json,
    JsonB,
    // Network
    Inet,
    // Enum, holding the name of the Rust enum we will generate
    Enum(String),
    // Array, holding the type of its elements
    Array(Box<AxionDataType>),
    // A placeholder for types we don't yet explicitly handle
    Unsupported(String),
}

/// Represents a reference to another column, for foreign keys.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ForeignKeyReference {
    pub schema: String,
    pub table: String,
    pub column: String,
}

/// Detailed metadata for a single column in a table or view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnMetadata {
    pub name: String,
    /// The original, raw data type name from the database (e.g., "character varying").
    pub sql_type_name: String,
    /// Axion's normalized, internal data type.
    pub axion_type: AxionDataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
    pub foreign_key: Option<ForeignKeyReference>,
}

/// Metadata for a database table.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableMetadata {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key_columns: Vec<String>,
    pub comment: Option<String>,
}

/// Metadata for a database view.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViewMetadata {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnMetadata>,
    /// The SQL definition of the view.
    pub definition: Option<String>,
    pub comment: Option<String>,
}

/// Metadata for a user-defined enum type.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnumMetadata {
    pub name: String,
    pub schema: String,
    /// The possible values for this enum.
    pub values: Vec<String>,
    pub comment: Option<String>,
}

/// The kind of database routine. This elegantly handles functions, procedures, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RoutineKind {
    Function,
    Procedure,
    Aggregate,
    Window,
    Trigger,
}

/// The mode of a parameter to a function or procedure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParameterMode {
    In,
    Out,
    InOut,
    Variadic,
}

/// Metadata for a single parameter to a function or procedure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterMetadata {
    pub name: String,
    pub sql_type_name: String,
    pub axion_type: AxionDataType,
    pub mode: ParameterMode,
    pub has_default: bool,
}

/// Metadata for a database function, procedure, or trigger.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionMetadata {
    pub name: String,
    pub schema: String,
    pub kind: Option<RoutineKind>,
    pub parameters: Vec<ParameterMetadata>,
    /// The return type for scalar functions or procedures.
    pub return_type: Option<AxionDataType>,
    /// For functions that `RETURNS TABLE(...)`, this holds the columns.
    pub return_table: Option<Vec<ColumnMetadata>>,
    pub comment: Option<String>,
}
