use ntex::web;

mod handlers;
mod models;
mod repository;

#[ntex::main]
async fn main() -> std::io::Result<()> {
    // enable logger
    std::env::set_var("RUST_LOG", "ntex=info,diesel=debug");
    env_logger::init();

    // set up database connection pool
    let pool = repository::database::new();
    // web::HttpServer can be shutdown gracefully.
    web::HttpServer::new(move || {
        let logger = web::middleware::Logger::default();

        web::App::new()
            // set up DB pool to be used with web::State<Pool> extractor
            .state(pool.clone())
            // enable logger
            .wrap(logger)
            // enable default headers
            .wrap(web::middleware::DefaultHeaders::new().header("content-type", "application/json"))
            // enable Compression, A response's Content-Encoding header defaults to ContentEncoding::Auto, which performs automatic content compression negotiation based on the request's Accept-Encoding header.
            // should add "compress" feature to the Cargo.toml
            .wrap(web::middleware::Compress::default())
            .configure(handlers::routes::config)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
