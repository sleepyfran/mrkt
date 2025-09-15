pub mod accounts_repo;
pub mod sessions_repo;
pub mod transactions_repo;
pub mod users_repo;

pub type Pool = sqlx::SqlitePool;
pub type DatabaseError = sqlx::Error;

/// Checks if the given error is an insertion error due to a unique constraint violation.
pub fn is_unique_constraint_violation(err: &DatabaseError) -> bool {
    let db_error = err.as_database_error();
    db_error.is_some() && db_error.unwrap().is_unique_violation()
}

/// Checks if the given error is an insertion error due to a foreign key constraint violation.
pub fn is_foreign_key_violation(err: &DatabaseError) -> bool {
    let db_error = err.as_database_error();
    db_error.is_some() && db_error.unwrap().is_foreign_key_violation()
}
