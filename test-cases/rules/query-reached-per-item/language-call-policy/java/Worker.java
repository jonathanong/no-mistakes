class Worker {
    static void fetch(String item) {
        example.store.read_row(item);
    }

    static void load(java.util.List<String> items) {
        for (String item : items) {
            fetch(item);
            item.fetch();
        }
    }
}
