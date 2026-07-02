use std::fs;
use std::sync::Arc;

use anyhow::Context as _;
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::transport::stdio;
use rmcp::{RoleServer, ServerHandler, ServiceExt};
use serde_json::{Map, Value};

use log_analyzer::{grep, level_counts, parse};

type McpError = rmcp::ErrorData;

#[derive(Clone)]
struct S;

impl ServerHandler for S {
    fn get_info(&self) -> ServerInfo {
        let mut result = InitializeResult::default();
        result.capabilities = ServerCapabilities::builder().enable_tools().build();
        result
    }

    #[allow(deprecated)]
    async fn list_tools(
        &self,
        _params: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let path_only_schema: Arc<Map<String, Value>> = {
            let mut props = Map::new();
            let mut path_prop = Map::new();
            path_prop.insert("type".to_string(), Value::String("string".to_string()));
            path_prop.insert(
                "description".to_string(),
                Value::String("Path to the log file".to_string()),
            );
            props.insert("path".to_string(), Value::Object(path_prop));

            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("object".to_string()));
            schema.insert("properties".to_string(), Value::Object(props));
            schema.insert(
                "required".to_string(),
                Value::Array(vec![Value::String("path".to_string())]),
            );
            Arc::new(schema)
        };

        let grep_schema: Arc<Map<String, Value>> = {
            let mut props = Map::new();

            let mut path_prop = Map::new();
            path_prop.insert("type".to_string(), Value::String("string".to_string()));
            path_prop.insert(
                "description".to_string(),
                Value::String("Path to the log file".to_string()),
            );
            props.insert("path".to_string(), Value::Object(path_prop));

            let mut needle_prop = Map::new();
            needle_prop.insert("type".to_string(), Value::String("string".to_string()));
            needle_prop.insert(
                "description".to_string(),
                Value::String("Substring to search for (case-sensitive)".to_string()),
            );
            props.insert("needle".to_string(), Value::Object(needle_prop));

            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("object".to_string()));
            schema.insert("properties".to_string(), Value::Object(props));
            schema.insert(
                "required".to_string(),
                Value::Array(vec![
                    Value::String("path".to_string()),
                    Value::String("needle".to_string()),
                ]),
            );
            Arc::new(schema)
        };

        let tools = vec![
            Tool::new(
                "parse_log",
                "Parse a log file and return a summary of its entries (level counts).",
                path_only_schema.clone(),
            ),
            Tool::new(
                "grep",
                "Search log entries for a substring (case-sensitive) and return matching lines.",
                grep_schema,
            ),
            Tool::new(
                "level_counts",
                "Return a count of log entries grouped by severity level.",
                path_only_schema,
            ),
        ];

        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        req: CallToolRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let args = req.arguments.unwrap_or_default();

        match req.name.as_ref() {
            "parse_log" => {
                let path = get_string_arg(&args, "path")?;
                let raw = read_file_contents(&path)?;
                let entries = parse(&raw);
                let counts = level_counts(&entries);
                let total = entries.len();
                let summary = serde_json::json!({
                    "total_entries": total,
                    "level_counts": counts,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string()),
                )]))
            }
            "grep" => {
                let path = get_string_arg(&args, "path")?;
                let needle = get_string_arg(&args, "needle")?;
                let raw = read_file_contents(&path)?;
                let entries = parse(&raw);
                let hits: Vec<&str> = grep(&entries, &needle)
                    .iter()
                    .map(|e| e.message.as_str())
                    .collect();
                let result = serde_json::json!({ "matches": hits });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]))
            }
            "level_counts" => {
                let path = get_string_arg(&args, "path")?;
                let raw = read_file_contents(&path)?;
                let entries = parse(&raw);
                let counts = level_counts(&entries);
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&counts).unwrap_or_else(|_| "{}".to_string()),
                )]))
            }
            other => Err(McpError {
                code: ErrorCode::METHOD_NOT_FOUND,
                message: format!("unknown tool: {other}").into(),
                data: None,
            }),
        }
    }
}

fn get_string_arg(args: &Map<String, Value>, key: &str) -> Result<String, McpError> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
        .ok_or_else(|| McpError {
            code: ErrorCode::INVALID_PARAMS,
            message: format!("missing or non-string argument `{key}`").into(),
            data: None,
        })
}

fn read_file_contents(path: &str) -> Result<String, McpError> {
    fs::read_to_string(path)
        .with_context(|| format!("reading {path}"))
        .map_err(|e| McpError {
            code: ErrorCode::INTERNAL_ERROR,
            message: e.to_string().into(),
            data: None,
        })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let svc = S {}.serve(stdio()).await?;
    svc.waiting().await?;
    Ok(())
}
