use std::net::SocketAddr;
use std::str::FromStr;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHasher};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use tokio::net::TcpListener;
use tracing::{error, info};

lazy_static!{
    static ref ADDR: Mutex<SocketAddr> = Mutex::new(
        SocketAddr::from_str("127.0.0.1:3000").unwrap()
    );
}

/// 用于测试代理的HTTP服务器
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    // args
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        error!("Usage: {} <address>", args[0]);
        std::process::exit(1);
    }
    let addr = SocketAddr::from_str(&args[1]).expect("Failed to parse socket address");
    {
        *ADDR.lock().unwrap() = addr;
    }
    let state = AppState::new();
    let app = Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/ping", get(ping))
        .route("/users/{id}", get(get_user))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}", put(update_user))
        .route("/users/{id}", delete(delete_user))
        .with_state(state);
    let listen = TcpListener::bind(addr).await.unwrap();
    info!("Listening on: {}", listen.local_addr().unwrap());
    axum::serve(listen, app).await.unwrap();
}


async fn ping() -> &'static str {
    info!("PING success: [{}]", *ADDR.lock().unwrap());
    "StatusOk"
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    #[serde(with = "my_date_format")]
    created_at: DateTime<Utc>,
    #[serde(with = "my_date_format")]
    updated_at: DateTime<Utc>,
}

/// Create User request body
#[derive(Debug, Deserialize)]
struct CreateUserReq {
    name: String,
    password: String,
    email: String,
}

/// Update User request body
#[derive(Debug, Deserialize)]
struct UpdateUserReq {
    id: u64,
    name: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

/// App state
#[derive(Clone)]
struct AppState {
    inner: Arc<AppStateInner>
}

/// 全局状态
struct AppStateInner {
    id_generator: AtomicU64,
    users: DashMap<u64, User>,
    encryptor: Argon2<'static>,
}

impl AppState {
    /// Create State
    fn new () -> Self {
        Self{
            inner: Arc::new(AppStateInner{
                id_generator: AtomicU64::new(1),
                users: DashMap::new(),
                encryptor: Argon2::default(),
            })
        }
    }

    /// 根据 ID 获取User
    fn get_user(&self, id: u64) -> Option<User> {
        self.inner.users.get(&id).map(|user| user.clone())
    }

    /// 创建User
    fn create_user(&self, req: CreateUserReq) -> Result<User, anyhow::Error> {
        let password = self.hash_password(req.password)?;
        let id = self.inner.id_generator.fetch_add(1, Ordering::SeqCst);

        let now = Utc::now();
        let user = User{
            id,
            name: req.name,
            email: req.email,
            password: password.to_string(),
            created_at: now,
            updated_at: now,
        };
        self.inner.users.insert(id, user.clone());
        Ok(user)
    }

    /// 根据ID，更新User
    fn update_user(&self,id: u64, req: UpdateUserReq) -> Option<User> {
        // let user = self.get_user(req.id).ok_or(anyhow::anyhow!("User not found"))?;
        let mut user = self.get_user(id)?;
        if let Some (name) = req.name {
            user.name = name;
        }
        if let Some(email) = req.email {
            user.email = email;
        }
        if let Some(password) = req.password {
            user.password = self.hash_password(password).ok()?;
        }
        // user.updated_at = Utc::now().with_timezone(&FixedOffset::east_opt(8).unwrap());
        user.updated_at = Utc::now();
        self.inner.users.insert(req.id, user.clone());
        Some(user)
    }

    /// 根据ID，删除User
    fn delete_user(&self, id: u64) -> Option<User> {
        self.inner.users.remove(&id).map(|(_, user)| user)
    }

    /// 获取所有用户
    fn list_users(&self) -> Vec<User> {
        self.inner.users
            .iter()
            .map(|ref_multi| ref_multi.value().clone())
            .collect()
    }

    /// 对明文密码加密
    fn hash_password(&self, password: String) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.inner.encryptor.hash_password(password.as_bytes(), &salt)
        .map_err(|_| anyhow::anyhow!("hash password failed"))?;
        Ok(password_hash.to_string())
    }
}


/// 创建用户
async fn get_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    state.get_user(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

/// 用户列表
async fn list_users(State(state): State<AppState>) -> Json<Vec<User>> {
    Json(state.list_users())
}


/// 创建用户
async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserReq>,
) -> Result<(StatusCode, Json<User>), (StatusCode, String)> {
    state.create_user(req)
        .map(|user| (StatusCode::CREATED, Json(user)))
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

/// 更新用户
async fn update_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
    Json(update_user): Json<UpdateUserReq>,
) -> Result<Json<User>, StatusCode> {
    state
        .update_user(id, update_user)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

/// 删除用户
async fn delete_user(
    Path(id): Path<u64>,
    State(state): State<AppState>,
) -> Result<Json<User>, StatusCode> {
    state.delete_user(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

// 自定义序列化模块
mod my_date_format {
    use chrono::{DateTime, FixedOffset, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    // 序列化时将 UTC 时间转换为上海时区（UTC+8）并格式化
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 创建东八区偏移
        let shanghai_offset = FixedOffset::east_opt(8 * 3600).unwrap(); // 8小时 = 8 * 3600秒
        // 转换时区
        let shanghai_time = date.with_timezone(&shanghai_offset);
        // 格式化为指定格式的字符串
        let formatted = shanghai_time.format("%Y-%m-%d %H:%M:%S").to_string();
        serializer.serialize_str(&formatted)
    }

    // 反序列化（如果需要）
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // 实现反序列化逻辑...
        // 这里简化处理，实际使用时可能需要更复杂的逻辑
        let _s = String::deserialize(deserializer)?;
        // 解析字符串为上海时区的时间，然后转回 UTC
        // ...
        Ok(Utc::now()) // 这只是占位符
    }
}