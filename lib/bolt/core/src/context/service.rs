use std::{
	collections::HashMap,
	hash::{Hash, Hasher},
	net::Ipv4Addr,
	path::{Path, PathBuf},
	sync::{Arc, Weak},
};

use anyhow::{ensure, Context, Result};
use async_recursion::async_recursion;
use indexmap::IndexMap;
use tokio::{fs, process::Command, sync::RwLock};

use super::BuildOptimization;
use crate::{
	config::{
		self,
		service::{RuntimeKind, ServiceKind},
	},
	context::{self, BuildContext, ProjectContext, RunContext},
	dep::{docker, k8s, terraform},
	utils,
};

pub type ServiceContext = Arc<ServiceContextData>;

pub struct ServiceContextData {
	project: RwLock<Weak<context::project::ProjectContextData>>,
	/// If this is overriding a service from an additional root, then this will be specified.
	overridden_service: Option<ServiceContext>,
	config: config::service::ServiceConfig,
	path: PathBuf,
	workspace_path: PathBuf,
	cargo: Option<config::service::CargoConfig>,
}

impl PartialEq for ServiceContextData {
	fn eq(&self, other: &Self) -> bool {
		self.config.service.name == other.config.service.name
	}
}

impl Hash for ServiceContextData {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.config.service.name.hash(state);
	}
}

impl ServiceContextData {
	/// Sets the reference to the project once the project context is finished initiating.
	#[async_recursion]
	pub(in crate::context) async fn set_project(
		&self,
		project: Weak<context::project::ProjectContextData>,
	) {
		*self.project.write().await = project.clone();
		if let Some(overridden_service) = &self.overridden_service {
			overridden_service.set_project(project.clone()).await;
		}
	}

	pub async fn project(&self) -> ProjectContext {
		self.project
			.read()
			.await
			.upgrade()
			.expect("missing project")
	}

	pub fn config(&self) -> &config::service::ServiceConfig {
		&self.config
	}

	pub fn path(&self) -> &Path {
		self.path.as_path()
	}

	pub fn workspace_path(&self) -> &Path {
		self.workspace_path.as_path()
	}
}

impl ServiceContextData {
	pub async fn from_path(
		project: Weak<context::project::ProjectContextData>,
		svc_ctxs_map: &HashMap<String, ServiceContext>,
		workspace_path: &Path,
		path: &Path,
	) -> Option<ServiceContext> {
		// Read config
		let config_path = path.join("Service.toml");
		let config_str = match fs::read_to_string(config_path).await {
			Ok(v) => v,
			Err(_) => return None,
		};
		let config = match toml::from_str::<config::service::ServiceConfig>(&config_str) {
			Result::Ok(x) => x,
			Result::Err(err) => {
				if let Some(span) = err.span().filter(|span| span.start != span.end) {
					panic!(
						"failed to parse service config ({}): {}\n\n{}\n",
						path.display(),
						err.message(),
						&config_str[span.clone()],
					);
				} else {
					panic!(
						"failed to parse service config ({}): {}",
						path.display(),
						err.message(),
					);
				}
			}
		};

		let cargo_path = path.join("Cargo.toml");
		let cargo = match fs::read_to_string(&cargo_path).await {
			Ok(v) => Some(
				toml::from_str::<config::service::CargoConfig>(&v).expect(&format!(
					"failed to read cargo config: {}",
					cargo_path.display()
				)),
			),
			Err(_) => None,
		};

		// Read overridden service
		let overridden_service = svc_ctxs_map.get(&config.service.name).cloned();

		Some(Arc::new(ServiceContextData::new(
			project,
			overridden_service,
			config,
			workspace_path,
			path,
			cargo,
		)))
	}

	fn new(
		project: Weak<context::project::ProjectContextData>,
		overridden_service: Option<ServiceContext>,
		config: config::service::ServiceConfig,
		workspace_path: &Path,
		path: &Path,
		cargo: Option<config::service::CargoConfig>,
	) -> ServiceContextData {
		// Build context
		let ctx = ServiceContextData {
			project: RwLock::new(project),
			overridden_service,
			config,
			path: path.to_owned(),
			workspace_path: workspace_path.to_owned(),
			cargo,
		};

		// Validate context
		ctx.validate();

		ctx
	}

	fn validate(&self) {
		// Max service name length is 63 characters (RFC 1123), so we need this
		// name to be much shorter so we can append things to it.
		assert!(
			self.config.service.name.len() <= 42,
			"name '{}' must be less than 42 characters",
			self.config.service.name
		);

		let component_class = self.config.kind.component_class();
		assert!(
			self.config
				.runtime
				.supports_component_class(&component_class),
			"runtime does not support component class"
		);

		// TODO: Validate that all services in `config.databases` are actually databases
	}
}

impl ServiceContextData {
	pub fn name(&self) -> String {
		self.config.service.name.clone()
	}

	pub fn name_snake(&self) -> String {
		self.config.service.name_snake()
	}

	pub fn name_screaming_snake(&self) -> String {
		self.config.service.name_screaming_snake()
	}

	pub fn name_camel(&self) -> String {
		self.config.service.name_camel_case()
	}

	pub fn name_single_word(&self) -> String {
		self.config.service.name_single_word()
	}

	pub fn name_matches_pattern(&self, pattern: impl AsRef<str>) -> bool {
		wildmatch::WildMatch::new(pattern.as_ref()).matches(&self.config.service.name)
	}

	pub fn cargo_name(&self) -> Option<&str> {
		self.cargo.as_ref().map(|x| x.package.name.as_str())
	}
}

impl ServiceContextData {
	pub async fn relative_path(&self) -> PathBuf {
		self.path()
			.strip_prefix(self.project().await.path())
			.expect("strip path")
			.to_owned()
	}

