// axion-db/src/client.rs
use crate::config::{DatabaseType, DbConfig};
use crate::error::{DbError, DbResult};
use crate::prelude::{
    ColumnMetadata, DatabaseMetadata, ForeignKeyReference, SchemaMetadata, TableMetadata,
};
use sqlx::any::AnyPoolOptions;
use sqlx::{AnyPool, Column, Connection};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

#[derive(Clone, Debug)]
pub struct DbClient {
    pub pool: Arc<AnyPool>,
    pub config: Arc<DbConfig>,
}

impl DbClient {
    pub async fn new(config: DbConfig) -> DbResult<Self> {
        info!("Initializing DbClient with config: {:?}", config.db_type);
        let cs = config.build_connection_string()?;
        debug!("Constructed connection string: [REDACTED]"); // Don't log full CS with password

        let mut pool_options = AnyPoolOptions::new();

        if let Some(pool_config) = &config.pool_options {
            if let Some(max_conn) = pool_config.max_connections {
                pool_options = pool_options.max_connections(max_conn);
            }
            if let Some(min_conn) = pool_config.min_connections {
                pool_options = pool_options.min_connections(min_conn);
            }
            // todo: Handle connect_timeout_seconds if needed...
            // if let Some(timeout_sec) = pool_config.connect_timeout_seconds {
            //     pool_options = pool_options.connect_timeout(Duration::from_secs(timeout_sec));
            // }
            if let Some(idle_timeout_sec) = pool_config.idle_timeout_seconds {
                pool_options = pool_options.idle_timeout(Duration::from_secs(idle_timeout_sec));
            }
            if let Some(max_lifetime_sec) = pool_config.max_lifetime_seconds {
                pool_options = pool_options.max_lifetime(Duration::from_secs(max_lifetime_sec));
            }
            if let Some(acquire_timeout_sec) = pool_config.acquire_timeout_seconds {
                pool_options =
                    pool_options.acquire_timeout(Duration::from_secs(acquire_timeout_sec));
            }
            if let Some(test_before_acquire) = pool_config.test_before_acquire {
                pool_options = pool_options.test_before_acquire(test_before_acquire);
            }
        }

        debug!("Connecting to database with type: {:?}", config.db_type);
        let pool = pool_options.connect(&cs).await?;
        info!(
            "Successfully connected to database: {:?}",
            config.database_name.as_deref().unwrap_or("default")
        );

        Ok(Self {
            pool: Arc::new(pool),
            config: Arc::new(config),
        })
    }

    pub async fn test_connection(&self) -> DbResult<()> {
        info!("Pinging database...");
        let mut conn = self.pool.acquire().await?;
        conn.ping().await?;
        info!("Database ping successful.");
        Ok(())
    }

    pub async fn get_db_version(&self) -> DbResult<String> {
        debug!("Fetching database version...");
        let mut conn = self.pool.acquire().await?;
        let query = match self.config.db_type {
            DatabaseType::Postgres => "SELECT version()",
            DatabaseType::Mysql => "SELECT VERSION()",
            DatabaseType::Sqlite => "SELECT sqlite_version()",
        };

        let (version,): (String,) = sqlx::query_as(query)
            .fetch_one(&mut *conn) // Dereference to get &mut <SpecificConnection>
            .await?;
        debug!("Database version: {}", version);
        Ok(version)
    }

    // Basic raw query execution for fetching results as JSON
    // This is a simplified version. Robust error handling and type mapping would be needed.
    pub async fn execute_raw_query_for_json(
        &self,
        query_str: &str,
    ) -> DbResult<Vec<serde_json::Value>> {
        use sqlx::Row;
        debug!("Executing raw query: {}", query_str);

        let rows: Vec<sqlx::any::AnyRow> = sqlx::query(query_str)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(DbError::QueryExecution)?;

        let mut results = Vec::new();
        for row in rows {
            let mut map = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                // Attempt to get value as common types. This is very basic.
                // sqlx provides more specific `try_get` for various types.
                let value = if let Ok(v_str) = row.try_get::<Option<String>, _>(i) {
                    serde_json::to_value(v_str).unwrap_or(serde_json::Value::Null)
                } else if let Ok(v_i64) = row.try_get::<Option<i64>, _>(i) {
                    serde_json::to_value(v_i64).unwrap_or(serde_json::Value::Null)
                } else if let Ok(v_f64) = row.try_get::<Option<f64>, _>(i) {
                    serde_json::to_value(v_f64).unwrap_or(serde_json::Value::Null)
                } else if let Ok(v_bool) = row.try_get::<Option<bool>, _>(i) {
                    serde_json::to_value(v_bool).unwrap_or(serde_json::Value::Null)
                }
                // Add more type checks as needed (chrono, uuid, etc.)
                else {
                    warn!(
                        "Could not determine type for column '{}' at index {}",
                        column.name(),
                        i
                    );
                    serde_json::Value::Null
                };
                map.insert(column.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(map));
        }
        debug!("Query returned {} rows.", results.len());
        Ok(results)
    }
}

