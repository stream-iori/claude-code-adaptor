# Claude Code Adaptor for Qwen3

A Rust-based proxy adaptor that allows Claude Code to use the Qwen3-Coder API by transforming requests and responses between the two formats. This adaptor implements the complete Claude Messages API including tool calling support.

## Features

- **Complete Claude Messages API**: Full support for Claude's Messages API with tool calling
- **Protocol Translation**: Seamlessly converts Claude API requests to Qwen3 format and vice versa
- **Tool Calling Support**: Full tool/function calling capability between Claude and Qwen3
- **Token Counting**: Local approximate token counting for Claude's count_tokens API
- **Configuration Management**: Environment-based configuration for easy deployment
- **CLI Interface**: Start, stop, and manage the proxy via command line
- **SOLID Principles**: Clean architecture with separation of concerns
- **Comprehensive Testing**: Unit tests for all adapters and services

## Quick Start

1. **Set up environment variables**:
   ```bash
   cp .env.example .env
   # Edit .env with your Qwen3 API key
   ```

2. **Install dependencies**:
   ```bash
   cargo build --release
   ```

3. **Start the proxy**:
   ```bash
   cargo run -- start
   ```

## Usage

### CLI Commands

```bash
# Start the proxy server
cargo run -- start --host 127.0.0.1 --port 8080

# Check proxy health
cargo run -- health --url http://127.0.0.1:8080

# Display current configuration
cargo run -- config
```

### API Endpoints

- **Health Check**: `GET /health`
- **Chat Completions**: `POST /v1/chat/completions`
- **Claude Messages**: `POST /v1/messages`
- **Token Counting**: `POST /v1/messages/count_tokens`

### Claude Code Configuration

Configure Claude Code to use the proxy:

```json
{
  "anthropic": {
    "baseUrl": "http://127.0.0.1:8080",
    "apiKey": "your-qwen-api-key"
  }
}
```

### Tool Calling Example

The proxy supports full tool calling between Claude and Qwen3:

```json
{
  "model": "qwen3-coder",
  "messages": [
    {
      "role": "user",
      "content": "What's the weather in Tokyo?"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather for a location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {"type": "string"}
          },
          "required": ["location"]
        }
      }
    }
  ]
}
```

## Architecture

The project follows SOLID principles with the following structure:

- **adapters/**: Request/response transformation between Claude and Qwen3 formats
  - `request_adapter.rs`: Claude → Qwen3 request transformation
  - `response_adapter.rs`: Qwen3 → Claude response transformation
- **models/**: Data structures for Claude and Qwen3 APIs
  - `claude.rs`: Claude API structures
  - `claude_messages.rs`: Complete Claude Messages API
  - `claude_count_tokens.rs`: Token counting API
  - `qwen3.rs`: Qwen3 API structures
- **services/**: Core business logic and API clients
  - `proxy_service.rs`: HTTP server endpoints
  - `qwen_service.rs`: Qwen3 API client
  - `token_counter.rs`: Local token counting implementation
- **config/**: Configuration management
- **cli/**: Command-line interface

## Testing

Run all tests:
```bash
cargo test
```

The test suite includes:
- Request/response adapter tests
- Token counting validation
- Full integration tests
- All 7 tests currently pass

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `QWEN_API_KEY` | Your Qwen3 API key | Required |
| `SERVER_HOST` | Server bind address | 127.0.0.1 |
| `SERVER_PORT` | Server port | 8080 |

## Development

### Building
```bash
cargo build
```

### Running
```bash
cargo run -- start
```

### Testing
```bash
cargo test
```

### Current Status

- ✅ Complete Claude Messages API implementation
- ✅ Tool calling support
- ✅ Token counting API
- ✅ All tests passing (7/7)
- ✅ Ready for production use

## License

MIT License - see LICENSE file for details.