const { invoke } = window.__TAURI__.core;

let dashboardData = {
    monthlyBudget: 0,
    totalSpent: 0,
    categories: [],
    recentTransactions: [],
    alerts: []
};

async function loadDashboard() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen dashboard">
            <div class="dashboard-header">
                <h1>Dashboard</h1>
                <span class="dashboard-date" id="current-month"></span>
            </div>

            <div class="card budget-overview-card" id="budget-overview">
                <div class="budget-header">
                    <h3>Monthly Budget</h3>
                    <span class="budget-amounts" id="budget-amounts"></span>
                </div>
                <div class="progress-bar-container">
                    <div class="progress-bar" id="budget-progress"></div>
                </div>
                <div class="left-to-spend" id="left-to-spend"></div>
            </div>

            <div class="card alerts-card" id="alerts-section">
                <h3>Budget Alerts</h3>
                <div id="alerts-list"></div>
            </div>

            <div class="card transactions-card">
                <div class="card-header">
                    <h3>Recent Transactions</h3>
                    <button class="btn-text" onclick="showAllTransactions()">See All</button>
                </div>
                <div id="recent-transactions-list"></div>
            </div>
        </div>
        <button class="fab" id="quick-add-fab" onclick="showQuickAdd()">
            <span>+</span>
        </button>
        <nav class="bottom-nav" id="bottom-nav">
            <button class="nav-item active" onclick="showDashboard()">
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
            <button class="nav-item" onclick="showSettings()">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    await fetchDashboardData();
    renderDashboard();
}

async function fetchDashboardData() {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        const currencySymbol = settings.currency === 'EUR' ? '€' :
                              settings.currency === 'GBP' ? '£' :
                              settings.currency === 'JPY' ? '¥' : '$';

        dashboardData.monthlyBudget = 0;
        dashboardData.totalSpent = 0;
        dashboardData.categories = [];
        dashboardData.recentTransactions = [];
        dashboardData.alerts = [];

        try {
            const categories = await invoke('get_categories');
            dashboardData.categories = categories || [];
            dashboardData.monthlyBudget = dashboardData.categories.reduce((sum, cat) => sum + (cat.budget || 0), 0);
        } catch (e) {
            console.log('Could not fetch categories:', e);
        }

        try {
            const transactions = await invoke('get_transactions', { limit: 5 });
            dashboardData.recentTransactions = transactions || [];
        } catch (e) {
            console.log('Could not fetch transactions:', e);
        }

        try {
            const spending = await invoke('get_monthly_spending');
            dashboardData.totalSpent = spending?.total || 0;
        } catch (e) {
            console.log('Could not fetch spending:', e);
        }

        dashboardData.currencySymbol = currencySymbol;
        generateAlerts();

    } catch (e) {
        console.error('Error fetching dashboard data:', e);
    }
}

function generateAlerts() {
    dashboardData.alerts = [];
    const spent = dashboardData.totalSpent;
    const budget = dashboardData.monthlyBudget;

    if (budget <= 0) return;

    const percentage = (spent / budget) * 100;

    if (percentage >= 100) {
        dashboardData.alerts.push({
            level: 'danger',
            message: 'Budget exceeded!',
            icon: '🚨'
        });
    } else if (percentage >= 80) {
        dashboardData.alerts.push({
            level: 'warning',
            message: 'Approaching limit (80%+)',
            icon: '⚠️'
        });
    } else if (percentage >= 50) {
        dashboardData.alerts.push({
            level: 'caution',
            message: 'Halfway there (50%+)',
            icon: '📍'
        });
    }

    dashboardData.categories.forEach(cat => {
        if (cat.spent && cat.budget) {
            const catPercentage = (cat.spent / cat.budget) * 100;
            if (catPercentage >= 100) {
                dashboardData.alerts.push({
                    level: 'danger',
                    message: `${cat.name} budget exceeded`,
                    icon: cat.icon || '📁'
                });
            } else if (catPercentage >= 80) {
                dashboardData.alerts.push({
                    level: 'warning',
                    message: `${cat.name} at ${Math.round(catPercentage)}%`,
                    icon: cat.icon || '📁'
                });
            }
        }
    });
}

