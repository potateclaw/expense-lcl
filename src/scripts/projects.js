// projects.js - Project management
const { invoke } = window.__TAURI__.core;

let currentProjectId = null;
let projectsData = [];

async function loadProjects() {
    const container = document.getElementById('screen-container');
    container.innerHTML = `
        <div class="screen projects-screen">
            <div class="screen-header">
                <h1>Projects</h1>
                <button class="btn btn-primary" onclick="showCreateProjectModal()">+ Add Project</button>
            </div>
            <div id="projects-list"></div>
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
            <button class="nav-item" onclick="showSettings()">
                <span class="nav-icon">⚙️</span>
                <span class="nav-label">Settings</span>
            </button>
        </nav>
    `;

    await fetchProjects();
    renderProjectsList();
}

async function fetchProjects() {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        projectsData.currencySymbol = settings.currency === 'EUR' ? '€' :
                                      settings.currency === 'GBP' ? '£' :
                                      settings.currency === 'JPY' ? '¥' : '$';
        projectsData.projects = await invoke('get_projects') || [];
    } catch (e) {
        console.error('Error fetching projects:', e);
        projectsData.projects = [];
    }
}

function renderProjectsList() {
    const list = document.getElementById('projects-list');
    const symbol = projectsData.currencySymbol || '$';

    if (projectsData.projects.length === 0) {
        list.innerHTML = `
            <div class="empty-state">
                <p>No projects yet</p>
                <button class="btn btn-primary" onclick="showCreateProjectModal()">Create Your First Project</button>
            </div>
        `;
        return;
    }

    list.innerHTML = projectsData.projects.map(project => {
        const spent = project.spent || 0;
        const budget = project.budget || 0;
        const remaining = Math.max(0, budget - spent);
        const percentage = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;
        const progressClass = percentage < 50 ? 'progress-green' : percentage < 80 ? 'progress-yellow' : 'progress-red';

        return `
            <div class="card project-card" onclick="loadProjectDetail(${project.id})">
                <div class="project-header">
                    <h3 class="project-name">${project.name}</h3>
                    <span class="project-budget">${symbol}${budget.toFixed(2)}</span>
                </div>
                <div class="progress-bar-container">
                    <div class="progress-bar ${progressClass}" style="width: ${percentage}%"></div>
                </div>
                <div class="project-stats">
                    <span class="spent-amount">Spent: ${symbol}${spent.toFixed(2)}</span>
                    <span class="remaining-amount">Remaining: ${symbol}${remaining.toFixed(2)}</span>
                </div>
            </div>
        `;
    }).join('');
}

async function loadProjectDetail(projectId) {
    currentProjectId = projectId;
    const container = document.getElementById('screen-container');

    container.innerHTML = `
        <div class="screen project-detail-screen">
            <div class="detail-header">
                <button class="back-btn" onclick="loadProjects()">← Back</button>
            </div>
            <div id="project-detail-content"></div>
        </div>
    `;

    await fetchProjectDetail(projectId);
    renderProjectDetail();
}

async function fetchProjectDetail(projectId) {
    try {
        const settings = JSON.parse(localStorage.getItem('settings') || '{}');
        projectsData.currencySymbol = settings.currency === 'EUR' ? '€' :
                                      settings.currency === 'GBP' ? '£' :
                                      settings.currency === 'JPY' ? '¥' : '$';

        const project = await invoke('get_projects');
        projectsData.currentProject = project.find(p => p.id === projectId) || null;

        if (projectsData.currentProject) {
            projectsData.categories = await invoke('get_project_categories', { projectId }) || [];
        } else {
            projectsData.categories = [];
        }
    } catch (e) {
        console.error('Error fetching project detail:', e);
        projectsData.currentProject = null;
        projectsData.categories = [];
    }
}

