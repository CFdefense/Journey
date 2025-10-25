/// Fetch only stores cookies when running in the browser.
/// When we run tests, it's running with Node, not the browser.
/// Node's implementation of fetch doesn't store cookies,
/// so we have to make a wrapper for it.
/// This file generates the source code for the api-calling functions
/// by doing simple text search and replacements.
/// The generated source code will use the wrapper fetch function to
/// use cookies.

import { promises as fs } from "fs";
import path from "path";

const HELPERS_FIND = "../helpers/";
const HELPERS_REPLACE = "../src/helpers/";

const MODELS_FIND = "../models/";
const MODELS_REPLACE = "../src/models/";

const FETCH_FIND = /await\s*fetch\s*\(\s*`\$\{API_BASE_URL\}\/api\//g;
const FETCH_REPLACE = "await customFetch(`${API_BASE_URL}/api/";

const URL_FIND = "const API_BASE_URL = import.meta.env.VITE_API_BASE_URL;";
const URL = 'const API_BASE_URL = "http://localhost:3001";\n';

const ENV_FIND = "import.meta.env.DEV";
const ENV_REPLACE = "test_state.dev_mode";
const ENV_DECL = `export const test_state = {
	dev_mode: true
};\n`;

const API_RESULT_FIND = 'import type { ApiResult } from "../src/helpers/global";';

const GEN_FILE_WARNING = `
//================================//
// GENERATED FILE - DO NOT MODIFY //
//================================//
\n`;

const IMPORT_CUSTOM_FETCH = 'import { customFetch } from "./customFetch";';

const API_DIR = "./src/api/";

const api_src = (await Promise.all((await fs
	.readdir(API_DIR, { withFileTypes: true }))
	.filter(file => file.isFile())
	.map(async file => await fs.readFile(path.join(API_DIR, file.name), "utf8"))))
	.join("\n");

let gen_src = GEN_FILE_WARNING
	+ URL
	+ ENV_DECL
	+ API_RESULT_FIND
	 + "\n"
	+ IMPORT_CUSTOM_FETCH
	+ api_src
		.replaceAll(HELPERS_FIND, HELPERS_REPLACE)
		.replaceAll(MODELS_FIND, MODELS_REPLACE)
		.replaceAll(FETCH_FIND, FETCH_REPLACE)
		.replaceAll(URL_FIND, "")
		.replaceAll(ENV_FIND, ENV_REPLACE)
		.replaceAll(API_RESULT_FIND, "");

await fs.writeFile("./tests/testApi.ts", gen_src, "utf8");

console.log("Generated file: './tests/testApi.js'");