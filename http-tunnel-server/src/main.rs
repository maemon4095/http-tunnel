use clap::Parser;
use futures::StreamExt;
use http::Response;
use http_body_util::{BodyExt, StreamBody};
use http_tunnel_util::error::BoxError;
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, clap::Parser)]
struct Args {
    connection: String,
    #[arg(long)]
    over: String,
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let args = Args::parse();
    println!("starting tunnel {}", &args.over);

    let listener = TcpListener::bind(args.over).await?;
    let (stream, addr) = listener.accept().await?;
    println!("tunnel connected: {}", addr);
    let client_io = TokioIo::new(stream);

    let connection = args.connection;
    http1::Builder::new()
        .serve_connection(
            client_io,
            service_fn(move |req| {
                let connection = connection.clone();
                async move {
                    println!("connect to {}", connection);
                    let stream = TcpStream::connect(connection).await?;
                    println!("connected");
                    let (downstream, mut upstream) = stream.into_split();
                    let downstream = tokio_util::io::ReaderStream::new(downstream);
                    let mut req_body: Incoming = req.into_body();
                    println!("start receiving");
                    tokio::spawn(async move {
                        while let Some(next) = req_body.frame().await {
                            let frame = next?;
                            if let Some(chunk) = frame.data_ref() {
                                upstream.write_all(&chunk).await?;
                            }
                        }
                        Ok::<_, BoxError>(())
                    });
                    println!("start sending");
                    Ok::<_, BoxError>(Response::new(StreamBody::new(
                        downstream.map(|e| e.map(|e| hyper::body::Frame::data(e))),
                    )))
                }
            }),
        )
        .await?;

    Ok(())
}
