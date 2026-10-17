fn value() -> i32 {
    include!("legacy.rs")
}

fn main() {
    assert_eq!(value(), 42);
}
