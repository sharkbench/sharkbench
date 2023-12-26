package com.example.demo;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.core.publisher.Mono;

import java.util.Map;

@RestController
class MyController {

    private final WebClient webClient = WebClient.create("http://web-data-source/data.json");

    @GetMapping("/api/v1/periodic-table/element")
    public Mono<Map<String, Object>> getElement(@RequestParam String symbol) {
        return webClient.get()
                .retrieve()
                .bodyToMono(Map.class)
                .map(elements -> {
                    Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);
                    return Map.of(
                            "name", elementData.get("name"),
                            "number", elementData.get("number"),
                            "group", elementData.get("group")
                    );
                });
    }

    @GetMapping("/api/v1/periodic-table/shells")
    public Mono<Map<String, Object>> getShells(@RequestParam String symbol) {
        return webClient.get()
                .retrieve()
                .bodyToMono(Map.class)
                .map(elements -> {
                    Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);
                    return Map.of("shells", elementData.get("shells"));
                });
    }
}
