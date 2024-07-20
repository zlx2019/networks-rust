use std::net::SocketAddr;
use anyhow::{Result};
use s2n_quic::Client;
use s2n_quic::client::Connect;

// QUIC 服务端与客户端通信使用的证书和秘钥
const CERT_PEM: &str = include_str!("../fixtures/cert.pem");

///
/// 基于s2n-quic实现的简单Quic客户端
///
#[tokio::main]
async fn main() -> Result<()>{
    // 创建quic客户端
    let client = Client::builder()
        .with_tls(CERT_PEM)?
        .with_io("0.0.0.0:0")?
        .start()?;
    eprintln!("Quic Client is Running...");
    // quic服务端地址
    let server_addr:SocketAddr = "127.0.0.1:4433".parse()?;
    // 连接服务端
    let mut connect = client.connect(Connect::new(server_addr).with_server_name("localhost")).await?;
    // 设置为长连接
    connect.keep_alive(true)?;

    eprintln!("Connect Server Successful...");

    // 从连接中打开一个流，拆分为接收流和发送流
    let  stream = connect.open_bidirectional_stream().await?;
    let (mut recv_stream, mut send_stream) = stream.split();

    // 读取服务端响应
    tokio::spawn(async move {
        // 将响应输出到终端
        let mut stdout = tokio::io::stdout();
        let _ = tokio::io::copy(&mut recv_stream,&mut stdout).await;
    });

    // 获取终端输入流
    let mut stdin = tokio::io::stdin();
    // 将终端输入 作为数据，通过发送流发送给服务端
    tokio::io::copy(&mut stdin, &mut send_stream).await?;
    Ok(())
}