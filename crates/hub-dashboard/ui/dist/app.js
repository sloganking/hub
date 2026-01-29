// Productivity Hub Dashboard

let invoke;

// Tool definitions - ALL are CLI tools that take hotkey args
const TOOLS = [
    {
        id: 'desk-talk',
        name: 'DeskTalk',
        description: 'Voice-to-text transcription with push-to-talk.',
        requiresApiKey: true,
        type: 'gui', // Has its own Tauri GUI config
    },
    {
        id: 'speak-selected',
        name: 'Speak Selected',
        description: 'Read selected text aloud using AI TTS.',
        requiresApiKey: true,
        type: 'cli',
        hotkeyArg: '--ptt-key',
        hasVoice: true,
    },
    {
        id: 'quick-assistant',
        name: 'Quick Assistant',
        description: 'Voice-activated AI assistant.',
        requiresApiKey: true,
        type: 'cli',
        hotkeyArg: '--ptt-key',
        hasVoice: true,
    },
    {
        id: 'flatten-string',
        name: 'Flatten String',
        description: 'Flatten clipboard text (remove newlines).',
        requiresApiKey: false,
        type: 'cli',
        hotkeyArg: '--trigger-key'
    },
    {
        id: 'typo-fix',
        name: 'Typo Fix',
        description: 'Fix typos in selected text using AI.',
        requiresApiKey: true,
        type: 'gui', // Has its own Tauri GUI config
    },
    {
        id: 'ocr-paste',
        name: 'OCR Paste',
        description: 'OCR from clipboard images.',
        requiresApiKey: false, // API key is optional (only for video extraction)
        type: 'cli',
        hotkeyArg: '--trigger-key'
    }
];

// Available AI voices for TTS tools
const VOICE_OPTIONS = [
    { value: '', label: '-- Default (Echo) --' },
    { value: 'alloy', label: 'Alloy' },
    { value: 'ash', label: 'Ash' },
    { value: 'coral', label: 'Coral' },
    { value: 'echo', label: 'Echo' },
    { value: 'fable', label: 'Fable' },
    { value: 'nova', label: 'Nova' },
    { value: 'onyx', label: 'Onyx' },
    { value: 'sage', label: 'Sage' },
    { value: 'shimmer', label: 'Shimmer' },
];

// Speech speed options for TTS tools
const SPEED_OPTIONS = [
    { value: 0.75, label: '0.75x (Slow)' },
    { value: 1.0, label: '1.0x (Normal)' },
    { value: 1.25, label: '1.25x' },
    { value: 1.5, label: '1.5x' },
    { value: 1.75, label: '1.75x' },
    { value: 2.0, label: '2.0x (Fast)' },
    { value: 2.5, label: '2.5x' },
    { value: 3.0, label: '3.0x (Very Fast)' },
];

// Default speech speeds per tool
const DEFAULT_SPEEDS = {
    'speak-selected': 1.5,
    'quick-assistant': 1.0,
};

