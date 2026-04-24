package com.limma.burp.ui;

import burp.api.montoya.MontoyaApi;
import com.limma.burp.LimmaPlugin;

import javax.swing.*;
import java.awt.*;
import java.awt.event.ActionEvent;

public class LimmaSettingsPanel extends JPanel {

    private final MontoyaApi api;
    private final LimmaPlugin plugin;

    private JTextField txtBackendUrl;
    private JTextField txtTargetUrl;
    private JButton btnConnect;
    private JButton btnDisconnect;
    private JLabel lblStatus;

    public LimmaSettingsPanel(MontoyaApi api, LimmaPlugin plugin) {
        this.api = api;
        this.plugin = plugin;
        
        setLayout(new BorderLayout());
        setBorder(BorderFactory.createEmptyBorder(20, 20, 20, 20));

        // Create main container
        JPanel mainPanel = new JPanel();
        mainPanel.setLayout(new BoxLayout(mainPanel, BoxLayout.Y_AXIS));

        // Title
        JLabel title = new JLabel("LIMMA Security Platform Integration");
        title.setFont(new Font("SansSerif", Font.BOLD, 18));
        title.setAlignmentX(Component.LEFT_ALIGNMENT);
        mainPanel.add(title);
        mainPanel.add(Box.createRigidArea(new Dimension(0, 20)));

        // Form Panel
        JPanel formPanel = new JPanel(new GridBagLayout());
        formPanel.setAlignmentX(Component.LEFT_ALIGNMENT);
        GridBagConstraints gbc = new GridBagConstraints();
        gbc.fill = GridBagConstraints.HORIZONTAL;
        gbc.insets = new Insets(5, 5, 5, 5);

        // Backend URL
        gbc.gridx = 0; gbc.gridy = 0; gbc.weightx = 0;
        formPanel.add(new JLabel("LIMMA Backend URL:"), gbc);

        gbc.gridx = 1; gbc.gridy = 0; gbc.weightx = 1;
        txtBackendUrl = new JTextField("http://127.0.0.1:8900", 30);
        formPanel.add(txtBackendUrl, gbc);

        // Target URL
        gbc.gridx = 0; gbc.gridy = 1; gbc.weightx = 0;
        formPanel.add(new JLabel("Target Scope URL:"), gbc);

        gbc.gridx = 1; gbc.gridy = 1; gbc.weightx = 1;
        txtTargetUrl = new JTextField("https://example.com", 30);
        formPanel.add(txtTargetUrl, gbc);

        mainPanel.add(formPanel);
        mainPanel.add(Box.createRigidArea(new Dimension(0, 20)));

        // Buttons Panel
        JPanel buttonPanel = new JPanel(new FlowLayout(FlowLayout.LEFT));
        buttonPanel.setAlignmentX(Component.LEFT_ALIGNMENT);
        
        btnConnect = new JButton("Connect & Start Streaming");
        btnConnect.setBackground(new Color(0, 150, 136));
        btnConnect.setForeground(Color.WHITE);
        
        btnDisconnect = new JButton("Disconnect");
        btnDisconnect.setEnabled(false);

        lblStatus = new JLabel("Status: Disconnected");
        lblStatus.setForeground(Color.GRAY);

        buttonPanel.add(btnConnect);
        buttonPanel.add(btnDisconnect);
        buttonPanel.add(Box.createRigidArea(new Dimension(15, 0)));
        buttonPanel.add(lblStatus);

        mainPanel.add(buttonPanel);
        mainPanel.add(Box.createVerticalGlue());

        add(mainPanel, BorderLayout.CENTER);

        // Add action listeners
        btnConnect.addActionListener(this::onConnect);
        btnDisconnect.addActionListener(this::onDisconnect);
    }

    private void onConnect(ActionEvent e) {
        String backendUrl = txtBackendUrl.getText().trim();
        String targetUrl = txtTargetUrl.getText().trim();

        if (backendUrl.isEmpty() || targetUrl.isEmpty()) {
            JOptionPane.showMessageDialog(this, "Both URLs are required.", "Error", JOptionPane.ERROR_MESSAGE);
            return;
        }

        lblStatus.setText("Status: Connecting...");
        lblStatus.setForeground(Color.ORANGE);
        btnConnect.setEnabled(false);

        // Run connection in a background thread to avoid freezing UI
        SwingWorker<Boolean, Void> worker = new SwingWorker<Boolean, Void>() {
            @Override
            protected Boolean doInBackground() throws Exception {
                return plugin.connectToLimma(backendUrl, targetUrl);
            }

            @Override
            protected void done() {
                try {
                    boolean success = get();
                    if (success) {
                        lblStatus.setText("Status: Connected & Streaming");
                        lblStatus.setForeground(new Color(0, 128, 0));
                        btnDisconnect.setEnabled(true);
                        txtBackendUrl.setEnabled(false);
                        txtTargetUrl.setEnabled(false);
                        api.logging().logToOutput("UI: Connection established successfully.");
                    } else {
                        lblStatus.setText("Status: Connection Failed");
                        lblStatus.setForeground(Color.RED);
                        btnConnect.setEnabled(true);
                        api.logging().logToError("UI: Connection failed.");
                    }
                } catch (Exception ex) {
                    lblStatus.setText("Status: Error");
                    lblStatus.setForeground(Color.RED);
                    btnConnect.setEnabled(true);
                    api.logging().logToError("UI Error: " + ex.getMessage());
                }
            }
        };
        worker.execute();
    }

    private void onDisconnect(ActionEvent e) {
        plugin.disconnectFromLimma();
        
        lblStatus.setText("Status: Disconnected");
        lblStatus.setForeground(Color.GRAY);
        
        btnConnect.setEnabled(true);
        btnDisconnect.setEnabled(false);
        txtBackendUrl.setEnabled(true);
        txtTargetUrl.setEnabled(true);
        
        api.logging().logToOutput("UI: Disconnected by user.");
    }
}
