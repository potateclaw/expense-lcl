// analytics.js - Charts and analytics for expense tracking
const { invoke } = window.__TAURI__.core;

// Chart instances
let dailyBarChart = null;
let categoryPieChart = null;
let trendLineChart = null;

// Analytics state
let analyticsData = {
    transactions: [],
    categories: [],
    dateRange: '30', // days
    viewMode: 'daily' // 'daily' or 'monthly'
};

// Chart colors
const CHART_COLORS = [
    '#6366f1', '#8b5cf6', '#a855f7', '#d946ef', '#ec4899',
    '#f43f5e', '#ef4444', '#f97316', '#f59e0b', '#eab308',
    '#84cc16', '#22c55e', '#10b981', '#14b8a6', '#06b6d4',
    '#0ea5e9', '#3b82f6', '#6366f1'
];

async function loadAnalytics() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen analytics-screen">
            <div class="analytics-header">
                <h1>Analytics</h1>
                <span class="analytics-date" id="analytics-current-period"></span>
            </div>

            <!-- Date Range Selector -->
            <div class="date-range-selector">
                <div class="range-presets">
                    <button class="range-btn" data-range="7" onclick="setDateRange(7)">7 Days</button>
                    <button class="range-btn active" data-range="30" onclick="setDateRange(30)">30 Days</button>
                    <button class="range-btn" data-range="90" onclick="setDateRange(90)">90 Days</button>
                </div>
                <div class="view-toggle">
                    <button class="toggle-btn active" data-view="daily" onclick="setViewMode('daily')">
                        <span class="toggle-icon">📊</span> Daily
                    </button>
                    <button class="toggle-btn" data-view="monthly" onclick="setViewMode('monthly')">
                        <span class="toggle-icon">📅</span> Monthly
                    </button>
                </div>
            </div>

            <!-- Summary Cards -->
            <div class="analytics-summary">
                <div class="summary-card">
                    <span class="summary-label">Total Spent</span>
                    <span class="summary-value" id="total-spent">$0.00</span>
                </div>
                <div class="summary-card">
                    <span class="summary-label">Daily Average</span>
                    <span class="summary-value" id="daily-average">$0.00</span>
                </div>
                <div class="summary-card">
                    <span class="summary-label">Top Category</span>
                    <span class="summary-value" id="top-category">-</span>
                </div>
            </div>

            <!-- Daily Bar Chart -->
            <div class="card chart-card">
                <div class="card-header">
                    <h3>Spending Overview</h3>
                    <span class="chart-period" id="bar-chart-period"></span>
                </div>
                <div class="chart-container">
                    <canvas id="dailyBarChart"></canvas>
                </div>
                <div class="chart-empty" id="bar-chart-empty" style="display: none;">
                    <p>No spending data for this period</p>
                </div>
            </div>

            <!-- Category Breakdown -->
            <div class="card chart-card">
                <div class="card-header">
                    <h3>Category Breakdown</h3>
                </div>
                <div class="category-chart-wrapper">
                    <div class="chart-container pie-container">
                        <canvas id="categoryPieChart"></canvas>
                    </div>
                    <div class="chart-empty" id="pie-chart-empty" style="display: none;">
                        <p>No category data</p>
                    </div>
                    <div class="category-legend" id="category-legend"></div>
                </div>
            </div>

            <!-- Spending Trends -->
            <div class="card chart-card">
                <div class="card-header">
                    <h3>Spending Trends</h3>
                    <span class="chart-period" id="trend-chart-period"></span>
                </div>
                <div class="chart-container">
                    <canvas id="trendLineChart"></canvas>
                </div>
                <div class="chart-empty" id="trend-chart-empty" style="display: none;">
                    <p>Not enough data for trends</p>
                </div>
            </div>

            <!-- Category Details List -->
            <div class="card category-details-card">
                <div class="card-header">
                    <h3>Category Details</h3>
                </div>
                <div id="category-details-list"></div>
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
                <span class="nav-icon">📈</span>
                <span class="nav-label">Analytics</span>
            </button>
            <button class="nav-item" onclick="showSettings()">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    // Load Chart.js if not already loaded
    await ensureChartJs();

    // Fetch and render analytics
    await fetchAnalyticsData();
    renderAnalytics();

    // Set up exports
    window.showAnalytics = loadAnalytics;
}

async function ensureChartJs() {
    if (window.Chart) return;

    return new Promise((resolve, reject) => {
        const script = document.createElement('script');
        script.src = 'https://cdn.jsdelivr.net/npm/chart.js@4.4.1/dist/chart.umd.min.js';
        script.onload = resolve;
        script.onerror = reject;
        document.head.appendChild(script);
    });
}