// All available hotkeys - lowercase kebab-case for clap ValueEnum
const HOTKEY_OPTIONS = [
    { value: '', label: '-- Select Key --' },
    // Function keys
    { value: 'f1', label: 'F1' },
    { value: 'f2', label: 'F2' },
    { value: 'f3', label: 'F3' },
    { value: 'f4', label: 'F4' },
    { value: 'f5', label: 'F5' },
    { value: 'f6', label: 'F6' },
    { value: 'f7', label: 'F7' },
    { value: 'f8', label: 'F8' },
    { value: 'f9', label: 'F9' },
    { value: 'f10', label: 'F10' },
    { value: 'f11', label: 'F11' },
    { value: 'f12', label: 'F12' },
    { value: 'f13', label: 'F13' },
    { value: 'f14', label: 'F14' },
    { value: 'f15', label: 'F15' },
    { value: 'f16', label: 'F16' },
    { value: 'f17', label: 'F17' },
    { value: 'f18', label: 'F18' },
    { value: 'f19', label: 'F19' },
    { value: 'f20', label: 'F20' },
    { value: 'f21', label: 'F21' },
    { value: 'f22', label: 'F22' },
    { value: 'f23', label: 'F23' },
    { value: 'f24', label: 'F24' },
    // Navigation
    { value: 'insert', label: 'Insert' },
    { value: 'delete', label: 'Delete' },
    { value: 'home', label: 'Home' },
    { value: 'end', label: 'End' },
    { value: 'page-up', label: 'Page Up' },
    { value: 'page-down', label: 'Page Down' },
    { value: 'up-arrow', label: 'Up Arrow' },
    { value: 'down-arrow', label: 'Down Arrow' },
    { value: 'left-arrow', label: 'Left Arrow' },
    { value: 'right-arrow', label: 'Right Arrow' },
    // Modifiers
    { value: 'alt', label: 'Alt' },
    { value: 'alt-gr', label: 'AltGr' },
    { value: 'control-left', label: 'Left Ctrl' },
    { value: 'control-right', label: 'Right Ctrl' },
    { value: 'shift-left', label: 'Left Shift' },
    { value: 'shift-right', label: 'Right Shift' },
    { value: 'meta-left', label: 'Left Win' },
    { value: 'meta-right', label: 'Right Win' },
    // Special
    { value: 'escape', label: 'Escape' },
    { value: 'tab', label: 'Tab' },
    { value: 'caps-lock', label: 'Caps Lock' },
    { value: 'space', label: 'Space' },
    { value: 'backspace', label: 'Backspace' },
    { value: 'return', label: 'Enter' },
    { value: 'print-screen', label: 'Print Screen' },
    { value: 'scroll-lock', label: 'Scroll Lock' },
    { value: 'pause', label: 'Pause' },
    { value: 'num-lock', label: 'Num Lock' },
    // Number row
    { value: 'back-quote', label: '` (Backtick)' },
    { value: 'num1', label: '1' },
    { value: 'num2', label: '2' },
    { value: 'num3', label: '3' },
    { value: 'num4', label: '4' },
    { value: 'num5', label: '5' },
    { value: 'num6', label: '6' },
    { value: 'num7', label: '7' },
    { value: 'num8', label: '8' },
    { value: 'num9', label: '9' },
    { value: 'num0', label: '0' },
    { value: 'minus', label: '- (Minus)' },
    { value: 'equal', label: '= (Equal)' },
    // Letters
    { value: 'key-q', label: 'Q' },
    { value: 'key-w', label: 'W' },
    { value: 'key-e', label: 'E' },
    { value: 'key-r', label: 'R' },
    { value: 'key-t', label: 'T' },
    { value: 'key-y', label: 'Y' },
    { value: 'key-u', label: 'U' },
    { value: 'key-i', label: 'I' },
    { value: 'key-o', label: 'O' },
    { value: 'key-p', label: 'P' },
    { value: 'left-bracket', label: '[ (Left Bracket)' },
    { value: 'right-bracket', label: '] (Right Bracket)' },
    { value: 'key-a', label: 'A' },
    { value: 'key-s', label: 'S' },
    { value: 'key-d', label: 'D' },
    { value: 'key-f', label: 'F' },
    { value: 'key-g', label: 'G' },
    { value: 'key-h', label: 'H' },
    { value: 'key-j', label: 'J' },
    { value: 'key-k', label: 'K' },
    { value: 'key-l', label: 'L' },
    { value: 'semi-colon', label: '; (Semicolon)' },
    { value: 'quote', label: "' (Quote)" },
    { value: 'back-slash', label: '\\ (Backslash)' },
    { value: 'key-z', label: 'Z' },
    { value: 'key-x', label: 'X' },
    { value: 'key-c', label: 'C' },
    { value: 'key-v', label: 'V' },
    { value: 'key-b', label: 'B' },
    { value: 'key-n', label: 'N' },
    { value: 'key-m', label: 'M' },
    { value: 'comma', label: ', (Comma)' },
    { value: 'dot', label: '. (Period)' },
    { value: 'slash', label: '/ (Slash)' },
    // Numpad
    { value: 'kp0', label: 'Numpad 0' },
    { value: 'kp1', label: 'Numpad 1' },
    { value: 'kp2', label: 'Numpad 2' },
    { value: 'kp3', label: 'Numpad 3' },
    { value: 'kp4', label: 'Numpad 4' },
    { value: 'kp5', label: 'Numpad 5' },
    { value: 'kp6', label: 'Numpad 6' },
    { value: 'kp7', label: 'Numpad 7' },
    { value: 'kp8', label: 'Numpad 8' },
    { value: 'kp9', label: 'Numpad 9' },
    { value: 'kp-return', label: 'Numpad Enter' },
    { value: 'kp-plus', label: 'Numpad +' },
    { value: 'kp-minus', label: 'Numpad -' },
    { value: 'kp-multiply', label: 'Numpad *' },
    { value: 'kp-divide', label: 'Numpad /' },
    { value: 'kp-delete', label: 'Numpad Delete' },
];

// State
let toolStatuses = {};
let config = {};
let tauriReady = false;
let hasApiKey = false;
let authStatus = null; // License/trial status

function initTauri() {
    if (window.__TAURI_INTERNALS__) {
        invoke = window.__TAURI_INTERNALS__.invoke;
        tauriReady = true;
        return true;
    }
    if (window.__TAURI__?.core) {
        invoke = window.__TAURI__.core.invoke;
        tauriReady = true;
        return true;
    }
    return false;
}

