const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

(async () => {
  const browser = await puppeteer.launch({
    headless: 'new',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-dev-shm-usage', '--disable-accelerated-2d-canvas', '--no-first-run', '--no-zygote']
  });

  const results = { allowed: null, disallowed: null };

  // Test Allowed Origin (http://localhost:3000)
  console.log('Running test for allowed origin...');
  const pageAllowed = await browser.newPage();
  await pageAllowed.goto('http://localhost:3000/index.html', { waitUntil: 'networkidle0' });
  await new Promise(resolve => setTimeout(resolve, 5000)); // Wait for tests to complete
  const dataAllowed = await pageAllowed.evaluate(() => ({
    results: window.testResults || [],
    summary: window.testSummary || { total: 0, passed: 0 },
    origin: window.location.origin,
    status: document.getElementById('status') ? document.getElementById('status').innerText : 'No status element'
  }));
  results.allowed = dataAllowed;
  await pageAllowed.screenshot({ path: 'allowed-screenshot.png', fullPage: true });
  await pageAllowed.close();
  console.log('Allowed origin results:', JSON.stringify(dataAllowed, null, 2));

  // Test Disallowed Origin (http://localhost:4001)
  console.log('Running test for disallowed origin...');
  const pageDisallowed = await browser.newPage();
  await pageDisallowed.goto('http://localhost:4001/index.html', { waitUntil: 'networkidle0' });
  await new Promise(resolve => setTimeout(resolve, 5000));
  const dataDisallowed = await pageDisallowed.evaluate(() => ({
    results: window.testResults || [],
    summary: window.testSummary || { total: 0, passed: 0 },
    origin: window.location.origin,
    status: document.getElementById('status') ? document.getElementById('status').innerText : 'No status element'
  }));
  results.disallowed = dataDisallowed;
  await pageDisallowed.screenshot({ path: 'disallowed-screenshot.png', fullPage: true });
  await pageDisallowed.close();
  console.log('Disallowed origin results:', JSON.stringify(dataDisallowed, null, 2));

  await browser.close();

  // Save results
  fs.writeFileSync('test-results.json', JSON.stringify(results, null, 2));
  console.log('Full test results saved to test-results.json');
  console.log('Screenshots saved: allowed-screenshot.png, disallowed-screenshot.png');
})();
