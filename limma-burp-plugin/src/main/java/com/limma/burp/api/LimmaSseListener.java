package com.limma.burp.api;

import burp.api.montoya.MontoyaApi;
import burp.api.montoya.http.message.HttpRequestResponse;
import burp.api.montoya.http.message.requests.HttpRequest;
import burp.api.montoya.scanner.audit.issues.AuditIssue;
import burp.api.montoya.scanner.audit.issues.AuditIssueConfidence;
import burp.api.montoya.scanner.audit.issues.AuditIssueSeverity;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.limma.burp.models.LimmaModels;
import okhttp3.Request;
import okhttp3.Response;
import okhttp3.sse.EventSource;
import okhttp3.sse.EventSourceListener;
import okhttp3.sse.EventSources;

import java.util.concurrent.TimeUnit;

public class LimmaSseListener extends EventSourceListener {

    private final LimmaApiClient apiClient;
    private final MontoyaApi api;
    private final ObjectMapper mapper;
    private EventSource eventSource;

    public LimmaSseListener(LimmaApiClient apiClient, MontoyaApi api) {
        this.apiClient = apiClient;
        this.api = api;
        this.mapper = new ObjectMapper();
    }

    public void startListening() {
        if (apiClient.getSessionId() == null) return;

        Request request = new Request.Builder()
            .url(apiClient.getBaseUrl() + "/api/burp/stream/" + apiClient.getSessionId())
            .header("Accept", "text/event-stream")
            .build();

        // Use a client with no read timeout for SSE
        okhttp3.OkHttpClient sseClient = apiClient.getClient().newBuilder()
            .readTimeout(0, TimeUnit.MILLISECONDS)
            .build();

        EventSource.Factory factory = EventSources.createFactory(sseClient);
        this.eventSource = factory.newEventSource(request, this);
        api.logging().logToOutput("Started SSE listener for LIMMA findings...");
    }

    public void stopListening() {
        if (eventSource != null) {
            eventSource.cancel();
            eventSource = null;
        }
    }

    @Override
    public void onEvent(EventSource eventSource, String id, String type, String data) {
        try {
            if ("message".equals(type)) {
                LimmaModels.SseEvent event = mapper.readValue(data, LimmaModels.SseEvent.class);
                if ("finding_detected".equals(event.type)) {
                    LimmaModels.NativeFinding finding = mapper.convertValue(event.data, LimmaModels.NativeFinding.class);
                    createBurpIssue(finding);
                }
            }
        } catch (Exception e) {
            api.logging().logToError("Failed to parse SSE event: " + e.getMessage());
        }
    }

    @Override
    public void onClosed(EventSource eventSource) {
        api.logging().logToOutput("LIMMA SSE Connection closed.");
    }

    @Override
    public void onFailure(EventSource eventSource, Throwable t, Response response) {
        api.logging().logToError("LIMMA SSE Connection failed: " + (t != null ? t.getMessage() : "Unknown error"));
        
        // Reconnect logic after 5 seconds
        api.logging().logToOutput("Attempting to reconnect in 5 seconds...");
        new java.util.Timer().schedule(new java.util.TimerTask() {
            @Override
            public void run() {
                startListening();
            }
        }, 5000);
    }

    private void createBurpIssue(LimmaModels.NativeFinding finding) {
        // Convert LIMMA confidence/severity to Burp Native
        AuditIssueSeverity severity = AuditIssueSeverity.INFORMATION;
        if ("High".equalsIgnoreCase(finding.severity) || "Critical".equalsIgnoreCase(finding.severity)) severity = AuditIssueSeverity.HIGH;
        else if ("Medium".equalsIgnoreCase(finding.severity)) severity = AuditIssueSeverity.MEDIUM;
        else if ("Low".equalsIgnoreCase(finding.severity)) severity = AuditIssueSeverity.LOW;

        AuditIssueConfidence confidence = AuditIssueConfidence.TENTATIVE;
        if ("Certain".equalsIgnoreCase(finding.confidence)) confidence = AuditIssueConfidence.CERTAIN;
        else if ("Firm".equalsIgnoreCase(finding.confidence)) confidence = AuditIssueConfidence.FIRM;

        HttpRequest req = HttpRequest.httpRequestFromUrl(finding.url);
        HttpRequestResponse reqRes = HttpRequestResponse.httpRequestResponse(req, null);

        AuditIssue issue = AuditIssue.auditIssue(
            finding.name,
            finding.detail,
            finding.remediation,
            finding.url,
            severity,
            confidence,
            null, // background
            null, // remediation background
            severity, // typical severity
            reqRes
        );

        api.siteMap().add(issue);
        api.logging().logToOutput("[NEW FINDING] " + finding.name + " (" + finding.severity + ") at " + finding.url);
    }
}
