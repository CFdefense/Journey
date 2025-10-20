import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		globals: true,
		environment: "node",
		coverage: {
			provider: "v8",
			exclude: [
				"coverage/",
				"dist/",
				"node_modules/",
				"public/",
				"src/components/",
				"src/pages/",
				"tests/",
				"**/*.d.ts",
				"*.config.*",
			],
			thresholds: {
				// lines: 80,
				// functions: 80,
				// branches: 80,
				// statements: 80
				lines: 0,
				functions: 0,
				branches: 0,
				statements: 0
			},
		},
	},
});
