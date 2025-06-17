// in axion-db/src/metadata.rs

use owo_colors::{OwoColorize, Style};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt; // The essential import for custom formatting


// =================================================================================
//  1. The Formatting Macro: A helper to create clean, aligned key-value output.
// =================================================================================

macro_rules! write_field {
    // For simple fields: `write_field!(f, "Name", &self.name)`
    ($f:expr, $name:literal, $value:expr) => {
        writeln!($f, "  {:<20} : {:?}", $name, $value)
    };
    // For fields that are collections: `write_field!(f, "Tables", &self.tables, collection)`
    ($f:expr, $name:literal, $collection:expr, collection) => {
        writeln!($f, "  {:<20} : {} items", $name, $collection.len())
    };
}

// =================================================================================
//  2. The Metadata Structs (Now with manual Display and Debug implementations)
// =================================================================================

// --- Root Metadata Structs ---

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DatabaseMetadata {
    pub schemas: HashMap<String, SchemaMetadata>,
}

impl fmt::Display for DatabaseMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database with {} schemas", self.schemas.len())
    }
}

impl fmt::Debug for DatabaseMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DatabaseMetadata ({} schemas):", self.schemas.len())?;
        for (name, schema) in &self.schemas {
            writeln!(f, "\nSchema '{}':\n{:#?}", name, schema)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SchemaMetadata {
    pub name: String,
    pub tables: HashMap<String, TableMetadata>,
    pub views: HashMap<String, ViewMetadata>,
    pub enums: HashMap<String, EnumMetadata>,
    pub functions: HashMap<String, FunctionMetadata>,
}

impl fmt::Display for SchemaMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Schema '{}' [{} tables, {} views, {} enums]",
            self.name,
            self.tables.len(),
            self.views.len(),
            self.enums.len()
        )
    }
}

impl fmt::Debug for SchemaMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Schema '{}':", self.name)?;
        write_field!(f, "Tables", self.tables, collection)?;
        write_field!(f, "Views", self.views, collection)?;
        write_field!(f, "Enums", self.enums, collection)?;
        write_field!(f, "Functions", self.functions, collection)?;
        Ok(())
    }
}

// --- Type and Reference Structs ---

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AxionDataType {
    Text,
    Integer(i32),
    Float(i32),
    Numeric,
    Boolean,
    Timestamp,
    TimestampTz,
    Date,
    Time,
    Bytes,
    Uuid,
    Json,
    JsonB,
    Inet,
    Enum(String),
    Array(Box<AxionDataType>),
    Unsupported(String),
}

// Display for AxionDataType will be its compact representation (e.g., "TEXT", "INT4", "UUID[]")
impl fmt::Display for AxionDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "TEXT"),
            Self::Integer(bits) => write!(f, "INT{}", bits),
            Self::Float(bits) => write!(f, "FLOAT{}", bits),
            Self::Numeric => write!(f, "NUMERIC"),
            Self::Boolean => write!(f, "BOOL"),
            Self::Timestamp => write!(f, "TIMESTAMP"),
            Self::TimestampTz => write!(f, "TIMESTAMPTZ"),
            Self::Date => write!(f, "DATE"),
            Self::Time => write!(f, "TIME"),
            Self::Bytes => write!(f, "BYTES"),
            Self::Uuid => write!(f, "UUID"),
            Self::Json => write!(f, "JSON"),
            Self::JsonB => write!(f, "JSONB"),
            Self::Inet => write!(f, "INET"),
            Self::Enum(name) => write!(f, "{}", name),
            Self::Array(inner) => write!(f, "{}[]", inner),
            Self::Unsupported(name) => write!(f, "UNSUPPORTED({})", name),
        }
    }
}
// Debug for AxionDataType will be the verbose, struct-like representation
impl fmt::Debug for AxionDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Enum(name) => f.debug_tuple("Enum").field(name).finish(),
            Self::Array(inner) => f.debug_tuple("Array").field(inner).finish(),
            Self::Unsupported(name) => f.debug_tuple("Unsupported").field(name).finish(),
            _ => write!(f, "{}", self), // For simple variants, Display and Debug are the same
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ForeignKeyReference {
    pub schema: String,
    pub table: String,
    pub column: String,
}
// Let's create a compact display for FKs
impl fmt::Display for ForeignKeyReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.schema, self.table, self.column)
    }
}
impl fmt::Debug for ForeignKeyReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For Debug, a more structured view is nice
        f.debug_struct("ForeignKey")
            .field("schema", &self.schema)
            .field("table", &self.table)
            .field("column", &self.column)
            .finish()
    }
}

// --- Core Entity Structs ---

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColumnMetadata {
    pub name: String,
    pub sql_type_name: String,
    pub axion_type: AxionDataType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
    pub foreign_key: Option<ForeignKeyReference>,
}
// This provides the `column_name    VARCHAR(255)    TEXT` format

