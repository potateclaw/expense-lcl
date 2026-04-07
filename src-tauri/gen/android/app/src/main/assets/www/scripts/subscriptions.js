
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

let subscriptionsData = {
    subscriptions: [],
    upcomingWarnings: []
};

async function loadSubscriptionsScreen() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen subscriptions-screen">
            <div class="screen-header">
                <h1>Subscriptions</h1>
                <button class="btn btn-small btn-primary" onclick="showAddSubscriptionModal()">+ Add</button>
            </div>

            <div class="card summary-card">
                <div class="summary-row">
                    <div class="summary-item">
                        <span class="summary-label">Active Subscriptions</span>
                        <span class="summary-value" id="sub-count">0</span>
                    </div>
                    <div class="summary-item">
                        <span class="summary-label">Monthly Total</span>
                        <span class="summary-value expense-value" id="sub-monthly">$0.00</span>
                    </div>
                </div>
            </div>

            <div class="card warnings-card" id="warnings-section" style="display: none;">
                <h3 class="card-title warning-title">Upcoming Payments</h3>
                <div id="warnings-list"></div>
            </div>

            <div class="card">
                <h3 class="card-title">Your Subscriptions</h3>
                <div id="subscriptions-list"></div>
            </div>
        </div>
    `;

    await fetchSubscriptionsData();
    renderSubscriptionsScreen();
}

async function fetchSubscriptionsData() {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        const currencySymbol = settings.currency === 'EUR' ? '€' :
                              settings.currency === 'GBP' ? '£' :
                              settings.currency === 'JPY' ? '¥' : '$';

        subscriptionsData.currencySymbol = currencySymbol;

        try {
            subscriptionsData.subscriptions = await window.__invoke__('get_subscriptions') || [];
        } catch (e) {
            console.log('Could not fetch subscriptions:', e);
            subscriptionsData.subscriptions = [];
        }

        // Check for upcoming payments (within 3 days)
        subscriptionsData.upcomingWarnings = subscriptionsData.subscriptions.filter(sub => {
            if (!sub.next_expected_date) return false;
            const nextDate = new Date(sub.next_expected_date);
            const today = new Date();
            const diffDays = Math.ceil((nextDate - today) / (1000 * 60 * 60 * 24));
            return diffDays >= 0 && diffDays <= 3;
        });

    } catch (e) {
        console.error('Error fetching subscriptions data:', e);
    }
}

function renderSubscriptionsScreen() {
    const symbol = subscriptionsData.currencySymbol || '$';
    const subscriptions = subscriptionsData.subscriptions;
    const warnings = subscriptionsData.upcomingWarnings;

    // Render count and monthly total
    const monthlyTotal = subscriptions.reduce((sum, sub) => {
        let monthlyAmount = sub.amount;
        if (sub.frequency === 'weekly') {
            monthlyAmount = sub.amount * 4.33;
        } else if (sub.frequency === 'yearly') {
            monthlyAmount = sub.amount / 12;
        }
        return sum + monthlyAmount;
    }, 0);

    document.getElementById('sub-count').textContent = subscriptions.length;
    document.getElementById('sub-monthly').textContent = `${symbol}${monthlyTotal.toFixed(2)}`;

    // Render warnings
    const warningsSection = document.getElementById('warnings-section');
    const warningsList = document.getElementById('warnings-list');

    if (warnings.length > 0) {
        warningsSection.style.display = 'block';
        warningsList.innerHTML = warnings.map(sub => {
            const nextDate = new Date(sub.next_expected_date);
            const today = new Date();
            const diffDays = Math.ceil((nextDate - today) / (1000 * 60 * 60 * 24));

            let warningText = 'Due today';
            if (diffDays === 1) warningText = 'Due tomorrow';
            else if (diffDays > 1) warningText = `Due in ${diffDays} days`;

            return `
                <div class="warning-item">
                    <span class="warning-icon">⚠️</span>
                    <div class="warning-info">
                        <span class="warning-name">${escapeHtml(sub.name)}</span>
                        <span class="warning-detail">${warningText} - ${symbol}${sub.amount.toFixed(2)}</span>
                    </div>
                </div>
            `;
        }).join('');
    } else {
        warningsSection.style.display = 'none';
    }

    // Render subscriptions list
    const listEl = document.getElementById('subscriptions-list');

    if (subscriptions.length === 0) {
        listEl.innerHTML = '<p class="empty-message">No subscriptions detected. Add one manually or scan receipts.</p>';
        return;
    }

    listEl.innerHTML = subscriptions.map(sub => {
        const monthlyAmount = sub.frequency === 'weekly' ? sub.amount * 4.33 :
                             sub.frequency === 'yearly' ? sub.amount / 12 : sub.amount;

        return `
            <div class="subscription-item" data-id="${sub.id}">
                <div class="subscription-icon">🔄</div>
                <div class="subscription-info">
                    <div class="subscription-header">
                        <span class="subscription-name">${escapeHtml(sub.name)}</span>
                        <span class="subscription-amount">${symbol}${sub.amount.toFixed(2)}</span>
                    </div>
                    <div class="subscription-details">
                        <span class="subscription-frequency">${sub.frequency}</span>
                        <span class="subscription-monthly">~${symbol}${monthlyAmount.toFixed(2)}/mo</span>
                    </div>
                    <div class="subscription-next">
                        <span class="next-label">Next:</span>
                        <span class="next-date">${formatDate(sub.next_expected_date)}</span>
                    </div>
                </div>
                <button class="btn-delete" onclick="deleteSubscription(${sub.id})" title="Delete">×</button>
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

function showAddSubscriptionModal() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Subscription</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Name</label>
                    <input type="text" id="sub-name" placeholder="e.g., Netflix, Spotify">
                </div>
                <div class="form-group">
                    <label>Amount</label>
                    <input type="number" id="sub-amount" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Frequency</label>
                    <select id="sub-frequency" class="form-select">
                        <option value="monthly">Monthly</option>
                        <option value="weekly">Weekly</option>
                        <option value="yearly">Yearly</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Next Expected Date</label>
                    <input type="date" id="sub-next-date">
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitSubscription()">Add Subscription</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitSubscription() {
    const name = document.getElementById('sub-name').value.trim();
    const amount = parseFloat(document.getElementById('sub-amount').value);
    const frequency = document.getElementById('sub-frequency').value;
    const nextDate = document.getElementById('sub-next-date').value;

    if (!name) {
        alert('Please enter a name for this subscription');
        return;
    }

    if (!amount || amount <= 0) {
        alert('Please enter a valid amount');
        return;
    }

    try {
        await window.__invoke__('add_subscription', {
            name,
            amount,
            frequency,
            nextExpectedDate: nextDate || new Date().toISOString()
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadSubscriptionsScreen();
    } catch (e) {
        console.error('Error adding subscription:', e);
        alert('Failed to add subscription');
    }
}

async function deleteSubscription(id) {
    if (!confirm('Are you sure you want to delete this subscription?')) {
        return;
    }

    try {
        // For now, just reload the screen (backend doesn't have delete yet)
        await loadSubscriptionsScreen();
    } catch (e) {
        console.error('Error deleting subscription:', e);
    }
}

window.loadSubscriptionsScreen = loadSubscriptionsScreen;
window.showAddSubscriptionModal = showAddSubscriptionModal;
window.submitSubscription = submitSubscription;
window.deleteSubscription = deleteSubscription;
