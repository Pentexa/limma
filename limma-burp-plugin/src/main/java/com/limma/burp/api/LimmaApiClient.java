package com.limma.burp.api;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.limma.burp.models.LimmaModels;
import okhttp3.*;
import java.io.IOException;
import java.util.List;
import java.util.concurrent.TimeUnit;
import burp.api.montoya.logging.Logging;

public class LimmaApiClient {

    private final OkHttpClient client;
    private final ObjectMapper mapper;
    private final String baseUrl;
    private final Logging logging;
    private String sessionId;

    public LimmaApiClient(String baseUrl, Logging logging) {
        this.baseUrl = baseUrl;
        this.logging = logging;
        this.mapper = new ObjectMapper();
        this.client = new OkHttpClient.Builder()
            .connectTimeout(10, TimeUnit.SECONDS)
            .writeTimeout(10, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .build();
    }

    public boolean handshake(String targetUrl, String burpVersion, String pluginVersion) {
        try {
            LimmaModels.HandshakeRequest req = new LimmaModels.HandshakeRequest();
            req.targetUrl = targetUrl;
            req.burpVersion = burpVersion;
            req.pluginVersion = pluginVersion;

            String json = mapper.writeValueAsString(req);
            RequestBody body = RequestBody.create(json, MediaType.parse("application/json"));
            
            Request request = new Request.Builder()
                .url(baseUrl + "/api/burp/handshake")
                .post(body)
                .build();

            try (Response response = client.newCall(request).execute()) {
                if (response.isSuccessful() && response.body() != null) {
                    LimmaModels.HandshakeResponse res = mapper.readValue(response.body().string(), LimmaModels.HandshakeResponse.class);
                    this.sessionId = res.sessionId;
                    logging.logToOutput("Handshake successful. Session ID: " + this.sessionId);
                    return true;
                } else {
                    logging.logToError("Handshake failed with status: " + response.code());
                }
            }
        } catch (Exception e) {
            logging.logToError("Handshake exception: " + e.getMessage());
        }
        return false;
    }

    public void sendTraffic(List<LimmaModels.TrafficItem> items) {
        if (this.sessionId == null || items.isEmpty()) return;

        try {
            LimmaModels.ImportTrafficRequest req = new LimmaModels.ImportTrafficRequest();
            req.sessionId = this.sessionId;
            req.items = items;

            String json = mapper.writeValueAsString(req);
            RequestBody body = RequestBody.create(json, MediaType.parse("application/json"));
            
            Request request = new Request.Builder()
                .url(baseUrl + "/api/burp/import-traffic")
                .post(body)
                .build();

            // Fire and forget, or handle async
            client.newCall(request).enqueue(new Callback() {
                @Override
                public void onFailure(Call call, IOException e) {
                    logging.logToError("Traffic import failed: " + e.getMessage());
                }

                @Override
                public void onResponse(Call call, Response response) throws IOException {
                    response.close();
                }
            });
        } catch (Exception e) {
            logging.logToError("Traffic import exception: " + e.getMessage());
        }
    }

    public String getSessionId() {
        return sessionId;
    }

    public OkHttpClient getClient() {
        return client;
    }

    public String getBaseUrl() {
        return baseUrl;
    }
}
