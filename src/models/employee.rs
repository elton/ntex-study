use diesel::{AsChangeset, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize, Debug, Clone, AsChangeset, Insertable)]
#[diesel(table_name=crate::models::schema::employees)]
pub struct Employee {
    pub id: i32,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Deserialize, Serialize, Debug, Clone, Insertable)]
#[diesel(table_name=crate::models::schema::employees)]
pub struct NewEmployee {
    pub name: String,
    pub created_at: Option<chrono::NaiveDateTime>,
}
