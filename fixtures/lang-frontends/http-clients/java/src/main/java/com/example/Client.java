package com.example;

public class Client {
    public Object load(org.springframework.web.client.RestTemplate restTemplate) {
        return restTemplate.getForObject("http://localhost:8080/api/users", Object.class);
    }
}
