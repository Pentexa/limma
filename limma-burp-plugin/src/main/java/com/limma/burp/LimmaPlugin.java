package com.limma.burp;

import burp.api.montoya.BurpExtension;
import burp.api.montoya.MontoyaApi;
import burp.api.montoya.extension.ExtensionUnloadingHandler;
import com.limma.burp.api.LimmaApiClient;
import com.limma.burp.api.LimmaSseListener;
import com.limma.burp.proxy.LimmaProxyHandler;
import com.limma.burp.ui.LimmaSettingsPanel;

public class LimmaPlugin implements BurpExtension, ExtensionUnloadingHandler {

    private MontoyaApi api;
    private LimmaApiClient apiClient;
    private LimmaSseListener sseListener;

    @Override
    public void initialize(MontoyaApi api) {
        this.api = api;
        
        // Set extension name
        api.extension().setName("LIMMA Security Integration");
        
        // Log startup
        api.logging().logToOutput("LIMMA Plugin is initializing...");
        api.logging().logToOutput("Version: 0.1.0");

        // Register unload handler
        api.extension().registerUnloadingHandler(this);
        
        // Register Settings UI Tab
        LimmaSettingsPanel uiPanel = new LimmaSettingsPanel(api, this);
        api.userInterface().registerSuiteTab("LIMMA", uiPanel);
        
        api.logging().logToOutput("LIMMA Plugin initialization complete. Please configure settings in the 'LIMMA' tab.");
    }

    public boolean connectToLimma(String backendUrl, String targetUrl) {
        api.logging().logToOutput("Attempting to connect to LIMMA at " + backendUrl);
        
        this.apiClient = new LimmaApiClient(backendUrl, api.logging());
        
        // Perform Handshake
        boolean success = apiClient.handshake(targetUrl, "Montoya", "0.1.0");
        
        if (success) {
            // Start SSE Listener
            this.sseListener = new LimmaSseListener(apiClient, api);
            this.sseListener.startListening();
            
            // Register Proxy Handler to intercept traffic
            api.http().registerHttpHandler(new LimmaProxyHandler(apiClient));
            api.logging().logToOutput("Successfully connected to LIMMA backend and registered interceptors.");
            return true;
        } else {
            api.logging().logToError("Failed to connect to LIMMA backend. Ensure it is running at " + backendUrl);
            return false;
        }
    }

    public void disconnectFromLimma() {
        if (sseListener != null) {
            sseListener.stopListening();
            sseListener = null;
        }
        apiClient = null;
        // Note: In Montoya API, you cannot unregister an HttpHandler once registered. 
        // We handle this by checking if apiClient is null inside the proxy handler, or ignoring traffic.
    }

    @Override
    public void extensionUnloaded() {
        if (api != null) {
            api.logging().logToOutput("LIMMA Plugin is unloading...");
            disconnectFromLimma();
        }
    }
}
