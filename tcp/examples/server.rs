

use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    thread,
};


/// 
/// TCP服务端
/// 
fn main() -> io::Result<()> {
    // 创建TCP服务
    let listener = TcpListener::bind("127.0.0.1:7896")?;

    // 创建一个线程集合
    let mut tasks: Vec<thread::JoinHandle<()>> = Vec::new();
    println!("tcp server running successful...");
    // 阻塞等待连接
    for stream in listener.incoming() {
        // 获取连接
        let tcp_stream = stream.expect("failed");
        let conn_addr = tcp_stream.peer_addr()?;
        println!("client connect to: {}", conn_addr);
        // 开启新的线程 处理连接数据
        let task = thread::spawn(move || {
            handler_client(tcp_stream).unwrap_or_else(|e| eprintln!("{:?}", e));
        });
        tasks.push(task);
    }
    // 等待线程执行完毕
    for task in tasks {
        task.join().unwrap();
    }
    Ok(())
}

// 连接处理函数
fn handler_client(mut stream: TcpStream) -> io::Result<()> {
    // 创建缓冲区
    let mut buf = [0; 512];
    loop {
        // 读取连接数据
        let len = stream.read(&mut buf)?;
        // len == 0 表示客户端已关闭
        if len == 0 {
            println!("client shutdown successful...");
            return Ok(());
        }
        // 将读取到的字节流，反序列化为String
        let message = String::from_utf8_lossy(&buf[.. len]);
        println!("{}", message);

        // 将数据写回客户端
        // stream.write_all(&buf[..len])?;
        // stream.write_all(message.as_bytes())?;
    }
}
