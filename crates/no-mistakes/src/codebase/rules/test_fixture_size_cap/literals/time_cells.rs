use super::{unwrap_ts_wrappers, ArrayExpressionElement, Expression};

#[derive(Default)]
pub(super) struct TimeCells {
    duration_seconds: Option<f64>,
    period: bool,
    date: bool,
}

impl TimeCells {
    pub(super) fn for_elements(elements: &[ArrayExpressionElement]) -> Self {
        let mut time = Self::default();
        for label in elements.iter().filter_map(string_cell) {
            time.duration_seconds = time.duration_seconds.or_else(|| interval_seconds(label));
            time.period |= label.ends_with(" period");
            time.date |= is_date_label(label);
        }
        time
    }

    pub(super) fn contains(&self, value: f64) -> bool {
        self.duration_seconds == Some(value)
            || (self.period && (1_000_000_000.0..10_000_000_000.0).contains(&value))
            || (self.date && is_compact_date(value))
    }
}

fn string_cell<'a>(cell: &'a ArrayExpressionElement<'a>) -> Option<&'a str> {
    match unwrap_ts_wrappers(cell.as_expression()?) {
        Expression::StringLiteral(literal) => literal.value.as_str(),
        _ => None,
    }
}

fn interval_seconds(label: &str) -> Option<f64> {
    if label == "daily" {
        return Some(86_400.0);
    }
    let interval = label.strip_prefix("every ")?;
    [("s", 1.0), ("m", 60.0), ("h", 3_600.0), ("d", 86_400.0)]
        .into_iter()
        .find_map(|(unit, seconds)| {
            let amount = interval.strip_suffix(unit)?.parse::<u32>().ok()?;
            Some(f64::from(amount) * seconds)
        })
}

fn is_date_label(label: &str) -> bool {
    let bytes = label.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[..4].iter().chain(&bytes[5..7]).chain(&bytes[8..]).all(u8::is_ascii_digit)
}

fn is_compact_date(value: f64) -> bool {
    if !(19_000_000.0..21_000_000.0).contains(&value) || value.fract() != 0.0 {
        return false;
    }
    let date = value as u32;
    (1..=12).contains(&(date / 100 % 100)) && (1..=31).contains(&(date % 100))
}
