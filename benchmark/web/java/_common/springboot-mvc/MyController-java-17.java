package com.example.demo;

import org.apache.hc.client5.http.impl.classic.CloseableHttpClient;
import org.apache.hc.client5.http.impl.classic.HttpClientBuilder;
import org.apache.hc.client5.http.impl.io.PoolingHttpClientConnectionManager;
import org.springframework.http.client.HttpComponentsClientHttpRequestFactory;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.client.RestTemplate;

import jakarta.annotation.PostConstruct;

import java.util.Map;

@RestController
class MyController {
    private RestTemplate restTemplate;

    @PostConstruct
    public void init() {
        PoolingHttpClientConnectionManager connectionManager = new PoolingHttpClientConnectionManager();
        connectionManager.setMaxTotal(64);
        connectionManager.setDefaultMaxPerRoute(32);

        CloseableHttpClient httpClient = HttpClientBuilder.create()
                .setConnectionManager(connectionManager)
                .build();

        HttpComponentsClientHttpRequestFactory requestFactory = new HttpComponentsClientHttpRequestFactory();
        requestFactory.setHttpClient(httpClient);

        this.restTemplate = new RestTemplate(requestFactory);
    }

    @GetMapping("/api/v1/periodic-table/element")
    public Map<String, Object> getElement(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://web-data-source/element.json", Map.class);
        Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);

        return Map.of(
                "name", elementData.get("name"),
                "number", elementData.get("number"),
                "group", elementData.get("group")
        );
    }

    @GetMapping("/api/v1/periodic-table/shells")
    public Map<String, Object> getShells(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://web-data-source/shells.json", Map.class);

        return Map.of("shells", elements.get(symbol));
    }
}
