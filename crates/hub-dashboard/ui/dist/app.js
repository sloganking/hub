// Productivity Hub Dashboard

// Tauri API - handle both Tauri 2.x API structures
let invoke;

// Tool definitions with type info
const TOOLS = [
    {
        id: 'desk-talk',
        name: 'DeskTalk',
        description: 'Voice-to-text transcription with push-to-talk. Press a key, speak, and your words are typed out.',
        requiresApiKey: true,
        type: 'gui', // Has its own GUI for hotkey config
        hotkeyNote: 'Configure in DeskTalk settings'
    },
    {
        id: 'speak-selected',
        name: 'Speak Selected',
        description: 'Read selected text aloud using AI text-to-speech. Perfect for proofreading or accessibility.',
        requiresApiKey: true,
        type: 'cli',
        hotkeyArg: '--ptt-key'
    },
    {
        id: 'quick-assistant',
        name: 'Quick Assistant',
        description: 'Voice-activated AI assistant. Ask questions and get instant responses.',
        requiresApiKey: true,
        type: 'cli',
        hotkeyArg: '--ptt-key'
    },
    {
        id: 'flatten-string',
        name: 'Flatten String',
        description: 'Flatten clipboard text by removing newlines and extra whitespace. Great for code or quotes.',
        requiresApiKey: false,
        type: 'cli',
        hotkeyArg: '--trigger-key'
    },
    {
        id: 'typo-fix',
        name: 'Typo Fix',
        description: 'Fix typos in selected text using AI. Select text, press hotkey, get corrected text.',
        requiresApiKey: true,
        type: 'gui',
        hotkeyNote: 'Configure in TypoFix settings'
    },
    {
        id: 'ocr-paste',
        name: 'OCR Paste',
        description: 'Extract text from clipboard images using OCR. Copy an image, press hotkey, get the text.',
        requiresApiKey: true,
        type: 'gui',
        hotkeyNote: 'Configure in OCR Paste settings'
    }
];

// Available hotkeys
const HOTKEY_OPTIONS = [
    { value: '', label: 'Not Set' },
    { value: 'F13', label: 'F13' },
    { value: 'F14', label: 'F14' },
    { value: 'F15', label: 'F15' },
    { value: 'F16', label: 'F16' },
    { value: 'F17', label: 'F17' },
    { value: 'F18', label: 'F18' },
    { value: 'F19', label: 'F19' },
    { value: 'F20', label: 'F20' },
    { value: 'F21', label: 'F21' },
    { value: 'F22', label: 'F22' },
    { value: 'F23', label: 'F23' },
    { value: 'F24', label: 'F24' },
    { value: 'Insert', label: 'Insert' },
    { value: 'Delete', label: 'Delete' },
    { value: 'Home', label: 'Home' },
    { value: 'End', label: 'End' },
    { value: 'PageUp', label: 'Page Up' },
    { value: 'PageDown', label: 'Page Down' },
    { value: 'ScrollLock', label: 'Scroll Lock' },
    { value: 'Pause', label: 'Pause' },
];

// State
let toolStatuses = {};
let config = {};
let tauriReady = false;

// Initialize Tauri API
function initTauri() {
    if (window.__TAURI_INTERNALS__) {
        invoke = window.__TAURI_INTERNALS__.invoke;
        tauriReady = true;
        console.log('Tauri API initialized (INTERNALS)');
        return true;
    }
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
    
    setupTabs();
    renderTools();
    renderAutoStartTools();
    renderHotkeyConfig();
    
    if (initTauri()) {
        try {
            await loadConfig();
            await loadToolStatuses();
            renderTools();
            renderAutoStartTools();
            renderHotkeyConfig();
        } catch (e) {
            console.error('Failed to load initial data:', e);
        }
        
        setupEventListeners();
        
        setInterval(async () => {
            try {
                await loadToolStatuses();
            } catch (e) {
                console.error('Status poll failed:', e);
            }
        }, 2000);
    } else {
        const grid = document.getElementById('toolsGrid');
        grid.innerHTML = '<p style="color: var(--error); padding: 20px;">Failed to connect to Tauri backend. Please restart the application.</p>';
    }
});

