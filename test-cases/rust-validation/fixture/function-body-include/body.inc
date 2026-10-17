{
    let mut values = vec![std::hint::black_box(40)];
    values.retain(|value| *value > 0);
    values[0] + 2
}