// * THE STUFF BELOW WILL CHANGE IT'S NOT READY YET

// Add these methods to impl DbClient in axion-db/src/client.rs
use sqlx::Row;

impl DbClient {
    fn map_pg_type_to_internal_axion_type(
        &self,
        pg_type_name: &str,
        udt_name: Option<&str>,
    ) -> String {
        // Prioritize UDT name if available and not empty (for custom enums, composites)
        if let Some(udt) = udt_name {
            if !udt.is_empty() {
                // For array UDTs like _my_enum, strip leading underscore
                return udt.strip_prefix('_').unwrap_or(udt).to_string();
            }
        }

        let lower_pg_type = pg_type_name.to_lowercase();
        match lower_pg_type.as_str() {
            "integer" | "int" | "int4" => "integer".to_string(),
            "bigint" | "int8" => "bigint".to_string(),
            "smallint" | "int2" => "smallint".to_string(),
            "character varying" | "varchar" => "varchar".to_string(),
            "char" | "bpchar" => "char".to_string(), // Fixed-length char
            "text" | "name" => "text".to_string(),   // NAME is often treated as text
            "boolean" | "bool" => "boolean".to_string(),
            "date" => "date".to_string(),
            "timestamp without time zone" | "timestamp" => "timestamp".to_string(),
            "timestamp with time zone" | "timestamptz" => "timestamptz".to_string(),
            "time without time zone" | "time" => "time".to_string(),
            "time with time zone" | "timetz" => "timetz".to_string(),
            "numeric" | "decimal" => "numeric".to_string(),
            "real" | "float4" => "real".to_string(),
            "double precision" | "float8" => "double_precision".to_string(),
            "bytea" => "bytea".to_string(),
            "uuid" => "uuid".to_string(),
            "json" => "json".to_string(),
            "jsonb" => "jsonb".to_string(),
            "inet" => "inet".to_string(),
            "cidr" => "cidr".to_string(),
            "macaddr" => "macaddr".to_string(),
            "macaddr8" => "macaddr8".to_string(),
            "interval" => "interval".to_string(),
            "point" => "point".to_string(),
            "line" => "line".to_string(),
            "lseg" => "lseg".to_string(),
            "box" => "box_type".to_string(), // Avoid conflict with Rust's Box
            "path" => "path_type".to_string(),
            "polygon" => "polygon".to_string(),
            "circle" => "circle".to_string(),
            "money" => "money".to_string(),
            "bit" => "bit".to_string(),
            "bit varying" | "varbit" => "varbit".to_string(),
            "xml" => "xml".to_string(),
            "tsvector" => "tsvector".to_string(),
            "tsquery" => "tsquery".to_string(),
            // Handle array types by stripping '[]' and recursing
            s if s.ends_with("[]") => {
                let element_type = s.trim_end_matches("[]");
                // For UDT arrays like `_myenum`, the `pg_type_name` might be `myenum` but `udt_name` is `_myenum`.
                // We need to ensure we map the base type correctly.
                let element_udt_name =
                    if udt_name.map_or(false, |u| u.starts_with('_') && u.ends_with("[]")) {
                        udt_name.map(|u| u.strip_prefix('_').unwrap_or(u).trim_end_matches("[]"))
                    } else {
                        None
                    };
                format!(
                    "{}[]",
                    self.map_pg_type_to_internal_axion_type(element_type, element_udt_name)
                )
            }
            // Fallback for other UDTs (composites, domains not handled above) or unmapped built-ins
            _ => udt_name.unwrap_or(pg_type_name).to_string(),
        }
    }