	pub async fn relative_gen_path(&self) -> PathBuf {
		self.gen_path()
			.await
			.strip_prefix(self.project().await.path())
			.expect("strip path")
			.to_owned()
	}

	pub async fn gen_path(&self) -> PathBuf {
		self.project()
			.await
			.gen_path()
			.join("svc")
			.join(self.name())
	}

	pub fn migrations_path(&self) -> PathBuf {
		self.path().join("migrations")
	}

	/// Path to the executable binary.
	pub async fn rust_bin_path(&self, optimization: &BuildOptimization) -> PathBuf {
		self.project()
			.await
			.cargo_target_dir()
			.join(match optimization {
				BuildOptimization::Release => "release",
				BuildOptimization::Debug => "debug",
			})
			.join(self.cargo_name().expect("no cargo name"))
	}
}

// Deploy meta
impl ServiceContextData {
	pub fn has_migrations(&self) -> bool {
		matches!(
			self.config().runtime,
			RuntimeKind::CRDB { .. } | RuntimeKind::ClickHouse { .. }
		)
	}

	pub fn crdb_db_name(&self) -> String {
		self.name_snake()
	}

	pub fn clickhouse_db_name(&self) -> String {
		self.name_snake()
	}

	pub fn redis_db_name(&self) -> String {
		self.config()
			.service
			.name
			.clone()
			.strip_prefix("redis-")
			.unwrap()
			.to_string()
	}

	pub async fn s3_bucket_name(&self) -> String {
		// Include the namespace name since it needs to be globally unique
		format!("{}-{}", self.project().await.ns_id(), self.name())
	}

	pub fn is_monolith_worker(&self) -> bool {
		self.config().service.name == "monolith-worker"
			|| self.config().service.name == "monolith-workflow-worker"
	}

	pub fn depends_on_nomad_api(&self) -> bool {
		true
		// self.name().starts_with("job-")
		// 	|| self.name().starts_with("nomad-")
		// 	|| self.name() == "api-job"
		// 	|| self.name() == "mm-lobby-create"
		// 	|| self.name() == "mm-lobby-stop"
		// 	|| self.name() == "job-gc"
		// 	|| self.name() == "nomad-monitor"
		// 	|| self.name() == "playground"
	}

	pub fn depends_on_traefik_api(&self) -> bool {
		true
		// self.name() == "job-run"
	}

	pub fn depends_on_prometheus_api(&self) -> bool {
		true
		// self.name() == "job-run-metrics-log"
	}

	pub fn depends_on_clickhouse(&self) -> bool {
		true
	}

	pub fn depends_on_jwt_key_private(&self) -> bool {
		true
		// self.name() == "token-create"
	}

	pub fn depends_on_sendgrid_key(&self, project_ctx: &ProjectContext) -> bool {
		// self.name() == "email-send"
		project_ctx.ns().email.as_ref().map_or(false, |x| {
			matches!(x.provider, config::ns::EmailProvider::SendGrid { .. })
		})
	}

	pub fn depends_on_s3(&self) -> bool {
		true
		// self.name().starts_with("upload-")
	}

	pub fn depends_on_s3_backfill(&self) -> bool {
		self.name() == "upload-provider-fill"
	}

	pub fn depends_on_cloudflare(&self) -> bool {
		true
		// 	|| self.name() == "cf-custom-hostname-create"
		// 	|| self.name() == "cf-custom-hostname-delete"
		// 	|| self.name() == "api-cloud"
	}

	pub fn depends_on_captcha(&self) -> bool {
		// TODO:
		true
	}

	pub fn depends_on_infra(&self) -> bool {
		self.name() == "cluster-worker"
			|| self.name() == "monolith-worker"
			|| self.name() == "monolith-workflow-worker"
	}

	pub fn depends_on_cluster_config(&self) -> bool {
		self.name() == "cluster-default-update"
	}

	pub fn depends_on_provision_margin(&self) -> bool {
		self.name() == "cluster-autoscale"
	}
}

impl ServiceContextData {
	pub fn enable_tokio_console(&self) -> bool {
		// TODO: This seems to have a significant performance impact
		false
	}
}

// Build info
#[derive(Debug, Clone)]
pub enum ServiceBuildPlan {
	/// Build the service locally.
	BuildLocally {
		/// Absolute path to the executable on the host system.
		exec_path: PathBuf,
	},

	/// Build exists on Docker.
	ExistingUploadedBuild { image_tag: String },

	/// Build the service and upload to Docker.
	BuildAndUpload {
		/// Push location (local repo)
		push_image_tag: String,
		/// Pull location (inside of k8s)
		pull_image_tag: String,
	},
}

impl ServiceContextData {
	/// Determines if this service needs to be recompiled.
	pub async fn build_plan(&self, build_context: &BuildContext) -> Result<ServiceBuildPlan> {
		let project_ctx = self.project().await;

		// Check if build exists on docker.io
		let pub_image_tag = self.docker_image_tag(&project_ctx, "docker.io/rivetgg/")?;
		if docker::cli::container_exists(&pub_image_tag).await {
			return Ok(ServiceBuildPlan::ExistingUploadedBuild {
				image_tag: pub_image_tag,
			});
		}

		// Check if build exists in config repo
		let image_tag = self.docker_image_tag(&project_ctx, &project_ctx.ns().docker.repository)?;
		if docker::cli::container_exists(&image_tag).await {
			return Ok(ServiceBuildPlan::ExistingUploadedBuild { image_tag });
		}

		if project_ctx.build_svcs_locally() {
			// Derive the build path
			let optimization = match &build_context {
				BuildContext::Bin { optimization } => optimization,
				BuildContext::Test { .. } => &BuildOptimization::Debug,
			};
			let output_path = self.rust_bin_path(optimization).await;

			// Rust libs always attempt to rebuild (handled by cargo)
			Ok(ServiceBuildPlan::BuildLocally {
				exec_path: output_path,
			})
		} else {
			let (push_repo, pull_repo) = project_ctx.docker_repos().await;

			Ok(ServiceBuildPlan::BuildAndUpload {
				push_image_tag: self.docker_image_tag(&project_ctx, &push_repo)?,
				pull_image_tag: self.docker_image_tag(&project_ctx, &pull_repo)?,
			})
		}
	}
}