// Tab navigation
function setupTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', (e) => {
            e.preventDefault();
            const tabId = button.dataset.tab;
            
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            button.classList.add('active');
            const targetContent = document.getElementById(tabId);
            if (targetContent) {
                targetContent.classList.add('active');
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
        
        const autoStartEl = document.getElementById('autoStart');
        const startMinEl = document.getElementById('startMinimized');
        const darkModeEl = document.getElementById('darkMode');
        
        if (autoStartEl) autoStartEl.checked = config.auto_start || false;
        if (startMinEl) startMinEl.checked = config.start_minimized || false;
        if (darkModeEl) darkModeEl.checked = config.dark_mode !== false;
        
        if (config.dark_mode === false) {
            document.body.classList.add('light-mode');
        }
        
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
        updateToolCards();
    } catch (e) {
        console.error('Failed to load tool statuses:', e);
    }
}

// Render tools grid
function renderTools() {
    const grid = document.getElementById('toolsGrid');
    if (!grid) return;
    
    grid.innerHTML = '';
    
    TOOLS.forEach(tool => {
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        const toolConfig = config.tools?.[tool.id] || {};
        const hasHotkey = toolConfig.hotkey || toolConfig.special_hotkey || tool.type === 'gui';
        
        const card = document.createElement('div');
        card.className = `tool-card ${isRunning ? 'running' : ''}`;
        card.id = `tool-${tool.id}`;
        
        let hotkeyInfo = '';
        if (tool.type === 'cli') {
            if (toolConfig.hotkey) {
                hotkeyInfo = `<span class="hotkey-badge">${toolConfig.hotkey}</span>`;
            } else if (toolConfig.special_hotkey) {
                hotkeyInfo = `<span class="hotkey-badge">Key ${toolConfig.special_hotkey}</span>`;
            } else {
                hotkeyInfo = `<span class="hotkey-badge missing">No hotkey set</span>`;
            }
        }
        
        card.innerHTML = `
            <div class="tool-header">
                <span class="tool-name">${tool.name}</span>
                <span class="tool-status ${isRunning ? 'running' : 'stopped'}">
                    <span class="status-dot"></span>
                    ${status}
                </span>
            </div>
            <p class="tool-description">${tool.description}</p>
            ${hotkeyInfo}
            ${tool.requiresApiKey ? '<p class="hint">Requires OpenAI API key</p>' : ''}
            <div class="tool-actions">
                <button class="btn ${isRunning ? 'btn-danger' : 'btn-success'}" 
                        onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}"
                        ${!hasHotkey && tool.type === 'cli' ? 'disabled title="Set hotkey first"' : ''}>
                    ${isRunning ? 'Stop' : 'Start'}
                </button>
                <button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')">Settings</button>
            </div>
        `;
        
        grid.appendChild(card);
    });
}