document.addEventListener('DOMContentLoaded', async () => {
    setupTabs();
    
    // Show "Checking..." state for all tools on startup
    TOOLS.forEach(tool => {
        toolStatuses[tool.id] = 'Checking...';
    });
    renderTools();
    renderAutoStartTools();
    
    if (initTauri()) {
        try {
            await loadConfig();
            await loadAuthStatus();
            // Re-render with config but still checking
            renderTools();
            renderAutoStartTools();
            renderLicenseTab();
            
            // If not authorized, show the License tab by default
            if (authStatus && !['Licensed', 'Trial'].includes(authStatus.type)) {
                switchToTab('license');
            }
            
            // Initial scan for external processes (one-time, can be slow)
            // Use setTimeout to let UI render the "Checking..." state first
            await new Promise(resolve => setTimeout(resolve, 50));
            await invoke('scan_external_processes');
            await loadToolStatuses();
            renderTools();
            renderAutoStartTools();
        } catch (e) {
            console.error('Failed to load initial data:', e);
            // On error, show as stopped
            TOOLS.forEach(tool => {
                toolStatuses[tool.id] = 'Stopped';
            });
            renderTools();
        }
        
        setupEventListeners();
        setupLicenseEventListeners();
        // Fast polling - only checks processes we spawned
        setInterval(loadToolStatuses, 2000);
    }
});

function setupTabs() {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(button => {
        button.addEventListener('click', (e) => {
            e.preventDefault();
            switchToTab(button.dataset.tab);
        });
    });
}

window.switchToTab = function(tabId) {
    const tabButtons = document.querySelectorAll('.tab-button');
    const tabContents = document.querySelectorAll('.tab-content');
    
    tabButtons.forEach(btn => btn.classList.remove('active'));
    tabContents.forEach(content => content.classList.remove('active'));
    
    document.querySelector(`[data-tab="${tabId}"]`)?.classList.add('active');
    document.getElementById(tabId)?.classList.add('active');
}

async function loadConfig() {
    if (!tauriReady) return;
    try {
        config = await invoke('get_config');
        
        document.getElementById('autoStart').checked = config.auto_start || false;
        document.getElementById('startMinimized').checked = config.start_minimized || false;
        document.getElementById('darkMode').checked = config.dark_mode !== false;
        
        if (config.dark_mode === false) {
            document.body.classList.add('light-mode');
        }
        
        hasApiKey = await invoke('has_api_key');
        updateApiKeyUI();
    } catch (e) {
        console.error('Failed to load config:', e);
    }
}

async function updateApiKeyUI() {
    const notSetDiv = document.getElementById('apiKeyNotSet');
    const isSetDiv = document.getElementById('apiKeyIsSet');
    const maskedSpan = document.getElementById('apiKeyMasked');
    
    if (hasApiKey) {
        notSetDiv.style.display = 'none';
        isSetDiv.style.display = 'block';
        try {
            const masked = await invoke('get_api_key_masked');
            maskedSpan.textContent = masked || '••••••••••••••••';
            maskedSpan.dataset.revealed = 'false';
        } catch (e) {
            maskedSpan.textContent = '••••••••••••••••';
        }
    } else {
        notSetDiv.style.display = 'block';
        isSetDiv.style.display = 'none';
        document.getElementById('apiKey').value = '';
    }
}

async function loadToolStatuses() {
    if (!tauriReady) return;
    try {
        toolStatuses = await invoke('get_tool_statuses');
        updateToolCards();
    } catch (e) {
        console.error('Failed to load tool statuses:', e);
    }
}

