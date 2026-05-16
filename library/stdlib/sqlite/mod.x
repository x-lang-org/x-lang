// X-SQLite - SQLite C Bindings for X Language
// Uses the actual SQLite library via C FFI

module sqlite

import std.prelude

// ============================================================================
// SQLite C API Bindings
// ============================================================================
// Note: Database and statement handles are passed as Int (pointer values)
// since X doesn't have native opaque type support yet.

// Open database
extern "C" function sqlite3_open(filename: *signed 8-bit integer, ppDb: *Int) -> signed 32-bit integer

// Close database
extern "C" function sqlite3_close(db: Int) -> signed 32-bit integer

// Execute SQL (for CREATE, INSERT, UPDATE, DELETE)
extern "C" function sqlite3_exec(db: Int, sql: *signed 8-bit integer, callback: Int, arg: *Int, errmsg: *(*signed 8-bit integer)) -> signed 32-bit integer

// Prepare statement
extern "C" function sqlite3_prepare_v2(db: Int, zSql: *signed 8-bit integer, nByte: signed 32-bit integer, ppStmt: *Int, pzTail: *(*signed 8-bit integer)) -> signed 32-bit integer

// Step through query results
extern "C" function sqlite3_step(stmt: Int) -> signed 32-bit integer

// Finalize statement
extern "C" function sqlite3_finalize(stmt: Int) -> signed 32-bit integer

// Get error message
extern "C" function sqlite3_errmsg(db: Int) -> *signed 8-bit integer

// Free memory
extern "C" function sqlite3_free(ptr: *signed 8-bit integer) -> unit

// Get number of columns in result
extern "C" function sqlite3_column_count(stmt: Int) -> signed 32-bit integer

// Get column type
extern "C" function sqlite3_column_type(stmt: Int, iCol: signed 32-bit integer) -> signed 32-bit integer

// Get column values
extern "C" function sqlite3_column_int64(stmt: Int, iCol: signed 32-bit integer) -> signed 64-bit integer
extern "C" function sqlite3_column_double(stmt: Int, iCol: signed 32-bit integer) -> float
extern "C" function sqlite3_column_text(stmt: Int, iCol: signed 32-bit integer) -> *signed 8-bit integer
extern "C" function sqlite3_column_bytes(stmt: Int, iCol: signed 32-bit integer) -> signed 32-bit integer

// ============================================================================
// SQLite Constants
// ============================================================================

const SQLITE_OK: signed 32-bit integer = 0
const SQLITE_ERROR: signed 32-bit integer = 1
const SQLITE_ROW: signed 32-bit integer = 100
const SQLITE_DONE: signed 32-bit integer = 101

const SQLITE_INTEGER: signed 32-bit integer = 1
const SQLITE_FLOAT: signed 32-bit integer = 2
const SQLITE_TEXT: signed 32-bit integer = 3
const SQLITE_NULL: signed 32-bit integer = 5

// ============================================================================
// X Language API
// ============================================================================

/// Open or create a SQLite database
/// Returns database handle (as integer/pointer)
export function open(path: string) -> Result<Int, string> {
    let db_handle: Int = 0
    let result = sqlite3_open(
        path as *signed 8-bit integer,
        &db_handle as *Int
    )

    if result == SQLITE_OK {
        Ok(db_handle)
    } else {
        Err("Failed to open database: " + path)
    }
}

/// Close a database connection
export function close(db: Int) -> Result<unit, string> {
    let result = sqlite3_close(db)
    if result == SQLITE_OK {
        Ok(())
    } else {
        Err("Failed to close database")
    }
}

/// Execute SQL (CREATE, INSERT, UPDATE, DELETE)
export function execute(db: Int, sql: string) -> Result<Int, string> {
    let errmsg: Int = 0
    let callback_arg: Int = 0
    let user_arg: Int = 0
    let result = sqlite3_exec(
        db,
        sql as *signed 8-bit integer,
        callback_arg,
        &user_arg as *Int,
        &errmsg as *(*signed 8-bit integer)
    )

    if result == SQLITE_OK {
        Ok(0)
    } else {
        if errmsg != 0 {
            sqlite3_free(errmsg as *signed 8-bit integer)
        }
        Err("SQL execution failed with code: " + result)
    }
}

