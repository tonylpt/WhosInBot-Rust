use chrono::{NaiveDateTime, Utc};

pub use status::CallStatus;

use crate::schema::w_roll_calls;

use super::type_aliases::*;

mod status;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "w_roll_calls"]
pub struct RollCall {
    pub id: CallId,
    pub chat_id: ChatId,
    pub status: CallStatus,
    pub title: String,
    pub quiet: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "w_roll_calls"]
pub struct NewRollCall<'a> {
    pub chat_id: ChatId,
    pub status: CallStatus,
    pub title: &'a str,
    pub quiet: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl<'a> NewRollCall<'a> {
    pub fn new(chat_id: ChatId, title: &'a str) -> NewRollCall<'a> {
        let now = Utc::now().naive_local();
        NewRollCall {
            chat_id,
            title,
            status: CallStatus::Open,
            quiet: false,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(AsChangeset, Debug)]
#[table_name = "w_roll_calls"]
pub struct UpdateRollCall<'a> {
    pub status: Option<CallStatus>,
    pub title: Option<&'a str>,
    pub quiet: Option<bool>,
    pub updated_at: Option<NaiveDateTime>,
}

impl<'a> UpdateRollCall<'a> {
    pub fn new() -> UpdateRollCall<'static> {
        UpdateRollCall {
            status: None,
            title: None,
            quiet: None,
            updated_at: Some(Utc::now().naive_local()),
        }
    }

    pub fn with_status(self, status: CallStatus) -> UpdateRollCall<'a> {
        UpdateRollCall {
            status: Some(status),
            ..self
        }
    }

    pub fn with_quiet(self, quiet: bool) -> UpdateRollCall<'a> {
        UpdateRollCall {
            quiet: Some(quiet),
            ..self
        }
    }

    pub fn with_title(self, title: &'a str) -> UpdateRollCall<'a> {
        UpdateRollCall {
            title: Some(title),
            ..self
        }
    }
}