async function fetchAnalyticsData() {
    const settings = JSON.parse(localStorage.getItem('settings') || '{}');
    analyticsData.currencySymbol = settings.currency === 'EUR' ? '€' :
                                   settings.currency === 'GBP' ? '£' :
                                   settings.currency === 'JPY' ? '¥' : '$';

    try {
        // Fetch categories
        try {
            analyticsData.categories = await invoke('get_categories');
        } catch (e) {
            analyticsData.categories = [];
        }

        // Fetch transactions within date range
        try {
            const transactions = await invoke('get_transactions', { limit: 1000 });
            analyticsData.transactions = filterTransactionsByDateRange(transactions);
        } catch (e) {
            // Generate sample data for demo
            analyticsData.transactions = generateSampleTransactions();
        }

        // Fetch dashboard summary
        try {
            analyticsData.summary = await invoke('get_dashboard_summary');
        } catch (e) {
            analyticsData.summary = { total_expenses: 0 };
        }

    } catch (e) {
        console.error('Error fetching analytics data:', e);
        analyticsData.transactions = generateSampleTransactions();
    }
}

function filterTransactionsByDateRange(transactions) {
    const now = new Date();
    const cutoffDate = new Date(now.getTime() - (analyticsData.dateRange * 24 * 60 * 60 * 1000));

    return transactions.filter(tx => {
        const txDate = new Date(tx.created_at);
        return txDate >= cutoffDate && tx.amount < 0; // Only expenses
    });
}

function generateSampleTransactions() {
    // Generate sample data for demonstration
    const transactions = [];
    const categories = ['Food & Dining', 'Transportation', 'Shopping', 'Entertainment', 'Utilities', 'Healthcare'];
    const now = new Date();

    for (let i = 0; i < 50; i++) {
        const daysAgo = Math.floor(Math.random() * parseInt(analyticsData.dateRange));
        const date = new Date(now.getTime() - (daysAgo * 24 * 60 * 60 * 1000));
        const category = categories[Math.floor(Math.random() * categories.length)];

        transactions.push({
            id: i,
            amount: -(Math.random() * 150 + 10).toFixed(2),
            category_name: category,
            note: '',
            created_at: date.toISOString()
        });
    }

    return transactions;
}

function setDateRange(days) {
    analyticsData.dateRange = days.toString();
    document.querySelectorAll('.range-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.range === days.toString());
    });
    fetchAnalyticsData().then(renderAnalytics);
}

function setViewMode(mode) {
    analyticsData.viewMode = mode;
    document.querySelectorAll('.toggle-btn').forEach(btn => {
        btn.classList.toggle('active', btn.dataset.view === mode);
    });
    renderAnalytics();
}

function renderAnalytics() {
    updatePeriodDisplay();
    renderSummaryCards();
    renderDailyBarChart();
    renderCategoryPieChart();
    renderTrendLineChart();
    renderCategoryDetails();
}

function updatePeriodDisplay() {
    const now = new Date();
    const pastDate = new Date(now.getTime() - (analyticsData.dateRange * 24 * 60 * 60 * 1000));
    const options = { month: 'short', day: 'numeric' };

    document.getElementById('analytics-current-period').textContent =
        `${pastDate.toLocaleDateString('en-US', options)} - ${now.toLocaleDateString('en-US', options)}`;
}

function renderSummaryCards() {
    const symbol = analyticsData.currencySymbol || '$';
    const transactions = analyticsData.transactions;

    // Calculate total spent
    const totalSpent = transactions.reduce((sum, tx) => sum + Math.abs(parseFloat(tx.amount)), 0);

    // Calculate daily average
    const daysInRange = parseInt(analyticsData.dateRange);
    const dailyAvg = totalSpent / daysInRange;

    // Find top category
    const categoryTotals = {};
    transactions.forEach(tx => {
        const cat = tx.category_name || 'Uncategorized';
        categoryTotals[cat] = (categoryTotals[cat] || 0) + Math.abs(parseFloat(tx.amount));
    });

    let topCategory = '-';
    let topAmount = 0;
    Object.entries(categoryTotals).forEach(([cat, amount]) => {
        if (amount > topAmount) {
            topAmount = amount;
            topCategory = cat;
        }
    });

    document.getElementById('total-spent').textContent = `${symbol}${totalSpent.toFixed(2)}`;
    document.getElementById('daily-average').textContent = `${symbol}${dailyAvg.toFixed(2)}`;
    document.getElementById('top-category').textContent = topCategory.length > 12
        ? topCategory.substring(0, 12) + '...'
        : topCategory;
}

