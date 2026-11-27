package com.example

class Computed {
    fun load(restTemplate: org.springframework.web.client.RestTemplate, url: String): Any? {
        return restTemplate.getForObject(url, Any::class.java)
    }
}
