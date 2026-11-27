package com.example;

public class Computed {
    java.net.URI value = java.net.URI.create("http://localhost:8080/api/users");
    public Object load(org.springframework.web.client.RestTemplate restTemplate, String url) {
        return restTemplate.getForObject(url, Object.class);
    }
}
