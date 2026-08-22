package com.example;

public class Computed {
    // v1 skips computed and empty Spring mappings.
    @GetMapping(PREFIX)
    public Object hidden() {
        return null;
    }

    @GetMapping
    public Object empty() {
        return null;
    }
}
