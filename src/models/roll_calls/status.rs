use std::fmt;
use std::io::Write;
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::Text;
use diesel::serialize::{self, ToSql, Output};
use diesel::backend::Backend;
use std::str::FromStr;


/** Boilerplate to enable enum in database model. */
#[derive(AsExpression, FromSqlRow)]
#[derive(Hash, Debug, PartialEq, Eq, Copy, Clone)]
#[sql_type = "Text"]
pub enum CallStatus {
    Open,
    Closed,
}

impl fmt::Display for CallStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            CallStatus::Open => "OPEN",
            CallStatus::Closed => "CLOSED",
        })
    }
}

impl FromStr for CallStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OPEN" => Ok(CallStatus::Open),
            "CLOSED" => Ok(CallStatus::Closed),
            x => Err(format!("Unrecognized variant {}", x)),
        }
    }
}

impl<DB> ToSql<Text, DB> for CallStatus
    where DB: Backend,
          String: ToSql<Text, DB>
{
    fn to_sql<W: Write>(&self, out: &mut Output<W, DB>) -> serialize::Result {
        self.to_string().to_sql(out)
    }
}

impl<DB> FromSql<Text, DB> for CallStatus
    where DB: Backend,
          String: FromSql<Text, DB>
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        String::from_sql(bytes)?
            .parse::<CallStatus>()
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_to_string() {
        assert_eq!("OPEN", CallStatus::Open.to_string());
        assert_eq!("CLOSED", CallStatus::Closed.to_string());
    }

    #[test]
    fn test_status_from_valid_string() -> Result<(), String> {
        assert_eq!(CallStatus::Open, "OPEN".parse::<CallStatus>()?);
        assert_eq!(CallStatus::Closed, "CLOSED".parse::<CallStatus>()?);
        Ok(())
    }

    #[test]
    fn test_status_from_invalid_string() {
        assert!("INVALID".parse::<CallStatus>().is_err())
    }
}
