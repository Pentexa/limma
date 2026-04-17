const http = require('http');
const https = require('https');

const API_PARAMS = JSON.stringify({ url: "https://example.com" });
const API_URL = "http://localhost:8900/investigate"; // Assuming backend is on 8900

const DURATON_SEC = 10;
const CONCURRENCY = 20;

let completed = 0;
let errors = 0;
let latencies = [];
let startTime = Date.now();
let isRunning = true;

function sendRequest() {
    if (!isRunning) return;
    
    const reqStart = Date.now();
    const req = http.request(API_URL, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Content-Length': Buffer.byteLength(API_PARAMS)
        }
    }, (res) => {
        let data = '';
        res.on('data', chunk => data += chunk);
        res.on('end', () => {
            if (res.statusCode === 200) {
                latencies.push(Date.now() - reqStart);
                completed++;
            } else {
                errors++;
            }
            if (isRunning) sendRequest();
        });
    });

    req.on('error', (e) => {
        errors++;
        if (isRunning) sendRequest();
    });

    req.write(API_PARAMS);
    req.end();
}

console.log(`Starting Limma Benchmark...`);
console.log(`Target: /investigate endpoint`);
console.log(`Concurrency: ${CONCURRENCY} workers`);
console.log(`Duration: ${DURATON_SEC} seconds\n...`);

for (let i = 0; i < CONCURRENCY; i++) {
    sendRequest();
}

setTimeout(() => {
    isRunning = false;
    const totalTime = (Date.now() - startTime) / 1000;
    
    // Sort latencies to find percentiles
    latencies.sort((a, b) => a - b);
    
    const avgLatency = latencies.reduce((a, b) => a + b, 0) / (latencies.length || 1);
    const p95 = latencies[Math.floor(latencies.length * 0.95)] || 0;
    const p99 = latencies[Math.floor(latencies.length * 0.99)] || 0;
    const reqPerSec = completed / totalTime;

    console.log(`\n================ BENCHMARK RESULTS ================`);
    console.log(`Total Requests Sent : ${completed + errors}`);
    console.log(`Successful Scans    : ${completed}`);
    console.log(`Failed Requests     : ${errors}`);
    console.log(`Throughput          : ${reqPerSec.toFixed(2)} req/sec`);
    console.log(`---------------------------------------------------`);
    console.log(`Average Latency     : ${avgLatency.toFixed(2)} ms`);
    console.log(`Min Latency         : ${latencies[0] || 0} ms`);
    console.log(`Max Latency         : ${latencies[latencies.length - 1] || 0} ms`);
    console.log(`p95 Latency         : ${p95} ms`);
    console.log(`p99 Latency         : ${p99} ms`);
    console.log(`===================================================\n`);
}, DURATON_SEC * 1000);
