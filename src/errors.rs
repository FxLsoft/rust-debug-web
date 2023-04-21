pub enum AppErrorType {
    DbError,
    NotFontError,
}

pub struct AppError {
    pub message: Option<string>,
    pub cause: Option<string>,
    pub error_type: AppErrorType,
}

