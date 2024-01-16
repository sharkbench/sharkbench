package com.example.demo;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

import java.util.HashMap;
import java.util.Map;

@RestController
class MyController {

    private final WebClient webClientElement = WebClient.create("http://web-data-source/element.json");
    private final WebClient webClientShells = WebClient.create("http://web-data-source/shells.json");

    @GetMapping("/api/v1/periodic-table/element")
    public Mono<Map<String, Object>> getElement(@RequestParam String symbol) {
        return webClientElement.get()
                .retrieve()
                .bodyToMono(Map.class)
                .map(elements -> {
                    Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);

                    Map<String, Object> map = new HashMap<String, Object>();
                    map.put("name", elementData.get("name"));
                    map.put("number", elementData.get("number"));
                    map.put("group", elementData.get("group"));
                    return map;
                });
    }

    @GetMapping("/api/v1/periodic-table/shells")
    public Mono<Map<String, Object>> getShells(@RequestParam String symbol) {
        return webClientShells.get()
                .retrieve()
                .bodyToMono(Map.class)
                .map(elements -> {
                    Map<String, Object> map = new HashMap<String, Object>();
                    map.put("shells", elements.get(symbol));
                    return map;
                });
    }
}
