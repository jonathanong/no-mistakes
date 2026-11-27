package com.example;

public class Client {
    public Object load(org.springframework.web.client.RestTemplate restTemplate) {
        return restTemplate.getForObject("/api/users", Object.class);
    }
}
