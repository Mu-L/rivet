import { RivetClient, RivetClientClient } from "@rivet-gg/api";
import { assertUnreachable, RivetEnvironment, ActorTags } from "../../common/src/utils.ts";
import { PORT_NAME } from "../../common/src/network.ts";
import { ActorQuery, CreateRequest } from "../../manager-protocol/src/query.ts";

export async function queryActor(
	client: RivetClientClient,
	environment: RivetEnvironment,
	query: ActorQuery,
): Promise<RivetClient.actor.Actor> {
	console.log('Query', query);
	if ("actorId" in query) {
		const res = await client.actor.get(
			query.actorId.actorId,
			environment,
		);
		if (res.actor.destroyedAt) {
			throw new Error(`Actor with ID ${query.actorId.actorId} already destroyed`);
		}
		return res.actor;
	} else if ("get" in query) {
		const getActor = await getWithTags(client, environment, query.get.tags);
		if (!getActor) throw new Error("Actor not found with tags");
		return getActor;
	} else if ("getOrCreate" in query) {
		const tags = query.getOrCreate.tags;
		if (!tags) throw new Error("Must define tags in getOrCreate");
		const getActor = await getWithTags(client, environment, tags as ActorTags);
		if (getActor) {
			return getActor;
		} else {
			return await createActor(client, environment, query.getOrCreate.create);
		}
	} else if ("create" in query) {
		return await createActor(client, environment, query.create);
	} else {
		assertUnreachable(query);
	}
}

async function getWithTags(
	client: RivetClientClient,
	environment: RivetEnvironment,
	tags: ActorTags,
): Promise<RivetClient.actor.Actor | undefined> {
	const req = {
		tagsJson: JSON.stringify(tags),
		...environment,
	};
	console.log("List request", req);
	let { actors } = await client.actor.list(req);

	// TODO(RVT-4248): Don't return actors that aren't networkable yet
	actors = actors.filter((a) => {
		for (const portName in a.network.ports) {
			const port = a.network.ports[portName];
			if (!port.publicHostname || !port.publicPort) return false;
		}
		return true;
	});

	if (actors.length == 0) {
		return undefined;
	}
	if (actors.length > 1) {
		actors.sort((a, b) => a.id.localeCompare(b.id));
	}

	return actors[0];
}

async function createActor(
	client: RivetClientClient,
	environment: RivetEnvironment,
	createRequest: CreateRequest = {} satisfies CreateRequest
): Promise<RivetClient.actor.Actor> {
	// TODO(RVT-4250):
	if (!createRequest.network) {
		createRequest.network = {};
	}
	if (!createRequest.network.mode) {
		// TODO: Don't force host
		createRequest.network.mode = "host";
	}
	if (!createRequest.network.ports) {
		createRequest.network.ports = {};
	}
	if (!(PORT_NAME in createRequest.network.ports)) {
		// TODO: Don't force TCP protocol
		createRequest.network.ports[PORT_NAME] = {
			protocol: "tcp",
			routing: { host: {} },
		}
	}

	const req = {
		...environment,
		body: createRequest,
	};
	console.log("Create actor", req);
	const { actor } = await client.actor.create(req);
	return actor;
}