// Dependencies
impl ServiceContextData {
	#[async_recursion]
	pub async fn dependencies(&self, run_context: &RunContext) -> Vec<ServiceContext> {
		let project = self.project().await;

		let all_svcs = project.all_services().await;

		let mut dep_ctxs = Vec::<ServiceContext>::new();

		// Add operation dependencies from Cargo.toml
		//
		// You cannot depend on these services from the Service.toml, only as a Cargo dependency
		if let Some(cargo) = &self.cargo {
			let svcs = cargo
				.dependencies
				.iter()
				// Add dev dependencies for tests
				.chain(
					cargo
						.dev_dependencies
						.iter()
						.filter(|_| matches!(run_context, RunContext::Test { .. })),
				)
				.filter_map(|(name, dep)| {
					if let config::service::CargoDependency::Path { .. } = dep {
						Some(name)
					} else {
						None
					}
				})
				// Remove overridden service from deps list
				.filter(|name| {
					self.overridden_service
						.as_ref()
						.map(|osvc| &&osvc.name() != name)
						.unwrap_or(true)
				})
				.filter_map(|name| {
					all_svcs
						.iter()
						.find(|x| x.cargo_name().map(|x| x == name).unwrap_or_default())
						.cloned()
				});
			// TODO: Use the path to find the service instead of the name. This is difficult with multiple roots.
			// .filter_map(|path| {
			// 	let absolute_path = svc_path.join(path);
			// 	all_svcs
			// 		.iter()
			// 		.filter(|x| x.path() == absolute_path)
			// 		.next()
			// 		.cloned()
			// });

			dep_ctxs.extend(svcs);
		}

		// Inherit dependencies from the service that was overridden
		if let Some(overridden_svc) = &self.overridden_service {
			dep_ctxs.extend(overridden_svc.dependencies(run_context).await);
		}

		// Check that these are services you can explicitly depend on in the Service.toml
		for dep in &dep_ctxs {
			if matches!(run_context, RunContext::Service { .. })
				&& !self.config().service.test_only
				&& !self.config().service.load_test
				&& dep.config().service.test_only
			{
				panic!(
					"{} -> {}: cannot depend on a `service.test-only` service outside of `test-dependencies` or if `service.load-test = true`",
					self.name(),
					dep.name()
				);
			}

			let can_depend = if self.is_monolith_worker() {
				matches!(
					dep.config().kind,
					ServiceKind::Database { .. }
						| ServiceKind::Cache { .. }
						| ServiceKind::Operation { .. }
						| ServiceKind::Package { .. }
						| ServiceKind::Consumer { .. }
				)
			} else if matches!(self.config().kind, ServiceKind::Api { .. }) {
				matches!(
					dep.config().kind,
					ServiceKind::Database { .. }
						| ServiceKind::Cache { .. }
						| ServiceKind::Operation { .. }
						| ServiceKind::Package { .. }
						| ServiceKind::ApiRoutes { .. }
						| ServiceKind::Consumer { .. }
				)
			} else {
				matches!(
					dep.config().kind,
					ServiceKind::Database { .. }
						| ServiceKind::Cache { .. }
						| ServiceKind::Operation { .. }
						| ServiceKind::Package { .. }
						| ServiceKind::Consumer { .. }
				)
			};

			if !can_depend {
				panic!(
					"{} -> {}: cannot explicitly depend on this kind of service",
					self.name(),
					dep.name()
				);
			}
		}

		dep_ctxs
	}

	pub async fn database_dependencies(
		&self,
		run_context: &RunContext,
	) -> HashMap<String, config::service::Database> {
		let dbs = self
			.project()
			.await
			.recursive_dependencies(&[self.name()], run_context)
			.await
			.into_iter()
			// Filter filter services to include only operations, since these run in-process
			.filter(|svc| {
				**svc == *self || matches!(svc.config().kind, ServiceKind::Operation { .. })
			})
			.flat_map(|x| x.config().databases.clone().into_iter())
			// Dedupe
			.collect::<HashMap<_, _>>();

		dbs
	}

	pub async fn crdb_dependencies(&self, run_context: &RunContext) -> Vec<ServiceContext> {
		let dep_names = self
			.database_dependencies(run_context)
			.await
			.into_iter()
			.map(|(k, _)| k)
			.collect::<Vec<_>>();
		self.project()
			.await
			.services_with_names(&dep_names)
			.await
			.into_iter()
			.filter(|svc| matches!(svc.config().runtime, RuntimeKind::CRDB { .. }))
			.collect()
	}

	pub async fn redis_dependencies(&self, run_context: &RunContext) -> Vec<ServiceContext> {
		let default_deps = [
			"redis-chirp".to_string(),
			"redis-chirp-ephemeral".into(),
			"redis-cache".to_string(),
		];

		let dep_names = self
			.database_dependencies(run_context)
			.await
			.into_iter()
			.map(|(k, _)| k)
			.chain(default_deps)
			.collect::<Vec<_>>();

		self.project()
			.await
			.services_with_names(&dep_names)
			.await
			.into_iter()
			.filter(|svc| matches!(svc.config().runtime, RuntimeKind::Redis { .. }))
			.collect()
	}

