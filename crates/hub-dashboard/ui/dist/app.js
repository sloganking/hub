// Productivity Hub Dashboard

const { invoke } = window.__TAURI__.core;

// Tool definitions
const TOOLS = [
    {
        id: 'desk-talk',
        name: 'DeskTalk',
        description: 'Voice-to-text transcription with push-to-talk. Press a key, speak, and your words are typed out.',
        requiresApiKey: true
    },
    {
        id: 'speak-selected',
        name: 'Speak Selected',
        description: 'Read selected text aloud using AI text-to-speech. Perfect for proofreading or accessibility.',
        requiresApiKey: true
    },
    {
        id: 'quick-assistant',
        name: 'Quick Assistant',
        description: 'Voice-activated AI assistant. Ask questions and get instant responses.',
        requiresApiKey: true
    },
    {
        id: 'flatten-string',
        name: 'Flatten String',
        description: 'Flatten clipboard text by removing newlines and extra whitespace. Great for code or quotes.',
        requiresApiKey: false
    },
    {
        id: 'typo-fix',
        name: 'Typo Fix',
        description: 'Fix typos in selected text using AI. Select text, press hotkey, get corrected text.',
        requiresApiKey: true
    },
    {
        id: 'ocr-paste',
        name: 'OCR Paste',
        description: 'Extract text from clipboard images using OCR. Copy an image, press hotkey, get the text.',
        requiresApiKey: true
    }
];

// State
let toolStatuses = {};
let config = {};

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    setupTabs();
    await loadConfig();
    await loadToolStatuses();
    renderTools();
    renderAutoStartTools();
    setupEventListeners();
    
    // Poll for status updates
    setInterval(loadToolStatuses, 2000);
});

// Tab navigation
function setupTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabId = button.dataset.tab;
            
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            button.classList.add('active');
            document.getElementById(tabId).classList.add('active');
        });
    });
}

