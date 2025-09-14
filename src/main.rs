use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use crate::app::new_app;

// enforce https on release to make sure no one forgets to use https
#[cfg(all(not(debug_assertions), not(feature = "https")))]
compile_error!("Feature `https` must be enabled on release.");

mod app;

#[allow(dead_code)]
struct Ports {
    http: u16,
    https: u16,
}

#[cfg(not(debug_assertions))]
const PORTS: Ports = Ports {
    http: 80,
    https: 443,
};
#[cfg(debug_assertions)]
const PORTS: Ports = Ports {
    http: 8080,
    https: 4430,
};

#[cfg(debug_assertions)]
const ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
#[cfg(not(debug_assertions))]
const ADDR: Ipv4Addr = Ipv4Addr::UNSPECIFIED;

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

    axum_server::bind(SocketAddr::V4(SocketAddrV4::new(ADDR, PORTS.http)))
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

    tokio::spawn(redirect_http_to_https(PORTS));

    let app = new_app();

    // run https server
    axum_server::bind_rustls(SocketAddr::V4(SocketAddrV4::new(ADDR, PORTS.https)), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(feature = "https")]
async fn redirect_http_to_https(ports: Ports) {
    use axum::{
        BoxError,
        handler::HandlerWithoutStateExt,
        http::{Uri, uri::Authority},
    };
    use axum_extra::extract::Host;

    fn make_https(host: &str, uri: Uri, https_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        let authority: Authority = host.parse()?;
        let bare_host = match authority.port() {
            Some(port_struct) => authority
                .as_str()
                .strip_suffix(port_struct.as_str())
                .unwrap()
                .strip_suffix(':')
                .unwrap(), // if authority.port() is Some(port) then we can be sure authority ends with :{port}
            None => authority.as_str(),
        };

        parts.authority = Some(format!("{bare_host}:{https_port}").parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        use axum::response::Redirect;

        match make_https(&host, uri, ports.https) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(_) => {
                use axum::http::StatusCode;

                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::V4(SocketAddrV4::new(ADDR, ports.http));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, redirect.into_make_service())
        .await
        .unwrap();
}