	pub async fn s3_dependencies(&self, run_context: &RunContext) -> Vec<ServiceContext> {
		let dep_names = self
			.database_dependencies(run_context)
			.await
			.into_iter()
			.map(|(k, _)| k)
			.collect::<Vec<_>>();
		self.project()
			.await
			.services_with_names(&dep_names)
			.await
			.into_iter()
			.filter(|svc| matches!(svc.config().runtime, RuntimeKind::S3 { .. }))
			.collect()
	}

	pub async fn nats_dependencies(&self, run_context: &RunContext) -> Vec<ServiceContext> {
		let dep_names = self
			.database_dependencies(run_context)
			.await
			.into_iter()
			.map(|(k, _)| k)
			.collect::<Vec<_>>();
		self.project()
			.await
			.services_with_names(&dep_names)
			.await
			.into_iter()
			.filter(|svc| matches!(svc.config().runtime, RuntimeKind::Nats { .. }))
			.collect()
	}
}

impl ServiceContextData {
	pub async fn source_hash_dev(&self, build_optimization: &BuildOptimization) -> Result<String> {
		match self.config().runtime {
			// Use binary modified timestamp for rust runtimes
			RuntimeKind::Rust { .. } => {
				let bin_ts = if let Ok(metadata) =
					fs::metadata(self.rust_bin_path(&build_optimization).await).await
				{
					metadata
						.modified()?
						.duration_since(std::time::UNIX_EPOCH)?
						.as_millis()
				} else {
					0
				};

				Ok(bin_ts.to_string())
			}
			// Use source folder hash for other runtimes
			_ => {
				if let Some(source_hash) = self.source_hash_git().await? {
					Ok(source_hash)
				} else {
					let path = self.path().to_owned();
					let modified_ts =
						tokio::task::spawn_blocking(move || utils::deep_modified_ts(&path))
							.await??;
					Ok(modified_ts.to_string())
				}
			}
		}
	}

	/// Checks if there's modifications in the svc directory before returning
	/// the source hash.
	pub async fn source_hash_git(&self) -> Result<Option<String>> {
		let git_diff_cmd = Command::new("git")
			.arg("diff-index")
			.arg("HEAD")
			.arg("--")
			.arg(self.relative_path().await.display().to_string())
			.output()
			.await?;
		assert!(git_diff_cmd.status.success());
		let git_diff = String::from_utf8(git_diff_cmd.stdout)?;

		if git_diff.trim().is_empty() {
			let source_hash = self.source_hash_git_unchecked().await?;
			let source_hash = source_hash.trim();
			if !source_hash.is_empty() {
				Ok(Some(source_hash.into()))
			} else {
				// Service is likely new and has not been committed in the past
				Ok(None)
			}
		} else {
			// Directory has changes
			Ok(None)
		}
	}

	/// Unique hash indicating the version of this service that's deployed.
	///
	/// This value doesn't have a consistent format. In development, it uses the
	/// folder's modified timestamp. In production, it uses the git hash of the
	/// directory.
	pub async fn source_hash_git_unchecked(&self) -> Result<String> {
		let git_hash_cmd = Command::new("git")
			.arg("rev-parse")
			.arg(format!("HEAD:{}", self.relative_path().await.display()))
			.output()
			.await?;
		if git_hash_cmd.status.success() {
			Ok(String::from_utf8(git_hash_cmd.stdout)?.trim()[..8].to_string())
		} else {
			// File is not committed yet
			Ok(String::new())
		}
	}

	async fn required_secrets(
		&self,
		run_context: &RunContext,
	) -> Result<Vec<(Vec<String>, config::service::Secret)>> {
		let mut secrets = self
			.project()
			.await
			.recursive_dependencies(&[self.name()], run_context)
			.await
			.into_iter()
			// Aggregate secrets from all dependencies
			.flat_map(|x| x.config().secrets.clone().into_iter())
			// Convert keys to string array
			.map(|(k, v)| (k.split('/').map(|x| x.to_string()).collect::<Vec<_>>(), v))
			// Dedupe
			.collect::<HashMap<_, _>>()
			.into_iter()
			.collect::<Vec<_>>();

		secrets.sort_by_cached_key(|x| x.0.clone());

		Ok(secrets)
	}

