void fetch(String item) {
  example.store.read_row(item);
}

void load(List items) {
  for (final item in items) {
    fetch(item);
    item.fetch();
  }
}
