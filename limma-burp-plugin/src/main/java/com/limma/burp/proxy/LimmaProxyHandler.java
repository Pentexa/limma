package com.limma.burp.proxy;

import burp.api.montoya.http.handler.*;
import burp.api.montoya.http.message.requests.HttpRequest;
import com.limma.burp.api.LimmaApiClient;
import com.limma.burp.models.LimmaModels;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class LimmaProxyHandler implements HttpHandler {

    private final LimmaApiClient apiClient;

    public LimmaProxyHandler(LimmaApiClient apiClient) {
        this.apiClient = apiClient;
    }

    @Override
    public RequestToBeSentAction handleHttpRequestToBeSent(HttpRequestToBeSent requestToBeSent) {
        // We only care about the response to capture full traffic
        return RequestToBeSentAction.continueWith(requestToBeSent);
    }

    @Override
    public ResponseReceivedAction handleHttpResponseReceived(HttpResponseReceived responseReceived) {
        try {
            HttpRequest request = responseReceived.initiatingRequest();
            
            // Only process in-scope or specific targets if configured. 
            // For now, we process all. In production, we'd check if the URL matches our Handshake target.
            
            LimmaModels.TrafficItem item = new LimmaModels.TrafficItem();
            item.url = request.url();
            item.method = request.method();
            
            // Extract headers
            Map<String, String> reqHeaders = new HashMap<>();
            request.headers().forEach(h -> reqHeaders.put(h.name(), h.value()));
            item.requestHeaders = reqHeaders;
            item.requestBody = request.bodyToString();
            
            item.responseStatus = responseReceived.statusCode();
            
            Map<String, String> resHeaders = new HashMap<>();
            responseReceived.headers().forEach(h -> resHeaders.put(h.name(), h.value()));
            item.responseHeaders = resHeaders;
            item.responseBody = responseReceived.bodyToString();
            
            item.timestamp = System.currentTimeMillis();
            item.toolSource = responseReceived.toolSource().toolType().name(); // PROXY, REPEATER, etc.

            // Send in a batch of 1 for now (real-time)
            List<LimmaModels.TrafficItem> items = new ArrayList<>();
            items.add(item);
            apiClient.sendTraffic(items);

        } catch (Exception e) {
            // Ignore extraction errors so proxy continues working
        }

        return ResponseReceivedAction.continueWith(responseReceived);
    }
}
