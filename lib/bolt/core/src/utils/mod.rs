use std::{convert::TryInto, fs, path::Path, process::Command, sync::Arc, time};

use anyhow::*;
use duct::cmd;
use futures_util::future::{BoxFuture, FutureExt};
use indicatif::{ProgressBar, ProgressStyle};
use rivet_term::console::style;
use tokio::{net::TcpStream, sync::Mutex};

use crate::context::ProjectContext;

pub mod command_helper;
pub mod media_resize;

pub fn progress_bar(len: usize) -> ProgressBar {
	let pb = ProgressBar::new(len as u64);
	pb.set_style(
		ProgressStyle::default_bar()
			.progress_chars("=> ")
			.template("{spinner} {elapsed_precise:.bold} [{bar:23}] {pos}/{len} {wide_msg}"),
	);
	pb.enable_steady_tick(250);
	pb
}

pub async fn join_set_progress(mut join_set: tokio::task::JoinSet<Result<()>>) -> Result<()> {
	// Run progress bar
	let pb = progress_bar(join_set.len());
	let mut errors = Vec::new();
	while let Some(res) = join_set.join_next().await {
		let res = res?;
		match res {
			Result::Ok(_) => {}
			Result::Err(err) => {
				errors.push(err);
			}
		}
		pb.inc(1);
	}
	pb.finish();

	// Log all errors
	for err in &errors {
		rivet_term::status::error("Error", err);
	}

	// Return error
	if let Some(err) = errors.into_iter().next() {
		Err(err)
	} else {
		Ok(())
	}
}

#[derive(Clone)]
pub struct MultiProgress {
	progress_bar: ProgressBar,
	running: Arc<Mutex<Vec<String>>>,
}

impl MultiProgress {
	pub fn new(len: usize) -> MultiProgress {
		MultiProgress {
			progress_bar: progress_bar(len),
			running: Arc::new(Mutex::new(Vec::new())),
		}
	}

	pub async fn insert(&self, name: &str) {
		let mut running = self.running.lock().await;
		running.push(name.to_owned());
		self.update(&running);
	}

	pub async fn remove(&self, name: &str) {
		let mut running = self.running.lock().await;
		running.retain(|n| n != name);
		self.progress_bar.inc(1);
		self.update(&running);
	}

	pub fn finish(&self) {
		self.progress_bar.finish_with_message("");
	}

	fn update(&self, running: &Vec<String>) {
		self.progress_bar.set_message(running.join(", "));
	}
}

pub fn now() -> i64 {
	time::SystemTime::now()
		.duration_since(time::UNIX_EPOCH)
		.unwrap_or_else(|err| unreachable!("time is broken: {}", err))
		.as_millis()
		.try_into()
		.expect("now doesn't fit in i64")
}

/// Returns the modified timestamp of all files recursively.
pub fn deep_modified_ts(path: &Path) -> Result<u128> {
	let mut max_modified_ts = 0;
	deep_modified_ts_inner(path, &mut max_modified_ts)?;
	Ok(max_modified_ts)
}

fn deep_modified_ts_inner(path: &Path, max_modified_ts: &mut u128) -> Result<()> {
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let file_name = entry.file_name();
		let file_name = file_name.to_str().unwrap();
		let file_type = entry.file_type()?;

		// Skip non-source files
		if file_name.starts_with('.')
			|| file_name == "node_modules"
			|| file_name == "target"
			|| file_name == "dist"
		{
			continue;
		}

		// Recurse
		if file_type.is_dir() {
			deep_modified_ts_inner(&path.join(entry.path()), max_modified_ts)?;
		}

		// Check if file is newer
		if file_type.is_file() {
			let meta = entry.metadata()?;
			let modified_ts = meta
				.modified()?
				.duration_since(std::time::UNIX_EPOCH)?
				.as_millis();
			if modified_ts > *max_modified_ts {
				*max_modified_ts = modified_ts;
			}
		}
	}

	Ok(())
}

/// The `ringadingding` function is used to generate a bell sound in the
/// terminal. This function only works on Unix-based systems. It does this by
/// printing the ASCII bell character (`\x07`) to the standard output.
pub fn ringadingding() {
	#[cfg(unix)]
	{
		print!("\x07");
	}
}

const GET_GIT_BRANCH: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

pub async fn get_git_branch() -> String {
	GET_GIT_BRANCH
		.get_or_init(|| async {
			let git_cmd = Command::new("git")
				.arg("rev-parse")
				.arg("--abbrev-ref")
				.arg("HEAD")
				.output()
				.unwrap();
			assert!(git_cmd.status.success());
			String::from_utf8(git_cmd.stdout)
				.unwrap()
				.trim()
				.to_string()
		})
		.await
		.clone()
}

const GET_GIT_COMMIT: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

pub async fn get_git_commit() -> String {
	GET_GIT_COMMIT
		.get_or_init(|| async {
			let git_cmd = Command::new("git")
				.arg("rev-parse")
				.arg("HEAD")
				.output()
				.unwrap();
			assert!(git_cmd.status.success());
			String::from_utf8(git_cmd.stdout)
				.unwrap()
				.trim()
				.to_string()
		})
		.await
		.clone()
}

