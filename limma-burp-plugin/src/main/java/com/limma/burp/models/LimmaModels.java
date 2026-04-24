package com.limma.burp.models;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.List;
import java.util.Map;

public class LimmaModels {

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class HandshakeRequest {
        @JsonProperty("target_url")
        public String targetUrl;
        @JsonProperty("burp_version")
        public String burpVersion;
        @JsonProperty("plugin_version")
        public String pluginVersion;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class HandshakeResponse {
        @JsonProperty("session_id")
        public String sessionId;
        public String status;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class TrafficItem {
        public String url;
        public String method;
        @JsonProperty("request_headers")
        public Map<String, String> requestHeaders;
        @JsonProperty("request_body")
        public String requestBody;
        @JsonProperty("response_status")
        public int responseStatus;
        @JsonProperty("response_headers")
        public Map<String, String> responseHeaders;
        @JsonProperty("response_body")
        public String responseBody;
        public long timestamp;
        @JsonProperty("tool_source")
        public String toolSource;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class ImportTrafficRequest {
        @JsonProperty("session_id")
        public String sessionId;
        public List<TrafficItem> items;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class NativeFinding {
        public String name;
        public String detail;
        public String severity;
        public String confidence;
        public String url;
        public String path;
        public String host;
        public int port;
        public String protocol;
        public String remediation;
        @JsonProperty("issue_type")
        public int issueType;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class SseEvent {
        public String type;
        public Object data; // Could be a map or a casted object
    }
}
