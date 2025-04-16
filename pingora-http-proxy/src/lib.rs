use async_trait::async_trait;
use pingora::prelude::*;
use tracing::info;

/// 一个极简的 HTTP 代理服务: 将所有请求转发到 `127.0.0.1:3000` 服务
#[derive(Default)]
pub struct MinimalHttpProxy {}

/// 实现 `HttpProxy`，成为 HTTP proxy
#[async_trait]
impl ProxyHttp for MinimalHttpProxy {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {
        ()
    }
    /// 请求处理：将请求转发到 `127.0.0.1:3000` 服务（这是我们本地的一个服务）
    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut Self::CTX) -> Result<Box<HttpPeer>> {
        // HttpPeer 表示一个远程服务器信息
        // address: 服务器地址
        // tls: 是否使用 tls
        // sni: 目标服务器的主机名
        let peer = HttpPeer::new("127.0.0.1:3000", false, "localhost".to_string());
        info!("request forward to: {}", peer.to_string());
        Ok(Box::new(peer))
    }
}