function renderDashboard() {
    const monthNames = ['January', 'February', 'March', 'April', 'May', 'June',
                        'July', 'August', 'September', 'October', 'November', 'December'];
    const now = new Date();
    document.getElementById('current-month').textContent = `${monthNames[now.getMonth()]} ${now.getFullYear()}`;

    const symbol = dashboardData.currencySymbol || '$';
    const budget = dashboardData.monthlyBudget;
    const spent = dashboardData.totalSpent;
    const left = Math.max(0, budget - spent);
    const percentage = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;

    document.getElementById('budget-amounts').textContent = `${symbol}${spent.toFixed(2)} / ${symbol}${budget.toFixed(2)}`;

    const progressBar = document.getElementById('budget-progress');
    progressBar.style.width = `${percentage}%`;
    progressBar.className = 'progress-bar';
    if (percentage < 50) {
        progressBar.classList.add('progress-green');
    } else if (percentage < 80) {
        progressBar.classList.add('progress-yellow');
    } else {
        progressBar.classList.add('progress-red');
    }

    document.getElementById('left-to-spend').textContent = `Left to spend: ${symbol}${left.toFixed(2)}`;

    const alertsList = document.getElementById('alerts-list');
    if (dashboardData.alerts.length > 0) {
        alertsList.innerHTML = dashboardData.alerts.map(alert => `
            <div class="alert-item alert-${alert.level}">
                <span class="alert-icon">${alert.icon}</span>
                <span class="alert-message">${alert.message}</span>
            </div>
        `).join('');
    } else {
        alertsList.innerHTML = '<p class="no-alerts">No budget alerts</p>';
    }

    const transactionsList = document.getElementById('recent-transactions-list');
    if (dashboardData.recentTransactions.length > 0) {
        transactionsList.innerHTML = dashboardData.recentTransactions.map(tx => `
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
    } else {
        transactionsList.innerHTML = '<p class="no-transactions">No transactions yet</p>';
    }
}

function showQuickAdd() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Transaction</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Amount</label>
                    <input type="number" id="quick-add-amount" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Category</label>
                    <select id="quick-add-category">
                        ${dashboardData.categories.map(c => `<option value="${c.id}">${c.icon} ${c.name}</option>`).join('')}
                    </select>
                </div>
                <div class="form-group">
                    <label>Note</label>
                    <input type="text" id="quick-add-note" placeholder="Optional note">
                </div>
                <div class="form-group">
                    <label>Type</label>
                    <div class="type-toggle">
                        <button class="type-btn active" data-type="expense" onclick="setTransactionType('expense')">Expense</button>
                        <button class="type-btn" data-type="income" onclick="setTransactionType('income')">Income</button>
                    </div>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitQuickAdd()">Add</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

let quickAddType = 'expense';

function setTransactionType(type) {
    quickAddType = type;
    document.querySelectorAll('.type-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.type === type);
    });
}

async function submitQuickAdd() {
    const amount = parseFloat(document.getElementById('quick-add-amount').value);
    const categoryId = document.getElementById('quick-add-category').value;
    const note = document.getElementById('quick-add-note').value;

    if (!amount || amount <= 0) {
        alert('Please enter a valid amount');
        return;
    }

    const finalAmount = quickAddType === 'expense' ? -Math.abs(amount) : Math.abs(amount);

    try {
        await invoke('add_transaction', {
            amount: finalAmount,
            categoryId: parseInt(categoryId),
            note: note
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadDashboard();
    } catch (e) {
        console.error('Error adding transaction:', e);
        alert('Failed to add transaction');
    }
}

function showAllTransactions() {
    if (typeof window.showTransactions === 'function') {
        window.showTransactions();
    }
}

function showTransactions() {
    if (typeof window.showTransactionsScreen === 'function') {
        window.showTransactionsScreen();
    }
}

function showCategories() {
    if (typeof window.showCategoriesScreen === 'function') {
        window.showCategoriesScreen();
    }
}

function showSettings() {
    if (typeof window.showSettingsScreen === 'function') {
        window.showSettingsScreen();
    }
}

function showDashboard() {
    loadDashboard();
}
