const LIMMA_API = "http://localhost:8900/master-report";
const targetUrl = "http://localhost:9001/safe/perfect";

async function run() {
    const response = await fetch(LIMMA_API, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: targetUrl })
    }).then(res => res.json());

    if (response.normalized_audit) {
        console.log(JSON.stringify(response.normalized_audit.findings.map(f => ({
            summary: f.summary,
            severity: f.severity,
            category: f.category
        })), null, 2));
    } else {
        console.log("No normalized_audit", response);
    }
}
run();
