module std.fs
import std.prelude
import std.types

/// Read entire file as text (Swift `String(contentsOfFile:)` / Kotlin `readText`).
/// Panics on failure — prefer `read_to_string` for fallible use.
export function read_file(path: string) -> string {
    unwrap_ok(__file_read(path))
}

/// Write text, panicking on failure.
export function write_file(path: string, content: string) -> unit {
    unwrap_ok(__file_write(path, content))
}

/// True if a file exists at `path`.
export function exists(path: string) -> boolean {
    __file_exists(path)
}

/// Delete a file, panicking on failure.
export function remove(path: string) -> unit {
    unwrap_ok(__file_delete(path))
}

/// Fallible read (Swift / Kotlin Result style).
export function read_to_string(path: string) -> Result<string, string> {
    __file_read(path)
}

/// Fallible write.
export function write(path: string, content: string) -> Result<unit, string> {
    __file_write(path, content)
}

/// Fallible delete.
export function remove_file(path: string) -> Result<unit, string> {
    __file_delete(path)
}
