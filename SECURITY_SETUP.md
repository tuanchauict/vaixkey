# VaixKey macOS Security Setup Guide

## 🔐 Required Permissions for Full VaixKey Functionality

VaixKey needs specific macOS permissions to capture keyboard input and inject Vietnamese text. Here's everything you need to know about setting up and verifying these permissions.

## 📋 **Required Permissions**

### 1. **Input Monitoring** (Essential)
- **Purpose**: Allows VaixKey to capture keyboard events system-wide
- **Without this**: VaixKey can't detect when you type Telex sequences
- **System Preference**: Privacy & Security → Input Monitoring

### 2. **Accessibility** (Essential)
- **Purpose**: Allows VaixKey to inject processed Vietnamese text into applications
- **Without this**: VaixKey can't replace typed text with Vietnamese characters
- **System Preference**: Privacy & Security → Accessibility

## 🔧 **Step-by-Step Setup Instructions**

### **Step 1: Open System Preferences**
1. Click the Apple menu (🍎) in the top-left corner
2. Select "System Preferences" (macOS Big Sur and earlier) or "System Settings" (macOS Ventura and later)

### **Step 2: Navigate to Privacy & Security**
- **macOS Big Sur and earlier**: Security & Privacy → Privacy tab
- **macOS Ventura and later**: Privacy & Security (in sidebar)

### **Step 3: Grant Input Monitoring Permission**
1. In the left sidebar, click **"Input Monitoring"**
2. Click the lock icon (🔒) in the bottom-left corner
3. Enter your administrator password
4. Look for **"vaixkey"** or **"Terminal"** (if running via `cargo run`)
5. **If VaixKey is listed**: Check the checkbox next to it
6. **If VaixKey is NOT listed**:
   - Run VaixKey once: `cargo run`
   - It should automatically appear in the list
   - Check the checkbox next to it

### **Step 4: Grant Accessibility Permission**
1. In the left sidebar, click **"Accessibility"**
2. Click the lock icon (🔒) if it's locked
3. Enter your administrator password
4. Look for **"vaixkey"** or **"Terminal"**
5. **If VaixKey is listed**: Check the checkbox next to it
6. **If VaixKey is NOT listed**:
   - Run VaixKey once: `cargo run`
   - It should automatically appear in the list
   - Check the checkbox next to it

## ✅ **How to Verify Permissions Are Working**

### **Method 1: Check System Preferences**
```bash
# Run this command to see what permissions are granted
cargo run -- --check-permissions
```

### **Method 2: Manual Verification**
1. **Open System Preferences → Privacy & Security**
2. **Check Input Monitoring**: VaixKey should be listed and ✅ checked
3. **Check Accessibility**: VaixKey should be listed and ✅ checked

### **Method 3: Test with VaixKey Debug Mode**
```bash
# This will show if permissions are working
cargo run -- --debug-permissions
```

**Expected output if permissions are working:**
```
🔐 VaixKey Permission Status
============================
✅ Input Monitoring: Granted
✅ Accessibility: Granted
🎯 VaixKey is ready for keyboard capture!
```

**Expected output if permissions are missing:**
```
🔐 VaixKey Permission Status
============================
❌ Input Monitoring: DENIED
❌ Accessibility: DENIED
⚠️  VaixKey cannot function without these permissions
```

## 🧪 **Testing Keyboard Capture**

Once permissions are granted, test that VaixKey can capture keystrokes:

```bash
# Start VaixKey in debug mode
cargo run -- --debug
```

**You should see:**
```
🔍 VaixKey Debug Mode
====================
📝 Comprehensive keystroke capture and processing logging

⌨️  Start typing to see real-time keystroke capture...
   (Press Ctrl+C to exit)

🔴 KEY PRESS: KeyA at Instant { ... }
📊 ENGINE STATE:
   Vietnamese Mode: true
   Current Buffer: 'a'
🔤 Processing character: 'a'
✨ Processing result: 'a' → 'a'
```

## 🚨 **Common Issues and Solutions**

### **Issue 1: "VaixKey doesn't appear in the permission list"**
**Solution:**
1. Run VaixKey at least once: `cargo run`
2. macOS will prompt for permission or add it to the list
3. Grant the permission when prompted
4. Restart VaixKey

