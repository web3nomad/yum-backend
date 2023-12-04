use diesel::prelude::{
    Queryable,
    Selectable,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::tasks)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Task {
    pub id: u32,
    pub task_id: String,
    pub params: String,
    pub result: String,
    pub starts_at: Option<chrono::NaiveDateTime>,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub callback_url: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