function renderProjectDetail() {
    const content = document.getElementById('project-detail-content');
    const project = projectsData.currentProject;
    const symbol = projectsData.currencySymbol || '$';

    if (!project) {
        content.innerHTML = '<p>Project not found</p>';
        return;
    }

    const spent = project.spent || 0;
    const budget = project.budget || 0;
    const remaining = Math.max(0, budget - spent);
    const percentage = budget > 0 ? Math.min(100, (spent / budget) * 100) : 0;
    const progressClass = percentage < 50 ? 'progress-green' : percentage < 80 ? 'progress-yellow' : 'progress-red';

    content.innerHTML = `
        <div class="card project-overview-card">
            <h2 class="project-title">${project.name}</h2>
            <div class="project-budget-detail">
                <span class="budget-label">Total Budget</span>
                <span class="budget-value">${symbol}${budget.toFixed(2)}</span>
            </div>
            <div class="progress-bar-container large">
                <div class="progress-bar ${progressClass}" style="width: ${percentage}%"></div>
            </div>
            <div class="budget-summary">
                <div class="summary-item">
                    <span class="summary-label">Spent</span>
                    <span class="summary-value spent">${symbol}${spent.toFixed(2)}</span>
                </div>
                <div class="summary-item">
                    <span class="summary-label">Remaining</span>
                    <span class="summary-value remaining">${symbol}${remaining.toFixed(2)}</span>
                </div>
            </div>
        </div>

        <div class="card categories-card">
            <div class="card-header">
                <h3>Category Breakdown</h3>
                <button class="btn-text" onclick="showAddCategoryModal()">+ Add Category</button>
            </div>
            <div id="categories-list">
                ${renderCategoriesList()}
            </div>
        </div>

        <div class="card actions-card">
            <button class="btn btn-secondary" onclick="showAddExpenseToProjectModal()">+ Add Expense</button>
            <button class="btn btn-secondary" onclick="showEditProjectModal()">Edit Project</button>
            <button class="btn btn-danger" onclick="confirmDeleteProject()">Delete Project</button>
        </div>
    `;
}

function renderCategoriesList() {
    const symbol = projectsData.currencySymbol || '$';
    const categories = projectsData.categories;

    if (categories.length === 0) {
        return '<p class="no-categories">No categories yet. Add one to track spending by area.</p>';
    }

    return categories.map(cat => {
        const catSpent = cat.spent || 0;
        const catPercentage = cat.budget && cat.budget > 0 ? Math.min(100, (catSpent / cat.budget) * 100) : 0;
        const progressClass = catPercentage < 50 ? 'progress-green' : catPercentage < 80 ? 'progress-yellow' : 'progress-red';

        return `
            <div class="category-breakdown-item">
                <div class="category-info">
                    <span class="category-name">${cat.name}</span>
                    <span class="category-spent">${symbol}${catSpent.toFixed(2)}</span>
                </div>
                <div class="progress-bar-container small">
                    <div class="progress-bar ${progressClass}" style="width: ${catPercentage}%"></div>
                </div>
            </div>
        `;
    }).join('');
}

function showCreateProjectModal() {
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Create New Project</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Project Name</label>
                    <input type="text" id="project-name" placeholder="e.g., Kitchen Renovation">
                </div>
                <div class="form-group">
                    <label>Total Budget</label>
                    <input type="number" id="project-budget" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Categories</label>
                    <div id="categories-input-list"></div>
                    <button class="btn btn-secondary btn-small" onclick="addCategoryInput()">+ Add Category</button>
                </div>
                <div class="form-group">
                    <button class="btn btn-llm" onclick="suggestCategories()">
                        <span class="llm-icon">✨</span> Suggest Categories (AI)
                    </button>
                    <div id="llm-suggestion-status" class="llm-status"></div>
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitCreateProject()">Create Project</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });

    // Initialize with one empty category field
    addCategoryInput();
}

let categoryInputCount = 0;
function addCategoryInput() {
    const list = document.getElementById('categories-input-list');
    const id = categoryInputCount++;
    const div = document.createElement('div');
    div.className = 'category-input-item';
    div.id = `cat-input-${id}`;
    div.innerHTML = `
        <input type="text" placeholder="Category name" class="category-name-input">
        <button class="btn-remove" onclick="removeCategoryInput(${id})">×</button>
    `;
    list.appendChild(div);
}

