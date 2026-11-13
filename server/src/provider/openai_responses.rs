//! Implements the OpenAI Responses provider adapter.
use async_stream::try_stream;
use base64::{engine::general_purpose::STANDARD, Engine};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use serde_json::{json, Map, Value};

use crate::{
    config::ProviderConfig,
    model::{
        ContentPart, ModelInvocation, ModelLatency, ModelRequest, ProjectedContent,
        ProjectedMessage, Role, Usage,
    },
    Error, Result,
};

use super::{
    apply_body_allowlist, apply_openai_prompt_cache_key,
    attempt::{send_once, Attempt},
    map_sse_error, merge_extra_params, provider_event_error,
    recorder::recorded_headers,
    CallRecorder, FinishReason, ModelEvent, Provider, ProviderStream,
};

#[derive(Default)]
struct ResponseToolState {
    call_id: Option<String>,
    name: Option<String>,
    arguments: String,
    emitted_arguments: usize,
    started: bool,
    ended: bool,
}

enum ResponseToolArguments<'a> {
    None,
    Delta(&'a str),
    Snapshot(&'a str),
}

pub struct OpenAiResponsesProvider {
    client: reqwest::Client,
    config: ProviderConfig,
    recorder: Option<CallRecorder>,
}

impl OpenAiResponsesProvider {
    pub fn new(client: reqwest::Client, config: ProviderConfig) -> Self {
        Self {
            client,
            config,
            recorder: None,
        }
    }

    pub fn with_recorder(mut self, recorder: Option<CallRecorder>) -> Self {
        self.recorder = recorder;
        self
    }
}

impl Provider for OpenAiResponsesProvider {
    fn stream(
        &self,
        invocation: ModelInvocation,
        cancellation: tokio_util::sync::CancellationToken,
    ) -> ProviderStream {
        let client = self.client.clone();
        let config = self.config.clone();
        let recorder = self.recorder.clone();
        Box::pin(try_stream! {
            let ModelInvocation { call_id, request, .. } = invocation;
            let mut replay_reasoning = has_responses_replay_state(&request.history);
            let mut body = responses_body(&request, &config, replay_reasoning)?;
            let request_headers = recorded_headers(&config, &[("content-type", "application/json")]);
            if let Some(recorder) = &recorder {
                recorder.request(request_headers.clone(), &body).await?;
            }
            let mut attempt = send_once(
                "OpenAI Responses",
                || responses_post(&client, &config, &body),
                &cancellation,
                recorder.as_ref(),
            ).await;
            // Azure OpenAI 把 item id 绑定到创建它的资源,中转站又会在多个资源之间轮转。
            // 一旦上游以跨资源为由拒绝,本请求立刻丢掉全部回放状态重发一次,避免把
            // 用户困在同一个 400 上。
            let stale_state = replay_reasoning
                && attempt
                    .as_ref()
                    .err()
                    .is_some_and(|error| is_foreign_resource_state(&error.to_string()));
            if stale_state {
                tracing::warn!(
                    model = %request.model.model_id,
                    "upstream rejected cross-resource reasoning state; retrying without replay"
                );
                replay_reasoning = false;
                body = responses_body(&request, &config, replay_reasoning)?;
                if let Some(recorder) = &recorder {
                    recorder.request(request_headers, &body).await?;
                }
                attempt = send_once(
                    "OpenAI Responses",
                    || responses_post(&client, &config, &body),
                    &cancellation,
                    recorder.as_ref(),
                ).await;
            }
            let attempt = attempt.map_err(readable_foreign_resource_error)?;
            let Attempt::Response(response) = attempt else { return };
            yield ModelEvent::Start { model_call_id: call_id };
            let chunk_recorder = recorder.clone();
            let chunks = response.bytes_stream()
                .map(|chunk| chunk.map_err(Error::from))
                .then(move |chunk| {
                    let recorder = chunk_recorder.clone();
                    async move {
                        let chunk = chunk?;
                        if let Some(recorder) = recorder { recorder.response_chunk(&chunk).await?; }
                        Ok::<_, Error>(chunk)
                    }
                });
            let source = chunks.eventsource();
            futures_util::pin_mut!(source);
            let mut text_open = false;
            let mut text = String::new();
            let mut thinking_open = false;
            let mut tools = std::collections::BTreeMap::<usize, ResponseToolState>::new();
            let mut reasoning_items = Vec::new();
            let mut saw_tool = false;
            let mut saw_completed_item = false;
            let mut terminal = false;
            loop {
                let event = tokio::select! {
                    _ = cancellation.cancelled() => { return; }
                    event = source.next() => event,
                };
                let Some(event) = event else { break };
                let event = event.map_err(|error| map_sse_error("OpenAI Responses", error))?;
                if event.data == "[DONE]" { break; }
                let value: Value = serde_json::from_str(&event.data)?;
                if let Some(error) = provider_event_error("OpenAI Responses", &value) {
                    Err(error)?;
                }
                let kind = value.get("type").and_then(Value::as_str).unwrap_or(&event.event);
                match kind {
                    "response.output_text.delta" => {
                        if thinking_open { thinking_open = false; yield ModelEvent::ThinkingEnd; }
                        if !text_open { text_open = true; yield ModelEvent::TextStart; }
                        if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                            text.push_str(delta);
                            yield ModelEvent::TextDelta(delta.into());
                        }
                    }
                    "response.output_text.done" => {
                        if let Some(final_text) = value.get("text").and_then(Value::as_str) {
                            for event in reconcile_response_text(&mut text_open, &mut text, final_text) { yield event; }
                        }
                        if text_open { text_open = false; yield ModelEvent::TextEnd; }
                    }
                    "response.reasoning_summary_text.delta" | "response.reasoning_text.delta" => {
                        if !thinking_open { thinking_open = true; yield ModelEvent::ThinkingStart; }
                        if let Some(delta) = value.get("delta").and_then(Value::as_str) { yield ModelEvent::ThinkingDelta(delta.into()); }
                    }
                    "response.reasoning_summary_text.done" | "response.reasoning_text.done"
                        if thinking_open => { thinking_open = false; yield ModelEvent::ThinkingEnd; }
                    "response.output_item.added" => {
                        let item = value.get("item").unwrap_or(&Value::Null);
                        if item.get("type").and_then(Value::as_str) == Some("function_call") {
                            let index = required_u64(&value, "output_index")? as usize;
                            saw_tool = true;
                            for event in update_response_tool(index, item, ResponseToolArguments::None, false, &mut tools)? { yield event; }
                        }
                    }
                    "response.output_item.done" => {
                        let item = value.get("item").unwrap_or(&Value::Null);
                        match item.get("type").and_then(Value::as_str) {
                            Some("reasoning") => {
                                if thinking_open { thinking_open = false; yield ModelEvent::ThinkingEnd; }
                                reasoning_items.push(item.clone());
                            }
                            Some("message") => {
                                saw_completed_item = true;
                                if let Some(final_text) = response_item_text(item) {
                                    for event in reconcile_response_text(&mut text_open, &mut text, &final_text) { yield event; }
                                }
                                if text_open { text_open = false; yield ModelEvent::TextEnd; }
                            }
                            Some("function_call") => {
                                saw_completed_item = true;
                                let index = required_u64(&value, "output_index")? as usize;
                                saw_tool = true;
                                let arguments = item
                                    .get("arguments")
                                    .and_then(Value::as_str)
                                    .map_or(ResponseToolArguments::None, ResponseToolArguments::Snapshot);
                                for event in update_response_tool(index, item, arguments, true, &mut tools)? { yield event; }
                            }
                            _ => {}
                        }
                    }
                    "response.function_call_arguments.delta" => {
                        let index = required_u64(&value, "output_index")? as usize;
                        if let Some(delta) = value.get("delta").and_then(Value::as_str) {
                            saw_tool = true;
                            for event in update_response_tool(index, &Value::Null, ResponseToolArguments::Delta(delta), false, &mut tools)? { yield event; }
                        }
                    }
                    "response.function_call_arguments.done" => {
                        let index = required_u64(&value, "output_index")? as usize;
                        match value.get("arguments").and_then(Value::as_str) {
                            Some("") => {
                                for event in update_response_tool(
                                    index,
                                    &Value::Null,
                                    ResponseToolArguments::None,
                                    false,
                                    &mut tools,
                                )? { yield event; }
                            }
                            arguments => {
                                let arguments = arguments.map_or(
                                    ResponseToolArguments::None,
                                    ResponseToolArguments::Snapshot,
                                );
                                for event in update_response_tool(index, &Value::Null, arguments, true, &mut tools)? { yield event; }
                            }
                        }
                    }
                    "response.completed" => {
                        if let Some(usage) = value.pointer("/response/usage") { yield ModelEvent::Usage(responses_usage(usage)); }
                        if thinking_open { thinking_open = false; yield ModelEvent::ThinkingEnd; }
                        if text_open { text_open = false; yield ModelEvent::TextEnd; }
                        for (index, tool) in tools.iter_mut().filter(|(_, tool)| tool.started && !tool.ended) {
                            tool.ended = true;
                            yield ModelEvent::ToolCallEnd { index: *index };
                        }
                        if tools.values().any(|tool| !tool.started) {
                            Err(Error::Provider("OpenAI Responses completed with incomplete tool metadata".into()))?;
                        }
                        terminal = true;
                        if !reasoning_items.is_empty() {
                            yield ModelEvent::ProviderReplayState(
                                crate::model::ProviderReplayState {
                                    provider_kind: "openai_responses".into(),
                                    value: json!({"items": std::mem::take(&mut reasoning_items)}),
                                },
                            );
                        }
                        yield ModelEvent::Done(if saw_tool { FinishReason::ToolUse } else { FinishReason::Stop });
                    }
                    "response.incomplete" => {
                        if thinking_open { thinking_open = false; yield ModelEvent::ThinkingEnd; }
                        if text_open { text_open = false; yield ModelEvent::TextEnd; }
                        for (index, tool) in tools.iter_mut().filter(|(_, tool)| tool.started && !tool.ended) {
                            tool.ended = true;
                            yield ModelEvent::ToolCallEnd { index: *index };
                        }
                        terminal = true;
                        yield ModelEvent::Done(FinishReason::Length);
                    }
                    _ => {}
                }
            }
            if !terminal && saw_completed_item {
                if thinking_open { yield ModelEvent::ThinkingEnd; }
                if text_open { yield ModelEvent::TextEnd; }
                if tools.values().any(|tool| !tool.ended) {
                    Err(Error::Provider("OpenAI Responses stream ended with an incomplete tool call".into()))?;
                }
                terminal = true;
                if !reasoning_items.is_empty() {
                    yield ModelEvent::ProviderReplayState(crate::model::ProviderReplayState {
                        provider_kind: "openai_responses".into(),
                        value: json!({"items": std::mem::take(&mut reasoning_items)}),
                    });
                }
                yield ModelEvent::Done(if saw_tool { FinishReason::ToolUse } else { FinishReason::Stop });
            }
            if !terminal {
                Err(Error::Provider("OpenAI Responses stream ended without response.completed or response.incomplete".into()))?;
            }
        })
    }
}

