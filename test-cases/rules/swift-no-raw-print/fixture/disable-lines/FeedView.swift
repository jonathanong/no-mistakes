func load() {
    print("[DEBUG]", message()) // no-mistakes-disable-line swift-no-raw-print: fixture exception
    // no-mistakes-disable-next-line swift-no-raw-print: fixture exception
    print("[DEBUG]", message())
    print("[DEBUG]", message())
}

func message() -> String { "loaded" }
