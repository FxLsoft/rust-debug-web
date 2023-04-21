use actix_cors::Cors;
use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    middleware::Logger,
    web, App, HttpResponse, HttpServer, Responder, Result, HttpRequest,
};
use chrono::Local;
use derive_more::{Display, Error};
use dotenv::dotenv;
use fast_log::{
    consts::LogSize,
    plugin::{file_split::RollingType, packer::LogPacker},
};
use log::LevelFilter;
use rbatis::{executor::RBatisConnExecutor, intercept::SqlIntercept, Rbatis, rbdc::uuid::{self, Uuid}, sql::PageRequest};
use rbdc_mysql::driver::MysqlDriver;
use serde::{Deserialize, Serialize};

use crate::{app_config::EnvVars, models::bug_log::{BugLog}};

mod app_config;
mod models;

#[derive(Debug)]
pub struct Intercept {}

impl SqlIntercept for Intercept {
    /// do intercept sql/args
    /// is_prepared_sql: if is run in prepared_sql=ture
    fn do_intercept(
        &self,
        _rb: &Rbatis,
        sql: &mut String,
        _args: &mut Vec<rbs::Value>,
        _is_prepared_sql: bool,
    ) -> Result<(), rbatis::Error> {
        log::info!("sql => {}", sql);
        Ok(())
    }
}

#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
    pub code: Option<i32>,
}

impl<T> ResResult<T> {
    fn ok(data: T) -> Self {
        return ResResult {
            success: true,
            data: Some(data),
            message: Some("请求成功".to_owned()),
            code: Some(200),
        };
    }
}

impl ResResult<String> {
    fn err(msg: &'static str, code: i32) -> Self {
        return ResResult {
            success: false,
            data: None,
            message: Some(msg.to_owned()),
            code: Some(code),
        };
    }
}

#[get("/getAllUsers")]
async fn get_all_users(app: web::Data<AppState>) -> Result<impl Responder> {
    let start_time = Local::now();
    log::info!("执行 开始时间{}", start_time);
    let mut executor = app.get_executor().await;
    let users = models::user::User::select_all(&mut executor)
        .await
        .expect("查询失败");
    let end_time = Local::now();
    log::info!(
        "执行 结束时间{}, 用时：{}",
        end_time,
        end_time.timestamp_millis() - start_time.timestamp_millis()
    );
    Ok(web::Json(ResResult::ok(users)))
}

#[derive(Debug, Deserialize)]
struct EventParams {
    event: String,
}

async fn do_insert_bug(event: String, app: web::Data<AppState>, req: HttpRequest) {
    let mut bug_info = serde_json::from_str::<BugLog>(&event).expect("参数错误❌");
    let remote_ip = req.connection_info().realip_remote_addr().unwrap_or_default().to_string();
    bug_info.id = Some(Uuid::new().0);
    bug_info.ip = Some(remote_ip);
    let mut executor = app.get_executor().await;
    BugLog::insert(&mut executor, &bug_info).await.unwrap();
}

#[get("/event")]
async fn do_event(app: web::Data<AppState>, query: web::Query<EventParams>, req: HttpRequest) -> HttpResponse {
    let event = query.into_inner().event;
    do_insert_bug(event, app, req).await;
    HttpResponse::Ok().status(StatusCode::NO_CONTENT).body("")
}

#[get("/getBugs")]
async fn get_event(app: web::Data<AppState>, query: web::Query<PageRequest>) -> Result<impl Responder> {
    let mut executor = app.get_executor().await;
    let page = BugLog::select_page(&mut executor, &query.into_inner()).await.unwrap();
    Ok(web::Json(ResResult::ok(page)))
}

pub struct AppState {
    pub rb: Rbatis,
    pub env_vars: EnvVars,
}

impl AppState {
    pub async fn get_executor(&self) -> RBatisConnExecutor {
        self.rb.acquire().await.expect("获取数据库连接池失败")
    }
}

async fn err_handling() -> Result<HttpResponse, MyError> {
    let result = ResResult::err("请求失败", 500);
    Ok(HttpResponse::InternalServerError().json(result))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let env_vars = EnvVars::new().unwrap();
    fast_log::init(
        fast_log::Config::new()
            .console()
            .chan_len(Some(100000))
            .file_split(
                "target/logs/",
                LogSize::MB(10),
                RollingType::All,
                LogPacker {},
            )
            .level(LevelFilter::Info),
    )
    .expect("log init fail");
    let rb = Rbatis::new();
    // rb.set_sql_intercepts(vec![Box::new(Intercept {})]);
    rb.init(MysqlDriver {}, env_vars.database_url.as_str())
        .unwrap();
    rb.get_pool().unwrap().resize(10);
    // 初始化数据连接池
    rb.acquire().await.expect("初始化数据库连接池失败");
    let app_state = web::Data::new(AppState {
        rb,
        env_vars: env_vars.clone(),
    });
    log::info!("正在启动web服务");
    return HttpServer::new(move || {
        let app = App::new()
            .wrap(Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("*.lotuscars.com.cn")
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"]),
            )
            .app_data(app_state.clone())
            .service(get_all_users)
            .service(do_event)
            .service(get_event)
            .default_service(web::route().to(err_handling));
        return app;
    })
    .workers(3)
    .bind(env_vars.server_address)?
    .run()
    .await;
}