fn responses_post(
    client: &reqwest::Client,
    config: &ProviderConfig,
    body: &Value,
) -> reqwest::RequestBuilder {
    client
        .post(&config.request_url)
        .bearer_auth(&config.api_key)
        .headers(config.custom_headers.clone())
        .json(body)
}

fn responses_body(
    request: &ModelRequest,
    config: &ProviderConfig,
    replay_reasoning: bool,
) -> Result<Value> {
    let input = responses_input(&request.history, replay_reasoning)?;
    let mut body = json!({
        "model": request.model.model_id, "input": input, "stream": true,
        "instructions": request.prompt.instructions,
        "include": ["reasoning.encrypted_content"]
    });
    if !request.prompt.tools.is_empty() {
        body["tools"] = json!(request.prompt.tools.iter().map(|tool| json!({
            "type":"function", "name":tool.name, "description":tool.description,
            "parameters":tool.parameters, "strict":false
        })).collect::<Vec<_>>());
    }
    apply_model(&mut body, &request.model, config.max_output_tokens)?;
    merge_extra_params(&mut body, &request.model.extra_params)?;
    apply_openai_prompt_cache_key(&mut body, &request.model.model_id)?;
    apply_body_allowlist(&mut body, config.allowed_body_fields.as_ref())?;
    Ok(body)
}

