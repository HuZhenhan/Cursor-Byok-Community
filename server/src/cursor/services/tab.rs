//! Implements Cursor tab services.
//!
//! `Local` mode answers `StreamCpp` with a configured BYOK model: the current
//! file, split at the cursor, becomes a fill-in-the-middle prompt and the
//! completion is returned as `StreamCppResponse.text`. Other tab endpoints keep
//! their transport behaviour (Cursor upstream or a custom TAB service).
use std::{convert::Infallible, sync::Arc, time::Duration};

use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{Extension, State},
    http::{header, HeaderValue, Request, Response, StatusCode},
    routing::post,
    Router,
};
use bytes::BytesMut;
use futures_util::StreamExt;
use prost::Message;
use tokio_util::sync::CancellationToken;

use crate::{
    api::cursor::proxy,
    cursor::{
        protocol::{
            connect::{self, END_STREAM_FLAG},
            proto::aiserver::v1 as ai,
        },
        transport::TransportRegistry,
    },
    model::{
        ContentPart, ModelInvocation, ModelRequest, ModelSpec, ModelType, ProjectedContent,
        ProjectedMessage, PromptSpec, Role,
    },
    plugin::ADAPTER_ID_PREFIX,
    provider::{ModelEvent, Provider, ProviderStream},
    store::TabMode,
    Error, Result,
};

pub const TAB_PATHS: [&str; 16] = [
    "/aiserver.v1.AiService/StreamCpp",
    "/aiserver.v1.AiService/StreamNextCursorPrediction",
    "/aiserver.v1.AiService/GetCppEditClassification",
    "/aiserver.v1.AiService/RefreshTabContext",
    "/aiserver.v1.AiService/CppConfig",
    "/aiserver.v1.AiService/CppEditHistoryStatus",
    "/aiserver.v1.AiService/CppAppend",
    "/aiserver.v1.AiService/CppEditHistoryAppend",
    "/aiserver.v1.AiService/ReportAiCodeChangeMetrics",
    "/aiserver.v1.AiService/WriteGitBranchName",
    "/aiserver.v1.CppService/AvailableModels",
    "/aiserver.v1.CppService/RecordCppFate",
    "/aiserver.v1.FileSyncService/FSSyncFile",
    "/aiserver.v1.FileSyncService/FSIsEnabledForUser",
    "/aiserver.v1.FileSyncService/FSConfig",
    "/aiserver.v1.FileSyncService/FSUploadFile",
];

const STREAM_CPP_PATH: &str = "/aiserver.v1.AiService/StreamCpp";
const PREFIX_LIMIT: usize = 8_000;
const SUFFIX_LIMIT: usize = 4_000;
const MAX_OUTPUT_TOKENS: u64 = 256;
const GENERATION_TIMEOUT: Duration = Duration::from_secs(15);

const INSTRUCTIONS: &str = "\
You are Cursor's inline code completion engine.
Complete the code at the <cursor/> marker in the user's file.
- Output only the code that should be inserted at the marker.
- Never repeat the code in <before> or <after>.
- Do not use markdown code fences and do not explain.
- Prefer a minimal, syntactically correct continuation and stop at a natural boundary.
- Output nothing when no useful completion exists.";

pub fn is_tab_path(path: &str) -> bool {
    TAB_PATHS.contains(&path)
}

pub fn router() -> Router<TransportRegistry> {
    TAB_PATHS
        .into_iter()
        .filter(|path| *path != STREAM_CPP_PATH)
        .fold(
            Router::new().route(STREAM_CPP_PATH, post(stream_cpp)),
            |router, path| router.route(path, post(forward)),
        )
}

async fn forward(
    State(registry): State<TransportRegistry>,
    Extension(upstream): Extension<proxy::CursorProxy>,
    request: Request<Body>,
) -> Result<Response<Body>> {
    let settings = registry.store().tab_settings().await?;
    match settings.service_url() {
        Some(service_url) => proxy::forward_to_service(&upstream, request, service_url).await,
        None => proxy::forward(Extension(upstream), request).await,
    }
}

