use std::{collections::HashMap, sync::Arc};

use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{
    sse::{Event, Sse},
    Response,
  },
  routing::{get, post},
  Json, Router,
};
use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio::io;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::{CancellationToken, PollSender};

use rmcp::{
  model::ClientJsonRpcMessage,
  service::{RxJsonRpcMessage, TxJsonRpcMessage},
  RoleServer, Service,
};

use crate::serve::AppState;
type SessionId = Arc<str>;
type TxStore =
  Arc<tokio::sync::RwLock<HashMap<SessionId, tokio::sync::mpsc::Sender<ClientJsonRpcMessage>>>>;

#[derive(Clone)]
pub(crate) struct McpState {
  txs: TxStore,
  transport_tx: tokio::sync::mpsc::UnboundedSender<SseServerTransport>,
  post_path: Arc<str>,
}

impl McpState {
  pub fn new(
    post_path: String,
  ) -> (
    Self,
    tokio::sync::mpsc::UnboundedReceiver<SseServerTransport>,
  ) {
    let (transport_tx, transport_rx) = tokio::sync::mpsc::unbounded_channel();
    (
      Self {
        txs: Default::default(),
        transport_tx,
        post_path: post_path.into(),
      },
      transport_rx,
    )
  }
}

fn session_id() -> SessionId {
  let id = format!("{:016x}", rand::random::<u128>());
  Arc::from(id)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostEventQuery {
  pub session_id: String,
}

async fn post_event_handler(
  State(app): State<Arc<AppState>>,
  Query(PostEventQuery { session_id }): Query<PostEventQuery>,
  Json(message): Json<ClientJsonRpcMessage>,
) -> Result<StatusCode, StatusCode> {
  let app = &app.mcp_state;
  tracing::debug!(session_id, ?message, "new client message");
  let tx = {
    let rg = app.txs.read().await;
    rg.get(session_id.as_str())
      .ok_or(StatusCode::NOT_FOUND)?
      .clone()
  };
  if tx.send(message).await.is_err() {
    tracing::error!("send message error");
    return Err(StatusCode::GONE);
  }
  Ok(StatusCode::ACCEPTED)
}

async fn sse_handler(
  State(app): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, io::Error>>>, Response<String>> {
  let session = session_id();
  tracing::info!(%session, "sse connection");
  use tokio_stream::wrappers::ReceiverStream;
  use tokio_util::sync::PollSender;
  let (from_client_tx, from_client_rx) = tokio::sync::mpsc::channel(64);
  let (to_client_tx, to_client_rx) = tokio::sync::mpsc::channel(64);
  let app = &app.mcp_state;
  app
    .txs
    .write()
    .await
    .insert(session.clone(), from_client_tx);
  let session = session.clone();
  let stream = ReceiverStream::new(from_client_rx);
  let sink = PollSender::new(to_client_tx);
  let transport = SseServerTransport {
    stream,
    sink,
    session_id: session.clone(),
    tx_store: app.txs.clone(),
  };
  let transport_send_result = app.transport_tx.send(transport);
  if transport_send_result.is_err() {
    tracing::warn!("send transport out error");
    let mut response =
      Response::new("fail to send out trasnport, it seems server is closed".to_string());
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    return Err(response);
  }
  let post_path = app.post_path.as_ref();
  let stream = futures::stream::once(futures::future::ok(
    Event::default()
      .event("endpoint")
      .data(format!("{post_path}?sessionId={session}")),
  ))
  .chain(ReceiverStream::new(to_client_rx).map(|message| {
    match serde_json::to_string(&message) {
      Ok(bytes) => Ok(Event::default().event("message").data(&bytes)),
      Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, e)),
    }
  }));
  Ok(Sse::new(stream))
}

pub struct SseServerTransport {
  stream: ReceiverStream<RxJsonRpcMessage<RoleServer>>,
  sink: PollSender<TxJsonRpcMessage<RoleServer>>,
  session_id: SessionId,
  tx_store: TxStore,
}

impl Sink<TxJsonRpcMessage<RoleServer>> for SseServerTransport {
  type Error = io::Error;

  fn poll_ready(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self
      .sink
      .poll_ready_unpin(cx)
      .map_err(std::io::Error::other)
  }

  fn start_send(
    mut self: std::pin::Pin<&mut Self>,
    item: TxJsonRpcMessage<RoleServer>,
  ) -> Result<(), Self::Error> {
    self
      .sink
      .start_send_unpin(item)
      .map_err(std::io::Error::other)
  }

  fn poll_flush(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    self
      .sink
      .poll_flush_unpin(cx)
      .map_err(std::io::Error::other)
  }

  fn poll_close(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Result<(), Self::Error>> {
    let inner_close_result = self
      .sink
      .poll_close_unpin(cx)
      .map_err(std::io::Error::other);
    if inner_close_result.is_ready() {
      let session_id = self.session_id.clone();
      let tx_store = self.tx_store.clone();
      tokio::spawn(async move {
        tx_store.write().await.remove(&session_id);
      });
    }
    inner_close_result
  }
}

impl Stream for SseServerTransport {
  type Item = RxJsonRpcMessage<RoleServer>;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    self.stream.poll_next_unpin(cx)
  }
}

#[derive(Debug, Clone)]
pub struct SseServerConfig {
  pub sse_path: String,
  pub post_path: String,
  pub ct: CancellationToken,
}

#[derive(Debug)]
pub struct SseServer {
  transport_rx: tokio::sync::mpsc::UnboundedReceiver<SseServerTransport>,
  pub config: SseServerConfig,
}

impl SseServer {
  /// Warning: This function creates a new SseServer instance with the provided configuration.
  /// `App.post_path` may be incorrect if using `Router` as an embedded router.
  pub fn new(config: SseServerConfig) -> (SseServer, Router<Arc<AppState>>, McpState) {
    let (app, transport_rx) = McpState::new(config.post_path.clone());
    let router = Router::new()
      .route(&config.sse_path, get(sse_handler))
      .route(&config.post_path, post(post_event_handler));

    let server = SseServer {
      transport_rx,
      config,
    };

    (server, router, app)
  }

  pub fn with_service<S, F>(mut self, service_provider: F) -> CancellationToken
  where
    S: Service<RoleServer>,
    F: Fn() -> S + Send + 'static,
  {
    use rmcp::service::ServiceExt;
    let ct = self.config.ct.clone();
    tokio::spawn(async move {
      while let Some(transport) = self.next_transport().await {
        let service = service_provider();
        let ct = self.config.ct.child_token();
        tokio::spawn(async move {
          let server = service.serve_with_ct(transport, ct).await?;
          server.waiting().await?;
          tokio::io::Result::Ok(())
        });
      }
    });
    ct
  }

  pub async fn next_transport(&mut self) -> Option<SseServerTransport> {
    self.transport_rx.recv().await
  }
}

impl Stream for SseServer {
  type Item = SseServerTransport;

  fn poll_next(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<Self::Item>> {
    self.transport_rx.poll_recv(cx)
  }
}
