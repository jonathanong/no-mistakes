package com.example

class Client {
    fun load(restTemplate: org.springframework.web.client.RestTemplate): Any? {
        return restTemplate.getForObject("/api/users", Any::class.java)
    }
}
