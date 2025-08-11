use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesRequest {
    pub model: String,
    pub messages: Vec<InputMessage>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    //TODO mcp_server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Vec<SystemMessage>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputMessage {
    //could be user, assistant
    pub role: MessageRole,
    pub content: InputMessageContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InputMessageContent {
    // string only
    Text(String),

    //or object[]
    Parts(Vec<InputMessageContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InputMessageContentPart {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(flatten)]
        cache_control: Option<serde_json::Value>,
    },
    Image {
        source: ImageSource,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    cache_control: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemMessage {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub cache_control: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub citations: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tool_type: Option<String>,
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<serde_json::Value>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_values: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    None,
    Auto {
        disable_parallel_tool_use: bool,
    },
    Any {
        disable_parallel_tool_use: bool,
    },
    Tool {
        name: String,
        disable_parallel_tool_use: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
}

//-----------------------------------------------------------------


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesResponse {
    pub id: String,
    pub content: Vec<OutputMessageContent>,
    #[serde(rename = "type")]
    pub response_type: String,
    pub model: String,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: Usage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ResponseContainer>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContainer {
    expires_at: String,
    id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputMessageContent {
    Text {
        text: String,
    },
    Thinking {
        signature: String,
        thinking: String,
    },
    Redacted_Thinking {
        data: String,
    },
    Tool_Use {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Server_Tool_Use {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Web_Search_Tool_Result {
        tool_use_id: String,
        content: serde_json::Value,
    },
    Code_Execution_Tool_Result {
        tool_use_id: String,
        content: serde_json::Value,
    },
    Mcp_Tool_Use {
        id: String,
        name: String,
        server_name: String,
        input: serde_json::Value,
    },
    Mcp_Tool_Result {
        content: String,
        is_error: bool,
        tool_use_id: String,
    },
    Container_Upload {
        file_id: String,
    },
}

//-------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMessagesStreamResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<StreamDelta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    // Keep other fields as JSON for extensibility
    #[serde(flatten)]
    pub other: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamDelta {
    pub content: String,
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_message() {
        let claude_request = r#"
            {
                "model": "claude-3-5-haiku-20241022",
                "max_tokens": 512,
                "messages": [
                    {
                        "role": "user",
                        "content": "who are you"
                    }
                ],
                "system": {
                    "type": "text",
                    "text": "Analyze if this message indicates a new conversation topic. If it does, extract a 2-3 word title that captures the new topic. Format your response as a JSON object with two fields: 'isNewTopic' (boolean) and 'title' (string, or null if isNewTopic is false). Only include these fields, no other text."
                },
                "temperature": 0,
                "stream": true
            }
        "#;

        let claude_messages_request: ClaudeMessagesRequest =
            serde_json::from_str(claude_request).expect("Failed to parse ClaudeMessagesRequest");
        assert_eq!(claude_messages_request.model, "claude-3-5-haiku-20241022");
        assert_eq!(claude_messages_request.max_tokens, Some(512));
        assert_eq!(claude_messages_request.messages.len(), 1);
    }

    #[test]
    fn test_claude_message_minimal() {
        let claude_request = r#"
            {
                "model": "claude-3-5-haiku-20241022",
                "messages": [
                    {
                        "role": "user",
                        "content": "Hello"
                    }
                ]
            }
        "#;

        let claude_messages_request: ClaudeMessagesRequest =
            serde_json::from_str(claude_request).expect("Failed to parse ClaudeMessagesRequest");
        assert_eq!(claude_messages_request.model, "claude-3-5-haiku-20241022");
        assert_eq!(claude_messages_request.messages.len(), 1);
        assert!(claude_messages_request.system.is_none());
        assert!(claude_messages_request.max_tokens.is_none());
    }

    #[test]
    fn test_claude_message_x() {
        let claude_request = r#"
        {
            "model": "claude-sonnet-4-20250514",
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": "<system-reminder>\nAs you answer the user's questions, you can use the following context:\n# important-instruction-reminders\nDo what has been asked; nothing more, nothing less.\nNEVER create files unless they're absolutely necessary for achieving your goal.\nALWAYS prefer editing an existing file to creating a new one.\nNEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.\n\n      \n      IMPORTANT: this context may or may not be relevant to your tasks. You should not respond to this context unless it is highly relevant to your task.\n</system-reminder>\n"
                        },
                        {
                            "type": "text",
                            "text": "who are you"
                        },
                        {
                            "type": "text",
                            "text": "<system-reminder>\nThis is a reminder that your todo list is currently empty. DO NOT mention this to the user explicitly because they are already aware. If you are working on tasks that would benefit from a todo list please use the TodoWrite tool to create one. If not, please feel free to ignore. Again do not mention this message to the user.\n</system-reminder>"
                        },
                        {
                            "type": "text",
                            "text": "who are you",
                            "cache_control": {
                                "type": "ephemeral"
                            }
                        }
                    ]
                }
            ],
            "temperature": 1,
            "system": [
                {
                    "type": "text",
                    "text": "You are Claude Code, Anthropic's official CLI for Claude.",
                    "cache_control": {
                        "type": "ephemeral"
                    }
                }
            ],
            "tools": [
                {
                    "name": "Task",
                    "description": "Launch a new agent to handle complex, multi-step tasks autonomously. \n\nAvailable agent types and the tools they have access to:\n- general-purpose: General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks. When you are searching for a keyword or file and are not confident that you will find the right match in the first few tries use this agent to perform the search for you. (Tools: *)\n- statusline-setup: Use this agent to configure the user's Claude Code status line setting. (Tools: Read, Edit)\n\nWhen using the Task tool, you must specify a subagent_type parameter to select which agent type to use.\n\n\n\nWhen NOT to use the Agent tool:\n- If you want to read a specific file path, use the Read or Glob tool instead of the Agent tool, to find the match more quickly\n- If you are searching for a specific class definition like \"class Foo\", use the Glob tool instead, to find the match more quickly\n- If you are searching for code within a specific file or set of 2-3 files, use the Read tool instead of the Agent tool, to find the match more quickly\n- Other tasks that are not related to the agent descriptions above\n\n\nUsage notes:\n1. Launch multiple agents concurrently whenever possible, to maximize performance; to do that, use a single message with multiple tool uses\n2. When the agent is done, it will return a single message back to you. The result returned by the agent is not visible to the user. To show the user the result, you should send a text message back to the user with a concise summary of the result.\n3. Each agent invocation is stateless. You will not be able to send additional messages to the agent, nor will the agent be able to communicate with you outside of its final report. Therefore, your prompt should contain a highly detailed task description for the agent to perform autonomously and you should specify exactly what information the agent should return back to you in its final and only message to you.\n4. The agent's outputs should generally be trusted\n5. Clearly tell the agent whether you expect it to write code or just to do research (search, file reads, web fetches, etc.), since it is not aware of the user's intent\n6. If the agent description mentions that it should be used proactively, then you should try your best to use it without the user having to ask for it first. Use your judgement.\n\nExample usage:\n\n<example_agent_descriptions>\n\"code-reviewer\": use this agent after you are done writing a signficant piece of code\n\"greeting-responder\": use this agent when to respond to user greetings with a friendly joke\n</example_agent_description>\n\n<example>\nuser: \"Please write a function that checks if a number is prime\"\nassistant: Sure let me write a function that checks if a number is prime\nassistant: First let me use the Write tool to write a function that checks if a number is prime\nassistant: I'm going to use the Write tool to write the following code:\n<code>\nfunction isPrime(n) {\n  if (n <= 1) return false\n  for (let i = 2; i * i <= n; i++) {\n    if (n % i === 0) return false\n  }\n  return true\n}\n</code>\n<commentary>\nSince a signficant piece of code was written and the task was completed, now use the code-reviewer agent to review the code\n</commentary>\nassistant: Now let me use the code-reviewer agent to review the code\nassistant: Uses the Task tool to launch the with the code-reviewer agent \n</example>\n\n<example>\nuser: \"Hello\"\n<commentary>\nSince the user is greeting, use the greeting-responder agent to respond with a friendly joke\n</commentary>\nassistant: \"I'm going to use the Task tool to launch the with the greeting-responder agent\"\n</example>\n",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "description": {
                                "type": "string",
                                "description": "A short (3-5 word) description of the task"
                            },
                            "prompt": {
                                "type": "string",
                                "description": "The task for the agent to perform"
                            },
                            "subagent_type": {
                                "type": "string",
                                "description": "The type of specialized agent to use for this task"
                            }
                        },
                        "required": [
                            "description",
                            "prompt",
                            "subagent_type"
                        ],
                        "additionalProperties": false,
                        "$schema": "http://json-schema.org/draft-07/schema#"
                    }
                },
                {
                    "name": "KillBash",
                    "description": "\n- Kills a running background bash shell by its ID\n- Takes a shell_id parameter identifying the shell to kill\n- Returns a success or failure status \n- Use this tool when you need to terminate a long-running shell\n- Shell IDs can be found using the /bashes command\n",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "shell_id": {
                                "type": "string",
                                "description": "The ID of the background shell to kill"
                            }
                        },
                        "required": [
                            "shell_id"
                        ],
                        "additionalProperties": false,
                        "$schema": "http://json-schema.org/draft-07/schema#"
                    }
                }
            ],
            "metadata": {
                "user_id": "user_8b8886105677a603d22a9d4b562314eac9258ce75f8c387d16fcd9b80475d6ec_account__session_7958862d-351f-4cd2-bbf1-7b9b4f0e0379"
            },
            "max_tokens": 21333
        }
        "#;

        let claude_messages_request: ClaudeMessagesRequest =
            serde_json::from_str(claude_request).expect("Failed to parse ClaudeMessagesRequest");
        assert_eq!(claude_messages_request.model, "claude-sonnet-4-20250514");
        assert_eq!(claude_messages_request.messages.len(), 1);
        assert_eq!(claude_messages_request.messages[0].role, MessageRole::User);
        assert!(claude_messages_request.system.is_some());
    }
}