// Load configuration
async function loadConfig() {
    try {
        config = await invoke('get_config');
        
        // Apply settings to UI
        document.getElementById('autoStart').checked = config.auto_start || false;
        document.getElementById('startMinimized').checked = config.start_minimized || false;
        document.getElementById('darkMode').checked = config.dark_mode || false;
        
        // Apply dark mode
        if (!config.dark_mode) {
            document.body.classList.add('light-mode');
        }
        
        // Check if API key exists
        const hasApiKey = await invoke('has_api_key');
        if (hasApiKey) {
            document.getElementById('apiKey').placeholder = '••••••••••••••••';
        }
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

// Load tool statuses
async function loadToolStatuses() {
    try {
        toolStatuses = await invoke('get_tool_statuses');
        updateToolCards();
    } catch (e) {
        console.error('Failed to load tool statuses:', e);
    }
}

// Render tools grid
function renderTools() {
    const grid = document.getElementById('toolsGrid');
    grid.innerHTML = '';
    
    TOOLS.forEach(tool => {
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        
        const card = document.createElement('div');
        card.className = `tool-card ${isRunning ? 'running' : ''}`;
        card.id = `tool-${tool.id}`;
        
        card.innerHTML = `
            <div class="tool-header">
                <span class="tool-name">${tool.name}</span>
                <span class="tool-status ${isRunning ? 'running' : 'stopped'}">
                    <span class="status-dot"></span>
                    ${status}
                </span>
            </div>
            <p class="tool-description">${tool.description}</p>
            ${tool.requiresApiKey ? '<p class="hint">Requires OpenAI API key</p>' : ''}
            <div class="tool-actions">
                ${isRunning 
                    ? `<button class="btn btn-danger" onclick="stopTool('${tool.id}')">Stop</button>`
                    : `<button class="btn btn-success" onclick="startTool('${tool.id}')">Start</button>`
                }
                <button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')">Settings</button>
            </div>
        `;
        
        grid.appendChild(card);
    });
}

// Update tool cards (just status, not full re-render)
function updateToolCards() {
    TOOLS.forEach(tool => {
        const card = document.getElementById(`tool-${tool.id}`);
        if (!card) return;
        
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        
        card.className = `tool-card ${isRunning ? 'running' : ''}`;
        
        const statusEl = card.querySelector('.tool-status');
        statusEl.className = `tool-status ${isRunning ? 'running' : 'stopped'}`;
        statusEl.innerHTML = `<span class="status-dot"></span>${status}`;
        
        const actionsEl = card.querySelector('.tool-actions');
        actionsEl.innerHTML = `
            ${isRunning 
                ? `<button class="btn btn-danger" onclick="stopTool('${tool.id}')">Stop</button>`
                : `<button class="btn btn-success" onclick="startTool('${tool.id}')">Start</button>`
            }
            <button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')">Settings</button>
        `;
    });
}

// Render auto-start checkboxes
function renderAutoStartTools() {
    const container = document.getElementById('autoStartTools');
    container.innerHTML = '';
    
    TOOLS.forEach(tool => {
        const toolConfig = config.tools?.[tool.id] || {};
        const isAutoStart = toolConfig.auto_start || false;
        
        const label = document.createElement('label');
        label.className = 'checkbox';
        label.innerHTML = `
            <input type="checkbox" id="autoStart-${tool.id}" ${isAutoStart ? 'checked' : ''}>
            <span>${tool.name}</span>
        `;
        container.appendChild(label);
    });
}

// Setup event listeners
function setupEventListeners() {
    // Toggle API key visibility
    document.getElementById('toggleApiKey').addEventListener('click', () => {
        const input = document.getElementById('apiKey');
        input.type = input.type === 'password' ? 'text' : 'password';
    });
    
    // Save API key
    document.getElementById('saveApiKeyBtn').addEventListener('click', async () => {
        const apiKey = document.getElementById('apiKey').value;
        const status = document.getElementById('apiKeyStatus');
        
        if (!apiKey) {
            status.textContent = 'Please enter an API key';
            status.className = 'status error';
            return;
        }
        
        try {
            await invoke('save_api_key', { apiKey });
            status.textContent = 'API key saved successfully!';
            status.className = 'status success';
            document.getElementById('apiKey').value = '';
            document.getElementById('apiKey').placeholder = '••••••••••••••••';
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
    
    // Validate API key
    document.getElementById('validateApiKeyBtn').addEventListener('click', async () => {
        const status = document.getElementById('apiKeyStatus');
        status.textContent = 'Validating...';
        status.className = 'status';
        
        try {
            const result = await invoke('validate_api_key');
            if (result.valid) {
                status.textContent = 'API key is valid!';
                status.className = 'status success';
            } else {
                status.textContent = `Invalid: ${result.error}`;
                status.className = 'status error';
            }
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
    
    // Dark mode toggle
    document.getElementById('darkMode').addEventListener('change', (e) => {
        if (e.target.checked) {
            document.body.classList.remove('light-mode');
        } else {
            document.body.classList.add('light-mode');
        }
    });
    
    // Save settings
    document.getElementById('saveSettingsBtn').addEventListener('click', async () => {
        const status = document.getElementById('settingsStatus');
        
        try {
            // Gather settings
            const newConfig = {
                auto_start: document.getElementById('autoStart').checked,
                start_minimized: document.getElementById('startMinimized').checked,
                dark_mode: document.getElementById('darkMode').checked,
                tools: {}
            };
            
            // Gather auto-start settings for each tool
            TOOLS.forEach(tool => {
                const checkbox = document.getElementById(`autoStart-${tool.id}`);
                newConfig.tools[tool.id] = {
                    enabled: true,
                    auto_start: checkbox ? checkbox.checked : false
                };
            });
            
            await invoke('save_config', { config: newConfig });
            config = newConfig;
            
            status.textContent = 'Settings saved!';
            status.className = 'status success';
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
}

// Tool actions (global functions for onclick handlers)
window.startTool = async function(toolId) {
    try {
        await invoke('start_tool', { toolId });
        toolStatuses[toolId] = 'Starting';
        updateToolCards();
    } catch (e) {
        console.error(`Failed to start ${toolId}:`, e);
        alert(`Failed to start tool: ${e}`);
    }
};

window.stopTool = async function(toolId) {
    try {
        await invoke('stop_tool', { toolId });
        toolStatuses[toolId] = 'Stopped';
        updateToolCards();
    } catch (e) {
        console.error(`Failed to stop ${toolId}:`, e);
        alert(`Failed to stop tool: ${e}`);
    }
};

window.openToolSettings = async function(toolId) {
    try {
        await invoke('open_tool_settings', { toolId });
    } catch (e) {
        console.error(`Failed to open settings for ${toolId}:`, e);
        alert(`This tool doesn't have a separate settings window.`);
    }
};
