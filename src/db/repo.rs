use std::time::Duration;

use diesel::result::QueryResult;
#[cfg(test)]
use mockers_derive::mocked;
use r2d2;

use crate::models::*;

use super::helpers::{self as h, Pool};

#[derive(Fail, Debug)]
pub enum DatabaseError {
    #[fail(display = "Database connection error: {}", _0)]
    ConnectError(#[fail(cause)] r2d2::Error),

    #[fail(display = "Database query error: {}", _0)]
    QueryError(#[fail(cause)] diesel::result::Error),
}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[cfg_attr(test, mocked)]
pub trait Repository {
    fn create_call(&self, chat_id: ChatId, title: &str) -> DatabaseResult<RollCall>;

    fn end_call(&self, chat_id: ChatId) -> DatabaseResult<Option<RollCall>>;

    fn update_title(&self, chat_id: ChatId, new_title: &str) -> DatabaseResult<Option<RollCall>>;

    fn update_quiet(
        &self,
        chat_id: ChatId,
        quiet: bool,
    ) -> DatabaseResult<Option<CallWithResponses>>;

    fn set_response(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        user_name: &str,
        attendance: &Attendance,
    ) -> DatabaseResult<Option<CallWithResponses>>;

    fn set_response_for(
        &self,
        chat_id: ChatId,
        user_name: &str,
        attendance: &Attendance,
    ) -> DatabaseResult<Option<CallWithResponses>>;

    fn get_call_with_responses(&self, chat_id: ChatId)
        -> DatabaseResult<Option<CallWithResponses>>;
}

pub struct PostgresRepository {
    pool: Pool,
}

impl PostgresRepository {
    pub fn new(database_url: &str, timeout: Duration) -> DatabaseResult<Self> {
        let pool = h::connect(database_url, timeout).map_err(DatabaseError::ConnectError)?;

        let repository = PostgresRepository { pool };
        Ok(repository)
    }

    fn exec_with_pool<T>(
        &self,
        exec: impl Fn(&h::PooledConnection) -> QueryResult<T>,
    ) -> DatabaseResult<T> {
        let pool = self.pool.clone();
        let connection = pool.get().map_err(DatabaseError::ConnectError)?;
        exec(&connection).map_err(DatabaseError::QueryError)
    }
}

impl Repository for PostgresRepository {
    fn create_call(&self, chat_id: ChatId, title: &str) -> DatabaseResult<RollCall> {
        self.exec_with_pool(|conn| h::create_call(conn, chat_id, title))
    }

    fn end_call(&self, chat_id: ChatId) -> DatabaseResult<Option<RollCall>> {
        self.exec_with_pool(|conn| h::end_call(conn, chat_id))
    }

    fn update_title(&self, chat_id: ChatId, new_title: &str) -> DatabaseResult<Option<RollCall>> {
        self.exec_with_pool(|conn| h::update_title(conn, chat_id, new_title))
    }

    fn update_quiet(
        &self,
        chat_id: ChatId,
        quiet: bool,
    ) -> DatabaseResult<Option<CallWithResponses>> {
        self.exec_with_pool(|conn| h::update_quiet(conn, chat_id, quiet))
    }

    fn set_response(
        &self,
        chat_id: ChatId,
        user_id: UserId,
        user_name: &str,
        attendance: &Attendance,
    ) -> DatabaseResult<Option<CallWithResponses>> {
        self.exec_with_pool(|conn| h::set_response(conn, chat_id, user_id, user_name, attendance))
    }

    fn set_response_for(
        &self,
        chat_id: ChatId,
        user_name: &str,
        attendance: &Attendance,
    ) -> DatabaseResult<Option<CallWithResponses>> {
        self.exec_with_pool(|conn| h::set_response_for(conn, chat_id, user_name, attendance))
    }

    fn get_call_with_responses(
        &self,
        chat_id: ChatId,
    ) -> DatabaseResult<Option<CallWithResponses>> {
        self.exec_with_pool(|conn| h::get_call_with_responses(conn, chat_id))
    }
}
