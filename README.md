# Polish Language - AI Text Enhancement Tool

A macOS system tray application that uses AI to polish and improve selected text anywhere on your system.

## Features

- **Global Text Enhancement**: Select any text in any application and press a shortcut to improve it with AI
- **Instant Translation**: Translate selected text to English with a separate hotkey
- **System Tray Integration**: Runs quietly in the background with easy access via system tray (no dock icon)
- **Smart Text Replacement**: Automatically replaces selected text or copies to clipboard
- **Configurable AI Models**: Support for OpenAI GPT models and compatible APIs
- **Customizable Shortcuts**: Set your preferred global keyboard shortcut
- **Persistent Settings**: All configurations are saved locally
- **Provider-Specific API Keys**: Automatically remembers API keys for each provider

## How to Use

1. **Install**: Run the built app from `src-tauri/target/release/bundle/macos/polish-language.app`
2. **Configure**: Click the system tray icon → Settings to configure:
   - Choose AI provider (OpenAI or Google Gemini)
   - Your API key for the selected provider (automatically saved per provider)
   - Preferred AI model
   - Polish text shortcut (default: Cmd+Shift+P)
   - Translate text shortcut (default: Cmd+Shift+T)
   - Custom system prompt for the AI
3. **Use**: 
   - Select any text in any application
   - **Polish text**: Press Cmd+Shift+P (or your custom shortcut)
   - **Translate to English**: Press Cmd+Shift+T (or your custom shortcut)
   - The processed text will be copied to your clipboard
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

## CI/CD

This project includes GitHub Actions workflows for:

### Automated Building
- **Build & Test**: Runs on every push and PR to main branch
- **macOS builds**: Automatically builds for macOS
- **PR Checks**: Validates code formatting, linting, and compilation

### Automated Releases
- **Release Workflow**: Triggered when you push a git tag (e.g., `v1.0.0`)
- **macOS Releases**: Automatically creates releases with macOS .dmg installer
- **Version Management**: Use the "Version Bump" workflow to automatically update versions

### Workflows
1. **`.github/workflows/build.yml`** - Build and test on push/PR
2. **`.github/workflows/release.yml`** - Create releases when tags are pushed
3. **`.github/workflows/pr-check.yml`** - Comprehensive PR validation
4. **`.github/workflows/version-bump.yml`** - Automated version management

### Creating a Release
1. Go to Actions → "Version Bump" → "Run workflow"
2. Choose version type (patch/minor/major)
3. The workflow will:
   - Update version in all config files
   - Create a git tag
   - Trigger the release workflow
   - Build and publish releases for all platforms

### Configuration

The app stores settings in your system's config directory:
- macOS: `~/Library/Application Support/polish-language/settings.json`

## API Support

Currently supports:
- **OpenAI API**: GPT-3.5 Turbo, GPT-4, GPT-4 Turbo, GPT-4o
- **Google Gemini**: Gemini 1.5 Flash, Gemini 1.5 Pro, Gemini Pro
- Any OpenAI-compatible API endpoint

### Getting API Keys
- **OpenAI**: Get your API key at [platform.openai.com/api-keys](https://platform.openai.com/api-keys)
- **Google Gemini**: Get your API key at [aistudio.google.com/app/apikey](https://aistudio.google.com/app/apikey)

### Smart API Key Management
The app automatically stores API keys per provider, so you can:
- Set up both OpenAI and Gemini API keys once
- Switch between providers seamlessly
- Your keys are automatically restored when switching back
- No need to re-enter keys when changing providers

## Privacy

- All settings are stored locally on your device
- Your API key never leaves your machine except to make requests to your configured AI service
- No telemetry or data collection

## License

This project is open source. Feel free to modify and distribute as needed.