/// 历史里是否存在可以回放的 Responses 推理状态。
fn has_responses_replay_state(history: &[ProjectedMessage]) -> bool {
    history.iter().any(|message| match &message.content {
        ProjectedContent::Assistant { replay_state, .. } => replay_state
            .as_ref()
            .is_some_and(|state| state.provider_kind == "openai_responses"),
        _ => false,
    })
}

/// Azure OpenAI 只允许在创建 item 的那个资源里引用它;中转站把同一个地址后面
/// 的请求分发到多个资源,所以这类拒绝无法在本地提前判断,只能按错误文本识别。
fn is_foreign_resource_state(message: &str) -> bool {
    message
        .to_ascii_lowercase()
        .contains("different azure openai resource")
}

/// 把上游跨资源拒绝翻译成用户能行动的错误,并保留原始响应便于排查。
fn readable_foreign_resource_error(error: Error) -> Error {
    let Error::Provider(message) = &error else {
        return error;
    };
    if !is_foreign_resource_state(message) {
        return error;
    }
    Error::Provider(format!(
        "OpenAI Responses 拒绝了由其他上游资源创建的会话状态;已改用无状态请求重试仍被拒绝。\
         请新建会话,或让该模型配置指向创建这份状态的资源。上游原始响应: {message}"
    ))
}