### **Issue 2: "Permission granted but keystroke capture not working"**
**Solution:**
1. Restart VaixKey completely
2. Check that you granted permission to the correct application:
   - If running via `cargo run`, grant permission to "Terminal"
   - If running as compiled binary, grant permission to "vaixkey"
3. Try logging out and back in (sometimes required for permissions to take effect)

### **Issue 3: "macOS keeps asking for permission repeatedly"**
**Solution:**
1. Remove VaixKey from both permission lists
2. Restart your Mac
3. Re-add permissions following the setup steps above

### **Issue 4: "VaixKey works in some apps but not others"**
**This is expected behavior:**
- Some applications (like password fields) block input monitoring for security
- System applications may have special restrictions
- This is normal macOS security behavior

## 🔍 **Permission Verification Commands**

VaixKey includes built-in commands to check permission status:

### **Quick Permission Check**
```bash
cargo run -- --permissions
```

### **Detailed Security Status**
```bash
cargo run -- --security-status
```

### **Permission Setup Helper**
```bash
cargo run -- --setup-permissions
```
*This command guides you through the permission setup process*

## ⚠️ **Security Notes**

### **Why These Permissions Are Needed**
- **Input Monitoring**: VaixKey needs to see what you type to detect Telex sequences like "aa" → "â"
- **Accessibility**: VaixKey needs to replace the text you typed with Vietnamese characters

### **Privacy Considerations**
- VaixKey only processes Vietnamese input sequences
- No data is sent over the network
- All processing happens locally on your Mac
- VaixKey respects secure input fields (passwords, etc.)

### **Revoking Permissions**
To stop VaixKey from monitoring input:
1. Open System Preferences → Privacy & Security
2. Go to Input Monitoring and Accessibility
3. Uncheck VaixKey in both lists
4. VaixKey will stop working until permissions are re-granted

## 📱 **macOS Version-Specific Instructions**

### **macOS Ventura (13.0+) and Sonoma (14.0+)**
- Settings app replaced System Preferences
- Navigate: Apple Menu → System Settings → Privacy & Security
- Same permission categories but different interface

### **macOS Big Sur (11.0+) and Monterey (12.0+)**
- Use System Preferences
- Navigate: Apple Menu → System Preferences → Security & Privacy → Privacy tab

### **macOS Catalina (10.15+) and earlier**
- Use System Preferences
- Navigate: Apple Menu → System Preferences → Security & Privacy → Privacy tab
- May have slightly different permission names

## 🎯 **Verification Checklist**

Before reporting that VaixKey "isn't working," verify:

- [ ] **Input Monitoring permission granted** ✅
- [ ] **Accessibility permission granted** ✅
- [ ] **VaixKey restarted after granting permissions**
- [ ] **Tested in a supported application** (TextEdit, Notes, etc.)
- [ ] **Vietnamese mode is enabled** (`cargo run -- --status`)
- [ ] **No conflicting input methods running**

## 🆘 **Getting Help**

If VaixKey still doesn't work after following this guide:

1. **Check the logs**: `RUST_LOG=debug cargo run -- --debug`
2. **Verify status**: `cargo run -- --status`
3. **Test the engine**: `cargo run -- --test`
4. **Create an issue** with your system info and error messages

## 🔧 **Developer Notes**

For developers working on VaixKey:

### **Testing Without Permissions**
```bash
# Run in simulation mode (no real keyboard capture)
cargo run -- --simulate
```

### **Permission API Usage**
VaixKey uses these macOS APIs for permission checking:
- `AXIsProcessTrustedWithOptions()` for Accessibility
- `CGPreflightScreenCaptureAccess()` pattern for Input Monitoring

### **Building for Distribution**
When building VaixKey for distribution:
1. Code signing is required for permissions to work properly
2. Notarization may be needed for Gatekeeper acceptance
3. Consider creating an installer that guides users through permission setup

---

**📝 Remember**: macOS security permissions are designed to protect users. VaixKey requests only the minimum permissions needed for Vietnamese input functionality, and all processing happens locally on your device.