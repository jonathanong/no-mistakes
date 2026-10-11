//! Reject precision loss before relying on serde_json's binary64 number values.
pub(super) fn validate(source: &str) -> Result<(), String> {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut quoted = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\\' if quoted => index += 2,
            b'"' => {
                quoted = !quoted;
                index += 1;
            }
            b'-' | b'0'..=b'9' if !quoted => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    index += 1;
                }
                let token = &source[start..index];
                let value: serde_json::Number =
                    serde_json::from_str(token).map_err(|e| e.to_string())?;
                if normalize(token) != normalize(&value.to_string()) {
                    return Err(format!("numeric token `{token}` loses decimal precision; exact compatibility is unproven"));
                }
            }
            _ => index += 1,
        }
    }
    Ok(())
}

fn normalize(token: &str) -> (bool, String, i64) {
    let (mantissa, exponent) = token.split_once(['e', 'E']).map_or((token, 0), |(m, e)| {
        (m, e.parse::<i64>().unwrap_or(i64::MAX))
    });
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.trim_start_matches('-');
    let fractional = mantissa
        .split_once('.')
        .map_or(0, |(_, fraction)| fraction.len() as i64);
    let digits = mantissa.replace('.', "");
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return (false, "0".into(), 0);
    }
    let significant = digits.trim_end_matches('0');
    (
        negative,
        significant.into(),
        exponent
            .saturating_sub(fractional)
            .saturating_add((digits.len() - significant.len()) as i64),
    )
}