async fn stream_cpp(
    State(registry): State<TransportRegistry>,
    Extension(upstream): Extension<proxy::CursorProxy>,
    request: Request<Body>,
) -> Result<Response<Body>> {
    let settings = registry.store().tab_settings().await?;
    if settings.mode != TabMode::Local {
        return forward(State(registry), Extension(upstream), request).await;
    }
    let (_, body) = request.into_parts();
    let body = to_bytes(body, usize::MAX)
        .await
        .map_err(|error| Error::Protocol(format!("cannot read StreamCpp body: {error}")))?;
    let request = decode_request(&body)?;
    let Some(file) = request.current_file.as_ref() else {
        tracing::warn!("Cursor StreamCpp request has no current file");
        return Ok(empty_completion());
    };
    if file.contents.is_empty() {
        return Ok(empty_completion());
    }
    let model_id = settings.model_id.trim().to_owned();
    if model_id.is_empty() {
        tracing::warn!("Cursor TAB is in local mode but no TAB model is selected");
        return Ok(empty_completion());
    }
    let offset = cursor_offset(file);
    let line = current_line(file, offset);
    tracing::debug!(
        path = file.relative_workspace_path,
        language = file.language_id,
        line = line.number,
        contents_len = file.contents.len(),
        "generating Cursor TAB completion locally"
    );
    let model = tab_model_spec(&registry, &model_id).await?;
    let invocation = build_invocation(model, completion_content(file, offset));
    let provider = registry.conversations().dependencies().provider.clone();
    Ok(completion_response(completion_body(provider, invocation, line)))
}

/// 解析 Connect 请求信封；带压缩标志的消息先解压再解码。
fn decode_request(body: &[u8]) -> Result<ai::StreamCppRequest> {
    if body.len() >= 5 {
        let flags = body[0];
        let length = u32::from_be_bytes([body[1], body[2], body[3], body[4]]) as usize;
        if flags & END_STREAM_FLAG == 0 && length == body.len() - 5 {
            let payload = connect::message_payload(flags, Bytes::copy_from_slice(&body[5..]))?;
            return Ok(ai::StreamCppRequest::decode(payload)?);
        }
    }
    Ok(ai::StreamCppRequest::decode(body)?)
}

/// 把光标位置换算成文件内容里的字节偏移。`CursorPosition` 与 VS Code 一致，
/// 行号和列号都从 0 开始。
fn cursor_offset(file: &ai::CurrentFileInfo) -> usize {
    let contents = file.contents.as_str();
    let position = file.cursor_position.as_ref();
    let start_line = file.contents_start_at_line.max(0) as usize;
    let target_line = position
        .map_or(0, |value| value.line.max(0) as usize)
        .saturating_sub(start_line);
    let target_column = position.map_or(0, |value| value.column.max(0) as usize);
    let mut line_start = 0;
    if target_line > 0 {
        let mut line = 0;
        for (index, byte) in contents.bytes().enumerate() {
            if byte == b'\n' {
                line += 1;
                if line == target_line {
                    line_start = index + 1;
                    break;
                }
            }
        }
        if line < target_line {
            return contents.len();
        }
    }
    let line_end = contents[line_start..]
        .find('\n')
        .map_or(contents.len(), |offset| line_start + offset);
    let mut column = 0;
    for (offset, _) in contents[line_start..line_end].char_indices() {
        if column >= target_column {
            return line_start + offset;
        }
        column += 1;
    }
    line_end
}

/// 光标所在行的原始内容。Cursor 的 Tab 客户端按「整行重写」解析响应：
/// `range_to_replace` 指向这一行，`text` 则是这一行重写后的完整内容。
struct TabLine {
    number: i32,
    before: String,
    after: String,
}

fn current_line(file: &ai::CurrentFileInfo, offset: usize) -> TabLine {
    let contents = file.contents.as_str();
    let line_start = contents[..offset]
        .rfind('\n')
        .map_or(0, |index| index + 1);
    let line_end = contents[offset..]
        .find('\n')
        .map_or(contents.len(), |index| offset + index);
    TabLine {
        number: contents[..line_start].matches('\n').count() as i32 + 1,
        before: contents[line_start..offset].to_owned(),
        after: contents[offset..line_end].trim_end_matches('\r').to_owned(),
    }
}

fn completion_content(file: &ai::CurrentFileInfo, offset: usize) -> String {
    let contents = file.contents.as_str();
    let before = tail_chars(&contents[..offset], PREFIX_LIMIT);
    let after = head_chars(&contents[offset..], SUFFIX_LIMIT);
    let path = if file.relative_workspace_path.is_empty() {
        "untitled"
    } else {
        file.relative_workspace_path.as_str()
    };
    let language = if !file.language_id.is_empty() {
        file.language_id.as_str()
    } else {
        file.relative_workspace_path
            .rsplit_once('.')
            .map_or("plaintext", |(_, extension)| extension)
    };
    format!(
        "File: {path}\nLanguage: {language}\n\n<before>\n{before}\n</before>\n<cursor/>\n<after>\n{after}\n</after>"
    )
}

