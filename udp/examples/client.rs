use std::{net::UdpSocket, io::{Result,stdin}, thread};



///
/// UDP 客户端
/// 
fn main() -> Result<()>{
    // 创建一个udp连接
    let client_socket = UdpSocket::bind("127.0.0.1:9876").expect("create socket failed");
    // 连接到服务端
    client_socket.connect("127.0.0.1:6780").expect("connect server failed");

    // 开启线程接收数据
    let read_socket =  client_socket.try_clone()?;
    thread::spawn(move ||{
        // 定义一个缓冲区
        let mut buf = [0u8;1024];
        loop {
            // 阻塞接收服务端数据
            let (len,_) = read_socket.recv_from(&mut buf).expect("client recv failed");
            if len == 0 {
                println!("udp server is shutdown");
                break;
            }
            let message = String::from_utf8_lossy(&buf[..len]);
            println!("server: {}",message);
        }
    });

    // 主动发送数据
    let mut line = String::new();
    loop {
        stdin().read_line(&mut line)?;
        // 发送数据
        client_socket.send(line.as_bytes()).expect("send message failed");
    }
}