const EXACT_TIMING_ORDER: &[(&str, u16)] = &[
    ("discovery", 10),
    ("read", 20),
    ("parse", 30),
    ("manifest", 40),
    ("resolve", 50),
    ("search", 60),
    ("ingest", 61),
    ("parse+analysis", 62),
    ("analysis", 63),
    ("prepare", 100),
    ("discover.dotnet", 110),
    ("discover.vitest", 111),
    ("discover.playwright", 112),
    ("discover.swift", 113),
    ("graph", 120),
    ("select.dotnet", 130),
    ("select.vitest", 131),
    ("select.playwright", 132),
    ("select.swift", 133),
    ("generic-checks", 140),
    ("analysis.react", 200),
    ("analysis.queues", 201),
    ("analysis.rules", 202),
    ("analysis.integration", 203),
    ("analysis.codebase", 204),
    ("analysis.filesystem_rules", 205),
    ("output", 900),
];

const PREFIX_TIMING_ORDER: &[(&str, u16)] = &[
    ("discovery.", 11),
    ("read.", 21),
    ("parse.", 31),
    ("manifest.", 41),
    ("resolve.", 51),
    ("graph.", 301),
    ("traversal.", 400),
    ("analysis.", 500),
    ("rules.", 600),
    ("playwright.", 700),
];

const UNKNOWN_TIMING_ORDER: u16 = 800;

pub(super) fn rank(label: &str) -> u16 {
    exact_rank(label)
        .or_else(|| prefix_rank(label))
        .unwrap_or(UNKNOWN_TIMING_ORDER)
}

fn exact_rank(label: &str) -> Option<u16> {
    EXACT_TIMING_ORDER
        .iter()
        .find_map(|(exact, rank)| (*exact == label).then_some(*rank))
}

fn prefix_rank(label: &str) -> Option<u16> {
    PREFIX_TIMING_ORDER
        .iter()
        .find_map(|(prefix, rank)| label.starts_with(prefix).then_some(*rank))
}
