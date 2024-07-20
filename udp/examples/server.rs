use std::{
    io::Result,
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
};

///
/// UDP 服务端
///
fn main() -> Result<()> {
    // 使用IPV4版本
    let addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    // 创建Socket地址
    let udp_addr = SocketAddr::new(addr, 6780);
    // 建立 udp 服务
    let socket = UdpSocket::bind(udp_addr)?;
    println!("udp server running successful...");

    loop {
        // 创建一个缓冲区
        let mut buf = [0u8; 1024];
        // 阻塞等待客户端发送数据
        // (读取的数据长度，发送数据的客户端地址)
        let (len, client_addr) = socket.recv_from(&mut buf)?;
        let message = String::from_utf8_lossy(&buf[..len]);
        // 输出消息
        println!("[{}]: {}", client_addr, message);
        // 将数据写回到客户端
        socket.send_to(message.as_bytes(), &client_addr)?;
        drop(buf);
    }
}