function renderTools() {
    const grid = document.getElementById('toolsGrid');
    if (!grid) return;
    grid.innerHTML = '';
    
    // Show license warning if not authorized
    const isAuthorized = authStatus && ['Licensed', 'Trial'].includes(authStatus.type);
    if (!isAuthorized && authStatus) {
        const banner = document.createElement('div');
        banner.className = 'license-warning-banner';
        banner.innerHTML = `
            <div class="license-warning-content">
                <span class="license-warning-icon">🔒</span>
                <span class="license-warning-text">License required to start tools. </span>
                <button class="btn btn-primary btn-small" onclick="switchToTab('license')">Get License</button>
            </div>
        `;
        grid.appendChild(banner);
    }
    
    TOOLS.forEach(tool => {
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        const isPending = status === 'Starting...' || status === 'Stopping...' || status === 'Checking...';
        const isChecking = status === 'Checking...';
        const toolConfig = config.tools?.[tool.id] || {};
        const currentHotkey = toolConfig.hotkey || '';
        const needsHotkey = tool.type === 'cli';
        const hasHotkeySet = currentHotkey || tool.type === 'gui';
        const needsApiKeyButMissing = tool.requiresApiKey && !hasApiKey;
        const canStart = hasHotkeySet && !needsApiKeyButMissing;
        
        const card = document.createElement('div');
        card.className = `tool-card ${isRunning ? 'running' : ''} ${isPending ? 'pending' : ''} ${isChecking ? 'checking' : ''}`;
        card.id = `tool-${tool.id}`;
        
        // Build hotkey selector for CLI tools
        let hotkeyHtml = '';
        if (needsHotkey) {
            const options = HOTKEY_OPTIONS.map(opt => 
                `<option value="${opt.value}" ${currentHotkey === opt.value ? 'selected' : ''}>${opt.label}</option>`
            ).join('');
            hotkeyHtml = `
                <div class="tool-hotkey">
                    <select class="hotkey-select" id="hotkey-${tool.id}" ${isRunning || isPending ? 'disabled' : ''}>
                        ${options}
                    </select>
                </div>
            `;
        } else {
            hotkeyHtml = `<div class="tool-hotkey"><span class="gui-note">Uses own settings</span></div>`;
        }
        
        // Build voice selector for TTS tools
        let voiceHtml = '';
        if (tool.hasVoice) {
            const currentVoice = toolConfig.voice || '';
            const voiceOptions = VOICE_OPTIONS.map(opt => 
                `<option value="${opt.value}" ${currentVoice === opt.value ? 'selected' : ''}>${opt.label}</option>`
            ).join('');
            voiceHtml = `
                <div class="tool-voice">
                    <label class="voice-label">AI Voice:</label>
                    <select class="voice-select" id="voice-${tool.id}" ${isRunning || isPending ? 'disabled' : ''}>
                        ${voiceOptions}
                    </select>
                </div>
            `;
        }
        
        // Build speech speed selector for TTS tools
        let speedHtml = '';
        if (tool.hasVoice) {
            const defaultSpeed = DEFAULT_SPEEDS[tool.id] || 1.0;
            const currentSpeed = toolConfig.speech_speed ?? defaultSpeed;
            const speedOptions = SPEED_OPTIONS.map(opt => 
                `<option value="${opt.value}" ${currentSpeed === opt.value ? 'selected' : ''}>${opt.label}</option>`
            ).join('');
            speedHtml = `
                <div class="tool-voice">
                    <label class="voice-label">Speed:</label>
                    <select class="speed-select" id="speed-${tool.id}" ${isRunning || isPending ? 'disabled' : ''}>
                        ${speedOptions}
                    </select>
                </div>
            `;
        }
        
        // Determine what's blocking the start
        let blockReason = '';
        if (!isRunning && !isPending) {
            if (needsApiKeyButMissing) {
                blockReason = 'Set API key in Settings first';
            } else if (!hasHotkeySet) {
                blockReason = 'Select a hotkey first';
            }
        }
        
        // Determine status class
        const statusClass = isChecking ? 'checking' : (isPending ? 'pending' : (isRunning ? 'running' : 'stopped'));
        
        // Build button HTML based on state
        let buttonHtml = '';
        const isGuiTool = tool.type === 'gui';
        // Only show Settings button when GUI tool is running
        const settingsBtn = (isGuiTool && isRunning) ? `<button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')" title="Open ${tool.name} settings">⚙️ Settings</button>` : '';
        
        if (isChecking) {
            buttonHtml = `<button class="btn btn-checking" disabled><span class="spinner"></span> Checking...</button>`;
        } else if (isPending) {
            const btnClass = status === 'Starting...' ? 'btn-pending-start' : 'btn-pending-stop';
            buttonHtml = `<button class="btn ${btnClass}" disabled><span class="spinner"></span> ${status}</button>`;
        } else {
            buttonHtml = `
                <button class="btn ${isRunning ? 'btn-danger' : 'btn-success'}" 
                        onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}"
                        ${!isRunning && !canStart ? `disabled title="${blockReason}"` : ''}>
                    ${isRunning ? 'Stop' : 'Start'}
                </button>
                ${settingsBtn}
            `;
        }
        
        card.innerHTML = `
            <div class="tool-header">
                <span class="tool-name">${tool.name}</span>
                <span class="tool-status ${statusClass}">
                    <span class="status-dot ${(isPending || isChecking) ? 'spinning' : ''}"></span>
                    ${status}
                </span>
            </div>
            <p class="tool-description">${tool.description}</p>
            ${tool.requiresApiKey ? `<p class="hint ${needsApiKeyButMissing ? 'warning' : ''}">Requires API key${needsApiKeyButMissing ? ' ⚠️' : ' ✓'}</p>` : ''}
            ${hotkeyHtml}
            ${voiceHtml}
            ${speedHtml}
            <div class="tool-actions">
                ${buttonHtml}
            </div>
            ${blockReason ? `<p class="block-reason">${blockReason}</p>` : ''}
        `;
        
        grid.appendChild(card);
    });
    
    // Add change listeners to hotkey selects
    TOOLS.filter(t => t.type === 'cli').forEach(tool => {
        const select = document.getElementById(`hotkey-${tool.id}`);
        if (select) {
            select.addEventListener('change', () => saveHotkeyForTool(tool.id, select.value));
        }
    });
    
    // Add change listeners to voice selects
    TOOLS.filter(t => t.hasVoice).forEach(tool => {
        const select = document.getElementById(`voice-${tool.id}`);
        if (select) {
            select.addEventListener('change', () => saveVoiceForTool(tool.id, select.value));
        }
    });
    
    // Add change listeners to speed selects
    TOOLS.filter(t => t.hasVoice).forEach(tool => {
        const select = document.getElementById(`speed-${tool.id}`);
        if (select) {
            select.addEventListener('change', () => saveSpeedForTool(tool.id, parseFloat(select.value)));
        }
    });
}

