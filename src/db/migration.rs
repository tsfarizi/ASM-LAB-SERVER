use crate::entities::{account, classroom, user};
use sea_orm::sea_query::{ColumnDef, Table};
use sea_orm::{ConnectionTrait, DbErr, Schema};

pub async fn run(db: &impl ConnectionTrait) -> Result<(), DbErr> {
    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    create_table_if_not_exists(db, schema.create_table_from_entity(account::Entity)).await?;
    create_table_if_not_exists(db, schema.create_table_from_entity(classroom::Entity)).await?;
    create_table_if_not_exists(db, schema.create_table_from_entity(user::Entity)).await?;

    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::LanguageLocked)
            .boolean()
            .not_null()
            .default(false)
            .to_owned(),
    )
    .await?;
    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::Tasks)
            .string()
            .not_null()
            .default("[]")
            .to_owned(),
    )
    .await?;
    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::IsExam)
            .boolean()
            .not_null()
            .default(false)
            .to_owned(),
    )
    .await?;
    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::TestCode)
            .string()
            .not_null()
            .default("")
            .to_owned(),
    )
    .await?;
    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::TimeLimit)
            .big_integer()
            .not_null()
            .default(0)
            .to_owned(),
    )
    .await?;
    add_column_if_not_exists(
        db,
        classroom::Entity,
        ColumnDef::new(classroom::Column::PresetupCode)
            .string()
            .not_null()
            .default("")
            .to_owned(),
    )
    .await?;

    add_column_if_not_exists(
        db,
        user::Entity,
        ColumnDef::new(user::Column::ExamStartedAt)
            .date_time()
            .null()
            .to_owned(),
    )
    .await?;

    add_column_if_not_exists(
        db,
        user::Entity,
        ColumnDef::new(user::Column::Active)
            .boolean()
            .not_null()
            .default(true)
            .to_owned(),
    )
    .await?;

    Ok(())
}

async fn add_column_if_not_exists(
    db: &impl ConnectionTrait,
    table: impl sea_orm::sea_query::IntoTableRef,
    mut column_def: ColumnDef,
) -> Result<(), DbErr> {
    let mut alter_table = Table::alter();
    alter_table.table(table).add_column(&mut column_def);

    let builder = db.get_database_backend();
    let query: String = builder.build(&alter_table).to_string();

    if let Err(err) = db.execute_unprepared(&query).await {
        let err_msg = err.to_string().to_lowercase();
        let already_exists =
            err_msg.contains("duplicate column") || err_msg.contains("already exists");

        if !already_exists {
            return Err(err);
        }
    }

    Ok(())
}

async fn create_table_if_not_exists(
    db: &impl ConnectionTrait,
    mut table: sea_query::TableCreateStatement,
) -> Result<(), DbErr> {
    table.if_not_exists();
    let builder = db.get_database_backend();
    db.execute_unprepared(&builder.build(&table).to_string()).await?;
    Ok(())
}