// This provides the `column_name    VARCHAR(255)    TEXT` format
impl fmt::Display for ColumnMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Define styles for different parts
        let pk_style = Style::new().green().bold();
        let fk_style = Style::new().cyan();
        let enum_style = Style::new().yellow();
        let type_style = Style::new().magenta();
        let nullable_style = Style::new().red().bold();

        // 1. Column Name and Nullable Marker
        // ======================= THE CORRECTED FIX =======================
        // A plain &str already implements Display. We just need to Box it.
        let nullable_marker: Box<dyn fmt::Display> = if self.is_nullable {
            Box::new(" ") // <-- Simply box the plain string slice.
        } else {
            Box::new("*".style(nullable_style))
        };
        // ===============================================================
        write!(f, "  {} {:<25}", nullable_marker, self.name.bold())?;

        // 2. SQL Type
        write!(f, "{:<20}", self.sql_type_name.dimmed())?;

        // 3. Axion Type (with special coloring for enums)
        let binding = self.axion_type.to_string();
        let axion_type_display: Box<dyn fmt::Display> = match &self.axion_type {
            AxionDataType::Enum(name) => Box::new(name.style(enum_style)),
            _ => Box::new(binding.style(type_style)),
        };


        write!(f, "{:<20}", axion_type_display)?;

        // 4. Constraints (PK, FK)
        let mut constraints = Vec::new();
        if self.is_primary_key {
            constraints.push(format!("{}", "PK".style(pk_style)));
        }
        if let Some(fk) = &self.foreign_key {
            constraints.push(format!("{} -> {}", "FK".style(fk_style), fk));
        }
        write!(f, "{}", constraints.join(" "))?;

        Ok(())
    }
}
impl fmt::Debug for ColumnMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Column '{}':", self.name)?;
        write_field!(f, "SQL Type", &self.sql_type_name)?;
        write_field!(f, "Axion Type", &self.axion_type)?;
        write_field!(f, "Nullable", &self.is_nullable)?;
        write_field!(f, "Primary Key", &self.is_primary_key)?;
        write_field!(f, "Default", &self.default_value)?;
        write_field!(f, "Foreign Key", &self.foreign_key)?;
        write_field!(f, "Comment", &self.comment)
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct TableMetadata {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnMetadata>,
    pub primary_key_columns: Vec<String>,
    pub comment: Option<String>,
}
impl fmt::Display for TableMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use bright_blue for the table name header
        writeln!(
            f,
            "{}",
            format!("{}.{}", self.schema, self.name)
                .bright_blue()
                .bold()
        )?;

        // Print columns
        for col in &self.columns {
            writeln!(f, "{}", col)?;
        }
        Ok(())
    }
}
impl fmt::Debug for TableMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Table '{}.{}':", self.schema, self.name)?;
        write_field!(f, "Primary Keys", &self.primary_key_columns)?;
        write_field!(f, "Comment", &self.comment)?;
        writeln!(f, "  Columns ({}):", self.columns.len())?;
        for col in &self.columns {
            writeln!(f, "{:#?}", col)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ViewMetadata {
    pub name: String,
    pub schema: String,
    pub columns: Vec<ColumnMetadata>,
    pub definition: Option<String>,
    pub comment: Option<String>,
}
// Views can use the same Display format as Tables
impl fmt::Display for ViewMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Use bright_green for the view name header
        writeln!(
            f,
            "{}",
            format!("{}.{}", self.schema, self.name)
                .bright_green()
                .bold()
        )?;

        for col in &self.columns {
            writeln!(f, "{}", col)?;
        }
        Ok(())
    }
}
impl fmt::Debug for ViewMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "View '{}.{}':", self.schema, self.name)?;
        write_field!(f, "Comment", &self.comment)?;
        if let Some(def) = &self.definition {
            writeln!(
                f,
                "  Definition           : {}...",
                &def.chars().take(50).collect::<String>()
            )?;
        }
        writeln!(f, "  Columns ({}):", self.columns.len())?;
        for col in &self.columns {
            writeln!(f, "{:#?}", col)?;
        }
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct EnumMetadata {
    pub name: String,
    pub schema: String,
    pub values: Vec<String>,
    pub comment: Option<String>,
}
impl fmt::Display for EnumMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{} ({})",
            self.schema,
            self.name,
            self.values.join(", ")
        )
    }
}
impl fmt::Debug for EnumMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Enum '{}.{}':", self.schema, self.name)?;
        write_field!(f, "Values", &self.values)?;
        write_field!(f, "Comment", &self.comment)
    }
}

// NOTE: Function-related structs are left with derived Debug for now,
// as they are not yet implemented in the introspector.

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RoutineKind {
    Function,
    Procedure,
    Aggregate,
    Window,
    Trigger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ParameterMode {
    In,
    Out,
    InOut,
    Variadic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterMetadata {
    pub name: String,
    pub sql_type_name: String,
    pub axion_type: AxionDataType,
    pub mode: ParameterMode,
    pub has_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionMetadata {
    pub name: String,
    pub schema: String,
    pub kind: Option<RoutineKind>,
    pub parameters: Vec<ParameterMetadata>,
    pub return_type: Option<AxionDataType>,
    pub return_table: Option<Vec<ColumnMetadata>>,
    pub comment: Option<String>,
}
