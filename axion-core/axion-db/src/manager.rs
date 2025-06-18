// axion-db/src/manager.rs
use crate::{
    client::DbClient,
    config::DbConfig,
    error::DbResult,
    introspection::{self, Introspector},
    // IMPORTANT: Make RoutineKind accessible for matching
    metadata::{DatabaseMetadata, RoutineKind},
};
use comfy_table::{presets::UTF8_FULL, Cell, CellAlignment, Table}; // Import comfy-table
use owo_colors::OwoColorize; // Import the colorize trait
use std::sync::Arc;
use tracing::info;

/// The ModelManager is the primary entry point for database introspection.
/// It holds the complete database schema and provides methods to interact with it.
#[derive(Clone)]
pub struct ModelManager {
    pub db_client: Arc<DbClient>,
    pub metadata: Arc<DatabaseMetadata>,
    introspector: Arc<dyn Introspector>,
}

impl ModelManager {
    /// Creates a new ModelManager by connecting to the database and performing a full introspection.
    pub async fn new(config: DbConfig) -> DbResult<Self> {
        info!("Initializing ModelManager...");
        let db_client = Arc::new(DbClient::new(config).await?);
        let introspector = introspection::new_introspector(db_client.clone())?;

        info!("Discovering user schemas...");
        let schemas = introspector.list_user_schemas().await?;

        info!("Performing full database introspection...");
        let metadata = introspector.introspect(&schemas).await?;
        info!(
            "Introspection complete. Found {} schemas.",
            metadata.schemas.len()
        );

        Ok(Self {
            db_client,
            metadata: Arc::new(metadata),
            introspector: Arc::from(introspector),
        })
    }

    // =================================================================================
    //  DX: Pretty-Printing Methods (WITH THE NEW `display_summary`)
    // =================================================================================

    /// Prints a rich, colorized, table-based summary of the database metadata.
    pub fn display_summary(&self) {
        println!(); // Add a newline for spacing

        let mut table = Table::new();

        // THE FIX: Use the UTF8_BORDERS_ONLY preset.
        // This preset creates the outer box and the line under the header,
        // but no horizontal lines between the data rows, achieving the clean look.
        table
            .load_preset(comfy_table::presets::UTF8_BORDERS_ONLY)
            .set_header(vec![
                Cell::new("Schema").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Tables").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Views").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Enums").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Functions").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Procedures").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Triggers").add_attribute(comfy_table::Attribute::Bold),
                Cell::new("Total").add_attribute(comfy_table::Attribute::Bold),
            ]);

        // --- Totals Initialization ---
        let mut total_tables = 0;
        let mut total_views = 0;
        let mut total_enums = 0;
        let mut total_functions = 0;
        let mut total_procedures = 0;
        let mut total_triggers = 0;

        // --- Sort schemas for consistent output ---
        let mut schemas: Vec<_> = self.metadata.schemas.keys().collect();
        schemas.sort();

        for schema_name in schemas {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                // --- Per-schema Counts ---
                let tables_count = schema_data.tables.len();
                let views_count = schema_data.views.len();
                let enums_count = schema_data.enums.len();

                let mut functions_count = 0;
                let mut procedures_count = 0;
                let mut triggers_count = 0;
                for func_meta in schema_data.functions.values() {
                    match func_meta.kind {
                        Some(RoutineKind::Function) => functions_count += 1,
                        Some(RoutineKind::Procedure) => procedures_count += 1,
                        Some(RoutineKind::Trigger) => triggers_count += 1,
                        _ => {}
                    }
                }

                let schema_total = tables_count + views_count + enums_count + functions_count + procedures_count + triggers_count;

                // --- Add to Grand Totals ---
                total_tables += tables_count;
                total_views += views_count;
                total_enums += enums_count;
                total_functions += functions_count;
                total_procedures += procedures_count;
                total_triggers += triggers_count;

                // --- Build and Add the Row ---
                table.add_row(vec![
                    Cell::new(schema_name).fg(comfy_table::Color::Cyan),
                    Cell::new(tables_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Blue),
                    Cell::new(views_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Green),
                    Cell::new(enums_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Magenta),
                    Cell::new(functions_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Red),
                    Cell::new(procedures_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Yellow),
                    Cell::new(triggers_count).set_alignment(CellAlignment::Right).fg(comfy_table::Color::DarkYellow),
                    Cell::new(schema_total).set_alignment(CellAlignment::Right).add_attribute(comfy_table::Attribute::Bold),
                ]);
            }
        }

        // --- Grand Total Calculation ---
        let grand_total = total_tables + total_views + total_enums + total_functions + total_procedures + total_triggers;

        // --- Add the TOTAL row which will act as the footer ---
        // This row will have the bottom border of the table drawn after it.
        table.add_row(vec![
            Cell::new("TOTAL").add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_tables).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Blue).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_views).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Green).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_enums).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Magenta).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_functions).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Red).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_procedures).set_alignment(CellAlignment::Right).fg(comfy_table::Color::Yellow).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(total_triggers).set_alignment(CellAlignment::Right).fg(comfy_table::Color::DarkYellow).add_attribute(comfy_table::Attribute::Bold),
            Cell::new(grand_total).set_alignment(CellAlignment::Right).add_attribute(comfy_table::Attribute::Bold),
        ]);

        // Print the title and the final table
        println!("{}", " ModelManager Statistics".green().bold().underline());
        println!("{table}");
    }

    /// Prints a detailed, prism-py-like breakdown of tables for the specified schemas.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_tables(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           TABLES OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display: Box<dyn Iterator<Item = &str>> = if schemas.is_empty() {
            Box::new(self.metadata.schemas.keys().map(|s| s.as_str()))
        } else {
            Box::new(schemas.iter().copied())
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                for table_data in schema_data.tables.values() {
                    // This now uses the beautiful `Display` implementation we wrote for TableMetadata
                    println!("{}\n", table_data);
                }
            }
        }
    }

    /// Prints a detailed, prism-py-like breakdown of views for the specified schemas.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_views(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           VIEWS OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display: Box<dyn Iterator<Item = &str>> = if schemas.is_empty() {
            Box::new(self.metadata.schemas.keys().map(|s| s.as_str()))
        } else {
            Box::new(schemas.iter().copied())
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                for view_data in schema_data.views.values() {
                    // Uses the `Display` implementation for ViewMetadata
                    println!("{}\n", view_data);
                }
            }
        }
    }

    /// Prints a summary of all enums for the specified schemas with enhanced formatting.
    /// If `schemas` is empty, it displays all schemas.
    pub fn display_enums(&self, schemas: &[&str]) {
        println!("\n{:=<80}", "");
        println!("           ENUMS OVERVIEW");
        println!("{:=<80}\n", "");

        let schemas_to_display: Box<dyn Iterator<Item = &str>> = if schemas.is_empty() {
            Box::new(self.metadata.schemas.keys().map(|s| s.as_str()))
        } else {
            Box::new(schemas.iter().copied())
        };

        for schema_name in schemas_to_display {
            if let Some(schema_data) = self.metadata.schemas.get(schema_name) {
                if !schema_data.enums.is_empty() {
                    println!("Schema '{}':", schema_name.cyan().bold());
                    for enum_data in schema_data.enums.values() {
                        // Print the enum name, indented and in yellow.
                        println!("  {}", enum_data.name.yellow());

                        // Format the values string, indented further, and styled.
                        let values_str = format!("({})", enum_data.values.join(", "));
                        println!("    {}", values_str.dimmed().italic());

                        // Add a blank line for spacing between enums.
                        println!();
                    }
                }
            }
        }
    }
}