function updateToolCards() {
    TOOLS.forEach(tool => {
        const card = document.getElementById(`tool-${tool.id}`);
        if (!card) return;
        
        const status = toolStatuses[tool.id] || 'Stopped';
        const isRunning = status === 'Running';
        const isPending = status === 'Starting...' || status === 'Stopping...' || status === 'Checking...';
        const isChecking = status === 'Checking...';
        const toolConfig = config.tools?.[tool.id] || {};
        const hasHotkeySet = toolConfig.hotkey || tool.type === 'gui';
        const needsApiKeyButMissing = tool.requiresApiKey && !hasApiKey;
        const canStart = hasHotkeySet && !needsApiKeyButMissing;
        
        card.className = `tool-card ${isRunning ? 'running' : ''} ${isPending ? 'pending' : ''} ${isChecking ? 'checking' : ''}`;
        
        const statusEl = card.querySelector('.tool-status');
        if (statusEl) {
            const statusClass = isChecking ? 'checking' : (isPending ? 'pending' : (isRunning ? 'running' : 'stopped'));
            statusEl.className = `tool-status ${statusClass}`;
            statusEl.innerHTML = `<span class="status-dot ${(isPending || isChecking) ? 'spinning' : ''}"></span>${status}`;
        }
        
        // Disable hotkey select when running or pending
        const select = card.querySelector('.hotkey-select');
        if (select) select.disabled = isRunning || isPending;
        
        // Determine what's blocking the start
        let blockReason = '';
        if (!isRunning && !isPending) {
            if (needsApiKeyButMissing) {
                blockReason = 'Set API key in Settings first';
            } else if (!hasHotkeySet) {
                blockReason = 'Select a hotkey first';
            }
        }
        
        const actionsEl = card.querySelector('.tool-actions');
        if (actionsEl) {
            const isGuiTool = tool.type === 'gui';
            // Only show Settings button when GUI tool is running
            const settingsBtn = (isGuiTool && isRunning) ? `<button class="btn btn-secondary" onclick="openToolSettings('${tool.id}')" title="Open ${tool.name} settings">⚙️ Settings</button>` : '';
            
            if (isChecking) {
                // Show checking button with spinner
                actionsEl.innerHTML = `
                    <button class="btn btn-checking" disabled>
                        <span class="spinner"></span> Checking...
                    </button>
                `;
            } else if (isPending) {
                // Show pending button with spinner
                const btnClass = status === 'Starting...' ? 'btn-pending-start' : 'btn-pending-stop';
                actionsEl.innerHTML = `
                    <button class="btn ${btnClass}" disabled>
                        <span class="spinner"></span> ${status}
                    </button>
                `;
            } else {
                actionsEl.innerHTML = `
                    <button class="btn ${isRunning ? 'btn-danger' : 'btn-success'}" 
                            onclick="${isRunning ? `stopTool('${tool.id}')` : `startTool('${tool.id}')`}"
                            ${!isRunning && !canStart ? `disabled title="${blockReason}"` : ''}>
                        ${isRunning ? 'Stop' : 'Start'}
                    </button>
                    ${settingsBtn}
                `;
            }
        }
        
        // Disable voice and speed selects when running or pending
        const voiceSelect = card.querySelector('.voice-select');
        if (voiceSelect) voiceSelect.disabled = isRunning || isPending;
        const speedSelect = card.querySelector('.speed-select');
        if (speedSelect) speedSelect.disabled = isRunning || isPending;
        
        // Update block reason
        let blockReasonEl = card.querySelector('.block-reason');
        if (blockReason) {
            if (!blockReasonEl) {
                blockReasonEl = document.createElement('p');
                blockReasonEl.className = 'block-reason';
                card.appendChild(blockReasonEl);
            }
            blockReasonEl.textContent = blockReason;
        } else if (blockReasonEl) {
            blockReasonEl.remove();
        }
    });
}

