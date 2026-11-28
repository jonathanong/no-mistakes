fn inspect<'static_life>(value: &'static_life Greeter) {
    let _ch = 'x';
    let _esc = '\n';
    let _ = value;
}
