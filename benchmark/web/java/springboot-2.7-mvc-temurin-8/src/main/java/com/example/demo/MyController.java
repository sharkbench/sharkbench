package com.example.demo;

import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;
import org.springframework.web.client.RestTemplate;

import java.util.HashMap;
import java.util.Map;

@RestController
class MyController {

    private final RestTemplate restTemplate = new RestTemplate();

    @GetMapping("/api/v1/periodic-table/element")
    public Map<String, Object> getElement(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://web-data-source/element.json", Map.class);
        Map<String, Object> elementData = (Map<String, Object>) elements.get(symbol);

        Map<String, Object> map = new HashMap<String, Object>();
        map.put("name", elementData.get("name"));
        map.put("number", elementData.get("number"));
        map.put("group", elementData.get("group"));
        return map;
    }

    @GetMapping("/api/v1/periodic-table/shells")
    public Map<String, Object> getShells(@RequestParam String symbol) {
        Map<String, Object> elements = restTemplate.getForObject("http://web-data-source/shells.json", Map.class);

        Map<String, Object> map = new HashMap<String, Object>();
        map.put("shells", elements.get(symbol));
        return map;
    }
}
