//! Serves the HTTP/2 Cursor Agent bidi `Run` stream.
//!
//! Cursor 3.x 默认用 HTTP/2 直接调用 `agent.v1.AgentService/Run`：
//! 请求体是一串 Connect envelope（AgentClientMessage），响应体同样是
//! 一串 Connect envelope（AgentServerMessage）。HTTP/1.1 下的
//! BidiAppend + RunSSE 组合只是该传输的退化形式。
use std::convert::Infallible;

use axum::{
    body::{Body, Bytes},
    extract::{Extension, State},
    http::{header, HeaderValue, Request, Response, StatusCode},
};
use bytes::{BufMut, BytesMut};
use futures_util::{Stream, StreamExt};
use prost::Message;
use tokio::sync::mpsc;

use crate::{
    api::cursor::{
        bidi,
        handlers::{header_text, parent_headers},
        proxy::{self, CursorProxy},
        run_sse::LocalRunGuard,
    },
    cursor::{
        protocol::{
            connect::{self, END_STREAM_FLAG},
            proto::agent::v1 as agent,
        },
        transport::{TransportHandle, TransportRegistry},
    },
    Error, Result,
};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn handler(
    State(registry): State<TransportRegistry>,
    Extension(proxy): Extension<CursorProxy>,
    request: Request<Body>,
) -> Result<Response<Body>> {
    let (parts, body) = request.into_parts();
    let header_request_id = header_text(&parts.headers, REQUEST_ID_HEADER)?
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let parent = parent_headers(&parts.headers)?;

    let mut stream = body.into_data_stream();
    let mut decoder = connect::FrameDecoder::new();
    let (flags, payload) = read_frame(&mut decoder, &mut stream)
        .await?
        .ok_or_else(|| Error::Protocol("Cursor Run stream closed before RunRequest".into()))?;
    if flags & END_STREAM_FLAG != 0 {
        return Err(Error::Protocol(
            "Cursor Run stream ended before RunRequest".into(),
        ));
    }
    let bytes = connect::message_payload(flags, payload.clone())?;
    let first = agent::AgentClientMessage::decode(bytes.as_ref())?;

    let Some(agent::agent_client_message::Message::RunRequest(run)) = first.message.as_ref() else {
        return Err(Error::Protocol(
            "first Cursor Run message must be a RunRequest".into(),
        ));
    };
    let model_id = model_id(run).ok_or_else(|| {
        Error::Protocol("first Cursor Run message must select a model".into())
    })?;
    let request_id = header_request_id
        .or_else(|| run.run_id.clone().filter(|id| !id.is_empty()))
        .ok_or_else(|| Error::Protocol("Cursor Run requires a request id".into()))?;
    let local = model_id.starts_with(crate::plugin::ADAPTER_ID_PREFIX)
        || registry.store().model(model_id).await?.is_some();
    if !local {
        let body = forwarded_body(flags, payload, &mut decoder, stream);
        return proxy::forward(Extension(proxy), Request::from_parts(parts, body)).await;
    }

    let conversation_id = run.conversation_id.clone();
    let handle = registry.get_or_create_for_append(&request_id, true).await?;
    if let Some(conversation_id) = conversation_id.as_deref() {
        handle.set_conversation_id(conversation_id)?;
    }
    if let Some(parent) = parent {
        handle.set_parent(parent)?;
    }
    {
        let _admission = handle.admit()?;
        bidi::deliver(&handle, 0, first).await?;
    }
    spawn_input_pump(handle.clone(), decoder, stream);

    let receiver = handle.subscribe();
    let mut response = Response::new(Body::from_stream(response_stream(receiver, handle)));
    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/connect+proto"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    Ok(response)
}

fn model_id(request: &agent::AgentRunRequest) -> Option<&str> {
    request
        .requested_model
        .as_ref()
        .map(|model| model.model_id.as_str())
        .filter(|model| !model.is_empty())
        .or_else(|| {
            request
                .model_details
                .as_ref()
                .map(|model| model.model_id.as_str())
                .filter(|model| !model.is_empty())
        })
}

async fn read_frame<S>(decoder: &mut connect::FrameDecoder, stream: &mut S) -> Result<Option<(u8, Bytes)>>
where
    S: Stream<Item = std::result::Result<Bytes, axum::Error>> + Unpin,
{
    loop {
        if let Some(frame) = decoder.pop()? {
            return Ok(Some(frame));
        }
        match stream.next().await {
            Some(Ok(chunk)) => decoder.push(&chunk),
            Some(Err(error)) => {
                return Err(Error::Protocol(format!(
                    "cannot read Cursor Run stream: {error}"
                )))
            }
            None => return Ok(None),
        }
    }
}

fn spawn_input_pump<S>(handle: TransportHandle, mut decoder: connect::FrameDecoder, mut stream: S)
where
    S: Stream<Item = std::result::Result<Bytes, axum::Error>> + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut seqno = 1i64;
        loop {
            let (flags, payload) = match read_frame(&mut decoder, &mut stream).await {
                Ok(Some(frame)) => frame,
                Ok(None) => break,
                Err(error) => {
                    tracing::warn!(%error, "failed to read Cursor Run input");
                    handle.disconnect().await;
                    break;
                }
            };
            if flags & END_STREAM_FLAG != 0 {
                break;
            }
            let bytes = match connect::message_payload(flags, payload) {
                Ok(bytes) => bytes,
                Err(error) => {
                    tracing::warn!(%error, "cannot decode Cursor Run message");
                    break;
                }
            };
            let message = match agent::AgentClientMessage::decode(bytes.as_ref()) {
                Ok(message) => message,
                Err(error) => {
                    tracing::warn!(%error, "invalid Cursor Run message");
                    break;
                }
            };
            if let Err(error) = bidi::deliver(&handle, seqno, message).await {
                tracing::warn!(%error, "Cursor Run input rejected");
                break;
            }
            seqno += 1;
        }
    });
}

fn response_stream(
    mut receiver: mpsc::UnboundedReceiver<Bytes>,
    handle: TransportHandle,
) -> impl Stream<Item = std::result::Result<Bytes, Infallible>> {
    async_stream::stream! {
        let mut guard = LocalRunGuard::new(handle);
        while let Some(frame) = receiver.recv().await {
            let terminal = frame
                .first()
                .is_some_and(|flags| flags & END_STREAM_FLAG != 0);
            if terminal {
                guard.complete();
            }
            yield Ok(frame);
            if terminal {
                return;
            }
        }
        guard.complete();
    }
}

/// 把已经读出的首帧和剩余字节重新拼成请求体，用于转发到 Cursor 官方上游。
fn forwarded_body<S>(
    flags: u8,
    payload: Bytes,
    decoder: &mut connect::FrameDecoder,
    mut stream: S,
) -> Body
where
    S: Stream<Item = std::result::Result<Bytes, axum::Error>> + Unpin + Send + 'static,
{
    let mut prefix = BytesMut::with_capacity(5 + payload.len());
    prefix.put_u8(flags);
    prefix.put_u32(payload.len() as u32);
    prefix.extend_from_slice(&payload);
    let prefix = prefix.freeze();
    let buffered = decoder.take_buffered();
    Body::from_stream(async_stream::stream! {
        yield Ok::<Bytes, axum::Error>(prefix);
        if !buffered.is_empty() {
            yield Ok(buffered);
        }
        while let Some(chunk) = stream.next().await {
            yield chunk;
        }
    })
}
