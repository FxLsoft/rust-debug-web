use std::env;

use serde::Deserialize;
#[derive(Debug, Default, Deserialize, Clone)]
pub struct EnvVars {
    // Server地址
    pub server_address: String,
    // mysql数据源地址
    pub database_url: String,
}

impl EnvVars {
    pub fn new() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").expect("没有配置数据源");
        let server_address = env::var("SERVER_ADDRESS").expect("没有配置Server地址");
        Ok(EnvVars {
            server_address,
            database_url,
        })
    }
}
