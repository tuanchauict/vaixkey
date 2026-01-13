// GUI module for VaixKey settings and status display
// Native macOS GUI implementation

use crate::config::Config;
use log::info;
use std::process::Command;

pub struct GuiManager {
    // GUI state management
}

impl GuiManager {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn show_settings(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Opening settings interface");

        // Create a simple HTML settings page and open it in the default browser
        self.create_settings_html().await?;

        // Open the settings page
        Command::new("open")
            .arg("file:///tmp/vaixkey_settings.html")
            .spawn()?;

        Ok(())
    }

    pub async fn show_status_indicator(&self, is_vietnamese: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mode_text = if is_vietnamese { "Vietnamese" } else { "English" };
        info!("Status: {} mode active", mode_text);

        // For now, just show a notification
        self.show_notification(&format!("VaixKey: {} Mode", mode_text)).await?;

        Ok(())
    }

    pub async fn update_config(&self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        info!("Updating configuration through GUI");
        config.save()?;
        Ok(())
    }

    async fn create_settings_html(&self) -> Result<(), Box<dyn std::error::Error>> {
        let html_content = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>VaixKey Settings</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 600px;
            margin: 40px auto;
            padding: 20px;
            background: #f5f5f7;
            color: #1d1d1f;
        }
        .container {
            background: white;
            border-radius: 12px;
            padding: 30px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
        }
        h1 {
            color: #1d1d1f;
            margin-bottom: 30px;
            text-align: center;
        }
        .section {
            margin-bottom: 30px;
            padding: 20px;
            background: #f8f9fa;
            border-radius: 8px;
        }
        .section h2 {
            margin-top: 0;
            color: #333;
        }
        .form-group {
            margin-bottom: 20px;
        }
        label {
            display: block;
            margin-bottom: 8px;
            font-weight: 500;
        }
        select, input {
            width: 100%;
            padding: 12px;
            border: 1px solid #ddd;
            border-radius: 6px;
            font-size: 14px;
        }
        .checkbox-group {
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .checkbox-group input {
            width: auto;
        }
        button {
            background: #007aff;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            font-size: 14px;
            cursor: pointer;
            margin-right: 10px;
        }
        button:hover {
            background: #0056b3;
        }
        .status {
            padding: 15px;
            background: #e8f5e8;
            border: 1px solid #4caf50;
            border-radius: 6px;
            margin-bottom: 20px;
            text-align: center;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🇻🇳 VaixKey Settings</h1>

        <div class="status">
            <strong>Status:</strong> VaixKey is running in Vietnamese mode
        </div>

        <div class="section">
            <h2>Input Method</h2>
            <div class="form-group">
                <label for="input-method">Select Input Method:</label>
                <select id="input-method">
                    <option value="telex" selected>Telex (aa → â, aw → ă)</option>
                    <option value="vni">VNI (a8 → â, a6 → ă)</option>
                    <option value="simple-telex">Simple Telex</option>
                </select>
            </div>
        </div>

        <div class="section">
            <h2>Hotkeys</h2>
            <div class="form-group">
                <label for="toggle-key">Toggle Vietnamese Mode:</label>
                <input type="text" id="toggle-key" value="Ctrl+Shift" readonly>
            </div>
            <div class="form-group">
                <label for="switch-key">Switch Input Method:</label>
                <input type="text" id="switch-key" value="Ctrl+Alt+V" readonly>
            </div>
        </div>

        <div class="section">
            <h2>Preferences</h2>
            <div class="form-group">
                <div class="checkbox-group">
                    <input type="checkbox" id="auto-start">
                    <label for="auto-start">Start VaixKey automatically at login</label>
                </div>
            </div>
            <div class="form-group">
                <div class="checkbox-group">
                    <input type="checkbox" id="show-notifications" checked>
                    <label for="show-notifications">Show status notifications</label>
                </div>
            </div>
        </div>

        <div style="text-align: center;">
            <button onclick="saveSettings()">Save Settings</button>
            <button onclick="resetSettings()" style="background: #6c757d;">Reset to Defaults</button>
        </div>
    </div>

    <script>
        function saveSettings() {
            alert('Settings saved! (This is a demo - settings will be persistent in the full implementation)');
        }

        function resetSettings() {
            if (confirm('Reset all settings to defaults?')) {
                document.getElementById('input-method').value = 'telex';
                document.getElementById('auto-start').checked = false;
                document.getElementById('show-notifications').checked = true;
                alert('Settings reset to defaults!');
            }
        }

        // Auto-save on changes
        document.addEventListener('change', function() {
            console.log('Setting changed - would auto-save in full implementation');
        });
    </script>
</body>
</html>
        "#;

        std::fs::write("/tmp/vaixkey_settings.html", html_content)?;
        Ok(())
    }

    async fn show_notification(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Use macOS native notifications
        Command::new("osascript")
            .arg("-e")
            .arg(&format!(
                "display notification \"{}\" with title \"VaixKey\"",
                message
            ))
            .spawn()?;

        Ok(())
    }
}