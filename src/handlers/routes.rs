use derive_more::{Display, Error};
use diesel::prelude::*;
use diesel::{self, r2d2::ConnectionManager};
use futures::{future::ok, stream::once, StreamExt};
use log::info;
use ntex::service;
use ntex::util::{Bytes, BytesMut};
use ntex::web::{self, Error};
use serde::{Deserialize, Serialize};

use crate::models;
use crate::models::employee::{Employee, NewEmployee};
use crate::repository::database;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Serialize)]
pub struct Response {
    status: String,
    message: String,
    data: Option<String>,
}

/// health check
#[web::get("/health")]
async fn health() -> Result<web::HttpResponse, Error> {
    Ok(web::HttpResponse::Ok().json(&Response {
        status: "success".to_string(),
        message: "Server is running".to_string(),
        data: None,
    }))
}

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
pub struct MyError {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl web::error::WebResponseError for MyError {}

#[derive(Serialize, Deserialize)]
struct Info {
    user_id: u32,
    friend: String,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

#[derive(Deserialize)]
struct MyInfo {
    name: String,
}

async fn index() -> web::HttpResponse {
    web::HttpResponse::Ok().body("Hello world!")
}

#[web::get("/error")]
async fn error() -> Result<&'static str, MyError> {
    let err = MyError { name: "test error" };
    info!("{}", err);
    Err(err)
}

/// extract path info from "/users/{user_id}/{friend}" url
/// {user_id} - deserializes to a u32
/// {friend} - deserializes to a String
#[web::get("/users/{user_id}/{friend}")]
async fn path(info: web::types::Path<Info>) -> Result<String, Error> {
    Ok(format!("Welcome {}! user_id:{}", info.friend, info.user_id))
}

/// extract query info from "/users/q?name={name}" url
/// {name} - deserializes to a String
#[web::get("/users/q")]
async fn query(info: web::types::Query<MyInfo>) -> Result<String, Error> {
    Ok(format!("Welcome {}!", info.name))
}

/// extract json info from "/users/json" url
/// {user_id} - deserializes to a u32
/// {friend} - deserializes to a String
async fn payload(mut payload: web::types::Payload) -> Result<web::HttpResponse, Error> {
    // payload is a stream of Bytes objects
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(web::error::ErrorBadRequest("overflow").into());
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<Info>(&body)?;
    Ok(web::HttpResponse::Ok().json(&obj))
} // <- send response}

/// extract json info from "/users/json" url
/// {user_id} - deserializes to a u32
/// {friend} - deserializes to a String

async fn json(info: web::types::Json<Info>) -> Result<web::types::Json<Info>, Error> {
    // let info = info.into_inner();
    // the following lines is equivalent to the above
    // let info = Info {
    //     user_id: info.user_id,
    //     friend: info.friend.clone(),
    // };

    // Ok(web::HttpResponse::Ok().json(&info))

    // Using the Json type this way instead of calling the .json method on a HttpResponse makes it immediately clear that the function returns JSON and not any other type of response.
    Ok(info)
}

/// stream response
#[web::get("/stream")]
async fn stream() -> web::HttpResponse {
    let body = once(ok::<_, web::Error>(Bytes::from_static(b"test")));

    web::HttpResponse::Ok().streaming(body)
}

// create a new employee
async fn create_employee(
    pool: web::types::State<DbPool>,
    employee: web::types::Json<NewEmployee>,
) -> Result<impl web::Responder, web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let mut employee = employee.into_inner();
    employee.created_at = Some(chrono::Local::now().naive_local());

    let new_employee = web::block(move || {
        // Obtaining a connection from the pool is also a potentially blocking operation.
        // So, it should be called within the `web::block` closure, as well.
        database::create_employee(&mut conn, employee)
    })
    .await
    .map_err(web::error::ErrorInternalServerError)?;

    Ok(web::HttpResponse::Ok().json(&new_employee))
}

// get a employee by id
#[web::get("/employee/{id}")]
async fn get_employee(
    pool: web::types::State<DbPool>,
    id: web::types::Path<i32>,
) -> Result<impl web::Responder, web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let employee = web::block(move || database::get_employee_by_id(&mut conn, id.into_inner()))
        .await
        .map_err(web::error::ErrorInternalServerError)?;

    Ok(web::HttpResponse::Ok().json(&employee))
}

// get all employees
#[web::get("/employees")]
async fn get_employees(pool: web::types::State<DbPool>) -> Result<impl web::Responder, web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let employees = web::block(move || database::get_all_employees(&mut conn))
        .await
        .map_err(web::error::ErrorInternalServerError)?;

    Ok(web::HttpResponse::Ok().json(&employees))
}

// update a employee by id
#[web::put("/employee/{id}")]
async fn update_employee(
    pool: web::types::State<DbPool>,
    id: web::types::Path<i32>,
    employee: web::types::Json<NewEmployee>,
) -> Result<impl web::Responder, web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let new_employee = models::employee::Employee {
        id: id.clone(),
        name: employee.name.clone(),
        created_at: employee
            .created_at
            .unwrap_or(chrono::Local::now().naive_local()),
    };

    let employee = web::block(move || {
        database::update_employee_by_id(&mut conn, id.into_inner(), new_employee)
    })
    .await
    .map_err(web::error::ErrorInternalServerError)?;

    Ok(web::HttpResponse::Ok().json(&employee))
}

// delete a employee by id
#[web::delete("/employee/{id}")]
async fn delete_employee(
    pool: web::types::State<DbPool>,
    id: web::types::Path<i32>,
) -> Result<impl web::Responder, web::Error> {
    let mut conn = pool.get().expect("couldn't get db connection from pool");

    let res = web::block(move || database::delete_employee_by_id(&mut conn, id.into_inner()))
        .await
        .map_err(web::error::ErrorInternalServerError)?;

    Ok(web::HttpResponse::Ok().json(&res))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health)
            // ...so this handles requests for `GET /app/index.html`
            .route("/index.html", web::get().to(index))
            .service(path)
            .service(query)
            .service(
                web::scope("/users")
                    .service(
                        // web::resource accepts a path pattern, guards, and one or more routes.
                        web::resource("/payload")
                            .name("payload")
                            .guard(web::guard::Header("content-type", "application/json"))
                            .route(web::post().to(payload)),
                    )
                    .service(
                        web::resource("/json")
                            .name("json")
                            .guard(web::guard::Header("content-type", "application/json"))
                            .route(web::post().to(json)),
                    ),
            )
            .service(stream)
            .service(error)
            // this handles requests for `GET /app/v1/resource`
            // with `text/plain` content type
            // and can be called with `GET` method
            // If a resource can not match any route, a "NOT FOUND" response is returned.
            .service(
                web::resource("/resource").route(
                    web::route()
                        .guard(web::guard::Get())
                        .guard(web::guard::Header("content-type", "text/plain"))
                        .to(|| async { web::HttpResponse::Ok().finish() }),
                ),
            )
            .service(
                web::resource("/employee")
                    .name("employee")
                    .route(web::post().to(create_employee)),
            )
            .service(get_employee)
            .service(get_employees)
            .service(update_employee)
            .service(delete_employee),
    );
}
