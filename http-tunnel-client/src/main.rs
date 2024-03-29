mod error;

use crate::error::{
    BaseSchemeNotSupportedError, ConnectionSchemeNotSupportedError, UriMustHasSchemeError,
};
use clap::Parser;
use futures::stream;
use http_body_util::{Empty, StreamBody};
use hyper::{body::Bytes, client::conn::http1};
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream},
    pin,
};

#[derive(Debug, clap::Parser)]
struct Args {
    connection: String,
    #[arg(long)]
    over: hyper::Uri,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let (listener, client) = futures::future::join(
        TcpListener::bind(&args.connection),
        TcpStream::connect((
            args.over.host().unwrap(),
            args.over.port_u16().unwrap_or(80),
        )),
    )
    .await;

    let listener = listener?;
    let base_stream = client?;

    let (client_stream, addr) = listener.accept().await?;
    let (client_downstream, client_upstream) = client_stream.into_split();
    let client_downstream = tokio_util::io::ReaderStream::new(client_downstream);

    let base_io = TokioIo::new(base_stream);
    let (mut sender, conn) = http1::handshake(base_io).await?;
    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection failed: {:?}", err);
        }
    });
    let req = http::Request::builder()
        .uri(&args.over)
        .body(StreamBody::new(TokioReaderStream(client_downstream)))?;

    let res = sender.send_request(req).await?;

    println!("Res {}", res.status());

    Ok(())
}
struct TokioReaderStream(tokio_util::io::ReaderStream<tokio::net::tcp::OwnedReadHalf>);

impl futures::Stream for TokioReaderStream {
    type Item = Result<hyper::body::Frame<Bytes>, std::io::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let poll = unsafe { self.map_unchecked_mut(|e| &mut e.0).poll_next(cx) };
        poll.map(|e| e.map(|e| e.map(|e| hyper::body::Frame::data(e))))
    }
}
