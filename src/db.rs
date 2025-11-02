pub mod migration;

use sea_orm::{
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr,Statement,
};
use sea_query::TableCreateStatement;

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
    migration::run(db).await
}

#[allow(dead_code)]
pub(crate) async fn create_table_if_not_exists(
    db: &impl ConnectionTrait,
    mut table: TableCreateStatement,
) -> Result<(), DbErr> {
    table.if_not_exists();
    let builder = db.get_database_backend();
    db.execute(builder.build(&table)).await?;
    Ok(())
}