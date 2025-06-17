// axion-db/src/introspection/postgres.rs
use crate::{
    client::DbClient,
    error::{DbError, DbResult},
    introspection::Introspector,
    metadata::*,
    types::{TypeMapper, postgres::PostgresTypeMapper},
};
use sqlx::FromRow;
use std::{collections::HashMap, sync::Arc};
use tracing::{info, instrument, warn};

// =================================================================================
//  1. FromRow Structs (Unchanged)
// =================================================================================
#[derive(Debug, FromRow)]
struct TableAndViewRow {
    table_name: String,
    table_type: String,
}

#[derive(Debug, FromRow, Clone)]
struct ColumnIntrospectionRow {
    column_name: String,
    data_type: String,
    udt_name: String,
    is_nullable: String,
    column_default: Option<String>,
    column_comment: Option<String>,
    is_primary_key: bool,
}

#[derive(Debug, FromRow)]
struct ForeignKeyIntrospectionRow {
    column_name: String,
    foreign_table_schema: String,
    foreign_table_name: String,
    foreign_column_name: String,
}

#[derive(Debug, FromRow)]
struct EnumIntrospectionRow {
    enum_name: String,
    enum_value: String,
}

// =================================================================================
//  2. The Introspector Implementation
// =================================================================================

pub struct PostgresIntrospector {
    client: Arc<DbClient>,
    type_mapper: PostgresTypeMapper,
}

impl PostgresIntrospector {
    pub fn new(client: Arc<DbClient>) -> Self {
        Self {
            client,
            type_mapper: PostgresTypeMapper,
        }
    }

    // --- Helper Methods using our validated queries ---

    #[instrument(skip(self), name = "list_db_entities")]
    async fn list_tables_and_views(&self, schema_name: &str) -> DbResult<Vec<TableAndViewRow>> {
        let query = "
            SELECT
                table_name::TEXT,
                table_type::TEXT
            FROM information_schema.tables
            WHERE table_schema = $1
            ORDER BY table_type, table_name;
        ";
        sqlx::query_as(query)
            .bind(schema_name)
            .fetch_all(&*self.client.pool)
            .await
            .map_err(DbError::from)
    }

    // (get_foreign_keys_for_table remains unchanged)
    #[instrument(skip(self), name = "get_foreign_keys")]
    async fn get_foreign_keys_for_table(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> DbResult<HashMap<String, ForeignKeyReference>> {
        let query = r#"
            SELECT
                kcu.column_name::TEXT,
                ccu.table_schema::TEXT AS foreign_table_schema,
                ccu.table_name::TEXT AS foreign_table_name,
                ccu.column_name::TEXT AS foreign_column_name
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name AND tc.constraint_schema = kcu.constraint_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name AND ccu.constraint_schema = tc.constraint_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = $1
            AND tc.table_name = $2
        "#;
        let rows: Vec<ForeignKeyIntrospectionRow> = sqlx::query_as(query)
            .bind(schema_name)
            .bind(table_name)
            .fetch_all(&*self.client.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    row.column_name,
                    ForeignKeyReference {
                        schema: row.foreign_table_schema,
                        table: row.foreign_table_name,
                        column: row.foreign_column_name,
                    },
                )
            })
            .collect())
    }
}

// =================================================================================
//  3. The Main Introspector Trait Implementation (Now with View/Enum Logic)
// =================================================================================

#[async_trait::async_trait]
impl Introspector for PostgresIntrospector {
    #[instrument(skip(self), name = "introspect_database")]
    async fn introspect(&self, schemas: &[String]) -> DbResult<DatabaseMetadata> {
        info!(
            "Starting full database introspection for schemas: {:?}",
            schemas
        );
        let mut db_meta = DatabaseMetadata::default();
        for schema_name in schemas {
            match self.introspect_schema(schema_name).await {
                Ok(schema_meta) => {
                    db_meta.schemas.insert(schema_name.clone(), schema_meta);
                }
                Err(e) => warn!("Could not introspect schema '{}': {}", schema_name, e),
            }
        }
        info!("Database introspection complete.");
        Ok(db_meta)
    }

    #[instrument(skip(self), name = "introspect_schema")]
    async fn introspect_schema(&self, schema_name: &str) -> DbResult<SchemaMetadata> {
        let mut schema_meta = SchemaMetadata {
            name: schema_name.to_string(),
            ..Default::default()
        };

        // Fetch all entities and enums for the schema concurrently
        let (entities_result, enums_result) = tokio::join!(
            self.list_tables_and_views(schema_name),
            self.introspect_enums_for_schema(schema_name)
        );

        schema_meta.enums = enums_result?;

        for entity in entities_result? {
            if entity.table_type == "BASE TABLE" {
                match self.introspect_table(schema_name, &entity.table_name).await {
                    Ok(table_md) => {
                        schema_meta.tables.insert(entity.table_name, table_md);
                    }
                    Err(e) => warn!(
                        "Skipping table {}.{}: {}",
                        schema_name, entity.table_name, e
                    ),
                }
            } else if entity.table_type == "VIEW" {
                match self.introspect_view(schema_name, &entity.table_name).await {
                    Ok(view_md) => {
                        schema_meta.views.insert(entity.table_name, view_md);
                    }
                    Err(e) => warn!("Skipping view {}.{}: {}", schema_name, entity.table_name, e),
                }
            }
        }

        Ok(schema_meta)
    }