	pub async fn env(&self, run_context: &RunContext) -> Result<IndexMap<String, String>> {
		let project_ctx = self.project().await;

		let mut env = IndexMap::new();

		// HACK: Link to dynamically linked libraries in /nix/store
		//
		// We build the binaries dynamically linked to dependencies from Nix, so
		// the output binaries are hardcoded to /nix/store.
		//
		// The `/nix/store` directory is mounted as a volume.
		if let config::ns::ClusterKind::SingleNode { .. } = project_ctx.ns().cluster.kind {
			env.insert(
				"LD_LIBRARY_PATH".into(),
				std::env::var("LD_LIBRARY_PATH").context("missing LD_LIBRARY_PATH")?,
			);
		}

		// TODO: This is re-running the hashing function for every service when we already did this in the planning step
		// Provide source hash to purge the cache when the service is updated
		let source_hash = self.source_hash_dev(&BuildOptimization::Debug).await?;
		env.insert("RIVET_SOURCE_HASH".into(), source_hash.clone());

		env.insert(
			"RIVET_PROJECT_SOURCE_HASH".into(),
			project_ctx.source_hash(),
		);

		let ns_service_config = self.ns_service_config().await;
		env.insert(
			"TOKIO_WORKER_THREADS".into(),
			if ns_service_config.resources.cpu >= 1000 {
				(ns_service_config.resources.cpu / 1000).max(1)
			} else {
				1
			}
			.to_string(),
		);

		// Generic context
		env.insert("RIVET_RUN_CONTEXT".into(), run_context.short().into());
		env.insert("RIVET_NAMESPACE".into(), project_ctx.ns_id().into());
		env.insert(
			"RIVET_CLUSTER_ID".into(),
			project_ctx.ns().cluster.id.to_string(),
		);

		if self.enable_tokio_console() {
			env.insert("TOKIO_CONSOLE_ENABLE".into(), "1".into());
			env.insert(
				"TOKIO_CONSOLE_BIND".into(),
				format!("0.0.0.0:{}", k8s::gen::TOKIO_CONSOLE_PORT),
			);
		}

		env.insert(
			"RIVET_ACCESS_KIND".into(),
			match project_ctx.ns().rivet.access {
				config::ns::RivetAccess::Private {} => "private".into(),
				config::ns::RivetAccess::Public {} => "public".into(),
			},
		);

		if project_ctx.ns().rivet.login.enable_admin {
			env.insert("RIVET_ACCESS_TOKEN_LOGIN".into(), "1".into());
		}

		// Domains
		if let Some(x) = project_ctx.domain_main() {
			env.insert("RIVET_DOMAIN_MAIN".into(), x);
		}
		if let Some(x) = project_ctx.domain_cdn() {
			env.insert("RIVET_DOMAIN_CDN".into(), x);
		}
		if let Some(x) = project_ctx.domain_job() {
			env.insert("RIVET_DOMAIN_JOB".into(), x);
		}
		if let Some(x) = project_ctx.domain_main_api() {
			env.insert("RIVET_DOMAIN_MAIN_API".into(), x);
		}
		if let Some(true) = project_ctx
			.ns()
			.dns
			.as_ref()
			.map(|x| x.deprecated_subdomains)
		{
			env.insert("RIVET_SUPPORT_DEPRECATED_SUBDOMAINS".into(), "1".into());
		}
		env.insert("RIVET_HOST_API".into(), project_ctx.host_api().await);
		env.insert("RIVET_HOST_TUNNEL".into(), project_ctx.host_tunnel().await);
		env.insert("RIVET_ORIGIN_API".into(), project_ctx.origin_api().await);
		env.insert("RIVET_ORIGIN_HUB".into(), project_ctx.origin_hub());

		// DNS
		if let Some(dns) = &project_ctx.ns().dns {
			if let Some(provider) = &dns.provider {
				env.insert(
					"RIVET_DNS_PROVIDER".into(),
					match provider {
						config::ns::DnsProvider::Cloudflare { .. } => "cloudflare".into(),
					},
				);
			}
		}

		// Networking
		if matches!(run_context, RunContext::Service { .. }) {
			env.insert("HEALTH_PORT".into(), k8s::gen::HEALTH_PORT.to_string());
			env.insert("METRICS_PORT".into(), k8s::gen::METRICS_PORT.to_string());
			if self.config().kind.has_server() {
				env.insert("PORT".into(), k8s::gen::HTTP_SERVER_PORT.to_string());
			}
		}

		// Add billing flag
		if let Some(billing) = &project_ctx.ns().rivet.billing {
			env.insert(
				"RIVET_BILLING".to_owned(),
				serde_json::to_string(&billing).unwrap(),
			);
		}

		// DNS
		if let Some(dns) = &project_ctx.ns().dns {
			match &dns.provider {
				Some(config::ns::DnsProvider::Cloudflare { account_id, .. }) => {
					env.insert("RIVET_DNS_PROVIDER".into(), "cloudflare".into());
					env.insert("CLOUDFLARE_ACCOUNT_ID".into(), account_id.clone());
				}
				None => {}
			}

			let dns_output = terraform::output::read_dns(&project_ctx).await;
			env.insert(
				"CLOUDFLARE_ZONE_ID_MAIN".into(),
				(*dns_output.cloudflare_zone_ids).main.clone(),
			);
			env.insert(
				"CLOUDFLARE_ZONE_ID_GAME".into(),
				(*dns_output.cloudflare_zone_ids).cdn.clone(),
			);
			env.insert(
				"CLOUDFLARE_ZONE_ID_JOB".into(),
				(*dns_output.cloudflare_zone_ids).job.clone(),
			);
		}

		// Infra Artifacts
		let infra_artifacts_output = terraform::output::read_infra_artifacts(&project_ctx).await;
		env.insert(
			"JOB_RUNNER_BINARY_KEY".into(),
			(*infra_artifacts_output.job_runner_binary_key).clone(),
		);

		// Backend
		if project_ctx.ns().rivet.backend.is_some() {
			let backend_output = terraform::output::read_backend(&project_ctx).await;

			env.insert(
				"CLOUDFLARE_BACKEND_DISPATCHER_NAMESPACE".into(),
				backend_output.dispatcher_namespace_name.to_string(),
			);
		}

		if self.depends_on_captcha() {
			if let Some(hcaptcha) = &project_ctx.ns().captcha.hcaptcha {
				env.insert(
					"HCAPTCHA_SITE_KEY_FALLBACK".into(),
					hcaptcha.site_key_fallback.clone(),
				);
			}

			if let Some(turnstile) = &project_ctx.ns().captcha.turnstile {
				env.insert(
					"TURNSTILE_SITE_KEY_MAIN".into(),
					turnstile.site_key_main.clone(),
				);
				env.insert(
					"TURNSTILE_SITE_KEY_CDN".into(),
					turnstile.site_key_cdn.clone(),
				);
			}
		}

		if self.depends_on_nomad_api() {
			// TODO: Read host url from terraform
			env.insert(
				"NOMAD_URL".into(),
				"http://nomad-server.nomad.svc.cluster.local:4646".into(),
			);
		}

		if let Some(x) = ns_service_config.crdb_min_connections {
			env.insert("CRDB_MIN_CONNECTIONS".into(), x.to_string());
		}
		if let Some(x) = ns_service_config.crdb_max_connections {
			env.insert("CRDB_MAX_CONNECTIONS".into(), x.to_string());
		}

		if project_ctx.ns().prometheus.is_some() && self.depends_on_prometheus_api() {
			env.insert(
				format!("PROMETHEUS_URL"),
				"http://prometheus-operated.prometheus.svc.cluster.local:9090".into(),
			);
		}

		// NATS
		env.insert(
			"NATS_URL".into(),
			// TODO: Add back passing multiple NATS nodes for failover instead of using DNS resolution
			"nats.nats.svc.cluster.local:4222".into(),
		);

		// Chirp config (used for both Chirp clients and Chirp workers)
		env.insert("CHIRP_SERVICE_NAME".into(), self.name());

		// Chirp worker config
		if (matches!(run_context, RunContext::Service { .. })
			&& matches!(&self.config().kind, ServiceKind::Consumer { .. }))
			|| self.is_monolith_worker()
		{
			env.insert(
				"CHIRP_WORKER_INSTANCE".into(),
				format!("{}-$(KUBERNETES_POD_ID)", self.name()),
			);

			env.insert("CHIRP_WORKER_KIND".into(), "consumer".into());
			env.insert("CHIRP_WORKER_CONSUMER_GROUP".into(), self.name());
		}

		// Fly
		if let Some(fly) = &project_ctx.ns().fly {
			env.insert("FLY_ORGANIZATION_ID".into(), fly.organization_id.clone());
			env.insert("FLY_REGION".into(), fly.region.clone());
		}

		// Add default provider
		let (default_provider, _) = project_ctx.default_s3_provider()?;
		env.insert(
			"S3_DEFAULT_PROVIDER".to_string(),
			default_provider.as_str().to_string(),
		);

		// Expose all S3 endpoints to services that need them
		let s3_deps = if self.depends_on_s3() {
			// self.s3_dependencies(&run_context).await
			project_ctx.all_services().await.to_vec()
		} else {
			Vec::new()
		};

		for s3_dep in s3_deps {
			if !matches!(s3_dep.config().runtime, RuntimeKind::S3 { .. }) {
				continue;
			}

			// Add all configured providers
			let providers = &project_ctx.ns().s3.providers;
			if providers.minio.is_some() {
				add_s3_env(&project_ctx, &mut env, &s3_dep, s3_util::Provider::Minio).await?;
			}
			if providers.backblaze.is_some() {
				add_s3_env(
					&project_ctx,
					&mut env,
					&s3_dep,
					s3_util::Provider::Backblaze,
				)
				.await?;
			}
			if providers.aws.is_some() {
				add_s3_env(&project_ctx, &mut env, &s3_dep, s3_util::Provider::Aws).await?;
			}
		}

		// S3 backfill
		if self.depends_on_s3_backfill() {
			if let Some(backfill) = &project_ctx.ns().s3.backfill {
				env.insert("S3_BACKFILL_PROVIDER".into(), backfill.as_str().to_string());
			}
		}

		// Runtime-specific
		match &self.config().runtime {
			RuntimeKind::Rust { .. } => {
				env.insert("RUST_BACKTRACE".into(), "1".into());
			}
			_ => {}
		}

		if project_ctx.ns().rivet.telemetry.disable {
			env.insert("RIVET_TELEMETRY_DISABLE".into(), "1".into());
		}

		env.insert(
			"RIVET_API_HUB_ORIGIN_REGEX".into(),
			project_ctx.origin_hub_regex(),
		);
		if project_ctx.ns().rivet.api.error_verbose {
			env.insert("RIVET_API_ERROR_VERBOSE".into(), "1".into());
		}
		if project_ctx.ns().rivet.profanity.filter_disable {
			env.insert("RIVET_PROFANITY_FILTER_DISABLE".into(), "1".into());
		}

		// Nomad
		env.insert(
			"NOMAD_SERVER_COUNT".into(),
			project_ctx.nomad_server_count().to_string(),
		);

		if let Some(provisioning) = &project_ctx.ns().rivet.provisioning {
			if self.depends_on_cluster_config() || matches!(run_context, RunContext::Test { .. }) {
				env.insert(
					"RIVET_DEFAULT_CLUSTER_CONFIG".into(),
					serde_json::to_string(&provisioning.cluster)?,
				);
			}

			if self.depends_on_provision_margin() {
				env.insert(
					"RIVET_JOB_SERVER_PROVISION_MARGIN".to_string(),
					provisioning.job_server_provision_margin.to_string(),
				);
			}

			env.insert(
				"TLS_ACME_DIRECTORY".to_string(),
				serde_json::to_value(&provisioning.acme_directory)?
					.as_str()
					.unwrap()
					.to_string(),
			);
		}

		if let Some(nsfw_check) = &project_ctx.ns().rivet.upload.nsfw_check {
			env.insert("RIVET_UPLOAD_NSFW_CHECK_ENABLED".into(), "1".into());

			if nsfw_check.error_verbose {
				env.insert("RIVET_UPLOAD_NSFW_ERROR_VERBOSE".into(), "1".into());
			}
		}

		Ok(env)
	}

