// The Actions runner renders numeric `format` replacements with .NET's
// invariant `Double.ToString("G15")` semantics, rather than Rust's shortest
// representation. Keep the fixed/scientific boundary and 15-digit rounding
// explicit so static conditions match the runner.
pub(super) fn github_format_number(value: f64) -> Option<String> {
    value.is_finite().then_some(())?;
    if value == 0.0 {
        return Some(if value.is_sign_negative() {
            "-0".to_string()
        } else {
            "0".to_string()
        });
    }
    let scientific = format!("{value:.14e}");
    let (mantissa, exponent) = scientific.split_once('e')?;
    let exponent = exponent.parse::<i32>().ok()?;
    if (-4..15).contains(&exponent) {
        let precision = usize::try_from(14 - exponent).ok()?;
        return Some(trim_decimal_zeros(format!("{value:.precision$}")));
    }
    Some(format!(
        "{}E{exponent:+03}",
        trim_decimal_zeros(mantissa.to_string())
    ))
}

fn trim_decimal_zeros(mut value: String) -> String {
    if value.contains('.') {
        value = value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();
    }
    value
}