fn tail_chars(value: &str, limit: usize) -> &str {
    match value.char_indices().rev().nth(limit) {
        Some((index, _)) => &value[index..],
        None => value,
    }
}

fn head_chars(value: &str, limit: usize) -> &str {
    match value.char_indices().nth(limit) {
        Some((index, _)) => &value[..index],
        None => value,
    }
}

async fn tab_model_spec(registry: &TransportRegistry, model_id: &str) -> Result<ModelSpec> {
    let mut spec = ModelSpec::new(model_id);
    spec.max_output_tokens = Some(MAX_OUTPUT_TOKENS);
    if let Some(model) = registry.store().model(model_id).await? {
        // 推理模型会把时间花在思考 token 上,而 TAB 需要尽快给出首字;
        // 仅在配置本身启用了思考时关闭,避免向不支持该参数的模型发送它。
        if matches!(model.model_type, ModelType::OpenAi) && model.reasoning_effort.is_some() {
            spec.reasoning.effort = Some("none".into());
        }
        return Ok(spec);
    }
    if model_id.starts_with(ADAPTER_ID_PREFIX) {
        return Ok(spec);
    }
    Err(Error::Provider(format!(
        "TAB model {model_id} is not configured; select a configured model in the TAB settings"
    )))
}

fn build_invocation(model: ModelSpec, content: String) -> ModelInvocation {
    let call_id = format!("tab-completion-{}", uuid::Uuid::new_v4());
    ModelInvocation {
        call_id: call_id.clone(),
        run_id: call_id.clone(),
        conversation_id: call_id,
        provider_call_index: 0,
        request: ModelRequest {
            prompt: PromptSpec {
                instructions: INSTRUCTIONS.to_owned(),
                tools: Vec::new(),
            },
            model,
            history: vec![ProjectedMessage {
                message_id: "tab-completion".into(),
                role: Role::User,
                content: ProjectedContent::Parts(vec![ContentPart::Text { text: content }]),
            }],
        },
    }
}

fn completion_body(provider: Arc<dyn Provider>, invocation: ModelInvocation, line: TabLine) -> Body {
    let cancellation = CancellationToken::new();
    let mut events = provider.stream(invocation, cancellation.clone());
    Body::from_stream(async_stream::stream! {
        let _guard = TabStreamGuard(cancellation);
        let collected = tokio::time::timeout(GENERATION_TIMEOUT, collect_text(&mut events)).await;
        let completion = match collected {
            Ok(Ok(raw)) => clean_completion(&raw),
            Ok(Err(error)) => {
                tracing::warn!(%error, "Cursor TAB completion failed");
                yield Ok::<Bytes, Infallible>(connect::encode_error_end_stream(
                    &connect::ConnectStreamError {
                        code: connect::ConnectCode::Internal,
                        message: error.to_string(),
                        details: Vec::new(),
                    },
                )
                .unwrap_or_else(|_| connect::encode_end_stream()));
                return;
            }
            Err(_) => {
                tracing::warn!("Cursor TAB completion timed out");
                yield Ok::<Bytes, Infallible>(connect::encode_error_end_stream(
                    &connect::ConnectStreamError {
                        code: connect::ConnectCode::Unavailable,
                        message: "TAB completion timed out".into(),
                        details: Vec::new(),
                    },
                )
                .unwrap_or_else(|_| connect::encode_end_stream()));
                return;
            }
        };
        // 客户端先读 model_info,再等到 range_to_replace,最后才消费 text。
        yield Ok::<Bytes, Infallible>(frame(&ai::StreamCppResponse {
            model_info: Some(ai::StreamCppResponseModelInfo {
                is_fused_cursor_prediction_model: Some(false),
                is_multidiff_model: Some(false),
            }),
            ..Default::default()
        }));
        yield Ok::<Bytes, Infallible>(frame(&ai::StreamCppResponse {
            range_to_replace: Some(ai::LineRange {
                start_line_number: line.number,
                end_line_number_inclusive: line.number,
            }),
            ..Default::default()
        }));
        if !completion.is_empty() {
            yield Ok::<Bytes, Infallible>(frame(&ai::StreamCppResponse {
                text: format!("{}{}{}", line.before, completion, line.after),
                ..Default::default()
            }));
        }
        yield Ok::<Bytes, Infallible>(frame(&ai::StreamCppResponse {
            done_stream: Some(true),
            ..Default::default()
        }));
        yield Ok::<Bytes, Infallible>(connect::encode_end_stream());
    })
}