pub fn copy_dir_all<'a>(src: &'a Path, dst: &'a Path) -> BoxFuture<'a, tokio::io::Result<()>> {
	async move {
		tokio::fs::create_dir_all(&dst).await?;

		let mut dir = tokio::fs::read_dir(src).await?;
		while let Some(entry) = dir.next_entry().await? {
			if tokio::fs::read_dir(entry.path()).await.is_ok() {
				copy_dir_all(&entry.path(), &dst.join(entry.file_name())).await?;
			} else {
				tokio::fs::copy(entry.path(), dst.join(entry.file_name())).await?;
			}
		}

		tokio::io::Result::Ok(())
	}
	.boxed()
}

pub fn pick_port() -> u16 {
	portpicker::pick_unused_port().expect("no free ports")
}

pub struct DroppablePort {
	local_port: u16,
	handle: duct::Handle,
}

impl DroppablePort {
	pub async fn check_all(ports: &Vec<DroppablePort>) -> Result<()> {
		let mut futures = Vec::new();
		for port in ports {
			futures.push(port.check());
		}
		futures_util::future::try_join_all(futures).await?;
		Ok(())
	}

	pub async fn check(&self) -> Result<()> {
		// Check that handle didn't close
		if let Some(output) = self.handle.try_wait()? {
			eprintln!("{}", std::str::from_utf8(&output.stdout)?);
			bail!(
				"port forward closed prematurely: {}",
				std::str::from_utf8(&output.stderr)?
			);
		}

		let mut retries = 0;
		loop {
			// Probe port
			match TcpStream::connect(format!("127.0.0.1:{}", self.local_port)).await {
				Result::Ok(_) => {
					break;
				}
				Err(err) => {
					if retries > 20 {
						bail!("port forward failed ({}): {err}", self.local_port)
					}

					retries += 1;
				}
			}

			tokio::time::sleep(std::time::Duration::from_millis(200)).await;
		}

		Ok(())
	}

	pub async fn wait(self) -> Result<()> {
		tokio::task::spawn_blocking(move || {
			self.handle.wait()?;
			Ok(())
		})
		.await?
	}
}

// Safety implementation to ensure port forward is cleaned up
impl Drop for DroppablePort {
	fn drop(&mut self) {
		self.handle.kill().unwrap();
	}
}

pub fn kubectl_port_forward(
	ctx: &ProjectContext,
	namespace: &str,
	name: &str,
	(local_port, remote_port): (u16, u16),
) -> Result<DroppablePort> {
	// println!(
	// 	"Forwarding {}: {} -> {}",
	// 	service_name, local_port, remote_port
	// );
	let handle = cmd!(
		"kubectl",
		"port-forward",
		"-n",
		namespace,
		name,
		format!("{local_port}:{remote_port}")
	)
	.env("KUBECONFIG", ctx.gen_kubeconfig_path())
	.stdout_capture()
	.stderr_capture()
	.start()?;

	Ok(DroppablePort { local_port, handle })
}

pub async fn is_port_in_use(port: u16) -> bool {
	let address = format!("127.0.0.1:{}", port);
	TcpStream::connect(address).await.is_ok()
}

pub fn render_diff(indent: usize, patches: &json_patch::Patch) {
	use json_patch::PatchOperation::*;
	for patch in &**patches {
		match patch {
			Add(op) => {
				eprintln!(
					"{}{}{} {}",
					" ".repeat(indent),
					op.path.replace('/', "."),
					style(":").dim(),
					style("added").green().bold()
				);
			}
			Remove(op) => {
				eprintln!(
					"{}{}{} {}",
					" ".repeat(indent),
					op.path.replace('/', "."),
					style(":").dim(),
					style("removed").red().bold()
				);
			}
			Replace(op) => {
				eprintln!(
					"{}{}{} {}",
					" ".repeat(indent),
					op.path.replace('/', "."),
					style(":").dim(),
					style("changed").yellow().bold()
				);
			}
			_ => unreachable!(),
		}
	}
}

pub fn indent_string(s: &str, indent: &str) -> String {
	let mut out = String::with_capacity(s.len());
	let mut iter = s.split("\n");

	if let Some(chunk) = iter.next() {
		out.push_str(indent);
		out.push_str(chunk);
	}

	while let Some(chunk) = iter.next() {
		out.push_str("\n");
		out.push_str(indent);
		out.push_str(chunk);
	}

	out
}

pub fn colored_json(value: &serde_json::Value) -> Result<String> {
	colored_json_inner(value, colored_json::PrettyFormatter::new())
}

pub fn colored_json_ugly(value: &serde_json::Value) -> Result<String> {
	colored_json_inner(value, colored_json::CompactFormatter {})
}

fn colored_json_inner<T: serde_json::ser::Formatter>(
	value: &serde_json::Value,
	formatter: T,
) -> Result<String> {
	use colored_json::{ColorMode, ColoredFormatter, Output, Style, Styler};
	use serde::Serialize;

	let mut writer = Vec::<u8>::with_capacity(128);

	let mode = ColorMode::Auto(Output::StdOut);
	if mode.use_color() {
		let formatter = ColoredFormatter::with_styler(
			formatter,
			Styler {
				object_brackets: Style::new(),
				array_brackets: Style::new(),
				..Default::default()
			},
		);

		let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
		value.serialize(&mut serializer)?;
	} else {
		let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
		value.serialize(&mut serializer)?;
	}

	Ok(String::from_utf8_lossy(&writer).to_string())
}
