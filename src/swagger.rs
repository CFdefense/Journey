use axum::Router;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use utoipa::{
	Modify, OpenApi,
	openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::controllers::{account::AccountApiDoc, chat::ChatApiDoc, itinerary::ItineraryApiDoc};

#[derive(OpenApi)]
#[openapi(
	modifiers(&SecurityAddon),
	security(
		(),
		("set-cookie"=[])
	),
    info(
    	title="Journey API",
    	description = "The public API documentation for the Journey web application."
    ),
    nest(
    	(path="/api/account", api=AccountApiDoc),
    	(path="/api/chat", api=ChatApiDoc),
    	(path="/api/itinerary", api=ItineraryApiDoc)
    ),
    servers(
    	(url="http://localhost:3001", description="Local host server for development"),
     	//TODO add deployed production server URL
    )
)]
#[allow(dead_code)]
struct ApiDoc;

#[allow(dead_code)]
pub struct SecurityAddon;

impl Modify for SecurityAddon {
	fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
		if let Some(components) = openapi.components.as_mut() {
			components.add_security_scheme(
                "set-cookie",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::with_description(
                	"auth-token",
                 	"An HTTP-only cookie which must encode a valid account id, expiration timestamp, and other information"
                ))),
            )
		}
	}
}

/// Merges swagger with the current routes
#[allow(dead_code)]
pub fn merge_swagger(router: OpenApiRouter) -> Router {
	let doc = ApiDoc::openapi();
	let docs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("docs");
	fs::create_dir_all(&docs_path).unwrap();
	let mut file = File::create(docs_path.join("openapi.json")).unwrap();
	file.write_all(doc.to_pretty_json().unwrap().as_bytes())
		.unwrap();
	let (router, api) = OpenApiRouter::with_openapi(doc)
		.merge(router)
		.split_for_parts();
	router.merge(SwaggerUi::new("/swagger").url("/docs/openapi.json", api.clone()))
}