fn response_item_text(item: &Value) -> Option<String> {
    let text = item
        .get("content")?
        .as_array()?
        .iter()
        .filter(|part| part.get("type").and_then(Value::as_str) == Some("output_text"))
        .filter_map(|part| part.get("text").and_then(Value::as_str))
        .collect::<String>();
    Some(text)
}

fn reconcile_response_text(
    open: &mut bool,
    streamed: &mut String,
    final_text: &str,
) -> Vec<ModelEvent> {
    let mut events = Vec::new();
    if final_text.starts_with(streamed.as_str()) && final_text.len() > streamed.len() {
        if !*open {
            *open = true;
            events.push(ModelEvent::TextStart);
        }
        let suffix = &final_text[streamed.len()..];
        streamed.push_str(suffix);
        events.push(ModelEvent::TextDelta(suffix.into()));
    }
    events
}

fn update_response_tool(
    index: usize,
    item: &Value,
    arguments: ResponseToolArguments<'_>,
    done: bool,
    tools: &mut std::collections::BTreeMap<usize, ResponseToolState>,
) -> Result<Vec<ModelEvent>> {
    let tool = tools.entry(index).or_default();
    if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
        tool.call_id.get_or_insert_with(|| call_id.into());
    }
    if let Some(name) = item.get("name").and_then(Value::as_str) {
        tool.name.get_or_insert_with(|| name.into());
    }
    match arguments {
        ResponseToolArguments::None => {}
        ResponseToolArguments::Delta(delta) => tool.arguments.push_str(delta),
        ResponseToolArguments::Snapshot(snapshot) if snapshot == tool.arguments => {}
        ResponseToolArguments::Snapshot(snapshot) if snapshot.starts_with(&tool.arguments) => {
            tool.arguments.push_str(&snapshot[tool.arguments.len()..]);
        }
        ResponseToolArguments::Snapshot(_) => {
            return Err(Error::Provider(
                "OpenAI Responses final tool arguments do not match streamed arguments".into(),
            ));
        }
    }

    let mut events = Vec::new();
    if !tool.started {
        if let (Some(call_id), Some(name)) = (&tool.call_id, &tool.name) {
            tool.started = true;
            events.push(ModelEvent::ToolCallStart {
                index,
                call_id: call_id.clone(),
                name: name.clone(),
            });
        }
    }
    if tool.started && tool.emitted_arguments < tool.arguments.len() {
        let delta = tool.arguments[tool.emitted_arguments..].to_string();
        tool.emitted_arguments = tool.arguments.len();
        events.push(ModelEvent::ToolCallArgumentsDelta { index, delta });
    }
    if done && !tool.ended {
        if !tool.started {
            return Err(Error::Provider(
                "OpenAI Responses function call is missing call_id or name".into(),
            ));
        }
        tool.ended = true;
        events.push(ModelEvent::ToolCallEnd { index });
    }
    Ok(events)
}