	pub async fn secret_env(&self, run_context: &RunContext) -> Result<IndexMap<String, String>> {
		let project_ctx = self.project().await;

		let mut env = IndexMap::new();

		// Write secrets
		for (secret_key, secret_config) in self.required_secrets(run_context).await? {
			let env_key = secret_env_var_key(&secret_key);
			if secret_config.optional {
				if let Some(value) = project_ctx.read_secret_opt(&secret_key).await? {
					env.insert(env_key, value);
				}
			} else {
				env.insert(env_key, project_ctx.read_secret(&secret_key).await?);
			}
		}

		// NATS
		env.insert("NATS_USERNAME".into(), "chirp".into());
		env.insert("NATS_PASSWORD".into(), "password".into());

		env.insert(
			"RIVET_JWT_KEY_PUBLIC".into(),
			project_ctx
				.read_secret(&["jwt", "key", "public_pem"])
				.await?,
		);
		if self.depends_on_jwt_key_private() {
			env.insert(
				"RIVET_JWT_KEY_PRIVATE".into(),
				project_ctx
					.read_secret(&["jwt", "key", "private_pem"])
					.await?,
			);
		}

		if self.depends_on_sendgrid_key(&project_ctx) {
			env.insert(
				"SENDGRID_KEY".into(),
				project_ctx.read_secret(&["sendgrid", "key"]).await?,
			);
		}

		// CRDB
		// TODO: Function is expensive
		{
			let crdb_data = terraform::output::read_crdb(&project_ctx).await;
			let crdb_host = format!("{}:{}", *crdb_data.host, *crdb_data.port);
			let username = project_ctx.read_secret(&["crdb", "username"]).await?;
			let password = project_ctx.read_secret(&["crdb", "password"]).await?;
			let sslmode = "verify-ca";

			let url = format!(
				"postgres://{}:{}@{crdb_host}/postgres?sslmode={sslmode}",
				username, password,
			);
			env.insert("CRDB_URL".into(), url);

			// TODO:
			let workflow_url = format!(
				"postgres://{}:{}@{crdb_host}/db_workflow?sslmode={sslmode}",
				username, password,
			);
			env.insert("CRDB_WORKFLOW_URL".into(), workflow_url);
		}

		// Redis
		// TODO: read_redis is expensive
		let redis_data = terraform::output::read_redis(&project_ctx).await;
		// Keeps track to avoid duplicates
		let mut has_ephemeral = false;
		let mut has_persistent = false;

		for redis_dep in self.redis_dependencies(run_context).await {
			let db_name = if let RuntimeKind::Redis { persistent } = redis_dep.config().runtime {
				if persistent {
					if has_persistent {
						continue;
					}
					has_persistent = true;

					"persistent".to_string()
				} else {
					if has_ephemeral {
						continue;
					}
					has_ephemeral = true;

					"ephemeral".to_string()
				}
			} else {
				unreachable!();
			};

			// Read host and port from terraform
			let hostname = redis_data
				.host
				.get(&db_name)
				.expect("terraform output for redis db not found");
			let port = redis_data
				.port
				.get(&db_name)
				.expect("terraform output for redis db not found");
			let host = format!("{}:{}", *hostname, *port);

			// Read auth secrets
			let (username, password) = match project_ctx.ns().redis.provider {
				config::ns::RedisProvider::Kubernetes {}
				| config::ns::RedisProvider::Aiven { .. } => (
					project_ctx
						.read_secret(&["redis", &db_name, "username"])
						.await?,
					project_ctx
						.read_secret_opt(&["redis", &db_name, "password"])
						.await?,
				),
				config::ns::RedisProvider::Aws {} => {
					let db_name = format!("rivet-{}-{}", project_ctx.ns_id(), db_name);
					let username = project_ctx
						.read_secret(&["redis", &db_name, "username"])
						.await?;
					let password = project_ctx
						.read_secret_opt(&["redis", &db_name, "password"])
						.await?;

					(username, password)
				}
			};

			// Build URL with auth
			let url = if let Some(password) = password {
				format!("rediss://{}:{}@{host}", username, password)
			} else {
				format!("rediss://{}@{host}", username)
			};

			env.insert(
				format!("REDIS_URL_{}", db_name.to_uppercase().replace('-', "_")),
				url,
			);
		}

		// ClickHouse
		if self.depends_on_clickhouse() {
			if project_ctx.ns().clickhouse.is_some() {
				let clickhouse_data = terraform::output::read_clickhouse(&project_ctx).await;
				let username = "chirp";
				let password = project_ctx
					.read_secret(&["clickhouse", "users", username, "password"])
					.await?;
				let uri = format!(
					"https://{}:{}@{}:{}",
					username, password, *clickhouse_data.host, *clickhouse_data.port_https
				);

				env.insert("CLICKHOUSE_URL".into(), uri);
			} else {
				env.insert("CLICKHOUSE_DISABLED".into(), "1".into());
			}
		}

		// Expose all S3 endpoints to services that need them
		let s3_deps = if self.depends_on_s3() {
			// self.s3_dependencies(&run_context).await
			project_ctx.all_services().await.to_vec()
		} else {
			Vec::new()
		};

		for s3_dep in s3_deps {
			if !matches!(s3_dep.config().runtime, RuntimeKind::S3 { .. }) {
				continue;
			}

			// Add all configured providers
			let providers = &project_ctx.ns().s3.providers;
			if providers.minio.is_some() {
				add_s3_secret_env(&project_ctx, &mut env, &s3_dep, s3_util::Provider::Minio)
					.await?;
			}
			if providers.backblaze.is_some() {
				add_s3_secret_env(
					&project_ctx,
					&mut env,
					&s3_dep,
					s3_util::Provider::Backblaze,
				)
				.await?;
			}
			if providers.aws.is_some() {
				add_s3_secret_env(&project_ctx, &mut env, &s3_dep, s3_util::Provider::Aws).await?;
			}
		}

		if let Some(dns) = &project_ctx.ns().dns {
			match &dns.provider {
				Some(config::ns::DnsProvider::Cloudflare { .. }) => {
					if self.depends_on_cloudflare() {
						env.insert(
							"CLOUDFLARE_AUTH_TOKEN".into(),
							project_ctx
								.read_secret(&["cloudflare", "terraform", "auth_token"])
								.await?,
						);
					}
				}
				None => {}
			}
		}

		// if self.depends_on_infra() && project_ctx.ns().rivet.provisioning.is_some() {
		let tls = terraform::output::read_tls(&project_ctx).await;

		env.insert(
			"TLS_CERT_LOCALLY_SIGNED_JOB_CERT_PEM".into(),
			tls.tls_cert_locally_signed_job.cert_pem.clone(),
		);
		env.insert(
			"TLS_CERT_LOCALLY_SIGNED_JOB_KEY_PEM".into(),
			tls.tls_cert_locally_signed_job.key_pem.clone(),
		);
		env.insert(
			"TLS_ACME_ACCOUNT_PRIVATE_KEY_PEM".into(),
			(*tls.acme_account_private_key_pem).clone(),
		);
		env.insert(
			"TLS_ROOT_CA_CERT_PEM".into(),
			(*tls.root_ca_cert_pem).clone(),
		);
		// }

		Ok(env)
	}
}

