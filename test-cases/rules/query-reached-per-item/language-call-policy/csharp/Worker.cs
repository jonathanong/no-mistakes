class Worker {
    static void fetch(string item) {
        example.store.read_row(item);
    }

    static void load(string[] items) {
        foreach (var item in items) {
            fetch(item);
            item.fetch();
        }
    }
}
