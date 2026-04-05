const { invoke } = window.__TAURI__.core;

// State
let capturedImageData = null;
let extractedReceiptData = null;
let categories = [];

// Camera capture flow
async function captureReceipt() {
    const container = document.getElementById('screen-container');

    // Create hidden file input for camera capture
    const fileInput = document.createElement('input');
    fileInput.type = 'file';
    fileInput.accept = 'image/*';
    fileInput.capture = 'environment';
    fileInput.id = 'camera-input';
    fileInput.style.display = 'none';
    document.body.appendChild(fileInput);

    fileInput.addEventListener('change', async (e) => {
        const file = e.target.files[0];
        if (file) {
            await processCameraCapture(file);
        }
        document.body.removeChild(fileInput);
    });

    fileInput.click();
}

async function processCameraCapture(file) {
    capturedImageData = await readFileAsBase64(file);
    showImagePreview();
}

function readFileAsBase64(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = reject;
        reader.readAsDataURL(file);
    });
}

function showImagePreview() {
    const container = document.getElementById('screen-container');

    container.innerHTML = `
        <div class="screen camera-preview-screen">
            <div class="camera-preview-header">
                <button class="btn-icon" onclick="cancelCapture()">
                    <span>←</span>
                </button>
                <h2>Preview</h2>
                <div style="width: 40px;"></div>
            </div>

            <div class="camera-preview-content">
                <div class="preview-image-container">
                    <img id="preview-image" src="${capturedImageData}" alt="Receipt preview">
                </div>
            </div>

            <div class="camera-preview-actions">
                <button class="btn btn-secondary btn-full" onclick="retakePhoto()">
                    Retake
                </button>
                <button class="btn btn-primary btn-full" onclick="confirmPhoto()">
                    Confirm
                </button>
            </div>
        </div>
    `;
}

function retakePhoto() {
    capturedImageData = null;
    captureReceipt();
}

function cancelCapture() {
    capturedImageData = null;
    extractedReceiptData = null;
    if (typeof window.loadDashboard === 'function') {
        window.loadDashboard();
    }
}

async function confirmPhoto() {
    showProcessingState();
    await processReceiptImage();
}

// Show loading state during processing
function showProcessingState() {
    const container = document.getElementById('screen-container');

    container.innerHTML = `
        <div class="screen processing-screen">
            <div class="processing-content">
                <div class="spinner"></div>
                <h2>Processing Receipt</h2>
                <p>Analyzing image and extracting data...</p>
            </div>
        </div>
    `;
}

async function processReceiptImage() {
    try {
        // Call backend to process the receipt image
        const result = await invoke('process_receipt_image', {
            imageData: capturedImageData
        });

        extractedReceiptData = result;
        await loadCategories();
        showReceiptReview();
    } catch (e) {
        console.error('Error processing receipt:', e);
        // If processing fails, show manual entry form
        extractedReceiptData = {
            total: 0,
            tax: 0,
            discount: 0,
            line_items: [],
            suggested_category: null
        };
        await loadCategories();
        showReceiptReview();
    }
}

async function loadCategories() {
    try {
        categories = await invoke('get_categories') || [];
    } catch (e) {
        console.log('Could not fetch categories:', e);
        categories = [];
    }
}

