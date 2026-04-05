const { invoke } = window.__TAURI__.core;

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

let incomeData = {
    sources: [],
    totalMonthly: 0,
    fixedBills: 0,
    savingsGoal: 0,
    disposable: 0
};

async function loadIncomeScreen() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen income-screen">
            <div class="screen-header">
                <h1>Income Sources</h1>
                <button class="btn btn-small btn-primary" onclick="showAddIncomeModal()">+ Add</button>
            </div>

            <div class="card summary-card">
                <div class="summary-grid">
                    <div class="summary-item">
                        <span class="summary-label">Monthly Income</span>
                        <span class="summary-value income-value" id="total-income">$0.00</span>
                    </div>
                    <div class="summary-item">
                        <span class="summary-label">Fixed Bills</span>
                        <span class="summary-value expense-value" id="fixed-bills">$0.00</span>
                    </div>
                    <div class="summary-item">
                        <span class="summary-label">Savings Goal</span>
                        <span class="summary-value savings-value" id="savings-goal">$0.00</span>
                    </div>
                    <div class="summary-item">
                        <span class="summary-label">Disposable</span>
                        <span class="summary-value disposable-value" id="disposable-income">$0.00</span>
                    </div>
                </div>
            </div>

            <div class="card">
                <h3 class="card-title">Your Income Sources</h3>
                <div id="income-sources-list"></div>
            </div>
        </div>
    `;

    await fetchIncomeData();
    renderIncomeScreen();
}

async function fetchIncomeData() {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        const currencySymbol = settings.currency === 'EUR' ? '€' :
                              settings.currency === 'GBP' ? '£' :
                              settings.currency === 'JPY' ? '¥' : '$';

        incomeData.currencySymbol = currencySymbol;

        // Fetch income sources
        try {
            incomeData.sources = await invoke('get_income_sources') || [];
        } catch (e) {
            console.log('Could not fetch income sources:', e);
            incomeData.sources = [];
        }

        // Calculate monthly total (normalize all frequencies to monthly)
        incomeData.totalMonthly = incomeData.sources.reduce((sum, source) => {
            let monthlyAmount = source.amount;
            if (source.frequency === 'weekly') {
                monthlyAmount = source.amount * 4.33;
            } else if (source.frequency === 'yearly') {
                monthlyAmount = source.amount / 12;
            }
            return sum + monthlyAmount;
        }, 0);

        // Fetch subscriptions as fixed bills
        try {
            const subscriptions = await invoke('get_subscriptions') || [];
            incomeData.fixedBills = subscriptions.reduce((sum, sub) => {
                let monthlyAmount = sub.amount;
                if (sub.frequency === 'weekly') {
                    monthlyAmount = sub.amount * 4.33;
                } else if (sub.frequency === 'yearly') {
                    monthlyAmount = sub.amount / 12;
                }
                return sum + monthlyAmount;
            }, 0);
        } catch (e) {
            console.log('Could not fetch subscriptions:', e);
            incomeData.fixedBills = 0;
        }

        // Fetch savings goals monthly allocation
        try {
            const goals = await invoke('get_savings_goals') || [];
            incomeData.savingsGoal = goals.reduce((sum, goal) => sum + (goal.monthly_allocation || 0), 0);
        } catch (e) {
            console.log('Could not fetch savings goals:', e);
            incomeData.savingsGoal = 0;
        }

        // Calculate disposable income
        incomeData.disposable = Math.max(0, incomeData.totalMonthly - incomeData.fixedBills - incomeData.savingsGoal);

    } catch (e) {
        console.error('Error fetching income data:', e);
    }
}

function renderIncomeScreen() {
    const symbol = incomeData.currencySymbol || '$';
    const sources = incomeData.sources;

    document.getElementById('total-income').textContent = `${symbol}${incomeData.totalMonthly.toFixed(2)}`;
    document.getElementById('fixed-bills').textContent = `${symbol}${incomeData.fixedBills.toFixed(2)}`;
    document.getElementById('savings-goal').textContent = `${symbol}${incomeData.savingsGoal.toFixed(2)}`;
    document.getElementById('disposable-income').textContent = `${symbol}${incomeData.disposable.toFixed(2)}`;

    const listEl = document.getElementById('income-sources-list');

    if (sources.length === 0) {
        listEl.innerHTML = '<p class="empty-message">No income sources added yet</p>';
        return;
    }

    listEl.innerHTML = sources.map(source => {
        const monthlyAmount = source.frequency === 'weekly' ? source.amount * 4.33 :
                             source.frequency === 'yearly' ? source.amount / 12 : source.amount;

        return `
            <div class="income-source-item" data-id="${source.id}">
                <div class="source-info">
                    <div class="source-header">
                        <span class="source-name">${escapeHtml(source.name)}</span>
                        <span class="source-amount">${symbol}${source.amount.toFixed(2)}</span>
                    </div>
                    <div class="source-details">
                        <span class="source-frequency">${source.frequency}</span>
                        <span class="source-next">Next: ${formatDate(source.next_date)}</span>
                    </div>
                    <div class="source-monthly">
                        <span class="monthly-label">Monthly equivalent:</span>
                        <span class="monthly-value">${symbol}${monthlyAmount.toFixed(2)}</span>
                    </div>
                </div>
                <button class="btn-delete" onclick="deleteIncomeSource(${source.id})" title="Delete">×</button>
            </div>
        `;
    }).join('');
}

function formatDate(dateStr) {
    if (!dateStr) return 'N/A';
    try {
        const date = new Date(dateStr);
        return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
    } catch {
        return dateStr;
    }
}

function showAddIncomeModal() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Income Source</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Name</label>
                    <input type="text" id="income-name" placeholder="e.g., Primary Salary">
                </div>
                <div class="form-group">
                    <label>Amount</label>
                    <input type="number" id="income-amount" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Frequency</label>
                    <select id="income-frequency" class="form-select">
                        <option value="monthly">Monthly</option>
                        <option value="weekly">Weekly</option>
                        <option value="yearly">Yearly</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Next Payment Date</label>
                    <input type="date" id="income-next-date">
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitIncomeSource()">Add Income</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitIncomeSource() {
    const name = document.getElementById('income-name').value.trim();
    const amount = parseFloat(document.getElementById('income-amount').value);
    const frequency = document.getElementById('income-frequency').value;
    const nextDate = document.getElementById('income-next-date').value;

    if (!name) {
        alert('Please enter a name for this income source');
        return;
    }

    if (!amount || amount <= 0) {
        alert('Please enter a valid amount');
        return;
    }

    try {
        await invoke('add_income_source', {
            name,
            amount,
            frequency,
            nextDate: nextDate || new Date().toISOString()
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadIncomeScreen();
    } catch (e) {
        console.error('Error adding income source:', e);
        alert('Failed to add income source');
    }
}

async function deleteIncomeSource(id) {
    if (!confirm('Are you sure you want to delete this income source?')) {
        return;
    }

    try {
        // For now, just reload the screen (backend doesn't have delete yet)
        await loadIncomeScreen();
    } catch (e) {
        console.error('Error deleting income source:', e);
    }
}

window.loadIncomeScreen = loadIncomeScreen;
window.showAddIncomeModal = showAddIncomeModal;
window.submitIncomeSource = submitIncomeSource;
window.deleteIncomeSource = deleteIncomeSource;
