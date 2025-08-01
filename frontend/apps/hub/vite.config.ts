import * as crypto from "node:crypto";
import path from "node:path";
import mdx from "@mdx-js/rollup";
import { sentryVitePlugin } from "@sentry/vite-plugin";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import { visualizer } from "rollup-plugin-visualizer";
import { defineConfig } from "vite";
import vitePluginFaviconsInject from "vite-plugin-favicons-inject";

// These are only needed in CI. They'll be undefined in dev.
const GIT_BRANCH = process.env.CF_PAGES_BRANCH;
const GIT_SHA = process.env.CF_PAGES_COMMIT_SHA;

// https://vitejs.dev/config/
export default defineConfig({
	base: "./",
	plugins: [
		react(),
		TanStackRouterVite(),
		vitePluginFaviconsInject(
			path.resolve(__dirname, "public", "icon-white.svg"),
			{
				appName: "Rivet Hub",
				theme_color: "#ff4f00",
			},
		),
		process.env.SENTRY_AUTH_TOKEN
			? sentryVitePlugin({
					org: "rivet-gaming",
					project: "hub",
					authToken: process.env.SENTRY_AUTH_TOKEN,
					release:
						GIT_BRANCH === "main" ? { name: GIT_SHA } : undefined,
				})
			: null,
		process.env.DEBUG_BUNDLE ? visualizer() : null,
	],
	server: {
		host: "0.0.0.0",
		port: 5080,
		allowedHosts: true,
		// Listen on a different port since we don't proxy WebSockets on /ui
		hmr: {
			port: 5080,
			host: "127.0.0.1",
		},
	},
	preview: {
		port: 5080,
	},
	define: {
		// Provide a unique build ID for cache busting
		__APP_BUILD_ID__: JSON.stringify(
			`${new Date().toISOString()}@${crypto.randomUUID()}`,
		),
	},
	resolve: {
		alias: {
			"@": path.resolve(__dirname, "./src"),
		},
	},
	build: {
		sourcemap: true,
		commonjsOptions: {
			include: [/@rivet-gg\/components/, /node_modules/],
		},
	},
	optimizeDeps: {
		include: ["@fortawesome/*", "@rivet-gg/icons"],
	},
	worker: {
		format: "es",
	},
});
