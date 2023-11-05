pub mod handlers;

use actix_web::{
    middleware::Logger,
    web::{get, put, scope, Data},
    App, HttpServer,
};
use mongodb::Client;
use nb_from_env::{FromEnv, FromEnvDerive};
use sms_verification_code_service::{
    core::service::Service, generators::random_generator::RandomGenerator,
    senders::fake_sender::FakeSender, stores::mongo_store::MongoStore,
};
use std::io;

#[derive(FromEnvDerive)]
struct Config {
    mongo_uri: String,
    mongo_database: String,
    mongo_collection: String,
    listen_address: String,
    #[env_default("info")]
    log_level: String,
    #[env_default("%t %r %s %D")]
    log_format: String,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv::dotenv().ok();
    let config = Config::from_env();
    let collection = Client::with_uri_str(config.mongo_uri)
        .await
        .expect("failed to connect to mongodb")
        .database(&config.mongo_database)
        .collection(&config.mongo_collection);

    let service = Data::new(Service::new(
        60,
        300,
        FakeSender,
        MongoStore::new(collection),
        RandomGenerator,
    ));

    env_logger::init_from_env(env_logger::Env::default().default_filter_or(config.log_level));
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::new(&config.log_format))
            .app_data(service.clone())
            .service(
                scope("")
                    .route(
                        "/phones/{phone}/codes",
                        put().to(handlers::send_code::<FakeSender, MongoStore, RandomGenerator>),
                    )
                    .route(
                        "/phones/{phone}/codes/{code}/verification",
                        put().to(handlers::verify_code::<FakeSender, MongoStore, RandomGenerator>),
                    ),
            )
    })
    .bind(config.listen_address)?
    .run()
    .await
}
