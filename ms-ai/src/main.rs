use fbc_starter::{AppResult, Server};

mod router;
mod model;

#[tokio::main]
async fn main() -> AppResult<()>{
    Server::run(|builder| {
        // 创建路由
        builder
    })
    .await
}
