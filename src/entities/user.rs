use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub classroom_id: i32,
    pub name: String,
    pub npm: String,
    pub code: String,
    pub active: bool,
    pub exam_started_at: Option<DateTimeUtc>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::classroom::Entity",
        from = "Column::ClassroomId",
        to = "super::classroom::Column::Id",
        on_delete = "Cascade"
    )]
    Classroom,
}

impl Related<super::classroom::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Classroom.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