function removeCategoryInput(id) {
    const item = document.getElementById(`cat-input-${id}`);
    if (item) {
        const list = document.getElementById('categories-input-list');
        if (list.children.length > 1) {
            item.remove();
        } else {
            item.querySelector('input').value = '';
        }
    }
}

async function suggestCategories() {
    const projectName = document.getElementById('project-name').value.trim();
    if (!projectName) {
        alert('Please enter a project name first');
        return;
    }

    const statusEl = document.getElementById('llm-suggestion-status');
    statusEl.textContent = 'Getting suggestions...';
    statusEl.className = 'llm-status loading';

    try {
        const response = await invoke('chat_query', { query: `Suggest 4-6 spending categories for a project called "${projectName}". Reply with just a comma-separated list of category names, no description.` });
        const categories = response.split(',').map(c => c.trim()).filter(c => c.length > 0);

        if (categories.length > 0) {
            const list = document.getElementById('categories-input-list');
            list.innerHTML = '';
            categoryInputCount = 0;

            categories.forEach(cat => {
                addCategoryInput();
                const inputs = list.querySelectorAll('.category-name-input');
                inputs[inputs.length - 1].value = cat;
            });

            statusEl.textContent = `Suggested: ${categories.join(', ')}`;
            statusEl.className = 'llm-status success';
        } else {
            statusEl.textContent = 'Could not generate suggestions';
            statusEl.className = 'llm-status error';
        }
    } catch (e) {
        console.error('Error suggesting categories:', e);
        statusEl.textContent = 'Failed to get suggestions';
        statusEl.className = 'llm-status error';
    }
}

async function submitCreateProject() {
    const name = document.getElementById('project-name').value.trim();
    const budget = parseFloat(document.getElementById('project-budget').value);

    if (!name) {
        alert('Please enter a project name');
        return;
    }
    if (isNaN(budget) || budget <= 0) {
        alert('Please enter a valid budget');
        return;
    }

    const categoryInputs = document.querySelectorAll('.category-name-input');
    const categories = Array.from(categoryInputs)
        .map(input => input.value.trim())
        .filter(name => name.length > 0);

    try {
        const projectId = await invoke('create_project', {
            name,
            budget,
            categories
        });

        document.querySelector('.modal-overlay')?.remove();
        await loadProjects();
    } catch (e) {
        console.error('Error creating project:', e);
        alert('Failed to create project');
    }
}

function showAddCategoryModal() {
    if (!currentProjectId) return;

    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Category</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Category Name</label>
                    <input type="text" id="new-category-name" placeholder="e.g., Materials, Labor">
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitAddCategory()">Add</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitAddCategory() {
    const name = document.getElementById('new-category-name').value.trim();

    if (!name) {
        alert('Please enter a category name');
        return;
    }

    try {
        await invoke('add_project_category', {
            projectId: currentProjectId,
            name
        });

        document.querySelector('.modal-overlay')?.remove();
        await fetchProjectDetail(currentProjectId);
        renderProjectDetail();
    } catch (e) {
        console.error('Error adding category:', e);
        alert('Failed to add category');
    }
}

function showAddExpenseToProjectModal() {
    if (!currentProjectId) return;

    const categories = projectsData.categories || [];
    const symbol = projectsData.currencySymbol || '$';

    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Add Expense to Project</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Amount</label>
                    <input type="number" id="expense-amount" step="0.01" placeholder="0.00">
                </div>
                <div class="form-group">
                    <label>Category (optional)</label>
                    <select id="expense-category">
                        <option value="">No category</option>
                        ${categories.map(c => `<option value="${c.id}">${c.name}</option>`).join('')}
                    </select>
                </div>
                <div class="form-group">
                    <label>Note (optional)</label>
                    <input type="text" id="expense-note" placeholder="What was this expense for?">
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitAddExpenseToProject()">Add Expense</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitAddExpenseToProject() {
    const amount = parseFloat(document.getElementById('expense-amount').value);
    const categoryId = document.getElementById('expense-category').value;
    const note = document.getElementById('expense-note').value.trim();

    if (isNaN(amount) || amount <= 0) {
        alert('Please enter a valid amount');
        return;
    }

    try {
        await invoke('add_transaction', {
            amount: -Math.abs(amount),
            categoryId: categoryId ? parseInt(categoryId) : null,
            note: note || `Project expense`
        });

        document.querySelector('.modal-overlay')?.remove();
        await fetchProjectDetail(currentProjectId);
        renderProjectDetail();
    } catch (e) {
        console.error('Error adding expense:', e);
        alert('Failed to add expense');
    }
}

