// chat.js - Persistent Chat Panel with LLM Integration

// Chat state - persists during session
let chatHistory = [];
let isChatOpen = false;
let chatInitialized = false;

// Initialize chat when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    initializeChat();
});

function initializeChat() {
    // Only create elements if they don't already exist in the DOM
    // This prevents duplicate FAB and panel creation since index.html already has them
    if (!document.getElementById('chat-fab')) {
        createChatFAB();
    }
    if (!document.getElementById('chat-panel')) {
        createChatPanel();
    }
    setupChatListeners();
    chatInitialized = true;
}

function createChatFAB() {
    const fab = document.createElement('div');
    fab.id = 'chat-fab';
    fab.className = 'chat-fab';
    fab.innerHTML = '💬';
    fab.title = 'Open Chat';
    document.body.appendChild(fab);
}

function createChatPanel() {
    const panel = document.createElement('div');
    panel.id = 'chat-panel';
    panel.className = 'chat-panel hidden';
    panel.innerHTML = `
        <div id="chat-header" class="chat-header">
            <div class="chat-title">Chat</div>
            <button id="chat-minimize" class="chat-minimize" title="Minimize">−</button>
        </div>
        <div id="chat-messages" class="chat-messages"></div>
        <div id="chat-loading" class="chat-loading hidden">
            <span class="loading-dots">Thinking<span class="dot">.</span><span class="dot">.</span><span class="dot">.</span></span>
        </div>
        <div id="chat-input" class="chat-input">
            <input type="text" id="chat-text" placeholder="Ask about your spending..." />
            <button id="chat-send" class="chat-send" title="Send">➤</button>
        </div>
    `;
    document.body.appendChild(panel);
}

function setupChatListeners() {
    const fab = document.getElementById('chat-fab');
    const panel = document.getElementById('chat-panel');
    const minimizeBtn = document.getElementById('chat-minimize');
    const sendBtn = document.getElementById('chat-send');
    const input = document.getElementById('chat-text');

    if (fab) {
        fab.addEventListener('click', toggleChat);
    }

    if (minimizeBtn) {
        minimizeBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            minimizeChat();
        });
    }

    if (sendBtn) {
        sendBtn.addEventListener('click', sendChat);
    }

    if (input) {
        input.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') {
                sendChat();
            }
        });
    }
}

function toggleChat() {
    const panel = document.getElementById('chat-panel');
    const fab = document.getElementById('chat-fab');

    if (isChatOpen) {
        panel.classList.add('hidden');
        fab.innerHTML = '💬';
        fab.title = 'Open Chat';
    } else {
        panel.classList.remove('hidden');
        fab.innerHTML = '💬';
        fab.title = 'Close Chat';
        scrollToBottom();
    }
    isChatOpen = !isChatOpen;
}

function minimizeChat() {
    const panel = document.getElementById('chat-panel');
    const fab = document.getElementById('chat-fab');

    panel.classList.add('hidden');
    fab.innerHTML = '💬';
    fab.title = 'Open Chat';
    isChatOpen = false;
}

async function sendChat() {
    const input = document.getElementById('chat-text');
    const sendBtn = document.getElementById('chat-send');
    const text = input.value.trim();

    if (!text) return;

    // Disable input and button while processing
    input.disabled = true;
    sendBtn.disabled = true;

    // Add user message to history and display
    const userMessage = {
        role: 'user',
        content: text,
        timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
    };
    chatHistory.push(userMessage);
    displayMessage(userMessage);

    // Clear input
    input.value = '';

    // Show loading indicator
    showLoading(true);

    try {
        // Check invoke is available
        if (typeof window.__invoke__ !== 'function') {
            throw new Error('Backend not available (window.__invoke__ missing). Make sure the app is connected to the server.');
        }

        // Call backend chat_query command
        const response = await window.__invoke__('chat_query', { query: text });

        // Add assistant response to history and display
        const assistantMessage = {
            role: 'assistant',
            content: response,
            timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
        };
        chatHistory.push(assistantMessage);
        displayMessage(assistantMessage);
    } catch (error) {
        console.error('Chat error:', error);
        // Display error message
        const errorMessage = {
            role: 'assistant',
            content: `Sorry, I encountered an error: ${error.message || 'Unable to get response. Please try again.'}`,
            timestamp: new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
            isError: true
        };
        chatHistory.push(errorMessage);
        displayMessage(errorMessage);
    } finally {
        // Re-enable input and button
        input.disabled = false;
        sendBtn.disabled = false;
        input.focus();
        showLoading(false);
        scrollToBottom();
    }
}

function displayMessage(message) {
    const messagesContainer = document.getElementById('chat-messages');
    if (!messagesContainer) return;

    const messageDiv = document.createElement('div');
    messageDiv.className = `chat-message ${message.role}`;

    if (message.isError) {
        messageDiv.classList.add('error');
    }

    const timeSpan = document.createElement('span');
    timeSpan.className = 'chat-timestamp';
    timeSpan.textContent = message.timestamp;

    messageDiv.innerHTML = `
        <div class="message-content">${escapeHtml(message.content)}</div>
        <div class="message-time">${message.timestamp}</div>
    `;

    messagesContainer.appendChild(messageDiv);
    scrollToBottom();
}

function showLoading(show) {
    const loading = document.getElementById('chat-loading');
    if (loading) {
        if (show) {
            loading.classList.remove('hidden');
        } else {
            loading.classList.add('hidden');
        }
    }
}

function scrollToBottom() {
    const messagesContainer = document.getElementById('chat-messages');
    if (messagesContainer) {
        setTimeout(() => {
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }, 10);
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Make functions available globally
window.toggleChat = toggleChat;
window.minimizeChat = minimizeChat;
window.sendChat = sendChat;
window.displayMessage = displayMessage;
