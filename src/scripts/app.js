const { invoke } = window.__TAURI__.core;

const screens = {
    onboarding: `
        <div class="screen" id="onboarding">
            <h1>Welcome to Budgy</h1>
            <p>Your privacy-first budget tracker</p>
            <div class="card">
                <label>Currency</label>
                <select id="currency"></select>
            </div>
            <div class="card">
                <label>Country</label>
                <select id="country"></select>
            </div>
            <button class="btn btn-primary" onclick="completeOnboarding()">Continue</button>
        </div>
    `,
    dashboard: `
        <div class="screen" id="dashboard">
            <h1>Dashboard</h1>
            <div class="card">
                <h3>Monthly Budget</h3>
                <div id="budget-summary"></div>
            </div>
            <div class="card">
                <h3>Recent Transactions</h3>
                <div id="recent-transactions"></div>
            </div>
        </div>
    `
};

async function init() {
    try {
        await invoke('init_db');
        const categories = await invoke('get_categories');
        renderScreen('onboarding');
    } catch (e) {
        console.error('Init error:', e);
    }
}

function renderScreen(name) {
    document.getElementById('screen-container').innerHTML = screens[name] || '';
}

async function completeOnboarding() {
    const currency = document.getElementById('currency').value;
    const country = document.getElementById('country').value;
    localStorage.setItem('settings', JSON.stringify({ currency, country }));
    renderScreen('dashboard');
}

function toggleChat() {
    const panel = document.getElementById('chat-panel');
    panel.classList.toggle('hidden');
}

init();