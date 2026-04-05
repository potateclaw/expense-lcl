const { invoke } = window.__TAURI__.core;

const CURRENCIES = [
    { code: 'USD', name: 'US Dollar', symbol: '$' },
    { code: 'EUR', name: 'Euro', symbol: '€' },
    { code: 'GBP', name: 'British Pound', symbol: '£' },
    { code: 'JPY', name: 'Japanese Yen', symbol: '¥' },
    { code: 'CAD', name: 'Canadian Dollar', symbol: 'C$' },
    { code: 'AUD', name: 'Australian Dollar', symbol: 'A$' },
    { code: 'CHF', name: 'Swiss Franc', symbol: 'Fr' },
    { code: 'CNY', name: 'Chinese Yuan', symbol: '¥' },
    { code: 'INR', name: 'Indian Rupee', symbol: '₹' },
    { code: 'BRL', name: 'Brazilian Real', symbol: 'R$' },
    { code: 'MXN', name: 'Mexican Peso', symbol: '$' },
    { code: 'SGD', name: 'Singapore Dollar', symbol: 'S$' }
];

const COUNTRIES = [
    { code: 'US', name: 'United States' },
    { code: 'GB', name: 'United Kingdom' },
    { code: 'DE', name: 'Germany' },
    { code: 'FR', name: 'France' },
    { code: 'JP', name: 'Japan' },
    { code: 'CA', name: 'Canada' },
    { code: 'AU', name: 'Australia' },
    { code: 'CH', name: 'Switzerland' },
    { code: 'CN', name: 'China' },
    { code: 'IN', name: 'India' },
    { code: 'BR', name: 'Brazil' },
    { code: 'MX', name: 'Mexico' },
    { code: 'SG', name: 'Singapore' },
    { code: 'NL', name: 'Netherlands' },
    { code: 'SE', name: 'Sweden' },
    { code: 'NO', name: 'Norway' },
    { code: 'DK', name: 'Denmark' },
    { code: 'NZ', name: 'New Zealand' }
];

const DEFAULT_CATEGORIES = [
    { name: 'Housing', icon: '🏠', defaultBudget: 1500 },
    { name: 'Food', icon: '🍔', defaultBudget: 600 },
    { name: 'Transportation', icon: '🚗', defaultBudget: 400 },
    { name: 'Utilities', icon: '💡', defaultBudget: 200 },
    { name: 'Healthcare', icon: '🏥', defaultBudget: 150 },
    { name: 'Entertainment', icon: '🎬', defaultBudget: 200 },
    { name: 'Shopping', icon: '🛍️', defaultBudget: 300 },
    { name: 'Personal', icon: '👤', defaultBudget: 100 }
];

let onboardingState = {
    step: 1,
    currency: 'USD',
    country: 'US',
    categories: [...DEFAULT_CATEGORIES]
};

