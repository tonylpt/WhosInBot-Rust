use chrono::{NaiveDateTime, Utc};
use crypto::{digest::Digest, sha2::Sha256};

pub use attendance::{Attendance, AttendanceStatus};

use crate::schema::w_roll_call_responses;

use super::type_aliases::*;

mod attendance;

#[derive(Queryable, Debug, Clone)]
pub struct RollCallResponse {
    pub id: ResponseId,
    pub roll_call_id: CallId,
    pub unique_token: String,
    pub user_id: Option<UserId>,
    pub user_name: Option<String>,
    pub status: AttendanceStatus,
    pub reason: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "w_roll_call_responses"]
pub struct NewRollCallResponse<'a> {
    pub roll_call_id: CallId,
    pub unique_token: String,
    pub user_id: Option<UserId>,
    pub user_name: &'a str,
    pub status: AttendanceStatus,
    pub reason: &'a str,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl<'a> NewRollCallResponse<'a> {
    pub fn new_self(
        call_id: CallId,
        user_id: UserId,
        user_name: &'a str,
        attendance: &'a Attendance,
    ) -> NewRollCallResponse<'a> {
        assert!(user_id > 0);

        let unique_token = format!("self:{}", Self::hash(&user_id.to_string()));
        let now = Utc::now().naive_local();

        NewRollCallResponse {
            roll_call_id: call_id,
            unique_token,
            user_id: Some(user_id),
            user_name,
            status: attendance.status,
            reason: &attendance.reason,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_for(
        call_id: CallId,
        user_name: &'a str,
        attendance: &'a Attendance,
    ) -> NewRollCallResponse<'a> {
        assert!(!user_name.is_empty());

        let unique_token = format!("for:{}", Self::hash(&user_name.to_lowercase()));
        let now = Utc::now().naive_local();

        NewRollCallResponse {
            roll_call_id: call_id,
            unique_token,
            user_id: None,
            user_name,
            status: attendance.status,
            reason: &attendance.reason,
            created_at: now,
            updated_at: now,
        }
    }

    fn hash(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(input);
        hasher.result_str()
    }
}

#[derive(AsChangeset, Debug)]
#[table_name = "w_roll_call_responses"]
pub struct UpdateRollCallResponse<'a> {
    pub user_name: Option<&'a str>,
    pub status: Option<AttendanceStatus>,
    pub reason: Option<&'a str>,
    pub updated_at: Option<NaiveDateTime>,
}

impl<'a> UpdateRollCallResponse<'a> {
    pub fn new(
        user_name: &'a str,
        status: AttendanceStatus,
        reason: &'a str,
    ) -> UpdateRollCallResponse<'a> {
        let now = Utc::now().naive_local();
        UpdateRollCallResponse {
            user_name: Some(user_name),
            status: Some(status),
            reason: Some(reason),
            updated_at: Some(now),
        }
    }
}
