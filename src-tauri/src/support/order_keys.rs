use crate::support::error::{AppError, Result};

const WIDTH: usize = 39;
const STEP: u128 = 1_000_000_000;

pub fn initial_key() -> String {
    format_key(STEP)
}

pub fn append_after(after: Option<&str>) -> Result<String> {
    let base = match after {
        Some(value) => parse_key(value)?,
        None => 0,
    };

    let next = base
        .checked_add(STEP)
        .ok_or_else(|| AppError::Validation("message order key overflow".to_string()))?;
    Ok(format_key(next))
}

pub fn between(after: &str, before: &str) -> Result<Option<String>> {
    let left = parse_key(after)?;
    let right = parse_key(before)?;
    if right <= left {
        return Err(AppError::Validation(
            "message order keys are not strictly ascending".to_string(),
        ));
    }

    if right - left <= 1 {
        return Ok(None);
    }

    Ok(Some(format_key((left + right) / 2)))
}

pub fn format_key(value: u128) -> String {
    format!("{value:0WIDTH$}")
}

pub fn parse_key(value: &str) -> Result<u128> {
    value
        .parse::<u128>()
        .map_err(|err| AppError::Validation(format!("invalid message order key '{value}': {err}")))
}
