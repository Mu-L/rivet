use std::collections::{HashMap, HashSet};

use proto::backend::{self, pkg::*};
use rivet_operation::prelude::*;

use crate::prewarm_ats::PrewarmAtsContext;

mod prewarm_ats;

#[operation(name = "mm-config-version-prepare")]
async fn handle(
	ctx: OperationContext<mm_config::version_prepare::Request>,
) -> GlobalResult<mm_config::version_prepare::Response> {
	let game_id = unwrap_ref!(ctx.game_id).as_uuid();
	let config = unwrap_ref!(ctx.config);

	// List of build paths that will be used to prewarm the ATS cache
	let mut prewarm_ctx = PrewarmAtsContext {
		region_ids: HashSet::new(),
		paths: HashMap::new(),
		total_size: 0,
	};

	let mut lobby_group_ctxs = Vec::new();
	for lobby_group in &config.lobby_groups {
		// Validate regions
		ensure!(!lobby_group.regions.is_empty(), "no regions provided");
		let region_ids = lobby_group
			.regions
			.iter()
			.flat_map(|r| r.region_id)
			.collect::<Vec<_>>();
		prewarm_ctx
			.region_ids
			.extend(region_ids.iter().map(common::Uuid::as_uuid));

		// Resolve region name IDs
		let (regions_res, tier_res) = tokio::try_join!(
			op!([ctx] region_get {
				region_ids: region_ids.clone(),
			}),
			chirp_workflow::compat::op(
				&ctx,
				::tier::ops::list::Input {
					datacenter_ids: region_ids.iter().map(|x| x.as_uuid()).collect(),
					pegboard: false,
				},
			),
		)?;

		// Validate regions exist
		for region in &lobby_group.regions {
			let has_region = regions_res
				.regions
				.iter()
				.any(|x| x.region_id == region.region_id);
			ensure!(has_region, "invalid region id");
		}

		// Check if we need to prewarm the ATS cache for this Docker build
		//
		// We only need to prewarm the cache if _not_ using idle lobbies or if using custom lobbies. If there are idle
		// lobbies, then we'll start a lobby immediately, so the prewarm is redundant.
		let needs_ats_prewarm = lobby_group
			.actions
			.as_ref()
			.and_then(|a| a.create.as_ref())
			.is_some()
			|| lobby_group.regions.iter().any(|x| {
				x.idle_lobbies
					.as_ref()
					.map_or(true, |y| y.min_idle_lobbies == 0)
			});

		// Prepare runtime
		let runtime = unwrap_ref!(lobby_group.runtime);
		let runtime = unwrap_ref!(runtime.runtime);
		let runtime_ctx = prepare_runtime(
			&ctx,
			game_id,
			runtime,
			&lobby_group.regions,
			&regions_res.regions,
			&tier_res.datacenters,
			&mut prewarm_ctx,
			needs_ats_prewarm,
		)
		.await?;

		lobby_group_ctxs.push(backend::matchmaker::LobbyGroupCtx {
			runtime: Some(runtime_ctx),
		});
	}

	crate::prewarm_ats::prewarm_ats_cache(&ctx, prewarm_ctx).await?;

	Ok(mm_config::version_prepare::Response {
		config_ctx: Some(backend::matchmaker::VersionConfigCtx {
			lobby_groups: lobby_group_ctxs,
		}),
	})
}

#[tracing::instrument(skip(prewarm_ctx))]
async fn prepare_runtime(
	ctx: &OperationContext<mm_config::version_prepare::Request>,
	game_id: Uuid,
	runtime: &backend::matchmaker::lobby_runtime::Runtime,
	lg_regions: &[backend::matchmaker::lobby_group::Region],
	regions_data: &[backend::region::Region],
	tier_dcs: &[::tier::ops::list::Datacenter],
	prewarm_ctx: &mut PrewarmAtsContext,
	needs_ats_prewarm: bool,
) -> GlobalResult<backend::matchmaker::LobbyRuntimeCtx> {
	let runtime: backend::matchmaker::LobbyRuntimeCtx = match runtime {
		backend::matchmaker::lobby_runtime::Runtime::Docker(runtime) => {
			// Validate the build
			let build_id = unwrap_ref!(runtime.build_id).as_uuid();
			let _ = validate_build(ctx, game_id, build_id, prewarm_ctx, needs_ats_prewarm).await?;

			// Validate regions
			for lg_region in lg_regions {
				// Validate region
				ensure!(
					regions_data
						.iter()
						.any(|x| x.region_id == lg_region.region_id),
					"invalid region id"
				);

				// Validate tier
				let region_id = unwrap!(lg_region.region_id).as_uuid();
				let tier_dc = unwrap!(tier_dcs.iter().find(|x| x.datacenter_id == region_id));
				ensure!(
					tier_dc
						.tiers
						.iter()
						.any(|x| x.tier_name_id == lg_region.tier_name_id),
					"invalid tier name id"
				);
			}

			backend::matchmaker::lobby_runtime_ctx::Docker {
				#[allow(deprecated)]
				job_template_id: None,
			}
			.into()
		}
	};

	Ok(runtime)
}

#[tracing::instrument(skip(prewarm_ctx))]
async fn validate_build(
	ctx: &OperationContext<mm_config::version_prepare::Request>,
	game_id: Uuid,
	build_id: Uuid,
	prewarm_ctx: &mut PrewarmAtsContext,
	needs_ats_prewarm: bool,
) -> GlobalResult<(Uuid, String)> {
	// Validate build exists and belongs to this game
	let build_get = op!([ctx] build_get {
		build_ids: vec![build_id.into()],
	})
	.await?;
	let build = unwrap!(build_get.builds.first(), "build not found");
	let build_upload_id = unwrap_ref!(build.upload_id).as_uuid();
	let build_game_id = unwrap_ref!(build.game_id).as_uuid();
	let build_kind = unwrap!(backend::build::BuildKind::from_i32(build.kind));
	let build_compression = unwrap!(backend::build::BuildCompression::from_i32(
		build.compression
	));
	ensure_eq!(game_id, build_game_id);

	tracing::info!(?build);

	// Validate build has completed uploading
	let upload_res = op!([ctx] upload_get {
		upload_ids: vec![build_upload_id.into()],
	})
	.await?;
	let upload = unwrap!(upload_res.uploads.first(), "build upload not found");
	let upload_id = unwrap_ref!(upload.upload_id).as_uuid();
	ensure!(upload.complete_ts.is_some(), "build upload is not complete");

	// Generate path used to request the image
	if needs_ats_prewarm {
		let path = format!(
			"/s3-cache/{namespace}-bucket-build/{upload_id}/{file_name}",
			namespace = ctx.config().server()?.rivet.namespace,
			file_name = util_build::file_name(build_kind, build_compression),
		);
		if !prewarm_ctx.paths.contains_key(&path) {
			let build_id_hash = util_build::build_hash(build_id);

			prewarm_ctx.paths.insert(path.clone(), build_id_hash);
			prewarm_ctx.total_size += upload.content_length;
		}
	}

	Ok((upload_id, build.image_tag.clone()))
}
