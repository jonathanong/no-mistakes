package com.example

class Client {
    fun load(restTemplate: org.springframework.web.client.RestTemplate): Any? {
        return restTemplate.getForObject("http://localhost:8080/api/users", Any::class.java)
    }
}
