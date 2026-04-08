import axios from 'axios';

const api = axios.create({
  baseURL: 'http://localhost:8900',
});

export function extractApiError(error: any): string {
  const responseData = error?.response?.data;

  if (typeof responseData?.error === "string") {
    return responseData.error;
  }

  if (typeof responseData?.error?.message === "string") {
    return responseData.error.message;
  }

  if (typeof responseData?.message === "string") {
    return responseData.message;
  }

  if (typeof error?.message === "string" && error.message.trim().length > 0) {
    return error.message;
  }

  return "An error occurred.";
}

export const reportApi = {
  analyzeWebsite: (url: string) => api.post('/analyze', { url }).then(res => res.data),
  analyzeWebsiteStreamUrl: (url: string) => `http://localhost:8900/analyze/stream?url=${encodeURIComponent(url)}`,
  investigateServer: (url: string) => api.post('/investigate', { url }).then(res => res.data),
  discoverApis: (url: string) => api.post('/discover-apis', { url }).then(res => res.data),
  collectServices: (url: string) => api.post('/collect-services', { url }).then(res => res.data),
  auditSecurity: (url: string) => api.post('/audit-security', { url }).then(res => res.data),
  mapForms: (url: string) => api.post('/map-forms', { url }).then(res => res.data),
  masterReport: (url: string) => api.post('/master-report', { url }).then(res => res.data),
  proxyRequest: (url: string, method: string, body?: string) => api.post('/proxy-request', { url, method, body }).then(res => res.data),
  verifyPort: (host: string, port: number) => api.post('/verify-port', { host, port }).then(res => res.data),
};

export default api;