    #[instrument(skip(self, table_name), name = "introspect_table")]
    async fn introspect_table(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> DbResult<TableMetadata> {
        let columns_query = r#"
            SELECT
                c.column_name::TEXT,
                c.data_type::TEXT,
                c.udt_name::TEXT,
                c.is_nullable::TEXT,
                c.column_default,
                pg_catalog.col_description((quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass::oid, c.ordinal_position) AS column_comment,
                EXISTS (
                    SELECT 1 FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu ON tc.constraint_name = kcu.constraint_name AND tc.constraint_schema = kcu.constraint_schema
                    WHERE tc.table_schema = c.table_schema AND tc.table_name = c.table_name AND kcu.column_name = c.column_name AND tc.constraint_type = 'PRIMARY KEY'
                ) AS is_primary_key
            FROM information_schema.columns c
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position;
        "#;

        let (columns_result, fks_result) = tokio::join!(
            sqlx::query_as::<_, ColumnIntrospectionRow>(columns_query)
                .bind(schema_name)
                .bind(table_name)
                .fetch_all(&*self.client.pool),
            self.get_foreign_keys_for_table(schema_name, table_name)
        );

        let column_rows = columns_result?;
        let foreign_keys = fks_result?;

        if column_rows.is_empty() {
            return Err(DbError::Introspection(format!(
                "Table {}.{} not found or has no columns",
                schema_name, table_name
            )));
        }

        let mut columns = Vec::new();
        let mut primary_key_columns = Vec::new();

        for row in column_rows {
            if row.is_primary_key {
                primary_key_columns.push(row.column_name.clone());
            }
            let foreign_key = foreign_keys.get(&row.column_name).cloned();

            columns.push(ColumnMetadata {
                name: row.column_name,
                sql_type_name: row.data_type.clone(),
                axion_type: self
                    .type_mapper
                    .sql_to_axion(&row.data_type, Some(&row.udt_name)),
                is_nullable: row.is_nullable.to_lowercase() == "yes",
                is_primary_key: row.is_primary_key,
                default_value: row.column_default,
                comment: row.column_comment,
                foreign_key,
            });
        }

        Ok(TableMetadata {
            name: table_name.to_string(),
            schema: schema_name.to_string(),
            columns,
            primary_key_columns,
            comment: None, // Table comments would require another small query
        })
    }

    // =================================== NEW METHODS ===================================

    #[instrument(skip(self, view_name), name = "introspect_view")]
    async fn introspect_view(&self, schema_name: &str, view_name: &str) -> DbResult<ViewMetadata> {
        let columns_query = r#"
            SELECT
                c.column_name::TEXT,
                c.data_type::TEXT,
                c.udt_name::TEXT,
                c.is_nullable::TEXT,
                c.column_default,
                pg_catalog.col_description((quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass::oid, c.ordinal_position) AS column_comment,
                -- Views do not have primary keys, so this is always false.
                false AS is_primary_key
            FROM information_schema.columns c
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position;
        "#;

        let definition_query = "
            SELECT view_definition::TEXT FROM information_schema.views 
            WHERE table_schema = $1 AND table_name = $2
        ";

        let (columns_result, definition_result) = tokio::join!(
            sqlx::query_as::<_, ColumnIntrospectionRow>(columns_query)
                .bind(schema_name)
                .bind(view_name)
                .fetch_all(&*self.client.pool),
            sqlx::query_scalar::<_, Option<String>>(definition_query)
                .bind(schema_name)
                .bind(view_name)
                .fetch_one(&*self.client.pool)
        );

        let column_rows = columns_result?;
        let definition = definition_result?;

        let columns = column_rows
            .into_iter()
            .map(|row| ColumnMetadata {
                name: row.column_name,
                sql_type_name: row.data_type.clone(),
                axion_type: self
                    .type_mapper
                    .sql_to_axion(&row.data_type, Some(&row.udt_name)),
                is_nullable: row.is_nullable.to_lowercase() == "yes",
                is_primary_key: false, // Views do not have primary keys
                default_value: row.column_default,
                comment: row.column_comment,
                foreign_key: None, // Views do not have foreign keys
            })
            .collect();

        Ok(ViewMetadata {
            name: view_name.to_string(),
            schema: schema_name.to_string(),
            columns,
            definition,
            comment: None, // View comments would require another query
        })
    }

    #[instrument(skip(self), name = "introspect_schema_enums")]
    async fn introspect_enums_for_schema(
        &self,
        schema_name: &str,
    ) -> DbResult<HashMap<String, EnumMetadata>> {
        let query = "
            SELECT
                t.typname::TEXT AS enum_name,
                e.enumlabel::TEXT AS enum_value
            FROM pg_catalog.pg_type t
            JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace
            JOIN pg_catalog.pg_enum e ON t.oid = e.enumtypid
            WHERE n.nspname = $1 AND t.typtype = 'e'
            ORDER BY enum_name, e.enumsortorder;
        ";

        let rows: Vec<EnumIntrospectionRow> = sqlx::query_as(query)
            .bind(schema_name)
            .fetch_all(&*self.client.pool)
            .await?;

        let mut enums = HashMap::new();
        for row in rows {
            enums
                .entry(row.enum_name.clone())
                .or_insert_with(|| EnumMetadata {
                    name: row.enum_name,
                    schema: schema_name.to_string(),
                    ..Default::default()
                })
                .values
                .push(row.enum_value);
        }
        Ok(enums)
    }
}
