use pingora::prelude::*;
use tracing::info;
use pingora_http_proxy::MinimalHttpProxy;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // 创建并初始化 pingora 服务器
    let mut server = Server::new(None)?;
    server.bootstrap();

    // 创建自定义的代理服务
    let proxy_addr = "0.0.0.0:7100";
    let http_proxy = MinimalHttpProxy::default();
    let mut http_proxy_service = http_proxy_service(&server.configuration, http_proxy);
    http_proxy_service.add_tcp(proxy_addr);
    info!("http proxy server running on {}", proxy_addr);
    // 将我们的代理服务，注册到服务器，并且运行.
    server.add_service(http_proxy_service);
    server.run_forever();
}