// Update tool cards
function updateToolCards() {
    TOOLS.forEach(tool => {
        const card = document.getElementById(`tool-${tool.id}`);
        if (!card) return;
        
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        const toolConfig = config.tools?.[tool.id] || {};
        const hasHotkey = toolConfig.hotkey || toolConfig.special_hotkey || tool.type === 'gui';
        
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
                        onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}"
                        ${!hasHotkey && tool.type === 'cli' ? 'disabled title="Set hotkey first"' : ''}>
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

// Render hotkey configuration
function renderHotkeyConfig() {
    const container = document.getElementById('hotkeyConfig');
    if (!container) return;
    
    container.innerHTML = '';
    
    TOOLS.forEach(tool => {
        const toolConfig = config.tools?.[tool.id] || {};
        const row = document.createElement('div');
        row.className = 'hotkey-row';
        
        if (tool.type === 'gui') {
            row.innerHTML = `
                <div class="tool-info">
                    <div class="tool-name">${tool.name}</div>
                    <div class="tool-type gui">GUI App</div>
                </div>
                <div style="color: var(--text-muted); font-size: 13px;">
                    ${tool.hotkeyNote}
                </div>
            `;
        } else {
            const currentHotkey = toolConfig.hotkey || '';
            const specialKey = toolConfig.special_hotkey || '';
            
            let optionsHtml = HOTKEY_OPTIONS.map(opt => 
                `<option value="${opt.value}" ${currentHotkey === opt.value ? 'selected' : ''}>${opt.label}</option>`
            ).join('');
            
            row.innerHTML = `
                <div class="tool-info">
                    <div class="tool-name">${tool.name}</div>
                    <div class="tool-type cli">CLI Tool (requires hotkey)</div>
                </div>
                <select id="hotkey-${tool.id}">
                    ${optionsHtml}
                </select>
                <span class="or-label">or code:</span>
                <input type="number" class="special-key-input" id="special-${tool.id}" 
                       placeholder="e.g. 123" value="${specialKey}" min="0" max="65535">
            `;
        }
        
        container.appendChild(row);
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
                const newConfig = {
                    auto_start: document.getElementById('autoStart')?.checked || false,
                    start_minimized: document.getElementById('startMinimized')?.checked || false,
                    dark_mode: document.getElementById('darkMode')?.checked !== false,
                    tools: {}
                };
                
                // Preserve existing hotkey settings, update auto_start
                TOOLS.forEach(tool => {
                    const checkbox = document.getElementById(`autoStart-${tool.id}`);
                    const existingConfig = config.tools?.[tool.id] || {};
                    newConfig.tools[tool.id] = {
                        enabled: true,
                        auto_start: checkbox ? checkbox.checked : false,
                        hotkey: existingConfig.hotkey || null,
                        special_hotkey: existingConfig.special_hotkey || null
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
    
    // Save hotkeys
    const saveHotkeysBtn = document.getElementById('saveHotkeysBtn');
    if (saveHotkeysBtn) {
        saveHotkeysBtn.addEventListener('click', async () => {
            const status = document.getElementById('hotkeysStatus');
            
            try {
                const newConfig = {
                    auto_start: config.auto_start || false,
                    start_minimized: config.start_minimized || false,
                    dark_mode: config.dark_mode !== false,
                    tools: {}
                };
                
                TOOLS.forEach(tool => {
                    const existingConfig = config.tools?.[tool.id] || {};
                    let hotkey = null;
                    let specialHotkey = null;
                    
                    if (tool.type === 'cli') {
                        const hotkeySelect = document.getElementById(`hotkey-${tool.id}`);
                        const specialInput = document.getElementById(`special-${tool.id}`);
                        
                        if (hotkeySelect && hotkeySelect.value) {
                            hotkey = hotkeySelect.value;
                        }
                        if (specialInput && specialInput.value) {
                            specialHotkey = parseInt(specialInput.value) || null;
                        }
                    }
                    
                    newConfig.tools[tool.id] = {
                        enabled: true,
                        auto_start: existingConfig.auto_start || false,
                        hotkey: hotkey,
                        special_hotkey: specialHotkey
                    };
                });
                
                await invoke('save_config', { config: newConfig });
                config = newConfig;
                
                // Re-render to update UI
                renderTools();
                renderHotkeyConfig();
                
                if (status) {
                    status.textContent = 'Hotkeys saved! Restart tools for changes to take effect.';
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

// Tool actions
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
    const tool = TOOLS.find(t => t.id === toolId);
    
    if (tool && tool.type === 'cli') {
        // For CLI tools, switch to hotkeys tab
        document.querySelector('[data-tab="hotkeys"]').click();
        return;
    }
    
    // For GUI tools, show a message
    alert(`Start ${tool?.name || toolId} and use its system tray icon to access settings.`);
};
