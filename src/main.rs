use clap::Parser;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
//use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug, Parser)]
#[command(name = "ociproxy")]
struct Cli {
    #[clap(long = "upstream")]
    upstream: String,
}

async fn proxy(upstream: Arc<String>, req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    println!("request: {req:?}");

    let (parts, _body) = req.into_parts();

    let mut builder = http::request::Builder::new()
        .method(parts.method.clone())
        .uri(format!("{}{}", upstream, parts.uri.clone()))
        .version(parts.version.clone());

    for (key, value) in parts.headers.iter() {
        builder = builder.header(key, value);
    }

    let new_req = builder.body(Body::empty()).unwrap();

    let client = hyper::Client::new();
    let res = client.request(new_req).await?;

    Ok(res)
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let Cli { upstream } = Cli::parse();
    let upstream = Arc::new(upstream);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));

    let make_svc = make_service_fn(move |_conn| {
        let upstream = Arc::clone(&upstream);
        async move { Ok::<_, anyhow::Error>(service_fn(move |req| proxy(Arc::clone(&upstream), req))) }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
