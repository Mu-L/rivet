import { RivetClient } from "@rivet-gg/api";

// Can be opt since they're not required for dev
const RIVET_ENDPOINT = Deno.env.get("RIVET_ENDPOINT");
const RIVET_SERVICE_TOKEN = Deno.env.get("RIVET_SERVICE_TOKEN");
const RIVET_PROJECT = Deno.env.get("RIVET_PROJECT");
const RIVET_ENVIRONMENT = Deno.env.get("RIVET_ENVIRONMENT");

let region = Deno.env.get("REGION");
if (!region || region.length === 0) {
	region = undefined;
}

const client = new RivetClient({
	environment: RIVET_ENDPOINT,
	token: RIVET_SERVICE_TOKEN,
});

async function run() {
	let actorId: string | undefined;
	try {
		console.log("Creating actor", { region });
		const { actor } = await client.actor.create({
			project: RIVET_PROJECT,
			environment: RIVET_ENVIRONMENT,
			body: {
				region,
				tags: {
					name: "ws",
				},
				buildTags: { name: "ws", current: "true" },
				network: {
					ports: {
						http: {
							protocol: "https",
							// internalPort: 80,
							routing: {
								guard: {},
							},
						},
					},
				},
				lifecycle: {
					durable: false,
				},
			},
		});
		actorId = actor.id;

		const port = actor.network.ports.http;
		const actorOrigin = `${port.protocol}://${port.hostname}:${port.port}${port.path ?? ""}`;
		console.log("Created actor at", actorOrigin);

		await new Promise((resolve, reject) => {
			// Open a WebSocket to that endpoint
			const ws = new WebSocket(`${actorOrigin}/ws`);

			ws.onmessage = (evt) => {
				const [type, body] = JSON.parse(evt.data);
				if (type === "init") {
					console.log("Init event data:", body);
				} else if (type === "pong") {
					console.log("Pong");
					ws.close();
					resolve(undefined);
				} else {
					console.warn("unknown message type", type);
				}
			};

			ws.onopen = () => {
				console.log("Ping");
				ws.send(JSON.stringify(["ping", 123]));
			};

			ws.onclose = () => {
				console.log("WebSocket connection closed");
			};

			ws.onerror = (err) => {
				console.error("WS error", err);
				reject("ws error");
			};
		});
	} catch (error) {
		console.error("Error:", error);
	} finally {
		if (actorId) {
			console.log("Destroying", actorId);
			await client.actor.destroy(actorId, {
				project: RIVET_PROJECT,
				environment: RIVET_ENVIRONMENT,
			});
		}
	}
}

await run();