impl ServiceContextData {
	pub fn docker_image_tag(&self, project_ctx: &ProjectContext, repo: &str) -> Result<String> {
		ensure!(repo.ends_with('/'), "docker repository must end with slash");

		let source_hash = project_ctx.source_hash();

		Ok(format!(
			"{}{}:{}",
			repo,
			self.cargo_name().expect("no cargo name"),
			source_hash
		))
	}

	pub async fn upload_build(&self) -> Result<()> {
		let project_ctx = self.project().await;
		let (push_repo, _) = project_ctx.docker_repos().await;
		let image_tag = self.docker_image_tag(&project_ctx, &push_repo)?;

		let mut cmd = Command::new("docker");
		cmd.arg("push");
		cmd.arg(image_tag);

		let status = cmd.status().await?;
		eprintln!();

		ensure!(status.success());

		Ok(())
	}
}

impl ServiceContextData {
	pub async fn ns_service_config(&self) -> config::ns::Service {
		let project_ctx = self.project().await;

		let is_singleton = matches!(
			&self.config().kind,
			ServiceKind::Headless {
				singleton: true,
				..
			} | ServiceKind::Api {
				singleton: true,
				..
			}
		);

		let mut service = project_ctx
			.ns()
			.services
			.get(&self.name())
			.cloned()
			.or_else(|| project_ctx.ns().services.get("default").cloned())
			.unwrap_or_else(|| match project_ctx.ns().cluster.kind {
				config::ns::ClusterKind::SingleNode { .. } => config::ns::Service {
					count: 1,
					resources: self.config().resources.single_node.clone(),
					crdb_min_connections: None,
					crdb_max_connections: None,
				},
				config::ns::ClusterKind::Distributed { .. } => config::ns::Service {
					count: 2,
					resources: self.config().resources.distributed.clone(),
					crdb_min_connections: None,
					crdb_max_connections: None,
				},
			});

		// Force single count if singleton
		if is_singleton {
			service.count = 1;
		}

		service
	}
}

