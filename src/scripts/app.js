const { invoke } = window.__TAURI__.core;

async function init() {
    try {
        await invoke('init_db');

        const settings = JSON.parse(localStorage.getItem('settings') || '{}');

        if (settings.onboardingComplete) {
            if (typeof loadDashboard === 'function') {
                loadDashboard();
            } else {
                window.loadDashboard();
            }
        } else {
            if (typeof initOnboarding === 'function') {
                initOnboarding();
            } else {
                window.initOnboarding();
            }
        }
    } catch (e) {
        console.error('Init error:', e);
        if (typeof initOnboarding === 'function') {
            initOnboarding();
        }
    }
}

function toggleChat() {
    const panel = document.getElementById('chat-panel');
    panel.classList.toggle('hidden');
}

function sendChat() {
    const input = document.getElementById('chat-text');
    const message = input.value.trim();
    if (!message) return;

    const messagesDiv = document.getElementById('chat-messages');
    messagesDiv.innerHTML += `<div class="chat-message user">${message}</div>`;
    input.value = '';

    setTimeout(() => {
        messagesDiv.innerHTML += `<div class="chat-message bot">Thanks for your message! I'll help you with your budget.</div>`;
        messagesDiv.scrollTop = messagesDiv.scrollHeight;
    }, 500);
}

window.loadDashboard = loadDashboard;
window.initOnboarding = initOnboarding;

init();