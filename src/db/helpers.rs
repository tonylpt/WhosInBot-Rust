use std::time::Duration;

use diesel::dsl::sql;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::sql_types::*;
use r2d2;

use crate::models::*;
use crate::schema;
use crate::util::collections::CollectionTools;

pub type Manager = ConnectionManager<PgConnection>;
pub type Pool = r2d2::Pool<Manager>;
pub type PooledConnection = r2d2::PooledConnection<Manager>;
pub type PoolResult = Result<Pool, r2d2::Error>;
pub type ConnectionResult = diesel::ConnectionResult<PgConnection>;

pub fn create_pool(database_url: &str, timeout: Duration) -> PoolResult {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .connection_timeout(timeout)
        .build(manager)
}

pub fn create_connection(database_url: &str) -> ConnectionResult {
    PgConnection::establish(database_url)
}

pub fn create_call(conn: &PgConnection, chat_id: ChatId, title: &str) -> QueryResult<RollCall> {
    use schema::w_roll_calls::{dsl, table};

    let close_all_calls = || -> QueryResult<usize> {
        let open_calls = table
            .filter(dsl::chat_id.eq(chat_id))
            .filter(dsl::status.ne(CallStatus::Closed));

        let update_close = UpdateRollCall::new().with_status(CallStatus::Closed);

        let updated = diesel::update(open_calls).set(update_close).execute(conn)?;

        debug!("Closed {} open calls", updated);
        Ok(updated)
    };

    let delete_old_calls = || -> QueryResult<usize> {
        const LIMIT: i32 = 10;
        let deleted = diesel::sql_query(
            "DELETE \
                FROM w_roll_calls \
                WHERE chat_id = $1 \
                  AND created_at < ( \
                  SELECT MIN(latest.created_at) \
                  FROM (SELECT created_at \
                        FROM w_roll_calls \
                        WHERE chat_id = $1 \
                        ORDER BY created_at DESC \
                        LIMIT $2) AS latest \
                )")
            .bind::<BigInt, _>(chat_id)
            .bind::<Integer, _>(LIMIT)
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
    use schema::w_roll_calls::table;

    let filter_open_call = table.filter(
        sql("id IN (SELECT id FROM w_roll_calls WHERE status = ")
            .bind::<VarChar, _>(CallStatus::Open)
            .sql("AND chat_id = ")
            .bind::<BigInt, _>(chat_id)
            .sql("ORDER BY created_at DESC LIMIT 1)"));

    let updated = diesel::update(filter_open_call)
        .set(update)
        .get_results(conn)?;

    let updated: Option<RollCall> = updated.take_first();
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

    Ok(open_calls.take_first())
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
