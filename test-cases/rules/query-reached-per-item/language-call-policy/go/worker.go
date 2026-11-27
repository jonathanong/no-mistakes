package worker

func fetch(item string) {
    example.store.read_row(item)
}

func load(items []string) {
    for _, item := range items {
        fetch(item)
        item.fetch()
    }
}
