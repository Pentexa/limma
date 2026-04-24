'use client';

import { useState } from 'react';
import { Download, Terminal, Code, Loader2 } from 'lucide-react';
import { exportToBurp, exportToNuclei, MasterReport } from '@/lib/api';

interface ExportButtonsProps {
  report: MasterReport;
}

/**
 * ExportButtons component
 * Provides 1-click export of Limma findings to Burp Suite XML and Nuclei YAML formats.
 */
export default function ExportButtons({ report }: ExportButtonsProps) {
  const [loadingBurp, setLoadingBurp] = useState(false);
  const [loadingNuclei, setLoadingNuclei] = useState(false);

  const handleExportBurp = async () => {
    setLoadingBurp(true);
    try {
      const response = await exportToBurp(report);
      
      // Trigger download
      const blob = new Blob([response.xml], { type: 'application/xml' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = response.filename || 'limma_burp_export.xml';
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export to Burp Suite:', error);
      alert('Failed to export to Burp Suite.');
    } finally {
      setLoadingBurp(false);
    }
  };

  const handleExportNuclei = async () => {
    setLoadingNuclei(true);
    try {
      const response = await exportToNuclei(report);
      
      // Trigger download
      const blob = new Blob([response.yaml], { type: 'text/yaml' });
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      const host = new URL(report.url).hostname || 'target';
      a.download = `${host}_limma_nuclei_templates.yaml`;
      a.click();
      window.URL.revokeObjectURL(url);
    } catch (error) {
      console.error('Failed to export to Nuclei:', error);
      alert('Failed to export to Nuclei.');
    } finally {
      setLoadingNuclei(false);
    }
  };

  return (
    <div style={{ display: 'flex', gap: 16, alignItems: 'center', padding: '4px' }}>
      <button
        onClick={handleExportBurp}
        disabled={loadingBurp}
        className="btn-export btn-export-burp"
      >
        <span className="icon-wrapper">
          {loadingBurp ? <Loader2 size={16} className="animate-spin" /> : <Code size={16} />}
        </span>
        <span style={{ letterSpacing: '0.01em' }}>Export to Burp Suite</span>
      </button>

      <button
        onClick={handleExportNuclei}
        disabled={loadingNuclei}
        className="btn-export btn-export-nuclei"
      >
        <span className="icon-wrapper">
          {loadingNuclei ? <Loader2 size={16} className="animate-spin" /> : <Terminal size={16} />}
        </span>
        <span style={{ letterSpacing: '0.01em' }}>Export as Nuclei</span>
      </button>
    </div>
  );
}
