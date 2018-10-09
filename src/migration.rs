use diesel::prelude::*;
use diesel::sql_query;

use crate::db::establish_connection;
use crate::{IntErrorKind, IntResult};

pub fn run() -> IntResult<()> {
    let con = establish_connection();

    let _ = sql_query(
        r#"CREATE TABLE users (

  id            INT UNSIGNED AUTO_INCREMENT NOT NULL,
  email         VARCHAR(255) NOT NULL UNIQUE,
  username      VARCHAR(20) NOT NULL UNIQUE,
  password      TINYTEXT NOT NULL,
  banned        BOOLEAN DEFAULT FALSE NOT NULL,
  verified      BOOLEAN DEFAULT FALSE NOT NULL,
  email_token   VARCHAR(255),

  PRIMARY KEY (id)
);"#,
    ).execute(&con)
    .map_err(|_| IntErrorKind::QueryError);

    let _ = sql_query(
        r#"CREATE TABLE roles (

  id            INT UNSIGNED NOT NULL,
  name          VARCHAR(20) NOT NULL,

  PRIMARY KEY (id),
  FOREIGN KEY(id) REFERENCES users(id)
);"#,
    ).execute(&con)
    .map_err(|_| IntErrorKind::QueryError);

    Ok(())
}
