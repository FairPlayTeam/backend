use std::{
    env::VarError,
    net::{Ipv4Addr, SocketAddrV4},
};

use tokio::net::TcpListener;

use crate::app::new_app;

mod app;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv_override(); // it doesn't matter if there isnt a .env
    let app = new_app();

    let port = dotenvy::var("HTTP_PORT")
        .map(|x| x.parse().unwrap())
        .unwrap_or_else(|err| match &err {
            dotenvy::Error::EnvVar(VarError::NotPresent) => {
                if cfg!(debug_assertions) {
                    8080
                } else {
                    80
                }
            }
            _ => panic!("{err}"),
        });

    let addr = dotenvy::var("HTTP_ADDR")
        .map(|x| x.parse().unwrap())
        .unwrap_or_else(|err| match &err {
            dotenvy::Error::EnvVar(VarError::NotPresent) => Ipv4Addr::LOCALHOST,
            _ => panic!("{err}"),
        });

    let listener = TcpListener::bind(SocketAddrV4::new(addr, port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
