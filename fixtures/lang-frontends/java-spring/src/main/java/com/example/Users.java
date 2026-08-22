package com.example;

@RequestMapping("/api")
public class Users {
    @GetMapping("/users")
    public Object listUsers() {
        return User.list();
    }

    @PostMapping("/users")
    public Object createUser() {
        return null;
    }
}