fn apply_model(
    body: &mut Value,
    model: &crate::model::ModelSpec,
    route_max_output_tokens: Option<u64>,
) -> Result<()> {
    let object = body
        .as_object_mut()
        .ok_or_else(|| Error::Provider("OpenAI Responses request body is not an object".into()))?;
    if let Some(max) = model.max_output_tokens.or(route_max_output_tokens) {
        object.insert("max_output_tokens".into(), json!(max));
    }
    if model.reasoning.enabled || model.reasoning.effort.is_some() {
        let mut reasoning = Map::new();
        reasoning.insert("summary".into(), json!("auto"));
        if let Some(effort) = &model.reasoning.effort {
            reasoning.insert("effort".into(), json!(effort));
        }
        object.insert("reasoning".into(), Value::Object(reasoning));
    }
    if model.latency == ModelLatency::Fast {
        object.insert("service_tier".into(), json!("fast"));
    }
    Ok(())
}

fn responses_input(messages: &[ProjectedMessage], replay_reasoning: bool) -> Result<Vec<Value>> {
    let mut input = Vec::new();
    for message in messages {
        match &message.content {
            ProjectedContent::Parts(parts) => {
                push_responses_parts(&mut input, &message.role, parts)?
            }
            ProjectedContent::ToolResult(result) => {
                let output = if result.provider_parts.is_empty() {
                    Value::String(result.content.clone())
                } else {
                    Value::Array(responses_content(&result.provider_parts, "input_text")?)
                };
                input.push(json!({
                    "type": "function_call_output",
                    "call_id": result.call_id,
                    "output": output,
                }));
            }
            ProjectedContent::Assistant {
                text,
                replay_state,
                calls,
                ..
            } => {
                if replay_reasoning {
                    if let Some(state) = replay_state
                        .as_ref()
                        .filter(|state| state.provider_kind == "openai_responses")
                    {
                        let items = state
                            .value
                            .get("items")
                            .and_then(Value::as_array)
                            .ok_or_else(|| {
                                Error::Protocol(
                                    "OpenAI Responses replay state is missing items".into(),
                                )
                            })?;
                        input.extend(
                            items
                                .iter()
                                .map(response_reasoning_input)
                                .collect::<Result<Vec<_>>>()?,
                        );
                    }
                }
                push_responses_text(&mut input, &message.role, text);
                for call in calls {
                    input.push(json!({
                        "type": "function_call",
                        "call_id": call.call_id,
                        "name": call.name,
                        "arguments": serde_json::to_string(&call.arguments)?,
                    }));
                }
            }
        }
    }
    Ok(input)
}

/// 只回放可移植字段。`id` 属于创建它的上游资源(Azure OpenAI 尤其如此),
/// 换资源或换中转后回放旧 `id` 会被拒绝,因此这里一律丢弃。
fn response_reasoning_input(item: &Value) -> Result<Value> {
    let source = item
        .as_object()
        .filter(|object| object.get("type").and_then(Value::as_str) == Some("reasoning"))
        .ok_or_else(|| {
            Error::Protocol("OpenAI Responses replay state contains a non-reasoning item".into())
        })?;
    let mut projected = Map::new();
    projected.insert("type".into(), json!("reasoning"));
    for field in ["summary", "content", "encrypted_content"] {
        if let Some(value) = source.get(field) {
            projected.insert(field.into(), value.clone());
        }
    }
    Ok(Value::Object(projected))
}

fn push_responses_parts(input: &mut Vec<Value>, role: &Role, parts: &[ContentPart]) -> Result<()> {
    let text_type = if *role == Role::Assistant {
        "output_text"
    } else {
        "input_text"
    };
    let content = responses_content(parts, text_type)?;
    if !content.is_empty() {
        input.push(json!({
            "type":"message",
            "role":role_name(role),
            "content":content,
        }));
    }
    Ok(())
}

