use std::sync::Arc;
use async_trait::async_trait;
use pingora::prelude::*;

/// # Pingora-lb
///
/// 此案例使用 pingora & pingora-proxy 构建一个简单的负载均衡器。
/// 目标： 对于每个传入的 HTTP 请求，转发到 https://1.1.1.1 或 https://1.0.0.1

/// 定义我们自己的负载均衡器, 保存上游的IP列表。
/// pingora 的 `pingora-load-balancing`模块为`LoadBalancer`结构体提供了常见的负载均衡算法，例如循环、哈希等。并且`LoadBalancer`还包含服务发现、健康检查等功能。
/// 这里使用 `RoundRobin` 轮询策略
pub struct LB(Arc<LoadBalancer<RoundRobin>>);


/// 为了使`LB`能够转发请求，我们需要为它实现`ProxyHttp` trait
/// 该特征定义了如何在代理中请求
#[async_trait]
impl ProxyHttp for LB {
    /// 自定义的上下文，这个案例中并不需要
    type CTX = ();
    /// 构建自定义上下文
    fn new_ctx(&self) -> Self::CTX {
        ()
    }
    /// 请求处理，最终返回上游服务的地址及连接方式
    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        // 从负载均衡器中获取上游服务，这里采用的是轮询策略
        let upstream = self.0.select(b"", 256).unwrap();
        println!("upstream peer is: {upstream:?}");
        let peer = Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    /// 请求过滤器：请求发送到上游服务之前执行
    /// 为了让 1.1.1.1 后端接受我们的请求，必须存在 host 标头。
    async fn upstream_request_filter(&self, _session: &mut Session, upstream_request: &mut RequestHeader, _ctx: &mut Self::CTX) -> Result<()>
    where
        Self::CTX: Send + Sync
    {
        upstream_request.insert_header("Host", "one.one.one.one")
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建并初始化 Pingora 服务器
    let mut server = Server::new(None)?;
    server.bootstrap();

    // 上游服务列表
    let upstreams = LoadBalancer::try_from_iter(["114.114.114.114:443","1.1.1.1:443", "1.0.0.1:443"]).unwrap();
    // 构建负载均衡器
    let lb = LB(Arc::new(upstreams));
    // 构建 HTTP 代理服务
    let mut http_proxy_service = http_proxy_service(&server.configuration, lb);
    // 绑定地址
    http_proxy_service.add_tcp("0.0.0.0:6188");
    // Pingora 运行 HTTP 代理服务器
    server.add_service(http_proxy_service);
    server.run_forever();
    Ok(())
}
