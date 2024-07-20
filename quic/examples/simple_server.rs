use s2n_quic::Server;
use anyhow::{Result};



// QUIC 服务端与客户端通信使用的证书和秘钥
const CERT_PEM: &str = include_str!("../fixtures/cert.pem");
const KEY_PEM: &str = include_str!("../fixtures/key.pem");


///
/// 基于s2n-quic实现的简单Quic服务端
///
#[tokio::main]
async fn main() -> Result<()>{
    // 服务的端点
    let addr = "127.0.0.1:4433";

    // 创建一个服务，设置证书与秘钥，以及服务端点
    let mut server = Server::builder()
        // 设置证书与秘钥
        .with_tls((CERT_PEM,KEY_PEM))?
        // 设置服务端点
        .with_io(addr)?
        // 启动服务
        .start()?;
    eprintln!("Quic Server is Running...");

    loop {
        // 阻塞等待客户端连接
        if let Some(mut conn) = server.accept().await{
            eprintln!("客户端[{:?}] 建立连接...",conn.remote_addr().unwrap());
            // 1. 为每个连接开启一个协程
            tokio::spawn(async move{
                loop {
                    // 阻塞等待连接打开新的流，并且获取打开的双向流
                    // QUIC协议中使用流Stream进行数据的并行传输，解决阻塞拥堵;
                    match conn.accept_bidirectional_stream().await {
                        Ok(Some(mut stream))=> {
                            eprintln!("客户端[{:?}] 打开一个数据流",conn.remote_addr().unwrap());
                            // 2. 为连接中打开的每一个stream流开启一个协程
                            tokio::spawn(async move{
                                loop {
                                    // 阻塞读取流中的数据
                                    match stream.receive().await {
                                        Ok(Some(bytes))=>{
                                            // 读取到数据处理
                                            let message = String::from_utf8_lossy(&bytes).trim().to_string();
                                            eprintln!("StreamID[{}]: {}",stream.id(), message);
                                            // 将数据再次写回客户端
                                            stream.send(bytes).await.expect("writer data to client error!");
                                        },
                                        Err(e)=> {
                                            eprintln!("读取流中的数据错误: {:?}",e);
                                            return;
                                        }
                                        _ => {}
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("accept conn stream failed: {:?}",e);
                            return;
                        }
                        _ => {}
                    }
                }
            });
        }
    }
}