use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr, Schema, Statement,
};
use sea_query::TableCreateStatement;

use crate::entities::{account, classroom, user};

pub async fn connect(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;

    if db.get_database_backend() == DatabaseBackend::Sqlite {
        db.execute(Statement::from_string(
            DatabaseBackend::Sqlite,
            "PRAGMA foreign_keys = ON",
        ))
        .await?;
    }

    Ok(db)
}

pub async fn init(db: &DatabaseConnection) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    create_table_if_not_exists(db, schema.create_table_from_entity(account::Entity)).await?;
    create_table_if_not_exists(db, schema.create_table_from_entity(classroom::Entity)).await?;
    create_table_if_not_exists(db, schema.create_table_from_entity(user::Entity)).await?;
    ensure_language_locked_column(db).await?;

    Ok(())
}

async fn create_table_if_not_exists(
    db: &DatabaseConnection,
    mut table: TableCreateStatement,
) -> Result<(), DbErr> {
    table.if_not_exists();
    let builder = db.get_database_backend();
    db.execute(builder.build(&table)).await?;
    Ok(())
}

async fn ensure_language_locked_column(db: &DatabaseConnection) -> Result<(), DbErr> {
    let backend = db.get_database_backend();
    let stmt = match backend {
        DatabaseBackend::Sqlite => Statement::from_string(
            DatabaseBackend::Sqlite,
            "ALTER TABLE classrooms ADD COLUMN language_locked BOOLEAN NOT NULL DEFAULT 0",
        ),
        DatabaseBackend::Postgres => Statement::from_string(
            DatabaseBackend::Postgres,
            "ALTER TABLE classrooms ADD COLUMN IF NOT EXISTS language_locked BOOLEAN NOT NULL DEFAULT FALSE",
        ),
        DatabaseBackend::MySql => Statement::from_string(
            DatabaseBackend::MySql,
            "ALTER TABLE classrooms ADD COLUMN language_locked BOOLEAN NOT NULL DEFAULT FALSE",
        ),
    };

    if let Err(err) = db.execute(stmt).await {
        // Ignore errors indicating the column already exists.
        let already_exists = matches!(
            backend,
            DatabaseBackend::Sqlite | DatabaseBackend::Postgres | DatabaseBackend::MySql
        ) && err.to_string().to_lowercase().contains("duplicate");

        if !already_exists {
            return Err(err);
        }
    }

    Ok(())
}