function renderOnboardingStep() {
    const container = document.getElementById('screen-container');

    switch (onboardingState.step) {
        case 1:
            container.innerHTML = `
                <div class="screen onboarding-step">
                    <div class="onboarding-header">
                        <div class="onboarding-icon">💰</div>
                        <h1>Welcome to Budgy</h1>
                        <p class="onboarding-subtitle">Your privacy-first, offline budget tracker</p>
                    </div>
                    <div class="card feature-card">
                        <div class="feature-item">
                            <span class="feature-icon">🔒</span>
                            <div>
                                <strong>100% Private</strong>
                                <p>Your data stays on your device. Always.</p>
                            </div>
                        </div>
                        <div class="feature-item">
                            <span class="feature-icon">📊</span>
                            <div>
                                <strong>Smart Budgeting</strong>
                                <p>Track spending across categories</p>
                            </div>
                        </div>
                        <div class="feature-item">
                            <span class="feature-icon">🔔</span>
                            <div>
                                <strong>Budget Alerts</strong>
                                <p>Know when you're approaching limits</p>
                            </div>
                        </div>
                        <div class="feature-item">
                            <span class="feature-icon">📱</span>
                            <div>
                                <strong>Works Offline</strong>
                                <p>No internet required</p>
                            </div>
                        </div>
                    </div>
                    <div class="onboarding-actions">
                        <button class="btn btn-primary btn-full" onclick="nextOnboardingStep()">Get Started</button>
                    </div>
                    <div class="onboarding-progress">
                        <span class="progress-dot active"></span>
                        <span class="progress-dot"></span>
                        <span class="progress-dot"></span>
                    </div>
                </div>
            `;
            break;
        case 2:
            container.innerHTML = `
                <div class="screen onboarding-step">
                    <div class="onboarding-header">
                        <h1>Setup Preferences</h1>
                        <p class="onboarding-subtitle">Tell us about your location</p>
                    </div>
                    <div class="card">
                        <label class="form-label">Currency</label>
                        <select id="currency-select" class="form-select" onchange="updateOnboardingCurrency(this.value)">
                            ${CURRENCIES.map(c => `<option value="${c.code}" ${c.code === onboardingState.currency ? 'selected' : ''}>${c.symbol} - ${c.name}</option>`).join('')}
                        </select>
                    </div>
                    <div class="card">
                        <label class="form-label">Country</label>
                        <select id="country-select" class="form-select" onchange="updateOnboardingCountry(this.value)">
                            ${COUNTRIES.map(c => `<option value="${c.code}" ${c.code === onboardingState.country ? 'selected' : ''}>${c.name}</option>`).join('')}
                        </select>
                    </div>
                    <div class="onboarding-actions">
                        <button class="btn btn-secondary" onclick="prevOnboardingStep()">Back</button>
                        <button class="btn btn-primary" onclick="nextOnboardingStep()">Continue</button>
                    </div>
                    <div class="onboarding-progress">
                        <span class="progress-dot"></span>
                        <span class="progress-dot active"></span>
                        <span class="progress-dot"></span>
                    </div>
                </div>
            `;
            break;
        case 3:
            container.innerHTML = `
                <div class="screen onboarding-step">
                    <div class="onboarding-header">
                        <h1>Budget Categories</h1>
                        <p class="onboarding-subtitle">Review your default categories</p>
                    </div>
                    <div class="card">
                        <div id="categories-list">
                            ${onboardingState.categories.map((cat, index) => `
                                <div class="category-item">
                                    <span class="category-icon">${cat.icon}</span>
                                    <span class="category-name">${cat.name}</span>
                                    <span class="category-budget">${getCurrencySymbol()}${cat.defaultBudget}</span>
                                </div>
                            `).join('')}
                        </div>
                        <p class="categories-hint">You can customize these later in settings</p>
                    </div>
                    <div class="onboarding-actions">
                        <button class="btn btn-secondary" onclick="prevOnboardingStep()">Back</button>
                        <button class="btn btn-primary" onclick="finishOnboarding()">Finish Setup</button>
                    </div>
                    <div class="onboarding-progress">
                        <span class="progress-dot"></span>
                        <span class="progress-dot"></span>
                        <span class="progress-dot active"></span>
                    </div>
                </div>
            `;
            break;
    }
}

function getCurrencySymbol() {
    const currency = CURRENCIES.find(c => c.code === onboardingState.currency);
    return currency ? currency.symbol : '$';
}

function updateOnboardingCurrency(value) {
    onboardingState.currency = value;
}

function updateOnboardingCountry(value) {
    onboardingState.country = value;
}

function nextOnboardingStep() {
    if (onboardingState.step < 3) {
        onboardingState.step++;
        renderOnboardingStep();
    }
}

function prevOnboardingStep() {
    if (onboardingState.step > 1) {
        onboardingState.step--;
        renderOnboardingStep();
    }
}

async function finishOnboarding() {
    const settings = {
        currency: onboardingState.currency,
        country: onboardingState.country,
        categories: onboardingState.categories,
        onboardingComplete: true
    };

    localStorage.setItem('settings', JSON.stringify(settings));

    try {
        await invoke('save_categories', { categories: onboardingState.categories });
    } catch (e) {
        console.log('Could not save categories to backend:', e);
    }

    if (typeof loadDashboard === 'function') {
        loadDashboard();
    } else if (typeof window.loadDashboard === 'function') {
        window.loadDashboard();
    }
}

function initOnboarding() {
    onboardingState = {
        step: 1,
        currency: 'USD',
        country: 'US',
        categories: [...DEFAULT_CATEGORIES]
    };
    renderOnboardingStep();
}
