use std::net::{Ipv4Addr, SocketAddr};

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{error, info};
use sqlx::postgres::PgPoolOptions;
use svc::{
    app::config::{self, ArgType, Args},
    routes::router,
    utils,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let args = Args::parse();
    match args.arg_type {
        ArgType::Run => {
            info!("configuration file: {}", &args.config_filename);
            let cfg = config::load(&args.config_filename);
            let db_string = cfg.db.db_string();

            let db_pool = match PgPoolOptions::new()
                .max_connections(10)
                .connect(&db_string)
                .await
            {
                Ok(pool) => {
                    info!("connection to the database is successful!");
                    pool
                }
                Err(err) => {
                    error!("failed to connect to the database: {:?}", err);
                    std::process::exit(1);
                }
            };

            let host = match utils::ip_string_to_array(&cfg.listen.host) {
                Some(ip_array) => ip_array,
                None => {
                    error!("invalid IP address format");
                    // Handle the error case gracefully, e.g., set a default value or exit the program
                    std::process::exit(1);
                }
            };

            let ip_addr = Ipv4Addr::new(host[0], host[1], host[2], host[3]);
            let port_ref = &cfg.listen.port;
            let port: u16 = *port_ref as u16;
            let addr = SocketAddr::from((ip_addr, port));
            let listener = TcpListener::bind(addr).await?;

            match sqlx::migrate!("src/database/migrations")
                .run(&db_pool)
                .await
            {
                Ok(_) => {
                    info!("success to apply migration to the database");
                }
                Err(err) => {
                    error!("failed to apply migration to the database: {:?}", err);
                    std::process::exit(1);
                }
            };

            info!("server started successfully on: {}", addr);
            loop {
                let (stream, _) = listener.accept().await?;
                let io = TokioIo::new(stream);
                let pool_clone = db_pool.clone();
                tokio::task::spawn(async move {
                    if let Err(err) = http1::Builder::new()
                        .serve_connection(
                            io,
                            service_fn(move |req| router(req, pool_clone.clone())),
                        )
                        .await
                    {
                        println!("error serving connection: {:?}", err);
                    }
                });
            }
        }
        ArgType::Others => todo!(),
    }
}