async function saveHotkeyForTool(toolId, hotkey) {
    if (!tauriReady) return;
    
    try {
        // Build updated config
        const newConfig = {
            auto_start: config.auto_start || false,
            start_minimized: config.start_minimized || false,
            dark_mode: config.dark_mode !== false,
            tools: {}
        };
        
        TOOLS.forEach(tool => {
            const existingConfig = config.tools?.[tool.id] || {};
            newConfig.tools[tool.id] = {
                enabled: true,
                auto_start: existingConfig.auto_start || false,
                hotkey: tool.id === toolId ? (hotkey || null) : (existingConfig.hotkey || null),
                special_hotkey: existingConfig.special_hotkey || null,
                voice: existingConfig.voice || null,
                speech_speed: existingConfig.speech_speed || null
            };
        });
        
        await invoke('save_config', { config: newConfig });
        config = newConfig;
        
        // Update button state
        updateToolCards();
        console.log(`Saved hotkey ${hotkey} for ${toolId}`);
    } catch (e) {
        console.error('Failed to save hotkey:', e);
        alert(`Failed to save hotkey: ${e}`);
    }
}

async function saveVoiceForTool(toolId, voice) {
    if (!tauriReady) return;
    
    try {
        // Build updated config
        const newConfig = {
            auto_start: config.auto_start || false,
            start_minimized: config.start_minimized || false,
            dark_mode: config.dark_mode !== false,
            tools: {}
        };
        
        TOOLS.forEach(tool => {
            const existingConfig = config.tools?.[tool.id] || {};
            newConfig.tools[tool.id] = {
                enabled: true,
                auto_start: existingConfig.auto_start || false,
                hotkey: existingConfig.hotkey || null,
                special_hotkey: existingConfig.special_hotkey || null,
                voice: tool.id === toolId ? (voice || null) : (existingConfig.voice || null),
                speech_speed: existingConfig.speech_speed || null
            };
        });
        
        await invoke('save_config', { config: newConfig });
        config = newConfig;
        
        console.log(`Saved voice ${voice} for ${toolId}`);
    } catch (e) {
        console.error('Failed to save voice:', e);
        alert(`Failed to save voice: ${e}`);
    }
}

async function saveSpeedForTool(toolId, speed) {
    if (!tauriReady) return;
    
    try {
        // Build updated config
        const newConfig = {
            auto_start: config.auto_start || false,
            start_minimized: config.start_minimized || false,
            dark_mode: config.dark_mode !== false,
            tools: {}
        };
        
        TOOLS.forEach(tool => {
            const existingConfig = config.tools?.[tool.id] || {};
            newConfig.tools[tool.id] = {
                enabled: true,
                auto_start: existingConfig.auto_start || false,
                hotkey: existingConfig.hotkey || null,
                special_hotkey: existingConfig.special_hotkey || null,
                voice: existingConfig.voice || null,
                speech_speed: tool.id === toolId ? speed : (existingConfig.speech_speed || null)
            };
        });
        
        await invoke('save_config', { config: newConfig });
        config = newConfig;
        
        console.log(`Saved speed ${speed} for ${toolId}`);
    } catch (e) {
        console.error('Failed to save speed:', e);
        alert(`Failed to save speed: ${e}`);
    }
}

function renderAutoStartTools() {
    const container = document.getElementById('autoStartTools');
    if (!container) return;
    container.innerHTML = '';
    
    TOOLS.forEach(tool => {
        const toolConfig = config.tools?.[tool.id] || {};
        const label = document.createElement('label');
        label.className = 'checkbox';
        label.innerHTML = `
            <input type="checkbox" id="autoStart-${tool.id}" ${toolConfig.auto_start ? 'checked' : ''}>
            <span>${tool.name}</span>
        `;
        container.appendChild(label);
    });
}

