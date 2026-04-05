// chat.js - Chat with LLM
async function sendChat() {
    const text = document.getElementById('chat-text').value;
    const response = await invoke('chat_query', { query: text });
    displayMessage(response);
}