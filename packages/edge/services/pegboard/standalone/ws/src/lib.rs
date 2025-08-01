use std::{
	collections::HashMap,
	net::SocketAddr,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use chirp_workflow::prelude::*;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use serde_json::json;
use tokio::{
	net::{TcpListener, TcpStream},
	sync::{Mutex, RwLock},
};
use tokio_tungstenite::{
	tungstenite::protocol::{
		frame::{coding::CloseCode, CloseFrame},
		Message,
	},
	WebSocketStream,
};

use pegboard::protocol;

const UPDATE_PING_INTERVAL: Duration = Duration::from_secs(3);

struct Connection {
	// Set after init packet is received
	workflow_id: RwLock<Option<Uuid>>,
	protocol_version: u16,
	flavor: protocol::ClientFlavor,
	tx: Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>,
	update_ping: AtomicBool,
}

type Connections = HashMap<Uuid, Arc<Connection>>;

pub async fn start(config: rivet_config::Config, pools: rivet_pools::Pools) -> GlobalResult<()> {
	run_from_env(config.clone(), pools.clone()).await
}

#[tracing::instrument(skip_all)]
pub async fn run_from_env(
	config: rivet_config::Config,
	pools: rivet_pools::Pools,
) -> GlobalResult<()> {
	let client = chirp_client::SharedClient::from_env(pools.clone())?.wrap_new("pegboard-ws");
	let cache = rivet_cache::CacheInner::from_env(&config, pools.clone())?;
	let ctx = StandaloneCtx::new(
		db::DatabaseFdbSqliteNats::from_pools(pools.clone())?,
		config,
		rivet_connection::Connection::new(client, pools, cache),
		"pegboard-ws",
	)
	.await?;

	let conns: Arc<RwLock<Connections>> = Arc::new(RwLock::new(HashMap::new()));

	let host = ctx.config().server()?.rivet.pegboard.host();
	let port = ctx.config().server()?.rivet.pegboard.port();
	let addr = SocketAddr::from((host, port));

	let listener = TcpListener::bind(addr).await?;
	tracing::info!(?port, ?port, "pegboard ws server listening");

	// None of these should ever exit
	//
	// If these do exit, then the `handle_connection` task will run indefinitely and never
	// send/receive anything to clients. Client workflows will then expire because of their ping,
	// their workflow will complete, and clients will be unusable unless they reconnect.
	tokio::join!(
		socket_thread(&ctx, conns.clone(), listener),
		msg_thread(&ctx, conns.clone()),
		update_ping_thread(&ctx, conns.clone()),
	);

	Ok(())
}

#[tracing::instrument(skip_all)]
async fn socket_thread(
	ctx: &StandaloneCtx,
	conns: Arc<RwLock<Connections>>,
	listener: TcpListener,
) {
	loop {
		match listener.accept().await {
			Ok((stream, addr)) => handle_connection(ctx, conns.clone(), stream, addr).await,
			Err(err) => tracing::error!(?err, "failed to connect websocket"),
		}
	}
}

#[tracing::instrument(skip_all)]
async fn handle_connection(
	ctx: &StandaloneCtx,
	conns: Arc<RwLock<Connections>>,
	raw_stream: TcpStream,
	addr: SocketAddr,
) {
	tracing::debug!(?addr, "new connection");

	let ctx = ctx.clone();

	tokio::spawn(async move {
		let (ws_stream, url_data) = match setup_connection(raw_stream, addr).await {
			Ok(x) => x,
			Err(err) => {
				tracing::error!(?addr, ?err, "setup connection failed");
				return;
			}
		};

		if let Err(err) = handle_connection_inner(&ctx, conns.clone(), ws_stream, url_data).await {
			tracing::error!(?addr, ?err, "handle connection inner failed");
		}

		tracing::info!(client_id=?url_data.client_id, "client connection closed");

		// Clean up
		let conn = conns.write().await.remove(&url_data.client_id);
		if let Some(conn) = conn {
			let close_frame = CloseFrame {
				code: CloseCode::Normal,
				reason: "handle_connection_inner event loop closed".into(),
			};
			if let Err(err) = conn
				.tx
				.lock()
				.await
				.send(Message::Close(Some(close_frame)))
				.await
			{
				tracing::error!(?addr, ?err, "failed closing socket");
			}
		}
	});
}

#[tracing::instrument(skip_all)]
async fn setup_connection(
	raw_stream: TcpStream,
	addr: SocketAddr,
) -> GlobalResult<(WebSocketStream<TcpStream>, UrlData)> {
	let mut uri = None;
	let ws_stream = tokio_tungstenite::accept_hdr_async(
		raw_stream,
		|req: &tokio_tungstenite::tungstenite::handshake::server::Request, res| {
			// Bootleg way of reading the uri
			uri = Some(req.uri().clone());

			tracing::debug!(?addr, ?uri, "handshake");

			Ok(res)
		},
	)
	.await?;

	// Parse URL
	let uri = unwrap!(uri, "socket has no associated request");
	let url_data = parse_url(addr, uri)?;

	Ok((ws_stream, url_data))
}

#[tracing::instrument(skip_all)]
async fn handle_connection_inner(
	ctx: &StandaloneCtx,
	conns: Arc<RwLock<Connections>>,
	ws_stream: WebSocketStream<TcpStream>,
	UrlData {
		protocol_version,
		client_id,
		flavor,
	}: UrlData,
) -> GlobalResult<()> {
	let (tx, mut rx) = ws_stream.split();

	tracing::info!(?client_id, "new client connection");

	let conn = Arc::new(Connection {
		workflow_id: RwLock::new(None),
		protocol_version,
		flavor,
		tx: Mutex::new(tx),
		update_ping: AtomicBool::new(false),
	});
	let mut workflow_id_guard = conn.workflow_id.write().await;

	// Store connection
	{
		let mut conns = conns.write().await;
		if let Some(old_conn) = conns.insert(client_id, conn.clone()) {
			tracing::warn!(
				?client_id,
				"client already connected, closing old connection"
			);

			let close_frame = CloseFrame {
				code: CloseCode::Normal,
				reason: "client already connected, closing old connection".into(),
			};
			old_conn
				.tx
				.lock()
				.await
				.send(Message::Close(Some(close_frame)))
				.await?;
		}
	}

	// Only create the client after receiving the init packet to prevent a race condition
	let workflow_id = if let Some(msg) = rx.next().await {
		match msg? {
			Message::Binary(buf) => {
				let packet = protocol::ToServer::deserialize(protocol_version, &buf)?;

				let workflow_id = if let protocol::ToServer::Init { .. } = &packet {
					// Spawn a new client workflow if one doesn't already exist
					let workflow_id = ctx
						.workflow(pegboard::workflows::client::Input { client_id, flavor })
						.tag("client_id", client_id)
						.unique()
						.dispatch()
						.await?;

					*workflow_id_guard = Some(workflow_id);
					drop(workflow_id_guard);

					workflow_id
				} else {
					bail!("unexpected initial packet: {packet:?}");
				};

				// Forward to client wf
				ctx.signal(packet)
					.to_workflow_id(workflow_id)
					.send()
					.await?;

				workflow_id
			}
			Message::Close(_) => {
				bail!("socket closed {client_id}");
			}
			msg => bail!("unexpected initial message: {msg:?}"),
		}
	} else {
		bail!("stream closed {client_id}");
	};

	// Receive messages from socket
	while let Some(msg) = rx.next().await {
		match msg? {
			Message::Binary(buf) => {
				let packet = protocol::ToServer::deserialize(protocol_version, &buf)?;

				// Forward to client wf
				ctx.signal(packet)
					.to_workflow_id(workflow_id)
					.send()
					.await?;
			}
			Message::Ping(_) => {
				conn.update_ping.store(true, Ordering::Relaxed);
			}
			Message::Close(_) => {
				bail!("socket closed {client_id}");
			}
			msg => tracing::warn!(?client_id, ?msg, "unexpected message"),
		}
	}

	bail!("stream closed {client_id}");

	// Only way I could figure out to help the complier infer type
	#[allow(unreachable_code)]
	GlobalResult::Ok(())
}

#[tracing::instrument(skip_all)]
async fn update_ping_thread(ctx: &StandaloneCtx, conns: Arc<RwLock<Connections>>) {
	loop {
		match update_ping_thread_inner(ctx, conns.clone()).await {
			Ok(_) => {
				tracing::warn!("update ping thread thread exited early");
			}
			Err(err) => {
				tracing::error!(?err, "update ping thread error");
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
	}
}

/// Updates the ping of all clients requesting a ping update at once.
#[tracing::instrument(skip_all)]
async fn update_ping_thread_inner(
	ctx: &StandaloneCtx,
	conns: Arc<RwLock<Connections>>,
) -> GlobalResult<()> {
	loop {
		tokio::time::sleep(UPDATE_PING_INTERVAL).await;

		let clients = {
			let conns = conns.read().await;

			// Select all clients that required a ping update
			let futs = conns
				.iter()
				.filter_map(|(client_id, conn)| {
					conn.update_ping
						.swap(false, Ordering::Relaxed)
						.then(|| async move {
							// Read workflow id
							(*client_id, conn.flavor, *conn.workflow_id.read().await)
						})
				})
				.collect::<Vec<_>>();

			futures_util::stream::iter(futs)
				.buffer_unordered(32)
				.collect::<Vec<_>>()
				.await
		};

		if clients.is_empty() {
			continue;
		}

		// TODO: Parallelize on the op level
		// Update ping in fdb idx for each client
		for (client_id, flavor, workflow_id) in clients {
			let Some(workflow_id) = workflow_id else {
				continue;
			};

			let Some(wf) = ctx
				.workflow::<pegboard::workflows::client::Input>(workflow_id)
				.get()
				.await?
			else {
				tracing::error!(?client_id, ?workflow_id, "workflow does not exist");
				continue;
			};

			// Only update ping if the workflow is not dead
			if wf.has_wake_condition {
				ctx.op(pegboard::ops::client::update_allocation_idx::Input {
					client_id,
					client_workflow_id: workflow_id,
					flavor,
					action: pegboard::ops::client::update_allocation_idx::Action::UpdatePing,
				})
				.await?;
			}
		}
	}
}

#[tracing::instrument(skip_all)]
async fn msg_thread(ctx: &StandaloneCtx, conns: Arc<RwLock<Connections>>) {
	loop {
		match msg_thread_inner(ctx, conns.clone()).await {
			Ok(_) => {
				tracing::warn!("msg thread exited early");
			}
			Err(err) => {
				tracing::error!(?err, "msg thread error");
			}
		}

		tokio::time::sleep(std::time::Duration::from_secs(2)).await;
	}
}

#[tracing::instrument(skip_all)]
async fn msg_thread_inner(
	ctx: &StandaloneCtx,
	conns: Arc<RwLock<Connections>>,
) -> GlobalResult<()> {
	// Listen for commands from client workflows
	let mut sub = ctx
		.subscribe::<pegboard::workflows::client::ToWs>(&json!({}))
		.await?;
	let mut close_sub = ctx
		.subscribe::<pegboard::workflows::client::CloseWs>(&json!({}))
		.await?;

	loop {
		tokio::select! {
			msg = sub.next() => {
				let msg = msg?;

				{
					let conns = conns.read().await;

					// Send command to socket
					if let Some(conn) = conns.get(&msg.client_id) {
						let buf = msg.inner.serialize(conn.protocol_version)?;
						conn.tx.lock().await.send(Message::Binary(buf)).await?;
					} else {
						tracing::debug!(
							client_id=?msg.client_id,
							"received command for client that isn't connected, ignoring"
						);
					}
				}
			}
			msg = close_sub.next() => {
				let msg = msg?;

				{
					let conns = conns.read().await;

					// Close socket
					if let Some(conn) = conns.get(&msg.client_id) {
						tracing::info!(client_id = ?msg.client_id, "received close ws event, closing socket");

						let close_frame = CloseFrame {
							code: CloseCode::Normal,
							reason: "received close ws event".into(),
						};
						conn.tx.lock().await.send(Message::Close(Some(close_frame))).await?;
					} else {
						tracing::debug!(
							client_id=?msg.client_id,
							"received close command for client that isn't connected, ignoring"
						);
					}
				}
			}
		}
	}
}

#[derive(Clone, Copy)]
struct UrlData {
	protocol_version: u16,
	client_id: Uuid,
	flavor: protocol::ClientFlavor,
}

fn parse_url(addr: SocketAddr, uri: hyper::Uri) -> GlobalResult<UrlData> {
	let url = url::Url::parse(&format!("ws://{addr}{uri}"))?;

	// Get protocol version from last path segment
	let last_segment = unwrap!(
		unwrap!(url.path_segments(), "invalid url").last(),
		"no path segments"
	);
	ensure!(last_segment.starts_with('v'), "invalid protocol version");
	let protocol_version = last_segment[1..].parse::<u16>()?;

	// Read client_id from query parameters
	let client_id = unwrap!(
		url.query_pairs()
			.find_map(|(n, v)| (n == "client_id").then_some(v)),
		"missing `client_id` query parameter"
	);
	let client_id = util::uuid::parse(client_id.as_ref())?;

	let flavor = unwrap!(
		url.query_pairs()
			.find_map(|(n, v)| (n == "flavor").then_some(v)),
		"missing `flavor` query parameter"
	);
	let flavor = flavor.parse()?;

	Ok(UrlData {
		protocol_version,
		client_id,
		flavor,
	})
}
