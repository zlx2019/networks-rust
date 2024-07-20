use std::sync::Arc;

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, signal, sync::Notify};

/**
 * 基于 Tokio 异步运行时实现 TCP 服务端.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    // 建立 TCP 服务
    let listener = TcpListener::bind("127.0.0.1:8667").await?;
    println!("Server listener runing...");

    // 信号通知器
    let notify = Arc::new(Notify::new());
    let notify_listener = notify.clone();
    tokio::spawn(async move {
        // 监听 SIGINT 信号
        let _ = signal::ctrl_c().await;
        notify_listener.notify_one();
    });

    // TCP 服务监听处理.
    loop {
        tokio::select! {
            Ok((client, client_addr)) = listener.accept() => {
                // 有客户端建立连接.
                println!("Clinet [{:?}] connect...", client_addr);
                // 异步处理客户端连接
                tokio::spawn(handler_client(client));
            },
            _ = notify.notified() => {
                // 接收到 SIGINT 信号，退出程序.
                println!("Receive [SIGINT] signal, program termination.");
                break;
            }
        }

    }
    Ok(())
}

/**
 * 客户端连接异步处理函数
 */
async fn handler_client(mut client: TcpStream){
    // 定义一个 u8 类型缓冲区
    let mut buf = [0u8; 1024];
    loop {
        // 阻塞读取数据
        let n = match client.read(&mut buf).await {
            Ok(n) if n == 0 => {
                // 连接关闭
                println!("Client [{:?}] closed...", client.peer_addr().unwrap());
                return;
            },
            Ok(n) => n,
            Err(e) => {
                eprintln!("Read client fail; err: {:?}", e);
                return ;
            }
        };
        // 将缓冲区数据写回客户端
        if let Err(e) = client.write_all(&buf[0..n]).await {
            eprintln!("Wreit to client fail; err: {:?}", e);
            return ;
        }
    }
}