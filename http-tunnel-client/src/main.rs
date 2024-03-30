use clap::Parser;
use futures::StreamExt;
use http::Uri;
use http_body_util::{BodyExt, StreamBody};
use http_tunnel_util::error::{BaseSchemeNotSupportedError, BoxError, UriMustHasSchemeError};
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt;
use tokio::{
    io::AsyncWrite,
    net::{TcpListener, TcpStream},
};

#[derive(Debug, clap::Parser)]
struct Args {
    connection: String,
    #[arg(long)]
    over: hyper::Uri,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    let base_scheme = args.over.scheme().ok_or(UriMustHasSchemeError)?;
    match base_scheme.as_str() {
        "http" | "https" => (),
        _ => return Err(BaseSchemeNotSupportedError(base_scheme.clone()))?,
    }
    println!(
        "starting connection {} over {}",
        &args.connection, args.over
    );

    let listener = TcpListener::bind(&args.connection).await?;
    let (client_stream, addr) = listener.accept().await?;
    println!("client connected: {}", addr);
    let (client_downstream, client_upstream) = client_stream.into_split();
    start_tunnnel(args.over, client_downstream, client_upstream).await
}

async fn start_tunnnel<
    D: tokio::io::AsyncRead + Send + 'static,
    U: AsyncWrite + std::marker::Unpin,
>(
    tunnel_uri: Uri,
    downstream: D,
    mut upstream: U,
) -> Result<(), BoxError> {
    println!("start tunnel over {}", tunnel_uri);
    let downstream = tokio_util::io::ReaderStream::new(downstream);
    let base_stream = TcpStream::connect((
        tunnel_uri.host().unwrap(),
        tunnel_uri.port_u16().unwrap_or(80),
    ))
    .await?;
    println!("tunnel connected.");
    let base_io = TokioIo::new(base_stream);
    let (mut sender, conn) = http1::handshake(base_io).await?;
    println!("tunnel handshake done.");
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });
    println!("start sending request.");
    let req = http::Request::builder()
        .uri(&tunnel_uri)
        .body(StreamBody::new(
            downstream.map(|e| e.map(|e| hyper::body::Frame::data(e))),
        ))?;

    println!("start receiving response.");
    let mut res = sender.send_request(req).await?;

    println!("tunnel established successfully.");

    while let Some(next) = res.frame().await {
        let frame = next?;
        if let Some(chunk) = frame.data_ref() {
            upstream.write_all(&chunk).await?;
        }
    }
    println!("connection closed.");

    Ok(())
}
