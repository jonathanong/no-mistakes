fn fetch(item: &str) {
    example.store.read_row(item);
}

fn load(items: &[&str]) {
    for item in items {
        fetch(item);
        item.fetch();
    }
}