async fn add_s3_env(
	project_ctx: &ProjectContext,
	env: &mut IndexMap<String, String>,
	s3_dep: &Arc<ServiceContextData>,
	provider: s3_util::Provider,
) -> Result<()> {
	let provider_upper = provider.as_str().to_uppercase();

	let s3_dep_name = s3_dep.name_screaming_snake();
	let s3_config = project_ctx.s3_config(provider).await?;

	env.insert(
		format!("S3_{provider_upper}_BUCKET_{s3_dep_name}"),
		s3_dep.s3_bucket_name().await,
	);
	env.insert(
		format!("S3_{provider_upper}_ENDPOINT_INTERNAL_{s3_dep_name}"),
		s3_config.endpoint_internal,
	);
	// External endpoint
	{
		let mut external_endpoint = s3_config.endpoint_external;

		// Switch to internal k8s url if public ip is loopback
		if let (
			s3_util::Provider::Minio,
			config::ns::ClusterKind::SingleNode {
				public_ip: Some(public_ip),
				minio_port,
				..
			},
		) = (provider, &project_ctx.ns().cluster.kind)
		{
			let is_loopback = public_ip
				.parse::<Ipv4Addr>()
				.ok()
				.map(|ip| ip.is_loopback())
				.unwrap_or_default();

			if is_loopback {
				external_endpoint = format!("http://minio.minio.svc.cluster.local:{minio_port}");
			}
		}

		env.insert(
			format!("S3_{provider_upper}_ENDPOINT_EXTERNAL_{s3_dep_name}",),
			external_endpoint,
		);
	}
	env.insert(
		format!("S3_{provider_upper}_REGION_{s3_dep_name}"),
		s3_config.region,
	);

	Ok(())
}

async fn add_s3_secret_env(
	project_ctx: &ProjectContext,
	env: &mut IndexMap<String, String>,
	s3_dep: &Arc<ServiceContextData>,
	provider: s3_util::Provider,
) -> Result<()> {
	let provider_upper = provider.as_str().to_uppercase();

	let s3_dep_name = s3_dep.name_screaming_snake();
	let s3_creds = project_ctx.s3_credentials(provider).await?;

	env.insert(
		format!("S3_{provider_upper}_ACCESS_KEY_ID_{s3_dep_name}"),
		s3_creds.access_key_id,
	);
	env.insert(
		format!("S3_{provider_upper}_SECRET_ACCESS_KEY_{s3_dep_name}"),
		s3_creds.access_key_secret,
	);

	Ok(())
}

/// TODO: Reuse code with lib/util/env/src/lib.r
pub fn secret_env_var_key(key: &[impl AsRef<str>]) -> String {
	key.iter()
		.map(|x| x.as_ref().to_uppercase())
		.collect::<Vec<_>>()
		.join("_")
}
