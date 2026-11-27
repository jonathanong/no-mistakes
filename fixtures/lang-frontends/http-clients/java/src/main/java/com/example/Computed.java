package com.example;

public class Computed {
    public Object load(org.springframework.web.client.RestTemplate restTemplate, String url) {
        return restTemplate.getForObject(url, Object.class);
    }
}