function renderDailyBarChart() {
    const canvas = document.getElementById('dailyBarChart');
    const ctx = canvas.getContext('2d');
    const symbol = analyticsData.currencySymbol || '$';

    // Destroy existing chart
    if (dailyBarChart) {
        dailyBarChart.destroy();
    }

    // Aggregate data by day
    const daysToShow = analyticsData.viewMode === 'daily' ? 7 : 30;
    const dailyData = aggregateByDay(daysToShow);

    if (dailyData.labels.length === 0) {
        document.getElementById('bar-chart-empty').style.display = 'flex';
        canvas.style.display = 'none';
        return;
    }

    document.getElementById('bar-chart-empty').style.display = 'none';
    canvas.style.display = 'block';

    // Update period label
    document.getElementById('bar-chart-period').textContent =
        analyticsData.viewMode === 'daily' ? 'Last 7 days' : 'Last 30 days';

    dailyBarChart = new Chart(ctx, {
        type: 'bar',
        data: {
            labels: dailyData.labels,
            datasets: [{
                label: 'Daily Spending',
                data: dailyData.values,
                backgroundColor: 'rgba(99, 102, 241, 0.8)',
                borderColor: 'rgba(99, 102, 241, 1)',
                borderWidth: 1,
                borderRadius: 4,
                barThickness: analyticsData.viewMode === 'daily' ? 30 : 12
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            return `${symbol}${context.raw.toFixed(2)}`;
                        }
                    }
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        callback: function(value) {
                            return symbol + value.toFixed(0);
                        }
                    },
                    grid: {
                        color: 'rgba(0, 0, 0, 0.05)'
                    }
                },
                x: {
                    grid: {
                        display: false
                    }
                }
            }
        }
    });
}

function aggregateByDay(days) {
    const now = new Date();
    const data = {};

    // Initialize all days
    for (let i = days - 1; i >= 0; i--) {
        const date = new Date(now.getTime() - (i * 24 * 60 * 60 * 1000));
        const key = date.toISOString().split('T')[0];
        data[key] = 0;
    }

    // Aggregate transactions
    analyticsData.transactions.forEach(tx => {
        const key = tx.created_at.split('T')[0];
        if (data.hasOwnProperty(key)) {
            data[key] += Math.abs(parseFloat(tx.amount));
        }
    });

    const labels = Object.keys(data).map(date => {
        const d = new Date(date);
        return d.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
    });

    return {
        labels,
        values: Object.values(data)
    };
}

function renderCategoryPieChart() {
    const canvas = document.getElementById('categoryPieChart');
    const ctx = canvas.getContext('2d');

    if (categoryPieChart) {
        categoryPieChart.destroy();
    }

    // Aggregate by category
    const categoryTotals = {};
    analyticsData.transactions.forEach(tx => {
        const cat = tx.category_name || 'Uncategorized';
        categoryTotals[cat] = (categoryTotals[cat] || 0) + Math.abs(parseFloat(tx.amount));
    });

    const sortedCategories = Object.entries(categoryTotals)
        .sort((a, b) => b[1] - a[1]);

    if (sortedCategories.length === 0) {
        document.getElementById('pie-chart-empty').style.display = 'flex';
        canvas.style.display = 'none';
        return;
    }

    document.getElementById('pie-chart-empty').style.display = 'none';
    canvas.style.display = 'block';

    const labels = sortedCategories.map(([cat]) => cat);
    const values = sortedCategories.map(([, amount]) => amount);
    const colors = sortedCategories.map((_, i) => CHART_COLORS[i % CHART_COLORS.length]);

    // Render legend
    const legendContainer = document.getElementById('category-legend');
    const symbol = analyticsData.currencySymbol || '$';
    const total = values.reduce((a, b) => a + b, 0);

    legendContainer.innerHTML = sortedCategories.map(([cat, amount], i) => {
        const percentage = ((amount / total) * 100).toFixed(1);
        return `
            <div class="legend-item">
                <span class="legend-color" style="background-color: ${colors[i]}"></span>
                <span class="legend-label">${cat}</span>
                <span class="legend-value">${symbol}${amount.toFixed(2)}</span>
                <span class="legend-percentage">${percentage}%</span>
            </div>
        `;
    }).join('');

    categoryPieChart = new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels,
            datasets: [{
                data: values,
                backgroundColor: colors,
                borderWidth: 0,
                hoverOffset: 4
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            const value = context.raw;
                            const percentage = ((value / total) * 100).toFixed(1);
                            return `${symbol}${value.toFixed(2)} (${percentage}%)`;
                        }
                    }
                }
            },
            cutout: '60%'
        }
    });
}

