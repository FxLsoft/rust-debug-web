use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Option<String>,
    pub name: Option<String>,
    pub login_name: Option<String>,
    pub pwd: Option<String>,
    pub app_key: Option<String>,
    pub update_time: Option<i64>,
    pub create_time: Option<i64>,
}

rbatis::crud!(User{});

rbatis::impl_select!(User{select_by_id(id: &str) => "`where id = #{id}`"});