fn responses_content(parts: &[ContentPart], text_type: &str) -> Result<Vec<Value>> {
    parts
        .iter()
        .filter_map(|part| match part {
            ContentPart::Text { text } if text.is_empty() => None,
            ContentPart::Text { text } => Some(Ok(json!({"type":text_type, "text":text}))),
            ContentPart::Image { mime_type, data } => Some(Ok(json!({
                "type":"input_image",
                "detail":"auto",
                "image_url":format!("data:{mime_type};base64,{}", STANDARD.encode(data)),
            }))),
        })
        .collect()
}

fn push_responses_text(input: &mut Vec<Value>, role: &Role, text: &str) {
    if text.is_empty() {
        return;
    }
    let content_type = if *role == Role::Assistant {
        "output_text"
    } else {
        "input_text"
    };
    input.push(json!({
        "type": "message",
        "role": role_name(role),
        "content": [{"type": content_type, "text": text}],
    }));
}

fn role_name(role: &Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

fn required_u64(value: &Value, name: &str) -> Result<u64> {
    value
        .get(name)
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::Provider(format!("OpenAI Responses event is missing {name}")))
}

pub(crate) fn responses_usage(value: &Value) -> Usage {
    let input_tokens = value.get("input_tokens").and_then(Value::as_u64);
    Usage {
        input_tokens,
        context_input_tokens: input_tokens,
        output_tokens: value.get("output_tokens").and_then(Value::as_u64),
        total_tokens: value.get("total_tokens").and_then(Value::as_u64),
        cache_read_tokens: value
            .pointer("/input_tokens_details/cached_tokens")
            .and_then(Value::as_u64),
        cache_write_tokens: None,
        reasoning_tokens: value
            .pointer("/output_tokens_details/reasoning_tokens")
            .and_then(Value::as_u64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ProviderReplayState;

    fn assistant_with_replay_state() -> ProjectedMessage {
        ProjectedMessage {
            message_id: "assistant-1".into(),
            role: Role::Assistant,
            content: ProjectedContent::Assistant {
                text: String::new(),
                thinking: String::new(),
                replay_state: Some(ProviderReplayState {
                    provider_kind: "openai_responses".into(),
                    value: json!({
                        "items": [{
                            "type": "reasoning",
                            "id": "item-1",
                            "status": "completed",
                            "summary": [{"type": "summary_text", "text": "why"}],
                            "content": [],
                            "encrypted_content": "opaque",
                            "output_only": true
                        }]
                    }),
                }),
                calls: Vec::new(),
            },
        }
    }

    #[test]
    fn reasoning_replay_drops_resource_scoped_item_ids() {
        // 复现上游报错:回放里带着另一个 Azure 资源创建的 item id。
        let messages = [assistant_with_replay_state()];

        assert_eq!(
            responses_input(&messages, true).unwrap(),
            vec![json!({
                "type": "reasoning",
                "summary": [{"type": "summary_text", "text": "why"}],
                "content": [],
                "encrypted_content": "opaque"
            })]
        );
    }

    #[test]
    fn replay_can_be_disabled_for_the_stateless_retry() {
        let messages = [assistant_with_replay_state()];

        assert!(responses_input(&messages, false).unwrap().is_empty());
        assert!(has_responses_replay_state(&messages));
    }

    #[test]
    fn foreign_resource_rejections_are_detected_and_explained() {
        let upstream = Error::Provider(format!(
            "OpenAI Responses 400 Bad Request: {{\"error\":{{\"message\":\"The requested item was \
             created under a different Azure OpenAI resource. Use the same resource that created \
             the item to access it.\"}}}}"
        ));
        let readable = readable_foreign_resource_error(upstream).to_string();
        assert!(readable.contains("无状态请求"));
        assert!(readable.contains("different Azure OpenAI resource"));

        let unrelated = Error::Provider("OpenAI Responses 429 Too Many Requests".into());
        assert_eq!(
            readable_foreign_resource_error(unrelated).to_string(),
            "provider error: OpenAI Responses 429 Too Many Requests"
        );
    }
}
