use std::fmt::Debug;

use rbatis::{crud, impl_select_page};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct BugLog {
    pub id: Option<String>,
    pub app_version: Option<String>,
    pub app_key: Option<String>,
    pub version: Option<String>,
    pub user_agent: Option<String>,
    pub locale: Option<String>,
    pub url: Option<String>,
    pub title: Option<String>,
    pub time: Option<i64>,
    #[serde(rename = "type")]
    pub bug_type: Option<String>,
    #[serde(serialize_with = "serialize_string")]
    pub detail: Option<Value>,
    #[serde(serialize_with = "serialize_string")]
    pub action_info: Option<Value>,
    #[serde(serialize_with = "serialize_string")]
    pub custom: Option<Value>,
    pub ip: Option<String>,
    pub event_count: Option<i32>,
    pub status: Option<String>,
}

crud!(BugLog {});

impl_select_page!(BugLog{select_page() => "`where 1 = 1`"});

fn serialize_string<S>(value: &Option<Value>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match *value {
        Some(ref v) => serializer.serialize_str(&v.to_string()),
        None => serializer.serialize_none(),
    }
}