// Receipt review screen
async function showReceiptReview() {
    const container = document.getElementById('screen-container');
    const data = extractedReceiptData || {};

    container.innerHTML = `
        <div class="screen receipt-review-screen">
            <div class="receipt-review-header">
                <button class="btn-icon" onclick="cancelReview()">
                    <span>←</span>
                </button>
                <h2>Review Receipt</h2>
                <div style="width: 40px;"></div>
            </div>

            <div class="receipt-image-container">
                <img id="review-image" src="${capturedImageData}" alt="Receipt">
            </div>

            <div class="receipt-form card">
                <h3>Extracted Data</h3>
                <p class="form-hint">Edit values if needed</p>

                <div class="form-group">
                    <label>Total Amount</label>
                    <input type="number" id="receipt-total" step="0.01" value="${data.total || 0}" placeholder="0.00">
                </div>

                <div class="form-group">
                    <label>Tax Amount</label>
                    <input type="number" id="receipt-tax" step="0.01" value="${data.tax || 0}" placeholder="0.00">
                </div>

                <div class="form-group">
                    <label>Discount Amount</label>
                    <input type="number" id="receipt-discount" step="0.01" value="${data.discount || 0}" placeholder="0.00">
                </div>

                <div class="form-group">
                    <label>Category</label>
                    <select id="receipt-category">
                        <option value="">Select category...</option>
                        ${categories.map(c => `
                            <option value="${c.id}" ${data.suggested_category == c.id ? 'selected' : ''}>
                                ${c.icon || ''} ${c.name}
                            </option>
                        `).join('')}
                    </select>
                </div>

                <div class="form-group">
                    <label>Project (Optional)</label>
                    <select id="receipt-project">
                        <option value="">No project</option>
                        <option value="1">Project A</option>
                        <option value="2">Project B</option>
                    </select>
                </div>

                <div class="line-items-section">
                    <div class="line-items-header">
                        <h4>Line Items</h4>
                        <button class="btn-text" onclick="addLineItem()">+ Add Item</button>
                    </div>
                    <div id="line-items-list">
                        ${(data.line_items || []).map((item, index) => `
                            <div class="line-item" data-index="${index}">
                                <input type="text" class="line-item-name" value="${item.name || ''}" placeholder="Item name">
                                <input type="number" class="line-item-qty" value="${item.qty || 1}" min="1" placeholder="Qty">
                                <input type="number" class="line-item-price" step="0.01" value="${item.price || 0}" placeholder="Price">
                                <button class="btn-icon-small" onclick="removeLineItem(${index})">×</button>
                            </div>
                        `).join('')}
                    </div>
                </div>

                <div class="receipt-actions">
                    <button class="btn btn-secondary" onclick="cancelReview()">Cancel</button>
                    <button class="btn btn-primary" onclick="saveReceipt()">Save Receipt</button>
                </div>
            </div>
        </div>
    `;
}

function cancelReview() {
    capturedImageData = null;
    extractedReceiptData = null;
    if (typeof window.loadDashboard === 'function') {
        window.loadDashboard();
    }
}

function addLineItem() {
    const list = document.getElementById('line-items-list');
    const index = list.children.length;
    const itemHtml = `
        <div class="line-item" data-index="${index}">
            <input type="text" class="line-item-name" value="" placeholder="Item name">
            <input type="number" class="line-item-qty" value="1" min="1" placeholder="Qty">
            <input type="number" class="line-item-price" step="0.01" value="0" placeholder="Price">
            <button class="btn-icon-small" onclick="removeLineItem(${index})">×</button>
        </div>
    `;
    list.insertAdjacentHTML('beforeend', itemHtml);
}

function removeLineItem(index) {
    const item = document.querySelector(`.line-item[data-index="${index}"]`);
    if (item) {
        item.remove();
    }
}

function getLineItems() {
    const items = [];
    document.querySelectorAll('.line-item').forEach((el) => {
        const name = el.querySelector('.line-item-name').value;
        const qty = parseFloat(el.querySelector('.line-item-qty').value) || 1;
        const price = parseFloat(el.querySelector('.line-item-price').value) || 0;
        if (name || price > 0) {
            items.push({ name, qty, price });
        }
    });
    return items;
}

async function saveReceipt() {
    const total = parseFloat(document.getElementById('receipt-total').value) || 0;
    const tax = parseFloat(document.getElementById('receipt-tax').value) || 0;
    const discount = parseFloat(document.getElementById('receipt-discount').value) || 0;
    const categoryId = document.getElementById('receipt-category').value;
    const projectId = document.getElementById('receipt-project').value;
    const lineItems = getLineItems();

    if (!categoryId) {
        alert('Please select a category');
        return;
    }

    if (total <= 0) {
        alert('Please enter a valid total amount');
        return;
    }

    try {
        // Build note from line items
        const note = lineItems.length > 0
            ? lineItems.map(item => `${item.qty}x ${item.name}`).join(', ')
            : '';

        await invoke('add_receipt', {
            amount: -Math.abs(total), // Expenses are negative
            categoryId: parseInt(categoryId),
            note: note,
            tax: tax,
            discount: discount,
            imageData: capturedImageData
        });

        // Clear state and return to dashboard
        capturedImageData = null;
        extractedReceiptData = null;

        if (typeof window.loadDashboard === 'function') {
            window.loadDashboard();
        }
    } catch (e) {
        console.error('Error saving receipt:', e);
        alert('Failed to save receipt. Please try again.');
    }
}

// Export for use from FAB
window.captureReceipt = captureReceipt;
window.retakePhoto = retakePhoto;
window.confirmPhoto = confirmPhoto;
window.cancelCapture = cancelCapture;
window.cancelReview = cancelReview;
window.addLineItem = addLineItem;
window.removeLineItem = removeLineItem;
window.saveReceipt = saveReceipt;