async fn collect_text(events: &mut ProviderStream) -> Result<String> {
    let mut text = String::new();
    let mut finished = false;
    while let Some(event) = events.next().await {
        match event? {
            ModelEvent::TextDelta(delta) => text.push_str(&delta),
            ModelEvent::Done(_) => {
                finished = true;
                break;
            }
            _ => {}
        }
    }
    if !finished && text.is_empty() {
        return Err(Error::Provider(
            "provider stream ended without producing a TAB completion".into(),
        ));
    }
    Ok(text)
}

fn clean_completion(raw: &str) -> String {
    let trimmed =
        raw.trim_start_matches(|character: char| character == '\n' || character == '\r');
    let body = strip_code_fence(trimmed);
    body.trim_end().to_owned()
}

fn strip_code_fence(value: &str) -> String {
    let content = value.trim_start();
    if !content.starts_with("```") {
        return value.to_owned();
    }
    let mut lines = content.lines();
    if lines.next().is_none() {
        return value.to_owned();
    }
    let mut body: Vec<&str> = lines.collect();
    if body
        .last()
        .is_some_and(|line| line.trim_start().starts_with("```"))
    {
        body.pop();
    }
    body.join("\n").trim_end().to_owned()
}

fn frame(message: &ai::StreamCppResponse) -> Bytes {
    connect::encode_message(message).expect("protobuf message encoding cannot fail")
}

fn empty_completion() -> Response<Body> {
    let mut body = BytesMut::new();
    body.extend_from_slice(&frame(&ai::StreamCppResponse {
        model_info: Some(ai::StreamCppResponseModelInfo {
            is_fused_cursor_prediction_model: Some(false),
            is_multidiff_model: Some(false),
        }),
        ..Default::default()
    }));
    body.extend_from_slice(&frame(&ai::StreamCppResponse {
        done_stream: Some(true),
        ..Default::default()
    }));
    body.extend_from_slice(&connect::encode_end_stream());
    completion_response(Body::from(body.freeze()))
}

fn completion_response(body: Body) -> Response<Body> {
    let mut response = Response::new(body);
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/connect+proto"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

struct TabStreamGuard(CancellationToken);

impl Drop for TabStreamGuard {
    fn drop(&mut self) {
        self.0.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(contents: &str, line: i32, column: i32) -> ai::CurrentFileInfo {
        ai::CurrentFileInfo {
            relative_workspace_path: "src/main.rs".into(),
            contents: contents.into(),
            cursor_position: Some(ai::CursorPosition { line, column }),
            language_id: "rust".into(),
            ..Default::default()
        }
    }

    #[test]
    fn cursor_offset_uses_zero_based_positions() {
        let contents = "abc\ndef\nghi";
        assert_eq!(cursor_offset(&file(contents, 0, 0)), 0);
        assert_eq!(cursor_offset(&file(contents, 0, 3)), 3);
        assert_eq!(cursor_offset(&file(contents, 1, 2)), 6);
        assert_eq!(cursor_offset(&file(contents, 2, 3)), contents.len());
        assert_eq!(cursor_offset(&file(contents, 9, 0)), contents.len());
    }

    #[test]
    fn completion_content_marks_the_cursor_position() {
        let contents = "fn main() {\n    \n}";
        let offset = cursor_offset(&file(contents, 1, 4));
        let content = completion_content(&file(contents, 1, 4), offset);
        assert!(content.contains("<before>\nfn main() {\n    \n</before>"));
        assert!(content.contains("<cursor/>"));
        assert!(content.contains("<after>\n\n}\n</after>"));
        assert!(content.contains("File: src/main.rs"));
        assert!(content.contains("Language: rust"));
    }

    #[test]
    fn cleaning_strips_fences_and_trailing_whitespace() {
        assert_eq!(clean_completion("```python\n    a + b\n```"), "    a + b");
        assert_eq!(clean_completion("a + b\n"), "a + b");
        assert_eq!(clean_completion("  \n"), "");
        assert_eq!(clean_completion("\n    a + b\n"), "    a + b");
    }

    #[test]
    fn tab_line_replaces_the_whole_cursor_line() {
        let contents = "def add(a, b):\n    return a";
        let offset = cursor_offset(&file(contents, 1, 12));
        let line = current_line(&file(contents, 1, 12), offset);
        assert_eq!(line.number, 2);
        assert_eq!(line.before, "    return a");
        assert_eq!(line.after, "");
        assert_eq!(
            format!("{}{}{}", line.before, " + b", line.after),
            "    return a + b"
        );
    }
}
