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

// create a new employee
pub fn create_employee(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
    employee: NewEmployee,
) -> diesel::QueryResult<Employee> {
    use crate::models::schema::employees::dsl::*;

    diesel::insert_into(employees)
        .values(&employee)
        .get_result(pool)
}

// get a employee by id
pub fn get_employee_by_id(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
    employee_id: i32,
) -> diesel::QueryResult<Employee> {
    use crate::models::schema::employees::dsl::*;

    employees.find(employee_id).first(pool)
}

// get all employees
pub fn get_all_employees(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
) -> diesel::QueryResult<Vec<Employee>> {
    use crate::models::schema::employees::dsl::*;

    employees.load(pool)
}

// update a employee by id
pub fn update_employee_by_id(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
    employee_id: i32,
    employee: Employee,
) -> diesel::QueryResult<Employee> {
    use crate::models::schema::employees::dsl::*;

    diesel::update(employees.find(employee_id))
        .set(&employee)
        .get_result(pool)
}

// delete a employee by id
pub fn delete_employee_by_id(
    pool: &mut PooledConnection<ConnectionManager<PgConnection>>,
    employee_id: i32,
) -> diesel::QueryResult<usize> {
    use crate::models::schema::employees::dsl::*;

    diesel::delete(employees.find(employee_id)).execute(pool)
}
