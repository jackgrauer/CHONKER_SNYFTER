# Claude Ecosystem Development Setup

## Overview
This project uses Pydantic, Instructor, and OpenHands configured specifically for the Claude ecosystem (not OpenAI).

## Installed Tools

### 1. Pydantic (v2.10.5)
- **Purpose**: Data validation and settings management using Python type annotations
- **Usage**: All data models, configuration, and validation
- **Claude Integration**: Works identically with Claude as with any Python application

### 2. Instructor (v1.8.1)
- **Purpose**: Structured extraction and validation of LLM outputs
- **Usage**: Converting Claude's responses into validated Pydantic models
- **Claude Integration**: Configure with Anthropic client, not OpenAI client

### 3. OpenHands (openai-function-calling v2.6.0)
- **Purpose**: Structured function calling for LLMs
- **Usage**: Define functions that Claude can call during execution
- **Claude Integration**: Requires conversion from OpenAI format to Claude's tool use format

## Key Configuration Files

1. **claude_openhands_config.py**
   - Configures OpenHands for Claude's XML-based tool format
   - Provides ClaudeFunctionCaller adapter class
   - Includes conversion utilities for Claude's tool use

2. **development_tools_integration.py**
   - Demonstrates all three tools working together
   - Shows rapid dev protocol with TASK:ACTION:TARGET:OUTPUT format
   - Includes examples of security auditing and test generation

## Important Notes for Claude Ecosystem

### Model Names
```python
# Correct (Claude):
model = "claude-3-opus"
model = "claude-3-sonnet"
model = "claude-3-haiku"

# Incorrect (OpenAI):
model = "gpt-4"  # Don't use this!
```

### Client Configuration
```python
# Correct (Claude):
from anthropic import Anthropic
client = Anthropic(api_key="your-claude-api-key")

# Incorrect (OpenAI):
from openai import OpenAI  # Don't use this!
```

### Tool/Function Format
- Claude uses XML-based tool definitions
- OpenHands functions need conversion to Claude format
- Tool results are sent back in Claude's specific format

## Usage Examples

### 1. Creating Structured Tasks (Pydantic)
```python
task = TaskExecution(
    task="FEAT-001",
    action="implement",
    target="new_feature.py",
    output="Feature implemented with tests"
)
```

### 2. Getting Structured Outputs (Instructor)
```python
# Configure for Claude
audit_result = instructor.from_anthropic(client).create(
    model="claude-3-opus",
    response_model=CodeAuditResult,
    messages=[...]
)
```

### 3. Defining Claude-Callable Functions (OpenHands)
```python
# Define function with OpenHands
security_func = Function(
    name="analyze_security",
    description="Analyze code for vulnerabilities",
    parameters={...}
)

# Convert for Claude's tool use
claude_tools = convert_to_claude_format([security_func])
```

## Rapid Development Protocol

Following the established protocol:
- **Format**: TASK:ACTION:TARGET:OUTPUT
- **Principle**: Generate code first, explain second
- **Models**: Use Pydantic for all structured data
- **Tools**: Leverage all three tools for maximum productivity

## Environment Setup

All dependencies are installed in the virtual environment:
```bash
source venv/bin/activate
```

Required packages:
- pydantic==2.10.5
- instructor==1.8.1
- openai-function-calling==2.6.0
- anthropic (for Claude client)

## Remember
⚠️ **This setup is configured for Claude, not OpenAI!** All examples and configurations assume you're using the Anthropic Claude API, not the OpenAI API.