// Tauri 2.x global API
const { invoke } = window.__TAURI__.core;

let currentReport = null;

async function init() {
    try {
        const version = await invoke('engine_version');
        document.getElementById('version').textContent = `engine v${version}`;

        const scanners = await invoke('list_scanners');
        const select = document.getElementById('scanner-select');
        for (const s of scanners) {
            const opt = document.createElement('option');
            opt.value = s.id;
            opt.textContent = `${s.name} (${s.id})`;
            select.appendChild(opt);
        }
    } catch (e) {
        console.error('Init failed:', e);
        showStatus(`Init error: ${e}`, 'error');
    }
}

function showStatus(msg, type) {
    const el = document.getElementById('status');
    el.innerHTML = msg;
    el.className = 'status' + (type ? ` ${type}` : '');
    el.hidden = false;
}

function fmtBytes(b) {
    if (b < 1024) return b + ' B';
    if (b < 1048576) return (b / 1024).toFixed(1) + ' KB';
    if (b < 1073741824) return (b / 1048576).toFixed(1) + ' MB';
    return (b / 1073741824).toFixed(2) + ' GB';
}

function shortenPath(p, max = 60) {
    if (p.length <= max) return p;
    return '...' + p.slice(p.length - (max - 3));
}

async function runScan() {
    const btn = document.getElementById('scan-btn');
    const scannerId = document.getElementById('scanner-select').value || null;
    const devCaches = document.getElementById('dev-caches-toggle').checked;

    btn.disabled = true;
    btn.textContent = '⏳ Scanning...';
    showStatus('<span class="spinner"></span>Scanning your system... (read-only)', 'scanning');

    try {
        const report = await invoke('run_scan', { scannerId, devCaches });
        currentReport = report;
        renderReport(report);
        showStatus(`✅ Scan complete — ${report.findings.length} findings in ${report.duration_ms}ms`);
    } catch (e) {
        showStatus(`❌ Scan failed: ${e}`, 'error');
    } finally {
        btn.disabled = false;
        btn.textContent = '🔍 Run Scan';
    }
}

function renderReport(report) {
    document.getElementById('summary').hidden = false;
    document.getElementById('filter-bar').hidden = false;

    const totalBytes = report.findings.reduce((s, f) => s + f.size_bytes, 0);
    document.getElementById('stat-count').textContent = report.findings.length;
    document.getElementById('stat-size').textContent = fmtBytes(totalBytes);
    document.getElementById('stat-high').textContent = report.findings.filter(f => f.risk === 'high').length;
    document.getElementById('stat-medium').textContent = report.findings.filter(f => f.risk === 'medium').length;
    document.getElementById('stat-low').textContent = report.findings.filter(f => f.risk === 'low').length;
    document.getElementById('stat-duration').textContent = report.duration_ms + ' ms';
    document.getElementById('scan-id').textContent = `scan: ${report.scan_id.slice(0, 8)}`;

    if (report.errors && report.errors.length > 0) {
        const errEl = document.getElementById('errors');
        errEl.hidden = false;
        errEl.innerHTML = `<h3>⚠️ ${report.errors.length} scanner error(s)</h3><ul>${
            report.errors.map(e => `<li>${escapeHtml(e)}</li>`).join('')
        }</ul>`;
    } else {
        document.getElementById('errors').hidden = true;
    }

    renderTable(report.findings);
}

function renderTable(findings) {
    const search = document.getElementById('search-input').value.toLowerCase();
    const riskFilter = document.getElementById('risk-filter').value;

    let filtered = findings;
    if (search) {
        filtered = filtered.filter(f =>
            f.target_ref.toLowerCase().includes(search) ||
            f.scanner_id.toLowerCase().includes(search) ||
            String(f.category).toLowerCase().includes(search)
        );
    }
    if (riskFilter) {
        filtered = filtered.filter(f => f.risk === riskFilter);
    }

    filtered = [...filtered].sort((a, b) => b.size_bytes - a.size_bytes);

    const container = document.getElementById('results');
    if (filtered.length === 0) {
        container.innerHTML = '<div class="empty">No findings match the current filter.</div>';
        return;
    }

    container.innerHTML = `
        <table>
            <thead>
                <tr>
                    <th>Risk</th>
                    <th>Scanner</th>
                    <th>Category</th>
                    <th>Size</th>
                    <th>Age</th>
                    <th>Path</th>
                    <th>Confidence</th>
                </tr>
            </thead>
            <tbody>
                ${filtered.slice(0, 500).map(f => `
                    <tr>
                        <td><span class="risk-badge ${f.risk}">${f.risk}</span></td>
                        <td>${escapeHtml(f.scanner_id)}</td>
                        <td>${escapeHtml(String(f.category))}</td>
                        <td>${fmtBytes(f.size_bytes)}</td>
                        <td>${f.age_days}d</td>
                        <td class="path-cell" title="${escapeHtml(f.target_ref)}">${escapeHtml(shortenPath(f.target_ref))}</td>
                        <td>${(f.confidence * 100).toFixed(0)}%</td>
                    </tr>
                `).join('')}
            </tbody>
        </table>
        ${filtered.length > 500 ? `<div class="empty">Showing top 500 of ${filtered.length} findings.</div>` : ''}
    `;
}

function escapeHtml(s) {
    return String(s)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;');
}

document.getElementById('scan-btn').addEventListener('click', runScan);
document.getElementById('search-input').addEventListener('input', () => {
    if (currentReport) renderTable(currentReport.findings);
});
document.getElementById('risk-filter').addEventListener('change', () => {
    if (currentReport) renderTable(currentReport.findings);
});

init();
