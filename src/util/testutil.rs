use slog::Logger;
use slog_scope;

pub fn with_test_logger<R, F>(func: F) -> R
where
    F: Fn(&Logger) -> R,
{
    func(&slog_scope::logger())
}

pub mod factories {
    use crate::models::{AttendanceStatus::*, CallStatus::*, *};

    pub fn create_call() -> RollCall {
        let now = chrono::Utc::now().naive_local();

        RollCall {
            id: 1,
            chat_id: 2,
            status: Open,
            title: "call title".to_string(),
            quiet: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn create_quiet_call() -> RollCall {
        RollCall {
            quiet: true,
            ..create_call()
        }
    }

    pub fn create_responses() -> Vec<RollCallResponse> {
        let now = chrono::Utc::now().naive_local();

        vec![
            RollCallResponse {
                id: 1,
                roll_call_id: 1,
                unique_token: "self:0001".to_string(),
                user_id: Some(1),
                user_name: Some("David".to_string()),
                status: In,
                reason: Some("will come".to_string()),
                created_at: now,
                updated_at: now,
            },
            RollCallResponse {
                id: 2,
                roll_call_id: 1,
                unique_token: "for:0002".to_string(),
                user_id: None,
                user_name: Some("Daniel".to_string()),
                status: Out,
                reason: Some("won't come".to_string()),
                created_at: now,
                updated_at: now,
            },
            RollCallResponse {
                id: 3,
                roll_call_id: 1,
                unique_token: "self:0003".to_string(),
                user_id: Some(2),
                user_name: Some("Henry".to_string()),
                status: In,
                reason: Some("also will come".to_string()),
                created_at: now,
                updated_at: now,
            },
            RollCallResponse {
                id: 4,
                roll_call_id: 1,
                unique_token: "for:0004".to_string(),
                user_id: None,
                user_name: Some("Albert".to_string()),
                status: Maybe,
                reason: Some("might come".to_string()),
                created_at: now,
                updated_at: now,
            },
        ]
    }
}
