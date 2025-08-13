# Polish Language - AI Text Enhancement Tool

A macOS system tray application that uses AI to polish and improve selected text anywhere on your system.

## Features

- **Global Text Enhancement**: Select any text in any application and press a shortcut to improve it with AI
- **System Tray Integration**: Runs quietly in the background with easy access via system tray (no dock icon)
- **Smart Text Replacement**: Automatically replaces selected text or copies to clipboard
- **Configurable AI Models**: Support for OpenAI GPT models and compatible APIs
- **Customizable Shortcuts**: Set your preferred global keyboard shortcut
- **Persistent Settings**: All configurations are saved locally

## How to Use

1. **Install**: Run the built app from `src-tauri/target/release/bundle/macos/polish-language.app`
2. **Configure**: Click the system tray icon → Settings to configure:
   - Your OpenAI API key
   - Preferred AI model (GPT-3.5, GPT-4, etc.)
   - Global keyboard shortcut (default: Cmd+Shift+P)
   - Custom system prompt for the AI
3. **Use**: 
   - Select any text in any application
   - Press your configured shortcut key
   - The AI-enhanced text will be copied to your clipboard
   - Paste it wherever you need it

## Development

### Prerequisites
- Node.js and npm
- Rust and Cargo
- Tauri CLI

### Build Commands
```bash
# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

### Configuration

The app stores settings in your system's config directory:
- macOS: `~/Library/Application Support/polish-language/settings.json`

## API Support

Currently supports:
- OpenAI API (GPT-3.5, GPT-4, GPT-4 Turbo, GPT-4o)
- Any OpenAI-compatible API endpoint

## Privacy

- All settings are stored locally on your device
- Your API key never leaves your machine except to make requests to your configured AI service
- No telemetry or data collection

## License

This project is open source. Feel free to modify and distribute as needed.