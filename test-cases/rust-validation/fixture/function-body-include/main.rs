fn value() -> i32 {
    include!("body.inc")
}

fn main() {
    assert_eq!(value(), 42);
}
