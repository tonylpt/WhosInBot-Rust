use std::fmt;
use std::io::Write;
use std::str::FromStr;

use diesel::backend::Backend;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::Text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attendance {
    pub status: AttendanceStatus,
    pub reason: String,
}

impl Attendance {
    pub fn new(status: AttendanceStatus, reason: String) -> Attendance {
        Attendance {
            status,
            reason,
        }
    }
}

/** Boilerplate to enable enum in database model. */
#[derive(AsExpression, FromSqlRow)]
#[derive(Hash, Debug, PartialEq, Eq, Copy, Clone)]
#[sql_type = "Text"]
pub enum AttendanceStatus {
    In,
    Out,
    Maybe,
}

impl fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            AttendanceStatus::In => "IN",
            AttendanceStatus::Out => "OUT",
            AttendanceStatus::Maybe => "MAYBE",
        })
    }
}

impl FromStr for AttendanceStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "IN" => Ok(AttendanceStatus::In),
            "OUT" => Ok(AttendanceStatus::Out),
            "MAYBE" => Ok(AttendanceStatus::Maybe),
            x => Err(format!("Unrecognized variant {}", x)),
        }
    }
}

impl<DB> ToSql<Text, DB> for AttendanceStatus
    where DB: Backend,
          String: ToSql<Text, DB>
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.to_string().to_sql(out)
    }
}

impl<DB> FromSql<Text, DB> for AttendanceStatus
    where DB: Backend,
          String: FromSql<Text, DB>
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        String::from_sql(bytes)?
            .parse::<AttendanceStatus>()
            .map_err(|e| e.into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_string() {
        assert_eq!("IN", AttendanceStatus::In.to_string());
        assert_eq!("OUT", AttendanceStatus::Out.to_string());
        assert_eq!("MAYBE", AttendanceStatus::Maybe.to_string());
    }

    #[test]
    fn test_status_from_valid_string() -> Result<(), String> {
        assert_eq!(AttendanceStatus::In, "IN".parse::<AttendanceStatus>()?);
        assert_eq!(AttendanceStatus::Out, "OUT".parse::<AttendanceStatus>()?);
        assert_eq!(AttendanceStatus::Maybe, "MAYBE".parse::<AttendanceStatus>()?);
        Ok(())
    }

    #[test]
    fn test_status_from_invalid_string() {
        assert!("INVALID".parse::<AttendanceStatus>().is_err())
    }
}
