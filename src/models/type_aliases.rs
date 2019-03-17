use super::roll_call_responses::RollCallResponse;
use super::roll_calls::RollCall;

pub type CallId = i64;
pub type ResponseId = i64;

pub type ChatId = i64;
pub type UserId = i64;

pub type CallWithResponses = (RollCall, Vec<RollCallResponse>);
