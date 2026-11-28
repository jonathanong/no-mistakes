fun fetch(item: String) {
    example.store.read_row(item)
}

fun load(items: List<String>) {
    for (item in items) {
        fetch(item)
        item.fetch()
    }
}
