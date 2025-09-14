use std::net::{Ipv4Addr, SocketAddrV4};

use crate::app::new_app;

// enforce https on release to make sure no one forgets to use https
#[cfg(all(not(debug_assertions), not(feature = "https")))]
compile_error!("Feature `https` must be enabled on release.");

mod app;

#[cfg(not(debug_assertions))]
const PORT: u16 = 443;
#[cfg(debug_assertions)]
const PORT: u16 = 8080;

#[cfg(debug_assertions)]
const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT);
#[cfg(not(debug_assertions))]
const ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, PORT);

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv_override(); // it doesn't matter if there isnt a .env
    #[cfg(not(feature = "https"))]
    http_main().await;
    #[cfg(feature = "https")]
    https_main().await;
}

#[cfg(not(feature = "https"))]
async fn http_main() {
    let app = new_app();

    axum_server::bind(std::net::SocketAddr::V4(ADDR))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "https")]
async fn https_main() {
    use axum_server::tls_rustls::RustlsConfig;

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        dotenvy::var("HTTPS_CERT_PATH").expect("Environment variable `HTTPS_CERT_PATH` not found.\nCreate it in a .env within the cwd or in the environment"),
        dotenvy::var("HTTPS_KEY_PATH").expect("Environment variable `HTTPS_KEY_PATH` not found.\nCreate it in a .env within the cwd or in the environment"),
    )
    .await
    .unwrap();

    let app = new_app();

    // run https server
    axum_server::bind_rustls(std::net::SocketAddr::V4(ADDR), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
