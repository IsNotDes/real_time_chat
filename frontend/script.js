const messagesList = document.getElementById('messages');
const messageForm = document.getElementById('form');
const messageInput = document.getElementById('input');
const sendButton = document.getElementById('sendButton');
const statusDiv = document.getElementById('status');

const socket = new WebSocket('ws://127.0.0.1:8080');

let username = null; // Will be set by the user's first message
let ownSenderAddr = null; // Store our own sender_addr to identify own messages if needed for styling

function addMessageToList(message, type = 'user') {
    const item = document.createElement('li');
    item.classList.add(type); // 'user', 'server', 'own'

    let prefix = '';
    if (message.username) {
        const usernameSpan = document.createElement('span');
        usernameSpan.className = 'username';
        usernameSpan.textContent = message.username + ': ';
        item.appendChild(usernameSpan);
    }
    item.appendChild(document.createTextNode(message.content));
    
    messagesList.appendChild(item);
    messagesList.scrollTop = messagesList.scrollHeight; // Auto-scroll to bottom
}

socket.onopen = () => {
    statusDiv.textContent = 'Connected. Please set your username.';
    messageInput.disabled = false;
    sendButton.disabled = false;
    console.log('WebSocket connection established');
};

socket.onmessage = (event) => {
    try {
        const message = JSON.parse(event.data);
        console.log('Message from server: ', message);

        if (!username && message.username === 'Server' && message.content.startsWith('Username set to:')) {
            // Extract and store the username confirmed by the server
            const parts = message.content.match(/Username set to: (\S+)\./);
            if (parts && parts[1]) {
                username = parts[1];
                statusDiv.textContent = `Connected as ${username}`;
                messageInput.placeholder = 'Type a message...';
            }
        }
        
        let messageType = 'user'; // Default for messages from other users
        if (message.username === 'Server') {
            messageType = 'server';
        }
        // We don't get our own messages back from the server due to server-side logic
        // If we did, we would check message.sender_addr against our own addr to style 'own' messages.

        addMessageToList(message, messageType);

    } catch (error) {
        console.error('Error parsing message from server:', error);
        addMessageToList({ content: 'Error: Could not parse server message.' }, 'server');
    }
};

socket.onerror = (error) => {
    statusDiv.textContent = 'Connection Error!';
    statusDiv.style.color = 'red';
    console.error('WebSocket Error: ', error);
    addMessageToList({ content: 'WebSocket connection error. Try refreshing.' }, 'server');
    messageInput.disabled = true;
    sendButton.disabled = true;
};

socket.onclose = () => {
    statusDiv.textContent = 'Disconnected. Try refreshing.';
    statusDiv.style.color = 'orange';
    console.log('WebSocket connection closed');
    addMessageToList({ content: 'Disconnected from chat server.' }, 'server');
    messageInput.disabled = true;
    sendButton.disabled = true;
};

messageForm.addEventListener('submit', (event) => {
    event.preventDefault();
    if (messageInput.value && socket.readyState === WebSocket.OPEN) {
        const content = messageInput.value;
        const messageToSend = {
            content: content
        };
        socket.send(JSON.stringify(messageToSend));
        
        // If this was the message to set the username, the server will confirm.
        // We don't add it to the list as an 'own' message yet until confirmed or if it's a regular chat message.
        // If the username IS already set, we could optimistically display it as 'own'
        if (username) {
             // Since server doesn't echo, we can add our own message directly for immediate feedback
            addMessageToList({ username: username, content: content }, 'own');
        }

        messageInput.value = '';
    } else if (socket.readyState !== WebSocket.OPEN) {
        addMessageToList({ content: 'Not connected to server.'}, 'server');
    }
});

// Initial state
messageInput.disabled = true;
sendButton.disabled = true; 