/// Query data (SELECT) - returns all matching rows
export function query(db: Int, sql: string) -> Result<[[SqlValue]], string> {
    let stmt_ptr: Int = 0
    let tail: Int = 0

    // Prepare statement
    let prep = sqlite3_prepare_v2(
        db,
        sql as *signed 8-bit integer,
        -1 as signed 32-bit integer,
        &stmt_ptr as *Int,
        &tail as *(*signed 8-bit integer)
    )

    if prep != SQLITE_OK {
        return Err("Failed to prepare statement")
    }

    let mut rows: [[SqlValue]] = []

    // Fetch all rows
    loop {
        let step_result = sqlite3_step(stmt_ptr)

        if step_result == SQLITE_ROW {
            let row = fetch_row(stmt_ptr)
            rows.push(row)
        } else if step_result == SQLITE_DONE {
            break
        } else {
            sqlite3_finalize(stmt_ptr)
            return Err("Query execution failed")
        }
    }

    sqlite3_finalize(stmt_ptr)
    Ok(rows)
}

/// Fetch a single row from prepared statement
function fetch_row(stmt: Int) -> [SqlValue] {
    let col_count = sqlite3_column_count(stmt) as Int
    let mut row: [SqlValue] = []

    let mut col: Int = 0
    while col < col_count {
        let col_type = sqlite3_column_type(stmt, col as signed 32-bit integer)

        if col_type == SQLITE_INTEGER {
            let col_val = sqlite3_column_int64(stmt, col as signed 32-bit integer)
            row.push(SqlValue { kind: SqlKind.Integer, int_value: col_val, float_value: 0.0, text_value: "" })
        } else if col_type == SQLITE_FLOAT {
            let col_val = sqlite3_column_double(stmt, col as signed 32-bit integer)
            row.push(SqlValue { kind: SqlKind.Float, int_value: 0, float_value: col_val, text_value: "" })
        } else if col_type == SQLITE_TEXT {
            let text_ptr = sqlite3_column_text(stmt, col as signed 32-bit integer)
            row.push(SqlValue { kind: SqlKind.Text, int_value: 0, float_value: 0.0, text_value: "" })
        } else {
            row.push(SqlValue { kind: SqlKind.Null, int_value: 0, float_value: 0.0, text_value: "" })
        }

        col = col + 1
    }

    row
}

// ============================================================================
// Data Types
// ============================================================================

export enum SqlKind {
    Null,
    Integer,
    Float,
    Text,
}

export record SqlValue {
    kind: SqlKind
    int_value: Int
    float_value: Float
    text_value: string
}

// ============================================================================
// Helper Constructors
// ============================================================================

export function integer(v: Int) -> SqlValue {
    SqlValue { kind: SqlKind.Integer, int_value: v, float_value: 0.0, text_value: "" }
}

export function float(v: Float) -> SqlValue {
    SqlValue { kind: SqlKind.Float, int_value: 0, float_value: v, text_value: "" }
}

export function text(v: string) -> SqlValue {
    SqlValue { kind: SqlKind.Text, int_value: 0, float_value: 0.0, text_value: v }
}

export function null_value() -> SqlValue {
    SqlValue { kind: SqlKind.Null, int_value: 0, float_value: 0.0, text_value: "" }
}

// ============================================================================
// High-level API
// ============================================================================

/// Run a SELECT query
export function select(db: Int, sql: string) -> Result<[[SqlValue]], string> {
    query(db, sql)
}

/// Create a table
export function create_table(db: Int, sql: string) -> Result<unit, string> {
    match execute(db, sql) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
    }
}

/// Insert a row
export function insert(db: Int, sql: string) -> Result<Int, string> {
    execute(db, sql)
}

/// Get last error message
export function last_error(db: Int) -> string {
    let msg_ptr = sqlite3_errmsg(db)
    ""
}

// ============================================================================
// Transaction Support (simplified)
// ============================================================================

// Transaction helper - executes SQL within a transaction
// Note: Simplified implementation for now
export function with_transaction(db: Int, sql: string) -> Result<unit, string> {
    execute(db, "BEGIN TRANSACTION")
    execute(db, sql)
    execute(db, "COMMIT")
    Ok(())
}