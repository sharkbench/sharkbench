package com.example.demo;

import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClientBuilder;
import org.apache.http.impl.conn.PoolingHttpClientConnectionManager;
import org.springframework.http.client.HttpComponentsClientHttpRequestFactory;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.client.RestTemplate;

import java.util.HashMap;
import java.util.Map;

@RestController
class MyController {

    private final RestTemplate restTemplate;

    MyController() {
        PoolingHttpClientConnectionManager connectionManager = new PoolingHttpClientConnectionManager();
        connectionManager.setMaxTotal(64);
        connectionManager.setDefaultMaxPerRoute(32);

        CloseableHttpClient httpClient = HttpClientBuilder.create()
                .setConnectionManager(connectionManager)
                .build();

        this.restTemplate = new RestTemplate(new HttpComponentsClientHttpRequestFactory(httpClient));
    }

    @GetMapping("/api/v1/periodic-table/element")
    public Map<String, Object> getElement(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://127.0.0.1:5002/element.json", Map.class);
        Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);

        Map<String, Object> map = new HashMap<String, Object>();
        map.put("name", elementData.get("name"));
        map.put("number", elementData.get("number"));
        map.put("group", elementData.get("group"));
        return map;
    }

    @GetMapping("/api/v1/periodic-table/shells")
    public Map<String, Object> getShells(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://127.0.0.1:5002/shells.json", Map.class);

        Map<String, Object> map = new HashMap<String, Object>();
        map.put("shells", elements.get(symbol));
        return map;
    }
}