    fn map_internal_axion_type_to_rust_type_string(
        &self,
        axion_type: &str,
        is_nullable: bool,
    ) -> String {
        let base_rust_type = match axion_type.to_lowercase().as_str() {
            "integer" => "i32".to_string(),
            "bigint" => "i64".to_string(),
            "smallint" => "i16".to_string(),
            "varchar" | "text" | "char" | "name" | "citext" => "String".to_string(),
            "boolean" => "bool".to_string(),
            "date" => "chrono::NaiveDate".to_string(),
            "timestamp" => "chrono::NaiveDateTime".to_string(),
            "timestamptz" => "chrono::DateTime<chrono::Utc>".to_string(),
            "time" => "chrono::NaiveTime".to_string(),
            "timetz" => "sqlx::postgres::types::PgTimeTz".to_string(),
            "numeric" | "decimal" => "sqlx::types::Decimal".to_string(), // rust_decimal
            "real" => "f32".to_string(),
            "double_precision" => "f64".to_string(),
            "bytea" => "Vec<u8>".to_string(),
            "uuid" => "uuid::Uuid".to_string(),
            "json" | "jsonb" => "serde_json::Value".to_string(),
            "inet" | "cidr" => "std::net::IpAddr".to_string(), // or ipnetwork::IpNetwork
            "macaddr" | "macaddr8" => "sqlx::types::mac_address::MacAddress".to_string(),
            "interval" => "sqlx::postgres::types::PgInterval".to_string(),
            "point" => "sqlx::postgres::types::PgPoint".to_string(),
            "line" => "sqlx::postgres::types::PgLine".to_string(),
            "lseg" => "sqlx::postgres::types::PgLSeg".to_string(),
            "box_type" => "sqlx::postgres::types::PgBox".to_string(),
            "path_type" => "sqlx::postgres::types::PgPath".to_string(),
            "polygon" => "sqlx::postgres::types::PgPolygon".to_string(),
            "circle" => "sqlx::postgres::types::PgCircle".to_string(),
            "money" => "sqlx::postgres::types::PgMoney".to_string(),
            "bit" | "varbit" => "sqlx::types::BitVec".to_string(), // bit_vec crate
            "xml" => "String".to_string(),                         // Typically handled as string
            "tsvector" | "tsquery" => "String".to_string(), // Specialized types exist but String is a safe default
            s if s.ends_with("[]") => {
                let element_axion_type = s.trim_end_matches("[]");
                let element_rust_type =
                    self.map_internal_axion_type_to_rust_type_string(element_axion_type, false);
                format!("Vec<{}>", element_rust_type)
            }
            // For custom enums or other UDTs, the codegen would create specific Rust types
            // For now, we'll placeholder them as String, but ideally codegen would map them
            // to `crate::generated_models::enums::MyEnum` etc.
            custom_type => {
                // A simple heuristic: if it looks like a known array base type, map it.
                // This is basic and should be improved by checking actual enum/composite defs.
                if custom_type.starts_with('_') && custom_type.ends_with("vector") {
                    // Example, not real PG
                    format!(
                        "Vec<{}>",
                        custom_type
                            .trim_start_matches('_')
                            .trim_end_matches("vector")
                    )
                } else {
                    // Convert to PascalCase for a potential Rust type name
                    // This is a very naive conversion and might need a proper library for complex cases
                    let mut pascal_case_name = String::new();
                    let mut capitalize_next = true;
                    for c in custom_type.chars() {
                        if c == '_' {
                            capitalize_next = true;
                        } else if capitalize_next {
                            pascal_case_name.push(c.to_ascii_uppercase());
                            capitalize_next = false;
                        } else {
                            pascal_case_name.push(c);
                        }
                    }
                    // Prepend with a common namespace for generated types or just use the name
                    format!("crate::models::enums::{}", pascal_case_name) // Assuming enums might live here
                }
            }
        };

        if is_nullable && !base_rust_type.starts_with("Option<") {
            format!("Option<{}>", base_rust_type)
        } else {
            base_rust_type
        }
    }

