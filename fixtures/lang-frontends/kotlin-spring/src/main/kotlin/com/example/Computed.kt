package com.example

class Computed {
    // v1 skips computed and empty Spring mappings.
    @GetMapping(PREFIX)
    fun hidden(): Any? = null

    @GetMapping
    fun empty(): Any? = null
}
