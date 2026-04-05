// export.js - Export functionality for JSON/CSV download
const { invoke } = window.__TAURI__.core;

let exportModal = null;

function showExportModal() {
    if (exportModal) {
        exportModal.remove();
    }

    exportModal = document.createElement('div');
    exportModal.className = 'modal-overlay';
    exportModal.innerHTML = `
        <div class="modal-content export-modal">
            <div class="modal-header">
                <h3>Export Data</h3>
                <button class="modal-close" onclick="closeExportModal()">×</button>
            </div>
            <div class="modal-body">
                <p class="export-description">Choose a format to export your data. All your receipts, categories, projects, income sources, subscriptions, and savings goals will be included.</p>

                <div class="export-formats">
                    <label class="format-option">
                        <input type="radio" name="export-format" value="json" checked>
                        <div class="format-card">
                            <span class="format-icon">{ }</span>
                            <span class="format-name">JSON</span>
                            <span class="format-desc">Full data backup, Excel-compatible</span>
                        </div>
                    </label>
                    <label class="format-option">
                        <input type="radio" name="export-format" value="csv">
                        <div class="format-card">
                            <span class="format-icon">📊</span>
                            <span class="format-name">CSV</span>
                            <span class="format-desc">Spreadsheet format, opens in Excel</span>
                        </div>
                    </label>
                </div>

                <div class="export-note">
                    <span class="note-icon">ℹ️</span>
                    <span>Excel can open CSV files directly. For XLSX support, use JSON format with Excel's import feature.</span>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="closeExportModal()">Cancel</button>
                <button class="btn btn-primary" onclick="executeExport()">
                    <span class="btn-icon">📥</span>
                    Export
                </button>
            </div>
        </div>
    `;

    document.body.appendChild(exportModal);
    exportModal.addEventListener('click', (e) => {
        if (e.target === exportModal) closeExportModal();
    });

    // Add modal styling
    addExportModalStyles();
}

function closeExportModal() {
    if (exportModal) {
        exportModal.remove();
        exportModal = null;
    }
}

async function executeExport() {
    const formatInputs = document.getElementsByName('export-format');
    let selectedFormat = 'json';
    for (const input of formatInputs) {
        if (input.checked) {
            selectedFormat = input.value;
            break;
        }
    }

    const exportBtn = document.querySelector('.export-modal .btn-primary');
    const originalContent = exportBtn.innerHTML;
    exportBtn.innerHTML = '<span class="spinner"></span> Exporting...';
    exportBtn.disabled = true;

    try {
        const data = await invoke('export_data', { format: selectedFormat });

        // Create download
        const timestamp = new Date().toISOString().split('T')[0];
        const filename = `budgy_export_${timestamp}.${selectedFormat}`;
        const mimeType = selectedFormat === 'json' ? 'application/json' : 'text/csv';

        downloadFile(filename, data, mimeType);

        closeExportModal();
        showToast(`Exported successfully as ${selectedFormat.toUpperCase()}!`);
    } catch (e) {
        console.error('Export error:', e);
        showToast('Export failed: ' + e, 'error');
    } finally {
        exportBtn.innerHTML = originalContent;
        exportBtn.disabled = false;
    }
}

function downloadFile(filename, content, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
}

function showExportButton(container) {
    const exportBtn = document.createElement('button');
    exportBtn.className = 'btn btn-export';
    exportBtn.innerHTML = '<span>📤</span> Export Data';
    exportBtn.onclick = showExportModal;
    container.appendChild(exportBtn);
}

function addExportModalStyles() {
    if (document.getElementById('export-modal-styles')) return;

    const style = document.createElement('style');
    style.id = 'export-modal-styles';
    style.textContent = `
        .export-modal {
            max-width: 400px;
        }
        .export-description {
            color: var(--text-secondary, #666);
            font-size: 0.9rem;
            margin-bottom: 1rem;
        }
        .export-formats {
            display: flex;
            flex-direction: column;
            gap: 0.75rem;
            margin-bottom: 1rem;
        }
        .format-option {
            cursor: pointer;
        }
        .format-option input {
            display: none;
        }
        .format-card {
            display: flex;
            align-items: center;
            padding: 1rem;
            border: 2px solid var(--border-color, #e0e0e0);
            border-radius: 12px;
            transition: all 0.2s;
        }
        .format-option input:checked + .format-card {
            border-color: var(--primary-color, #6366f1);
            background: var(--primary-light, rgba(99, 102, 241, 0.1));
        }
        .format-icon {
            font-size: 1.5rem;
            margin-right: 1rem;
            width: 40px;
            text-align: center;
        }
        .format-name {
            font-weight: 600;
            flex: 1;
        }
        .format-desc {
            font-size: 0.8rem;
            color: var(--text-secondary, #666);
        }
        .export-note {
            display: flex;
            align-items: flex-start;
            gap: 0.5rem;
            padding: 0.75rem;
            background: var(--info-bg, rgba(59, 130, 246, 0.1));
            border-radius: 8px;
            font-size: 0.8rem;
            color: var(--text-secondary, #666);
        }
        .note-icon {
            flex-shrink: 0;
        }
        .btn-export {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.75rem 1rem;
            background: var(--primary-color, #6366f1);
            color: white;
            border: none;
            border-radius: 12px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s;
        }
        .btn-export:hover {
            background: var(--primary-dark, #4f46e5);
        }
        .spinner {
            display: inline-block;
            width: 16px;
            height: 16px;
            border: 2px solid rgba(255,255,255,0.3);
            border-top-color: white;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
    `;
    document.head.appendChild(style);
}

// Mobile-first toast notification
function showToast(message, type = 'success') {
    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    toast.innerHTML = `
        <span class="toast-icon">${type === 'success' ? '✓' : '✕'}</span>
        <span class="toast-message">${message}</span>
    `;

    const toastStyles = `
        .toast {
            position: fixed;
            bottom: 80px;
            left: 50%;
            transform: translateX(-50%);
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.75rem 1.25rem;
            background: var(--toast-bg, #333);
            color: white;
            border-radius: 24px;
            font-size: 0.9rem;
            z-index: 10000;
            animation: toastIn 0.3s ease;
            box-shadow: 0 4px 12px rgba(0,0,0,0.2);
        }
        .toast-success {
            background: #22c55e;
        }
        .toast-error {
            background: #ef4444;
        }
        .toast-icon {
            font-weight: bold;
        }
        @keyframes toastIn {
            from {
                opacity: 0;
                transform: translateX(-50%) translateY(20px);
            }
            to {
                opacity: 1;
                transform: translateX(-50%) translateY(0);
            }
        }
    `;

    let styleEl = document.getElementById('toast-styles');
    if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = 'toast-styles';
        styleEl.textContent = toastStyles;
        document.head.appendChild(styleEl);
    }

    document.body.appendChild(toast);

    setTimeout(() => {
        toast.style.animation = 'toastIn 0.3s ease reverse';
        setTimeout(() => toast.remove(), 300);
    }, 3000);
}

// Show export button in settings or dashboard
function showExportInSettings() {
    const container = document.getElementById('screen-container');
    // Look for a settings section to add the export button
    const settingsCard = container.querySelector('.card');
    if (settingsCard) {
        showExportButton(settingsCard);
    }
}

window.showExportModal = showExportModal;
window.closeExportModal = closeExportModal;
window.executeExport = executeExport;
window.showExportButton = showExportButton;
