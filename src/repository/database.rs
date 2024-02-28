use ::r2d2::PooledConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;

use crate::models::employee::{Employee, NewEmployee};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn new() -> DbPool {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn create_employee(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
    employee: NewEmployee,
) -> diesel::QueryResult<Employee> {
    use crate::models::schema::employees::dsl::*;

    diesel::insert_into(employees)
        .values(&employee)
        .get_result(pool)
}
