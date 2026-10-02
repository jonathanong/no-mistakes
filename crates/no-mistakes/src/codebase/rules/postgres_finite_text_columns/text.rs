pub(super) fn literal_text(data_type: &str, values: &[String], peers: &[String]) -> String {
    format!(
        "{data_type} column holds a fixed set of values ({}) enforced by CHECK; use an enum type or a foreign key to a lookup table{}",
        format_values(values),
        peer_clause(peers)
    )
}

pub(super) fn name_text(data_type: &str, pattern: &str) -> String {
    format!(
        "{data_type} column name matches {pattern}, which marks a fixed set of values; use an enum type or a foreign key to a lookup table, or add an allow entry with a reason"
    )
}

fn format_values(values: &[String]) -> String {
    let shown = values
        .iter()
        .take(10)
        .map(|value| format!("'{}'", value.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(", ");
    if values.len() > 10 {
        format!("{shown}, … ({} more)", values.len() - 10)
    } else {
        shown
    }
}

fn peer_clause(peers: &[String]) -> String {
    if peers.is_empty() {
        return String::new();
    }
    let verb = if peers.len() == 1 { "has" } else { "have" };
    format!(
        "; {} {verb} the same values and can share the type",
        join_peers(peers)
    )
}

fn join_peers(peers: &[String]) -> String {
    if peers.len() > 5 {
        return format!("{} and {} more", peers[..5].join(", "), peers.len() - 5);
    }
    match peers {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} and {second}"),
        many => {
            let last = many.last().expect("at least three peers");
            format!("{} and {last}", many[..many.len() - 1].join(", "))
        }
    }
}
