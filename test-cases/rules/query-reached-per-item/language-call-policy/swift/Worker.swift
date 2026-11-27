func fetch(item: String) {
    example.store.read_row(item)
}

func load(items: [String]) {
    for item in items {
        fetch(item)
        item.fetch()
    }
}
