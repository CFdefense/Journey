import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		globals: true,
		environment: "jsdom",
		testTimeout: 3600000,
		coverage: {
			provider: "v8",
			reporter: ["text", "html"],
			reportsDirectory: "../docs/frontend-codecov",
			exclude: [
				"src/api/",
				"src/helpers/config.ts",
				"src/helpers/global.ts",
				"src/models",
				"**/*.d.ts",
				"**/*.tsx", // if you want to test a react component, you should specifically include that file
				"tests/testApi.ts" // Generated file, exclude from coverage
			],
			include: [
				"src/**"
			],
			thresholds: {
				lines: 80,
				functions: 80,
				branches: 80,
				statements: 80
			},
		},
	},
});