function showEditProjectModal() {
    if (!projectsData.currentProject) return;

    const project = projectsData.currentProject;
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Edit Project</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <div class="form-group">
                    <label>Project Name</label>
                    <input type="text" id="edit-project-name" value="${project.name}">
                </div>
                <div class="form-group">
                    <label>Total Budget</label>
                    <input type="number" id="edit-project-budget" step="0.01" value="${project.budget}">
                </div>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-primary" onclick="submitEditProject()">Save Changes</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitEditProject() {
    const name = document.getElementById('edit-project-name').value.trim();
    const budget = parseFloat(document.getElementById('edit-project-budget').value);

    if (!name) {
        alert('Please enter a project name');
        return;
    }
    if (isNaN(budget) || budget <= 0) {
        alert('Please enter a valid budget');
        return;
    }

    try {
        await invoke('update_project', {
            id: currentProjectId,
            name,
            budget
        });

        document.querySelector('.modal-overlay')?.remove();
        await fetchProjectDetail(currentProjectId);
        renderProjectDetail();
    } catch (e) {
        console.error('Error updating project:', e);
        alert('Failed to update project');
    }
}

function confirmDeleteProject() {
    if (!projectsData.currentProject) return;

    const project = projectsData.currentProject;
    const modal = document.createElement('div');
    modal.className = 'modal-overlay';
    modal.innerHTML = `
        <div class="modal-content">
            <div class="modal-header">
                <h3>Delete Project</h3>
                <button class="modal-close" onclick="this.closest('.modal-overlay').remove()">×</button>
            </div>
            <div class="modal-body">
                <p>Are you sure you want to delete "<strong>${project.name}</strong>"?</p>
                <p class="warning-text">This action cannot be undone.</p>
            </div>
            <div class="modal-footer">
                <button class="btn btn-secondary" onclick="this.closest('.modal-overlay').remove()">Cancel</button>
                <button class="btn btn-danger" onclick="submitDeleteProject()">Delete</button>
            </div>
        </div>
    `;
    document.body.appendChild(modal);
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}

async function submitDeleteProject() {
    try {
        await invoke('delete_project', { id: currentProjectId });
        document.querySelector('.modal-overlay')?.remove();
        await loadProjects();
    } catch (e) {
        console.error('Error deleting project:', e);
        alert('Failed to delete project');
    }
}

function showDashboard() {
    if (typeof window.loadDashboard === 'function') {
        window.loadDashboard();
    }
}

function showTransactions() {
    if (typeof window.showTransactions === 'function') {
        window.showTransactions();
    } else if (typeof window.showTransactionsScreen === 'function') {
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

// Export functions to window for global access
window.loadProjects = loadProjects;
window.loadProjectDetail = loadProjectDetail;
window.showCreateProjectModal = showCreateProjectModal;
window.addCategoryInput = addCategoryInput;
window.removeCategoryInput = removeCategoryInput;
window.suggestCategories = suggestCategories;
window.submitCreateProject = submitCreateProject;
window.showAddCategoryModal = showAddCategoryModal;
window.submitAddCategory = submitAddCategory;
window.showAddExpenseToProjectModal = showAddExpenseToProjectModal;
window.submitAddExpenseToProject = submitAddExpenseToProject;
window.showEditProjectModal = showEditProjectModal;
window.submitEditProject = submitEditProject;
window.confirmDeleteProject = confirmDeleteProject;
window.submitDeleteProject = submitDeleteProject;
window.showDashboard = showDashboard;
window.showTransactions = showTransactions;
window.showCategories = showCategories;
window.showSettings = showSettings;
