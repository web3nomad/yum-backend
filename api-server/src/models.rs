use diesel::prelude::{
    Queryable,
    Selectable,
    Insertable,
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

#[derive(Insertable)]
#[diesel(table_name = crate::schema::tasks)]
pub struct NewTask<'a> {
    pub task_id: &'a str,
    pub params: &'a str,
    pub result: &'a str,
    pub callback_url: &'a str,
}
