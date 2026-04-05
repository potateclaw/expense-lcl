const { invoke } = window.__TAURI__.core;

async function init() {
    try {
        await invoke('init_db');

        const settings = JSON.parse(localStorage.getItem('settings') || '{}');

        if (settings.onboardingComplete) {
            if (typeof loadDashboard === 'function') {
                loadDashboard();
            } else {
                window.loadDashboard();
            }
        } else {
            if (typeof initOnboarding === 'function') {
                initOnboarding();
            } else {
                window.initOnboarding();
            }
        }
    } catch (e) {
        console.error('Init error:', e);
        if (typeof initOnboarding === 'function') {
            initOnboarding();
        }
    }
}

// Navigation placeholder functions for screens not yet implemented
function showSettingsScreen() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen settings-screen">
            <div class="screen-header">
                <h1>Settings</h1>
            </div>
            <div class="card">
                <h3>Currency</h3>
                <p class="settings-value" id="settings-currency">-</p>
            </div>
            <div class="card">
                <h3>Country</h3>
                <p class="settings-value" id="settings-country">-</p>
            </div>
            <div class="card">
                <h3>Data Management</h3>
                <button class="btn btn-secondary" onclick="showExportModal()">Export All Data</button>
            </div>
            <div class="card">
                <h3>About</h3>
                <p>Budgy v0.1.0</p>
                <p class="text-secondary">Your privacy-first, offline budget tracker</p>
            </div>
        </div>
        <nav class="bottom-nav" id="bottom-nav">
            <button class="nav-item" onclick="showDashboard()">
                <span class="nav-icon">🏠</span>
                <span class="nav-label">Home</span>
            </button>
            <button class="nav-item" onclick="showTransactions()">
                <span class="nav-icon">📊</span>
                <span class="nav-label">Transactions</span>
            </button>
            <button class="nav-item" onclick="showCategories()">
                <span class="nav-icon">📁</span>
                <span class="nav-label">Categories</span>
            </button>
            <button class="nav-item active">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    // Load settings values
    const settings = JSON.parse(localStorage.getItem('settings') || '{}');
    const currencyEl = document.getElementById('settings-currency');
    const countryEl = document.getElementById('settings-country');

    const currencies = { USD: '$', EUR: '€', GBP: '£', JPY: '¥' };
    currencyEl.textContent = (currencies[settings.currency] || '$') + ' (' + (settings.currency || 'USD') + ')';
    countryEl.textContent = settings.country || 'US';
}

function showTransactionsScreen() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen transactions-screen">
            <div class="screen-header">
                <h1>Transactions</h1>
            </div>
            <div id="transactions-list" class="transactions-list">
                <p class="empty-message">Loading transactions...</p>
            </div>
        </div>
        <nav class="bottom-nav" id="bottom-nav">
            <button class="nav-item" onclick="showDashboard()">
                <span class="nav-icon">🏠</span>
                <span class="nav-label">Home</span>
            </button>
            <button class="nav-item active">
                <span class="nav-icon">📊</span>
                <span class="nav-label">Transactions</span>
            </button>
            <button class="nav-item" onclick="showCategories()">
                <span class="nav-icon">📁</span>
                <span class="nav-label">Categories</span>
            </button>
            <button class="nav-item" onclick="showSettings()">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    // Load transactions
    loadTransactionsList();
}

async function loadTransactionsList() {
    const listEl = document.getElementById('transactions-list');
    try {
        const transactions = await invoke('get_transactions', { limit: 100 }) || [];
        if (transactions.length === 0) {
            listEl.innerHTML = '<p class="empty-message">No transactions yet</p>';
            return;
        }

        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        const symbol = settings.currency === 'EUR' ? '€' :
                      settings.currency === 'GBP' ? '£' :
                      settings.currency === 'JPY' ? '¥' : '$';

        listEl.innerHTML = transactions.map(tx => `
            <div class="transaction-item">
                <div class="transaction-info">
                    <span class="transaction-category">${tx.category_name || 'Uncategorized'}</span>
                    <span class="transaction-note">${tx.note || ''}</span>
                </div>
                <div class="transaction-amount ${tx.amount < 0 ? 'amount-negative' : 'amount-positive'}">
                    ${tx.amount < 0 ? '-' : '+'}${symbol}${Math.abs(tx.amount).toFixed(2)}
                </div>
            </div>
        `).join('');
    } catch (e) {
        console.error('Error loading transactions:', e);
        listEl.innerHTML = '<p class="empty-message">Failed to load transactions</p>';
    }
}

function showCategories() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen categories-screen">
            <div class="screen-header">
                <h1>Categories</h1>
            </div>
            <div id="categories-list" class="categories-list">
                <p class="empty-message">Loading categories...</p>
            </div>
        </div>
        <nav class="bottom-nav" id="bottom-nav">
            <button class="nav-item" onclick="showDashboard()">
                <span class="nav-icon">🏠</span>
                <span class="nav-label">Home</span>
            </button>
            <button class="nav-item" onclick="showTransactions()">
                <span class="nav-icon">📊</span>
                <span class="nav-label">Transactions</span>
            </button>
            <button class="nav-item active">
                <span class="nav-icon">📁</span>
                <span class="nav-label">Categories</span>
            </button>
            <button class="nav-item" onclick="showSettings()">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    loadCategoriesList();
}

async function loadCategoriesList() {
    const listEl = document.getElementById('categories-list');
    try {
        const categories = await invoke('get_categories') || [];
        if (categories.length === 0) {
            listEl.innerHTML = '<p class="empty-message">No categories yet</p>';
            return;
        }

        listEl.innerHTML = categories.map(cat => `
            <div class="category-item">
                <span class="category-icon">📁</span>
                <span class="category-name">${escapeHtml(cat.name)}</span>
            </div>
        `).join('');
    } catch (e) {
        console.error('Error loading categories:', e);
        listEl.innerHTML = '<p class="empty-message">Failed to load categories</p>';
    }
}

function escapeHtml(text) {
    if (text === null || text === undefined) return '';
    const div = document.createElement('div');
    div.textContent = String(text);
    return div.innerHTML;
}

function showTransactions() {
    if (typeof window.showTransactionsScreen === 'function') {
        window.showTransactionsScreen();
    }
}

function showSettings() {
    showSettingsScreen();
}

window.loadDashboard = loadDashboard;
window.initOnboarding = initOnboarding;
window.showSettingsScreen = showSettingsScreen;
window.showTransactionsScreen = showTransactionsScreen;
window.showCategories = showCategories;
window.showTransactions = showTransactions;
window.showSettings = showSettings;

init();