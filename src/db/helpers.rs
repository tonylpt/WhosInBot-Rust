use std::time::Duration;

use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2;

use crate::models::*;
use crate::schema;
use crate::util::collections::first;

pub type Manager = ConnectionManager<PgConnection>;
pub type Pool = r2d2::Pool<Manager>;
pub type PooledConnection = r2d2::PooledConnection<Manager>;
pub type PoolResult = Result<Pool, r2d2::Error>;

pub fn connect(database_url: &str, timeout: Duration) -> PoolResult {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .connection_timeout(timeout)
        .build(manager)
}

pub fn create_call(conn: &PgConnection, chat_id: ChatId, title: &str) -> QueryResult<RollCall> {
    use schema::w_roll_calls::{dsl, table};

    let close_all_calls = || -> QueryResult<usize> {
        let open_calls = table
            .filter(dsl::chat_id.eq(chat_id))
            .filter(dsl::status.eq(CallStatus::Open));

        let update_close = UpdateRollCall::new().with_status(CallStatus::Closed);

        let updated = diesel::update(open_calls).set(update_close).execute(conn)?;

        debug!("Closed {} open calls", updated);
        Ok(updated)
    };

    let delete_old_calls = || -> QueryResult<usize> {
        let to_keep: Vec<_> = table
            .filter(dsl::chat_id.eq(chat_id))
            .filter(dsl::status.eq(CallStatus::Closed))
            .order(dsl::created_at.desc())
            .limit(10)
            .select(dsl::created_at)
            .load::<NaiveDateTime>(conn)?;

        let earliest_to_keep = to_keep.iter().min();
        let earliest_to_keep = match earliest_to_keep {
            None => return Ok(0_usize),
            Some(value) => value,
        };

        let deleted = diesel::delete(
            table
                .filter(dsl::chat_id.eq(chat_id))
                .filter(dsl::status.eq(CallStatus::Closed))
                .filter(dsl::created_at.lt(earliest_to_keep)),
        )
        .execute(conn)?;

        debug!("Deleted {} old closed calls", deleted);
        Ok(deleted)
    };

    let insert_new_call = || -> QueryResult<RollCall> {
        let new_call = NewRollCall::new(chat_id, title);
        let result = diesel::insert_into(table)
            .values(new_call)
            .get_result(conn)?;

        debug!("Inserted new call: {:?}", result);
        Ok(result)
    };

    conn.transaction(|| {
        close_all_calls()?;
        delete_old_calls()?;
        insert_new_call()
    })
}

pub fn end_call(conn: &PgConnection, chat_id: ChatId) -> QueryResult<Option<RollCall>> {
    let update = UpdateRollCall::new().with_status(CallStatus::Closed);
    update_call(conn, chat_id, update)
}

pub fn update_title(
    conn: &PgConnection,
    chat_id: ChatId,
    new_title: &str,
) -> QueryResult<Option<RollCall>> {
    let update = UpdateRollCall::new().with_title(new_title);
    update_call(conn, chat_id, update)
}

pub fn update_quiet(
    conn: &PgConnection,
    chat_id: ChatId,
    quiet: bool,
) -> QueryResult<Option<CallWithResponses>> {
    let update = UpdateRollCall::new().with_quiet(quiet);
    match update_call(conn, chat_id, update)? {
        None => Ok(None),
        Some(call) => get_responses(conn, call.id).map(|responses| Some((call, responses))),
    }
}

fn update_call(
    conn: &PgConnection,
    chat_id: ChatId,
    update: UpdateRollCall,
) -> QueryResult<Option<RollCall>> {
    use schema::w_roll_calls::{dsl, table};

    let open_call = table
        .filter(dsl::chat_id.eq(chat_id))
        .filter(dsl::status.eq(CallStatus::Open))
        .order(dsl::updated_at.desc())
        .limit(1)
        .select(dsl::id)
        .load::<CallId>(conn)?;

    if open_call.is_empty() {
        return Ok(None);
    }

    let filter_open_call = table.filter(dsl::id.eq_any(open_call));
    let updated = diesel::update(filter_open_call)
        .set(update)
        .get_results(conn)?;

    let updated: Option<RollCall> = first(updated);
    debug!("Updated call: {:?}", updated; "call_id" => updated.as_ref().map(|u| u.id));
    Ok(updated)
}

fn get_current_call(conn: &PgConnection, chat_id: ChatId) -> QueryResult<Option<RollCall>> {
    use schema::w_roll_calls::{dsl, table};
    let open_calls = table
        .filter(dsl::chat_id.eq(chat_id))
        .filter(dsl::status.eq(CallStatus::Open))
        .order(dsl::updated_at.desc())
        .limit(1)
        .load::<RollCall>(conn)?;

    Ok(first(open_calls))
}

fn get_responses(conn: &PgConnection, call_id: CallId) -> QueryResult<Vec<RollCallResponse>> {
    use schema::w_roll_call_responses::{self, dsl};
    w_roll_call_responses::table
        .filter(dsl::roll_call_id.eq(call_id))
        .order(dsl::updated_at.desc())
        .load::<RollCallResponse>(conn)
}

pub fn get_call_with_responses(
    conn: &PgConnection,
    chat_id: ChatId,
) -> QueryResult<Option<CallWithResponses>> {
    let open_call = match get_current_call(conn, chat_id)? {
        Some(call) => call,
        None => return Ok(None),
    };

    let responses = get_responses(conn, open_call.id)?;
    Ok(Some((open_call, responses)))
}

pub fn set_response(
    conn: &PgConnection,
    chat_id: ChatId,
    user_id: UserId,
    user_name: &str,
    attendance: &Attendance,
) -> QueryResult<Option<CallWithResponses>> {
    set_response_base(conn, chat_id, |call_id| {
        NewRollCallResponse::new_self(call_id, user_id, user_name, attendance)
    })
}

pub fn set_response_for(
    conn: &PgConnection,
    chat_id: ChatId,
    user_name: &str,
    attendance: &Attendance,
) -> QueryResult<Option<CallWithResponses>> {
    set_response_base(conn, chat_id, |call_id| {
        NewRollCallResponse::new_for(call_id, user_name, attendance)
    })
}

fn set_response_base<'a, F>(
    conn: &PgConnection,
    chat_id: ChatId,
    value_fn: F,
) -> QueryResult<Option<CallWithResponses>>
where
    F: Fn(CallId) -> NewRollCallResponse<'a>,
{
    let open_call = match get_current_call(conn, chat_id)? {
        Some(call) => call,
        None => return Ok(None),
    };

    let record = value_fn(open_call.id);
    let update = UpdateRollCallResponse::new(record.user_name, record.status, record.reason);

    use schema::w_roll_call_responses::{dsl, table};
    let inserted = diesel::insert_into(table)
        .values(&record)
        .on_conflict((dsl::roll_call_id, dsl::unique_token))
        .do_update()
        .set(&update)
        .execute(conn)?;

    debug!("Added {} response", inserted; "call_id" => open_call.id);
    let responses = get_responses(conn, open_call.id)?;

    Ok(Some((open_call, responses)))
}
