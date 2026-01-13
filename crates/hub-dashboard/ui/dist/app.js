// Productivity Hub Dashboard

// Tauri API - handle both Tauri 2.x API structures
let invoke;

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
let tauriReady = false;

// Initialize Tauri API
function initTauri() {
    // Tauri 2.x uses __TAURI_INTERNALS__ 
    if (window.__TAURI_INTERNALS__) {
        invoke = window.__TAURI_INTERNALS__.invoke;
        tauriReady = true;
        console.log('Tauri API initialized (INTERNALS)');
        return true;
    }
    // Fallback for other Tauri 2.x structures
    if (window.__TAURI__ && window.__TAURI__.core) {
        invoke = window.__TAURI__.core.invoke;
        tauriReady = true;
        console.log('Tauri API initialized (core)');
        return true;
    }
    if (window.__TAURI__ && window.__TAURI__.tauri) {
        invoke = window.__TAURI__.tauri.invoke;
        tauriReady = true;
        console.log('Tauri API initialized (tauri)');
        return true;
    }
    console.error('Tauri API not available');
    return false;
}

// Initialize
document.addEventListener('DOMContentLoaded', async () => {
    console.log('DOM loaded, initializing...');
    
    // Setup tabs first (doesn't need Tauri)
    setupTabs();
    
    // Render tools immediately with default status
    renderTools();
    renderAutoStartTools();
    
    // Initialize Tauri
    if (initTauri()) {
        try {
            await loadConfig();
            await loadToolStatuses();
            renderTools(); // Re-render with actual statuses
            renderAutoStartTools();
        } catch (e) {
            console.error('Failed to load initial data:', e);
        }
        
        // Setup event listeners that need Tauri
        setupEventListeners();
        
        // Poll for status updates
        setInterval(async () => {
            try {
                await loadToolStatuses();
            } catch (e) {
                console.error('Status poll failed:', e);
            }
        }, 2000);
    } else {
        // Show error to user
        const grid = document.getElementById('toolsGrid');
        grid.innerHTML = '<p style="color: var(--error); padding: 20px;">Failed to connect to Tauri backend. Please restart the application.</p>';
    }
});

// Tab navigation
function setupTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    console.log('Setting up tabs:', tabButtons.length, 'buttons,', tabContents.length, 'contents');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', (e) => {
            e.preventDefault();
            const tabId = button.dataset.tab;
            console.log('Tab clicked:', tabId);
            
            // Remove active from all
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            // Add active to clicked
            button.classList.add('active');
            const targetContent = document.getElementById(tabId);
            if (targetContent) {
                targetContent.classList.add('active');
                console.log('Activated tab:', tabId);
            } else {
                console.error('Tab content not found:', tabId);
            }
        });
    });
}