function renderTrendLineChart() {
    const canvas = document.getElementById('trendLineChart');
    const ctx = canvas.getContext('2d');
    const symbol = analyticsData.currencySymbol || '$';

    if (trendLineChart) {
        trendLineChart.destroy();
    }

    // Use more data points for trend line
    const daysToShow = 30;
    const dailyData = aggregateByDay(daysToShow);

    if (dailyData.labels.length < 2) {
        document.getElementById('trend-chart-empty').style.display = 'flex';
        canvas.style.display = 'none';
        return;
    }

    document.getElementById('trend-chart-empty').style.display = 'none';
    canvas.style.display = 'block';

    // Calculate cumulative spending for trend
    let cumulative = 0;
    const cumulativeData = dailyData.values.map(val => {
        cumulative += val;
        return cumulative;
    });

    document.getElementById('trend-chart-period').textContent = 'Cumulative spending';

    trendLineChart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: dailyData.labels,
            datasets: [{
                label: 'Cumulative Spending',
                data: cumulativeData,
                borderColor: 'rgba(99, 102, 241, 1)',
                backgroundColor: 'rgba(99, 102, 241, 0.1)',
                fill: true,
                tension: 0.4,
                pointRadius: 0,
                pointHoverRadius: 4,
                borderWidth: 2
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    display: false
                },
                tooltip: {
                    callbacks: {
                        label: function(context) {
                            return `${symbol}${context.raw.toFixed(2)}`;
                        }
                    }
                }
            },
            scales: {
                y: {
                    beginAtZero: true,
                    ticks: {
                        callback: function(value) {
                            return symbol + value.toFixed(0);
                        }
                    },
                    grid: {
                        color: 'rgba(0, 0, 0, 0.05)'
                    }
                },
                x: {
                    grid: {
                        display: false
                    },
                    ticks: {
                        maxTicksLimit: 7
                    }
                }
            }
        }
    });
}

function renderCategoryDetails() {
    const container = document.getElementById('category-details-list');
    const symbol = analyticsData.currencySymbol || '$';

    // Aggregate by category
    const categoryTotals = {};
    analyticsData.transactions.forEach(tx => {
        const cat = tx.category_name || 'Uncategorized';
        categoryTotals[cat] = (categoryTotals[cat] || 0) + Math.abs(parseFloat(tx.amount));
    });

    const totalSpent = Object.values(categoryTotals).reduce((a, b) => a + b, 0);

    const sortedCategories = Object.entries(categoryTotals)
        .sort((a, b) => b[1] - a[1]);

    if (sortedCategories.length === 0) {
        container.innerHTML = '<p class="no-categories">No spending data available</p>';
        return;
    }

    container.innerHTML = sortedCategories.map(([cat, amount]) => {
        const percentage = totalSpent > 0 ? (amount / totalSpent) * 100 : 0;
        const icon = getCategoryIcon(cat);

        return `
            <div class="category-breakdown-item">
                <div class="category-info">
                    <span class="category-icon">${icon}</span>
                    <span class="category-name">${cat}</span>
                    <span class="category-spent">${symbol}${amount.toFixed(2)}</span>
                </div>
                <div class="progress-bar-container small">
                    <div class="progress-bar" style="width: ${percentage}%; background: #6366f1;"></div>
                </div>
                <div class="category-percentage">${percentage.toFixed(1)}% of total</div>
            </div>
        `;
    }).join('');
}

function getCategoryIcon(categoryName) {
    const iconMap = {
        'Food & Dining': '🍽️',
        'Transportation': '🚗',
        'Shopping': '🛍️',
        'Entertainment': '🎬',
        'Utilities': '💡',
        'Healthcare': '🏥',
        'Travel': '✈️',
        'Education': '📚',
        'Groceries': '🛒',
        'Other': '📦'
    };
    return iconMap[categoryName] || '📁';
}

// Navigation functions
function showDashboard() {
    if (typeof window.loadDashboard === 'function') {
        window.loadDashboard();
    }
}

function showTransactions() {
    if (typeof window.showTransactionsScreen === 'function') {
        window.showTransactionsScreen();
    }
}

function showSettings() {
    if (typeof window.showSettingsScreen === 'function') {
        window.showSettingsScreen();
    }
}

// Export the load function
window.loadAnalytics = loadAnalytics;
