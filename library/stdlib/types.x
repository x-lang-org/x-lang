module std.types

/// Presence or absence of a value (Swift Optional / Kotlin T?).
export enum Option<T> {
    None,
    Some(T),
}

/// Success or failure (Swift Result / Kotlin Result).
export enum Result<T, E> {
    Ok(T),
    Err(E),
}

/// True when `opt` is `Some`.
export function is_some<T>(opt: Option<T>) -> boolean {
    match opt {
        Some(_) => true,
        None => false,
    }
}

/// True when `opt` is `None`.
export function is_none<T>(opt: Option<T>) -> boolean {
    match opt {
        Some(_) => false,
        None => true,
    }
}

/// Unwrap or return `default` (Kotlin `getOrDefault` / Swift `??`).
export function unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,
        None => default,
    }
}

/// True when `res` is `Ok`.
export function is_ok<T, E>(res: Result<T, E>) -> boolean {
    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// True when `res` is `Err`.
export function is_err<T, E>(res: Result<T, E>) -> boolean {
    match res {
        Ok(_) => false,
        Err(_) => true,
    }
}
