use std::net::TcpStream;
use std::io::{Result,stdin, Write, Read};
use std::thread;


///
/// TCP 客户端
/// 
fn main() -> Result<()>{
    // 连接服务端
    let mut stream = TcpStream::connect("127.0.0.1:7896")?;
    // 开启线程 读取服务端的数据
    let mut read_stream = stream.try_clone()?;
    thread::spawn(move || {
        let mut buf = [0;512];
        loop {
            let len = read_stream.read(&mut buf).unwrap();
            if len == 0 {
                println!("与服务端的连接已关闭");
                break;
            }
            let msg = String::from_utf8_lossy(&buf[.. len]);
            println!("{}",msg);
        }
    });


    let mut line = String::new();
    loop {
        // 读取终端输入
        stdin().read_line(&mut line).expect("read stdin failed");
        // 发送到服务端
        stream.write(line.as_bytes()).expect("send message to server failed");
    }
}