    pub async fn get_table_metadata(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> DbResult<TableMetadata> {
        if self.config.db_type != DatabaseType::Postgres {
            return Err(DbError::UnsupportedDbType(
                "PostgreSQL only for this get_table_metadata implementation".to_string(),
            ));
        }
        tracing::debug!(
            "Fetching metadata for table: {}.{}",
            schema_name,
            table_name
        );

        let query = r#"
            SELECT
                c.column_name,
                c.ordinal_position,
                c.data_type,          -- Standard SQL type (e.g., "character varying", "integer")
                c.udt_name,           -- Underlying UDT name (e.g., "varchar", "int4", custom enum/composite name)
                c.is_nullable,
                c.column_default,
                c.character_maximum_length,
                c.numeric_precision,
                c.numeric_scale,
                pg_catalog.col_description(
                    (quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass::oid,
                    c.ordinal_position
                ) as column_comment,
                (
                    SELECT pg_catalog.obj_description(cls.oid, 'pg_class')
                    FROM pg_catalog.pg_class cls
                    JOIN pg_catalog.pg_namespace n ON n.oid = cls.relnamespace
                    WHERE n.nspname = c.table_schema AND cls.relname = c.table_name
                ) as table_comment,
                EXISTS (
                    SELECT 1
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.constraint_schema = kcu.constraint_schema
                    WHERE tc.table_schema = c.table_schema
                        AND tc.table_name = c.table_name
                        AND kcu.column_name = c.column_name
                        AND tc.constraint_type = 'PRIMARY KEY'
                ) as is_primary_key,
                EXISTS (
                    SELECT 1
                    FROM information_schema.table_constraints tc
                    JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.constraint_schema = kcu.constraint_schema
                    WHERE tc.table_schema = c.table_schema
                        AND tc.table_name = c.table_name
                        AND kcu.column_name = c.column_name
                        AND tc.constraint_type = 'UNIQUE'
                ) as is_unique,
                fk.foreign_table_schema,
                fk.foreign_table_name,
                fk.foreign_column_name
            FROM
                information_schema.columns c
            LEFT JOIN (
                SELECT
                    kcu.table_schema,
                    kcu.table_name,
                    kcu.column_name,
                    ccu.table_schema AS foreign_table_schema,
                    ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name
                FROM
                    information_schema.table_constraints AS tc
                JOIN
                    information_schema.key_column_usage AS kcu
                    ON tc.constraint_name = kcu.constraint_name AND tc.constraint_schema = kcu.constraint_schema
                JOIN
                    information_schema.constraint_column_usage AS ccu
                    ON ccu.constraint_name = tc.constraint_name AND ccu.constraint_schema = tc.constraint_schema
                WHERE
                    tc.constraint_type = 'FOREIGN KEY'
            ) AS fk ON c.table_schema = fk.table_schema AND c.table_name = fk.table_name AND c.column_name = fk.column_name
            WHERE
                c.table_schema = $1 AND c.table_name = $2
            ORDER BY
                c.ordinal_position;
        "#;

        let rows = sqlx::query(query)
            .bind(schema_name)
            .bind(table_name)
            .fetch_all(self.pool.as_ref())
            .await?;

        if rows.is_empty() {
            return Err(DbError::Introspection(format!(
                "Table or view {}.{} not found or has no columns",
                schema_name, table_name
            )));
        }

        let mut columns_meta = Vec::new();
        let mut pks = Vec::new();
        let mut table_comment_opt: Option<String> = None;

        for row in rows {
            let column_name: String = row.try_get("column_name")?;
            let sql_data_type_standard: String = row.try_get("data_type")?; // e.g. "character varying", "ARRAY"
            let udt_name: Option<String> = row.try_get("udt_name").ok(); // e.g. "varchar", "_int4" (for int4[])
            let is_nullable_str: String = row.try_get("is_nullable")?;
            let is_nullable = is_nullable_str.to_lowercase() == "yes";

            let internal_axion_type = self
                .map_pg_type_to_internal_axion_type(&sql_data_type_standard, udt_name.as_deref());
            let rust_type_string =
                self.map_internal_axion_type_to_rust_type_string(&internal_axion_type, is_nullable);

            let is_primary_key: bool = row.try_get("is_primary_key")?;
            if is_primary_key {
                pks.push(column_name.clone());
            }

            let foreign_key_reference = match (
                row.try_get("foreign_table_schema").ok(),
                row.try_get("foreign_table_name").ok(),
                row.try_get("foreign_column_name").ok(),
            ) {
                (Some(fts), Some(ftn), Some(fcn)) => Some(ForeignKeyReference {
                    foreign_table_schema: fts,
                    foreign_table_name: ftn,
                    foreign_column_name: fcn,
                }),
                _ => None,
            };

            if table_comment_opt.is_none() {
                table_comment_opt = row.try_get("table_comment").ok().flatten();
            }

            columns_meta.push(ColumnMetadata {
                name: column_name,
                ordinal_position: row.try_get("ordinal_position")?,
                sql_type_name: sql_data_type_standard,
                udt_name,
                internal_axion_type,
                rust_type_string,
                is_nullable,
                is_primary_key,
                is_unique: row.try_get("is_unique")?,
                default_value: row.try_get("column_default").ok(),
                foreign_key_reference,
                character_maximum_length: row.try_get("character_maximum_length").ok(),
                numeric_precision: row.try_get("numeric_precision").ok(),
                numeric_scale: row.try_get("numeric_scale").ok(),
                comment: row.try_get("column_comment").ok().flatten(),
            });
        }

        Ok(TableMetadata {
            schema: schema_name.to_string(),
            name: table_name.to_string(),
            columns: columns_meta,
            primary_key_columns: pks,
            comment: table_comment_opt,
        })
    }

    pub async fn list_tables_in_schema(&self, schema_name: &str) -> DbResult<Vec<String>> {
        if self.config.db_type != DatabaseType::Postgres {
            return Err(DbError::UnsupportedDbType(
                "PostgreSQL only for list_tables_in_schema".to_string(),
            ));
        }
        tracing::debug!("Listing tables in schema: {}", schema_name);
        let query = "SELECT table_name FROM information_schema.tables \
                     WHERE table_schema = $1 AND table_type = 'BASE TABLE' \
                     ORDER BY table_name";
        let rows: Vec<(String,)> = sqlx::query_as(query)
            .bind(schema_name)
            .fetch_all(self.pool.as_ref())
            .await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn list_all_schemas(&self, include_system_schemas: bool) -> DbResult<Vec<String>> {
        if self.config.db_type != DatabaseType::Postgres {
            return Err(DbError::UnsupportedDbType(
                "PostgreSQL only for list_all_schemas".to_string(),
            ));
        }
        tracing::debug!(
            "Listing all schemas (include_system: {})",
            include_system_schemas
        );
        let query = if include_system_schemas {
            "SELECT schema_name FROM information_schema.schemata ORDER BY schema_name"
        } else {
            "SELECT schema_name FROM information_schema.schemata \
             WHERE schema_name NOT IN ('pg_catalog', 'information_schema') \
             AND schema_name NOT LIKE 'pg_toast%' \
             AND schema_name NOT LIKE 'pg_temp_%' \
             ORDER BY schema_name"
        };
        let rows: Vec<(String,)> = sqlx::query_as(query).fetch_all(self.pool.as_ref()).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn get_full_database_metadata(
        &self,
        schemas_to_include: Option<Vec<String>>,
    ) -> DbResult<DatabaseMetadata> {
        info!("Fetching full database metadata...");
        let mut db_meta = DatabaseMetadata::default();

        let schemas_to_query = match schemas_to_include {
            Some(s_vec) if !s_vec.is_empty() => s_vec,
            _ => self.list_all_schemas(false).await?, // Default to non-system schemas
        };

        debug!("Processing schemas: {:?}", schemas_to_query);

        for schema_name in schemas_to_query {
            tracing::info!("Introspecting schema: {}", schema_name);
            let mut schema_meta = SchemaMetadata {
                name: schema_name.clone(),
                ..Default::default()
            };

            match self.list_tables_in_schema(&schema_name).await {
                Ok(table_names) => {
                    debug!(
                        "Found {} tables in schema '{}'",
                        table_names.len(),
                        schema_name
                    );
                    for table_name in table_names {
                        match self.get_table_metadata(&schema_name, &table_name).await {
                            Ok(table_md) => {
                                schema_meta.tables.insert(table_name.clone(), table_md);
                                debug!(
                                    "Successfully fetched metadata for table '{}.{}'",
                                    schema_name, table_name
                                );
                            }
                            Err(e) => warn!(
                                "Error fetching metadata for table {}.{}: {}",
                                schema_name, table_name, e
                            ),
                        }
                    }
                }
                Err(e) => warn!("Error listing tables for schema {}: {}", schema_name, e),
            }

            // TODO: Populate views, functions, enums for schema_meta in a similar fashion
            // For example:
            // match self.list_views_in_schema(&schema_name).await {
            //     Ok(view_names) => { /* ... fetch metadata for each view ... */ },
            //     Err(e) => warn!("Error listing views for schema {}: {}", schema_name, e),
            // }
            // ... and so on for functions and enums

            db_meta.schemas.insert(schema_name.clone(), schema_meta);
            info!("Finished introspecting schema: {}", schema_name);
        }
        info!("Full database metadata fetch complete.");
        Ok(db_meta)
    }
}
