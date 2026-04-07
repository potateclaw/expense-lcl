
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

let savingsData = {
    goals: [],
    totalSaved: 0,
    totalTarget: 0
};

async function loadSavingsScreen() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen savings-screen">
            <div class="screen-header">
                <h1>Savings Goals</h1>
                <button class="btn btn-small btn-primary" onclick="showAddGoalModal()">+ Add Goal</button>
            </div>

            <div class="card summary-card">
                <div class="summary-row">
                    <div class="summary-item">
                        <span class="summary-label">Total Saved</span>
                        <span class="summary-value savings-value" id="total-saved">$0.00</span>
                    </div>
                    <div class="summary-item">
                        <span class="summary-label">Total Target</span>
                        <span class="summary-value" id="total-target">$0.00</span>
                    </div>
                </div>
            </div>

            <div class="card">
                <h3 class="card-title">Your Goals</h3>
                <div id="savings-goals-list"></div>
            </div>
        </div>
    `;

    await fetchSavingsData();
    renderSavingsScreen();
}

async function fetchSavingsData() {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        const currencySymbol = settings.currency === 'EUR' ? '€' :
                              settings.currency === 'GBP' ? '£' :
                              settings.currency === 'JPY' ? '¥' : '$';

        savingsData.currencySymbol = currencySymbol;

        try {
            savingsData.goals = await window.__invoke__('get_savings_goals') || [];
        } catch (e) {
            console.log('Could not fetch savings goals:', e);
            savingsData.goals = [];
        }

        savingsData.totalSaved = savingsData.goals.reduce((sum, goal) => sum + (goal.current_progress || 0), 0);
        savingsData.totalTarget = savingsData.goals.reduce((sum, goal) => sum + (goal.target_amount || 0), 0);

    } catch (e) {
        console.error('Error fetching savings data:', e);
    }
}

function renderSavingsScreen() {
    const symbol = savingsData.currencySymbol || '$';
    const goals = savingsData.goals;

    document.getElementById('total-saved').textContent = `${symbol}${savingsData.totalSaved.toFixed(2)}`;
    document.getElementById('total-target').textContent = `${symbol}${savingsData.totalTarget.toFixed(2)}`;

    const listEl = document.getElementById('savings-goals-list');

    if (goals.length === 0) {
        listEl.innerHTML = '<p class="empty-message">No savings goals yet. Create one to start tracking!</p>';
        return;
    }

    listEl.innerHTML = goals.map(goal => {
        const progress = goal.current_progress || 0;
        const target = goal.target_amount || 0;
        const percentage = target > 0 ? Math.min(100, (progress / target) * 100) : 0;
        const monthlyAllocation = goal.monthly_allocation || 0;

        // Determine progress bar color
        let progressClass = 'progress-green';
        if (percentage >= 75) {
            progressClass = 'progress-green';
        } else if (percentage >= 50) {
            progressClass = 'progress-yellow';
        } else if (percentage >= 25) {
            progressClass = 'progress-yellow';
        }

        return `
            <div class="savings-goal-item" data-id="${goal.id}">
                <div class="goal-header">
                    <span class="goal-name">${escapeHtml(goal.name)}</span>
                    <button class="btn-delete-small" onclick="deleteGoal(${goal.id})" title="Delete">×</button>
                </div>
                <div class="goal-progress-section">
                    <div class="goal-amounts">
                        <span class="current-amount">${symbol}${progress.toFixed(2)}</span>
                        <span class="target-amount">of ${symbol}${target.toFixed(2)}</span>
                    </div>
                    <div class="progress-bar-container">
                        <div class="progress-bar ${progressClass}" style="width: ${percentage}%"></div>
                    </div>
                    <div class="goal-percentage">${percentage.toFixed(1)}%</div>
                </div>
                <div class="goal-footer">
                    <div class="goal-allocation">
                        <span class="allocation-label">Monthly allocation:</span>
                        <span class="allocation-value">${symbol}${monthlyAllocation.toFixed(2)}</span>
                    </div>
                    <button class="btn btn-small btn-secondary" onclick="showUpdateProgressModal(${goal.id}, ${progress}, ${target})">Add Funds</button>
                </div>
            </div>
        `;
    }).join('');
}

function showAddGoalModal() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Savings Goal</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Goal Name</label>
                    <input type="text" id="goal-name" placeholder="e.g., Emergency Fund, Vacation">
                </div>
                <div class="form-group">
                    <label>Target Amount</label>
                    <input type="number" id="goal-target" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Monthly Allocation</label>
                    <input type="number" id="goal-allocation" step="0.01" placeholder="0.00">
                    <span class="form-hint">How much you plan to save each month</span>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitGoal()">Create Goal</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitGoal() {
    const name = document.getElementById('goal-name').value.trim();
    const targetAmount = parseFloat(document.getElementById('goal-target').value);
    const monthlyAllocation = parseFloat(document.getElementById('goal-allocation').value) || 0;

    if (!name) {
        alert('Please enter a name for this goal');
        return;
    }

    if (!targetAmount || targetAmount <= 0) {
        alert('Please enter a valid target amount');
        return;
    }

    try {
        await window.__invoke__('add_savings_goal', {
            name,
            targetAmount,
            monthlyAllocation
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadSavingsScreen();
    } catch (e) {
        console.error('Error adding savings goal:', e);
        alert('Failed to add savings goal');
    }
}

function showUpdateProgressModal(goalId, currentProgress, targetAmount) {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add to Savings</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Current Progress</label>
                    <div class="current-progress-display">${savingsData.currencySymbol}${(currentProgress || 0).toFixed(2)}</div>
                </div>
                <div class="form-group">
                    <label>Add Amount</label>
                    <input type="number" id="add-amount" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>New Progress</label>
                    <div class="new-progress-display" id="new-progress-preview">${savingsData.currencySymbol}${(currentProgress || 0).toFixed(2)}</div>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitProgressUpdate(${goalId}, ${currentProgress})">Update</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);

    // Live preview of new progress
    const amountInput = document.getElementById('add-amount');
    amountInput.addEventListener('input', () => {
        const addAmount = parseFloat(amountInput.value) || 0;
        const newProgress = currentProgress + addAmount;
        document.getElementById('new-progress-preview').textContent =
            `${savingsData.currencySymbol}${newProgress.toFixed(2)}`;
    });

    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitProgressUpdate(goalId, currentProgress) {
    const addAmount = parseFloat(document.getElementById('add-amount').value);

    if (!addAmount || addAmount <= 0) {
        alert('Please enter a valid amount to add');
        return;
    }

    const newProgress = currentProgress + addAmount;

    try {
        await window.__invoke__('update_savings_progress', {
            id: goalId,
            currentProgress: newProgress
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadSavingsScreen();
    } catch (e) {
        console.error('Error updating savings progress:', e);
        alert('Failed to update savings progress');
    }
}

async function deleteGoal(id) {
    if (!confirm('Are you sure you want to delete this savings goal?')) {
        return;
    }

    try {
        // For now, just reload the screen (backend doesn't have delete yet)
        await loadSavingsScreen();
    } catch (e) {
        console.error('Error deleting goal:', e);
    }
}

window.loadSavingsScreen = loadSavingsScreen;
window.showAddGoalModal = showAddGoalModal;
window.submitGoal = submitGoal;
window.showUpdateProgressModal = showUpdateProgressModal;
window.submitProgressUpdate = submitProgressUpdate;
window.deleteGoal = deleteGoal;