function setupEventListeners() {
    document.getElementById('toggleApiKey')?.addEventListener('click', async () => {
        const maskedSpan = document.getElementById('apiKeyMasked');
        const toggleBtn = document.getElementById('toggleApiKey');
        const isRevealed = maskedSpan.dataset.revealed === 'true';
        
        if (isRevealed) {
            // Hide it
            try {
                const masked = await invoke('get_api_key_masked');
                maskedSpan.textContent = masked || '••••••••••••••••';
            } catch (e) {
                maskedSpan.textContent = '••••••••••••••••';
            }
            maskedSpan.dataset.revealed = 'false';
            toggleBtn.textContent = '👁️';
            toggleBtn.title = 'Show';
        } else {
            // Reveal it
            try {
                const fullKey = await invoke('get_api_key');
                maskedSpan.textContent = fullKey;
                maskedSpan.dataset.revealed = 'true';
                toggleBtn.textContent = '🙈';
                toggleBtn.title = 'Hide';
            } catch (e) {
                console.error('Failed to get API key:', e);
            }
        }
    });
    
    document.getElementById('saveApiKeyBtn')?.addEventListener('click', async () => {
        const apiKey = document.getElementById('apiKey')?.value;
        const status = document.getElementById('apiKeyStatus');
        if (!apiKey) {
            status.textContent = 'Please enter an API key';
            status.className = 'status error';
            return;
        }
        try {
            await invoke('save_api_key', { apiKey });
            status.textContent = 'API key saved!';
            status.className = 'status success';
            // Update state and re-render
            hasApiKey = true;
            await updateApiKeyUI();
            renderTools();
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
    
    document.getElementById('deleteApiKeyBtn')?.addEventListener('click', async () => {
        const status = document.getElementById('apiKeyStatus');
        if (!confirm('Are you sure you want to delete the API key?')) {
            return;
        }
        try {
            await invoke('delete_api_key');
            status.textContent = 'API key deleted';
            status.className = 'status success';
            // Update state and re-render
            hasApiKey = false;
            await updateApiKeyUI();
            renderTools();
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
    
    document.getElementById('darkMode')?.addEventListener('change', (e) => {
        document.body.classList.toggle('light-mode', !e.target.checked);
    });
    
    document.getElementById('saveSettingsBtn')?.addEventListener('click', async () => {
        const status = document.getElementById('settingsStatus');
        try {
            const newConfig = {
                auto_start: document.getElementById('autoStart')?.checked || false,
                start_minimized: document.getElementById('startMinimized')?.checked || false,
                dark_mode: document.getElementById('darkMode')?.checked !== false,
                tools: {}
            };
            
            TOOLS.forEach(tool => {
                const existingConfig = config.tools?.[tool.id] || {};
                newConfig.tools[tool.id] = {
                    enabled: true,
                    auto_start: document.getElementById(`autoStart-${tool.id}`)?.checked || false,
                    hotkey: existingConfig.hotkey || null,
                    special_hotkey: existingConfig.special_hotkey || null,
                    voice: existingConfig.voice || null,
                    speech_speed: existingConfig.speech_speed || null
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
    
    // OpenAI API key help links
    document.getElementById('openaiApiKeysLink')?.addEventListener('click', async (e) => {
        e.preventDefault();
        try {
            await invoke('open_checkout', { plan: 'openai-keys' });
        } catch (error) {
            window.open('https://platform.openai.com/api-keys', '_blank');
        }
    });
    
    document.getElementById('apiKeyVideoLink')?.addEventListener('click', async (e) => {
        e.preventDefault();
        try {
            await invoke('open_checkout', { plan: 'video-tutorial' });
        } catch (error) {
            window.open('https://youtu.be/SzPE_AE0eEo?si=WbJP-ABj0uG5s-XV', '_blank');
        }
    });
    
    document.getElementById('viewUsageBtn')?.addEventListener('click', async () => {
        try {
            await invoke('open_checkout', { plan: 'openai-usage' });
        } catch (error) {
            window.open('https://platform.openai.com/usage', '_blank');
        }
    });
}

window.startTool = async function(toolId) {
    if (!tauriReady) return;
    
    // Show pending state immediately
    toolStatuses[toolId] = 'Starting...';
    updateToolCards();
    
    // Use setTimeout to let the UI update before the blocking call
    setTimeout(async () => {
        try {
            await invoke('start_tool', { toolId });
            toolStatuses[toolId] = 'Running';
            updateToolCards();
        } catch (e) {
            toolStatuses[toolId] = 'Stopped';
            updateToolCards();
            alert(`Failed to start tool: ${e}`);
        }
    }, 10);
};

window.stopTool = async function(toolId) {
    if (!tauriReady) return;
    
    // Show pending state immediately
    toolStatuses[toolId] = 'Stopping...';
    updateToolCards();
    
    // Use setTimeout to let the UI update before the blocking call
    setTimeout(async () => {
        try {
            await invoke('stop_tool', { toolId });
            toolStatuses[toolId] = 'Stopped';
            updateToolCards();
        } catch (e) {
            toolStatuses[toolId] = 'Running';
            updateToolCards();
            alert(`Failed to stop tool: ${e}`);
        }
    }, 10);
};

window.openToolSettings = async function(toolId) {
    if (!tauriReady) return;
    
    try {
        await invoke('open_tool_settings', { toolId });
    } catch (e) {
        alert(`Failed to open settings: ${e}`);
    }
};

// === License Functions ===

async function loadAuthStatus() {
    if (!tauriReady) return;
    try {
        authStatus = await invoke('get_auth_status');
        console.log('Auth status:', authStatus);
    } catch (e) {
        console.error('Failed to load auth status:', e);
        authStatus = { type: 'NoLicense' };
    }
}

function renderLicenseTab() {
    const statusSection = document.getElementById('licenseStatusSection');
    const statusContent = document.getElementById('licenseStatusContent');
    const trialSection = document.getElementById('trialSection');
    const activateSection = document.getElementById('activateLicenseSection');
    const buySection = document.getElementById('buySection');
    const deactivateSection = document.getElementById('deactivateSection');
    
    if (!authStatus) {
        statusContent.innerHTML = '<p>Loading...</p>';
        return;
    }
    
    // Reset classes
    statusSection.classList.remove('licensed', 'trial', 'expired');
    
    switch (authStatus.type) {
        case 'Licensed':
            statusSection.classList.add('licensed');
            statusContent.innerHTML = `
                <div class="license-status-icon">✓</div>
                <div class="license-status-title">Licensed - ${authStatus.plan}</div>
                <div class="license-status-subtitle">All features unlocked</div>
                <div class="license-key-display">Key: ${authStatus.key_preview}</div>
            `;
            trialSection.style.display = 'none';
            activateSection.style.display = 'none';
            buySection.style.display = 'none';
            deactivateSection.style.display = 'block';
            break;
            
        case 'Trial':
            statusSection.classList.add('trial');
            statusContent.innerHTML = `
                <div class="license-status-icon">⏱️</div>
                <div class="license-status-title">Free Trial</div>
                <div class="trial-countdown">${authStatus.days_remaining}d ${authStatus.hours_remaining}h remaining</div>
                <div class="license-status-subtitle">All features unlocked during trial</div>
            `;
            trialSection.style.display = 'none';
            activateSection.style.display = 'block';
            buySection.style.display = 'block';
            deactivateSection.style.display = 'none';
            break;
            
        case 'TrialExpired':
            statusSection.classList.add('expired');
            statusContent.innerHTML = `
                <div class="license-status-icon">⚠️</div>
                <div class="license-status-title">Trial Expired</div>
                <div class="license-status-subtitle">Purchase a license to continue using all features</div>
            `;
            trialSection.style.display = 'none';
            activateSection.style.display = 'block';
            buySection.style.display = 'block';
            deactivateSection.style.display = 'none';
            break;
            
        case 'NoLicense':
        default:
            statusContent.innerHTML = `
                <div class="license-status-icon">🔒</div>
                <div class="license-status-title">No License</div>
                <div class="license-status-subtitle">Start a free trial or purchase a license</div>
            `;
            trialSection.style.display = 'block';
            activateSection.style.display = 'block';
            buySection.style.display = 'block';
            deactivateSection.style.display = 'none';
            break;
    }
}

function setupLicenseEventListeners() {
    // Start trial button
    document.getElementById('startTrialBtn')?.addEventListener('click', async () => {
        try {
            await invoke('start_trial');
            await loadAuthStatus();
            renderLicenseTab();
        } catch (e) {
            alert(`Failed to start trial: ${e}`);
        }
    });
    
    // Activate license button
    document.getElementById('activateLicenseBtn')?.addEventListener('click', async () => {
        const keyInput = document.getElementById('licenseKeyInput');
        const status = document.getElementById('licenseActivateStatus');
        const key = keyInput.value.trim();
        
        if (!key) {
            status.textContent = 'Please enter a license key';
            status.className = 'status error';
            return;
        }
        
        status.textContent = 'Activating...';
        status.className = 'status';
        
        try {
            const result = await invoke('activate_license', { licenseKey: key });
            if (result.success) {
                status.textContent = 'License activated!';
                status.className = 'status success';
                keyInput.value = '';
                await loadAuthStatus();
                renderLicenseTab();
            } else {
                status.textContent = result.error || 'Activation failed';
                status.className = 'status error';
            }
        } catch (e) {
            status.textContent = `Error: ${e}`;
            status.className = 'status error';
        }
    });
    
    // Deactivate license button
    document.getElementById('deactivateLicenseBtn')?.addEventListener('click', async () => {
        if (!confirm('Are you sure you want to deactivate your license on this machine?')) {
            return;
        }
        
        try {
            await invoke('deactivate_license');
            await loadAuthStatus();
            renderLicenseTab();
        } catch (e) {
            alert(`Failed to deactivate: ${e}`);
        }
    });
}

// Open checkout URL
window.openCheckout = async function(plan) {
    if (!tauriReady) return;
    
    try {
        // Use our Rust command to open the URL in the default browser
        await invoke('open_checkout', { plan });
    } catch (e) {
        console.error('Failed to open checkout:', e);
        alert('Failed to open checkout page. Please visit: slking.lemonsqueezy.com');
    }
};
