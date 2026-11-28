def fetch(item):
    return example.store.read_row(item)

def load(items):
    for item in items:
        fetch(item)
        item.fetch()