// Load configuration
async function loadConfig() {
    if (!tauriReady) return;
    
    try {
        config = await invoke('get_config');
        console.log('Config loaded:', config);
        
        // Apply settings to UI
        const autoStartEl = document.getElementById('autoStart');
        const startMinEl = document.getElementById('startMinimized');
        const darkModeEl = document.getElementById('darkMode');
        
        if (autoStartEl) autoStartEl.checked = config.auto_start || false;
        if (startMinEl) startMinEl.checked = config.start_minimized || false;
        if (darkModeEl) darkModeEl.checked = config.dark_mode !== false; // Default to dark
        
        // Apply dark mode (default is dark, so only add light-mode if explicitly disabled)
        if (config.dark_mode === false) {
            document.body.classList.add('light-mode');
        }
        
        // Check if API key exists
        try {
            const hasApiKey = await invoke('has_api_key');
            if (hasApiKey) {
                const apiKeyEl = document.getElementById('apiKey');
                if (apiKeyEl) apiKeyEl.placeholder = '••••••••••••••••';
            }
        } catch (e) {
            console.error('Failed to check API key:', e);
        }
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

// Load tool statuses
async function loadToolStatuses() {
    if (!tauriReady) return;
    
    try {
        toolStatuses = await invoke('get_tool_statuses');
        console.log('Tool statuses:', toolStatuses);
        updateToolCards();
    } catch (e) {
        console.error('Failed to load tool statuses:', e);
    }
}

// Render tools grid
function renderTools() {
    const grid = document.getElementById('toolsGrid');
    if (!grid) {
        console.error('Tools grid not found!');
        return;
    }
    
    console.log('Rendering', TOOLS.length, 'tools');
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
                <button class="btn ${isRunning ? 'btn-danger' : 'btn-success'}" 
                        onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}">
                    ${isRunning ? 'Stop' : 'Start'}
                </button>
                <button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')">Settings</button>
            </div>
        `;
        
        grid.appendChild(card);
    });
    
    console.log('Tools rendered');
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
        if (statusEl) {
            statusEl.className = `tool-status ${isRunning ? 'running' : 'stopped'}`;
            statusEl.innerHTML = `<span class="status-dot"></span>${status}`;
        }
        
        const actionsEl = card.querySelector('.tool-actions');
        if (actionsEl) {
            actionsEl.innerHTML = `
                <button class="btn ${isRunning ? 'btn-danger' : 'btn-success'}" 
                        onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}">
                    ${isRunning ? 'Stop' : 'Start'}
                </button>
                <button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')">Settings</button>
            `;
        }
    });
}

// Render auto-start checkboxes
function renderAutoStartTools() {
    const container = document.getElementById('autoStartTools');
    if (!container) return;
    
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
    const toggleBtn = document.getElementById('toggleApiKey');
    if (toggleBtn) {
        toggleBtn.addEventListener('click', () => {
            const input = document.getElementById('apiKey');
            if (input) {
                input.type = input.type === 'password' ? 'text' : 'password';
            }
        });
    }
    
    // Save API key
    const saveApiKeyBtn = document.getElementById('saveApiKeyBtn');
    if (saveApiKeyBtn) {
        saveApiKeyBtn.addEventListener('click', async () => {
            const apiKey = document.getElementById('apiKey')?.value;
            const status = document.getElementById('apiKeyStatus');
            
            if (!apiKey) {
                if (status) {
                    status.textContent = 'Please enter an API key';
                    status.className = 'status error';
                }
                return;
            }
            
            try {
                await invoke('save_api_key', { apiKey });
                if (status) {
                    status.textContent = 'API key saved successfully!';
                    status.className = 'status success';
                }
                const apiKeyInput = document.getElementById('apiKey');
                if (apiKeyInput) {
                    apiKeyInput.value = '';
                    apiKeyInput.placeholder = '••••••••••••••••';
                }
            } catch (e) {
                if (status) {
                    status.textContent = `Error: ${e}`;
                    status.className = 'status error';
                }
            }
        });
    }
    
    // Validate API key
    const validateBtn = document.getElementById('validateApiKeyBtn');
    if (validateBtn) {
        validateBtn.addEventListener('click', async () => {
            const status = document.getElementById('apiKeyStatus');
            if (status) {
                status.textContent = 'Validating...';
                status.className = 'status';
            }
            
            try {
                const result = await invoke('validate_api_key');
                if (status) {
                    if (result.valid) {
                        status.textContent = 'API key is valid!';
                        status.className = 'status success';
                    } else {
                        status.textContent = `Invalid: ${result.error}`;
                        status.className = 'status error';
                    }
                }
            } catch (e) {
                if (status) {
                    status.textContent = `Error: ${e}`;
                    status.className = 'status error';
                }
            }
        });
    }
    
    // Dark mode toggle
    const darkModeCheckbox = document.getElementById('darkMode');
    if (darkModeCheckbox) {
        darkModeCheckbox.addEventListener('change', (e) => {
            if (e.target.checked) {
                document.body.classList.remove('light-mode');
            } else {
                document.body.classList.add('light-mode');
            }
        });
    }
    
    // Save settings
    const saveSettingsBtn = document.getElementById('saveSettingsBtn');
    if (saveSettingsBtn) {
        saveSettingsBtn.addEventListener('click', async () => {
            const status = document.getElementById('settingsStatus');
            
            try {
                // Gather settings
                const newConfig = {
                    auto_start: document.getElementById('autoStart')?.checked || false,
                    start_minimized: document.getElementById('startMinimized')?.checked || false,
                    dark_mode: document.getElementById('darkMode')?.checked !== false,
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
                
                if (status) {
                    status.textContent = 'Settings saved!';
                    status.className = 'status success';
                }
            } catch (e) {
                if (status) {
                    status.textContent = `Error: ${e}`;
                    status.className = 'status error';
                }
            }
        });
    }
}

// Tool actions (global functions for onclick handlers)
window.startTool = async function(toolId) {
    if (!tauriReady) {
        alert('Tauri not ready');
        return;
    }
    
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
    if (!tauriReady) {
        alert('Tauri not ready');
        return;
    }
    
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
    if (!tauriReady) {
        alert('Tauri not ready');
        return;
    }
    
    try {
        await invoke('open_tool_settings', { toolId });
    } catch (e) {
        console.error(`Failed to open settings for ${toolId}:`, e);
        alert(`This tool doesn't have a separate settings window. Start the tool and use its tray icon for settings.`);
    }
};
