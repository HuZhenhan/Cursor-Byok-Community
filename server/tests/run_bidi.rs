//! Verifies the HTTP/2 AgentService/Run bidi bridge.
#[path = "support/fake_provider.rs"]
mod fake_provider;
#[path = "support/fixtures.rs"]
mod fixtures;

use std::sync::Arc;

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use futures_util::StreamExt;
use cursor_server::{
    api::cursor,
    cursor::{
        prompting::{PromptAssets, PromptCompiler},
        protocol::{connect, proto::agent::v1 as pb},
        transport::TransportRegistry,
    },
    model::{ModelConfigInput, ModelType, OPENAI_CHAT_ENDPOINT},
    network::NetworkClients,
    provider::{FinishReason, ModelEvent},
};
use prost::Message;
use tower::ServiceExt;

fn model_input(model_id: &str) -> ModelConfigInput {
    ModelConfigInput {
        sort_order: 1,
        display_name: "Qwen Flash".into(),
        group_name: None,
        model_type: ModelType::OpenAi,
        base_url: "https://example.com/v1".into(),
        use_full_url: false,
        api_key: "test-key".into(),
        tooltip_data: "模型介绍".into(),
        model_id: model_id.into(),
        reasoning_effort: None,
        openai_endpoint: OPENAI_CHAT_ENDPOINT.into(),
        openai_extra_params_enabled: false,
        openai_extra_params: serde_json::json!({}),
        custom_headers_enabled: false,
        custom_headers: serde_json::json!({}),
        anthropic_extra_params_enabled: false,
        anthropic_extra_params: serde_json::json!({}),
        context_window_tokens: None,
        max_completion_tokens: None,
        anthropic_max_tokens: None,
        anthropic_thinking_effort: None,
        thinking_budget_tokens: None,
    }
}

fn run_request(request_id: &str, model_id: &str) -> pb::AgentClientMessage {
    pb::AgentClientMessage {
        message: Some(pb::agent_client_message::Message::RunRequest(
            pb::AgentRunRequest {
                action: Some(pb::ConversationAction {
                    action: Some(pb::conversation_action::Action::UserMessageAction(
                        pb::UserMessageAction {
                            user_message: Some(pb::UserMessage {
                                text: "hello".into(),
                                message_id: "bidi-user-message".into(),
                                mode: pb::AgentMode::Agent as i32,
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    )),
                    ..Default::default()
                }),
                conversation_id: Some("bidi-conversation".into()),
                run_id: Some(request_id.into()),
                requested_model: Some(pb::RequestedModel {
                    model_id: model_id.into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )),
    }
}

#[tokio::test]
async fn run_stream_serves_local_model_and_closes_with_end_stream() {
    let (_directory, store) = fixtures::temp_store().await;
    let created = store.create_model(&model_input("test-model")).await.unwrap();
    let provider = fake_provider::FakeProvider::default();
    provider.push(vec![
        ModelEvent::Start {
            model_call_id: "bidi-call".into(),
        },
        ModelEvent::TextStart,
        ModelEvent::TextDelta("hello".into()),
        ModelEvent::TextEnd,
        ModelEvent::Done(FinishReason::Stop),
    ]);
    let assets = PromptAssets::load(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("prompt/cursor")
            .as_path(),
    )
    .unwrap();
    let clients = NetworkClients::new(store.clone());
    let registry = TransportRegistry::new(store, Arc::new(provider), PromptCompiler::new(assets));

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<
        std::result::Result<bytes::Bytes, std::io::Error>,
    >();
    sender
        .send(Ok(connect::encode_message(&run_request(
            "bidi-request",
            &created.model_hash,
        ))
        .unwrap()))
        .unwrap();
    let request = Request::post("/agent.v1.AgentService/Run")
        .header(header::CONTENT_TYPE, "application/connect+proto")
        .header("x-request-id", "bidi-request")
        .body(Body::from_stream(
            tokio_stream::wrappers::UnboundedReceiverStream::new(receiver),
        ))
        .unwrap();
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        cursor::router(registry, clients).unwrap().oneshot(request),
    )
    .await
    .expect("router did not answer the Run request")
    .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/connect+proto"
    );

    let mut output = response.into_body().into_data_stream();
    let mut decoder = connect::FrameDecoder::new();
    let mut text = String::new();
    let mut terminal = false;
    while !terminal {
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(5), output.next())
            .await
            .expect("Run stream stalled")
            .expect("Run stream closed before EndStream")
            .unwrap();
        decoder.push(&chunk);
        while let Some((flags, payload)) = decoder.pop().unwrap() {
            if flags & connect::END_STREAM_FLAG != 0 {
                terminal = true;
                break;
            }
            let message = pb::AgentServerMessage::decode(payload).unwrap();
            match message.message {
                Some(pb::agent_server_message::Message::KvServerMessage(kv)) => {
                    // 客户端的 kv ack 必须走同一条 Run 请求流回传。
                    sender
                        .send(Ok(connect::encode_message(&kv_ack(kv.id)).unwrap()))
                        .unwrap();
                }
                Some(pb::agent_server_message::Message::InteractionUpdate(update)) => {
                    if let Some(pb::interaction_update::Message::TextDelta(delta)) = update.message
                    {
                        text.push_str(&delta.text);
                    }
                }
                _ => {}
            }
        }
    }
    assert!(terminal, "Run must close the response with an end-stream frame");
    assert_eq!(text, "hello");
}

fn kv_ack(id: u32) -> pb::AgentClientMessage {
    pb::AgentClientMessage {
        message: Some(pb::agent_client_message::Message::KvClientMessage(
            pb::KvClientMessage {
                id,
                message: Some(pb::kv_client_message::Message::SetBlobResult(
                    pb::SetBlobResult { error: None },
                )),
            },
        )),
    }
}
