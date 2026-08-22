package com.example

@RequestMapping("/api")
class Users {
    @GetMapping("/users")
    fun listUsers(): Any? = User.list()

    @PostMapping("/users")
    fun createUser(): Any? = null
}
