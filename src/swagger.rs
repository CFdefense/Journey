use axum::Router;
use utoipa::{openapi::security::{ApiKey, ApiKeyValue, SecurityScheme}, Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;
use std::fs::{self, File};
use std::io::Write;

#[derive(OpenApi)]
#[openapi(
	paths(
		crate::controllers::account::api_update,
		crate::controllers::account::api_current,
		crate::controllers::account::api_validate,
		crate::controllers::account::api_logout,
		crate::controllers::account::api_signup,
		crate::controllers::account::api_login,
	),
	modifiers(&SecurityAddon),
    info(description = "My Api description"),
)]
struct ApiDoc;

struct SecurityAddon;

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
pub fn merge_swagger(router: OpenApiRouter) -> Router {
	let doc = ApiDoc::openapi();
	fs::create_dir_all("api-docs").unwrap();
	let mut file = File::create("api-docs/openapi.json").unwrap();
	file.write_all(doc.to_pretty_json().unwrap().as_bytes()).unwrap();
	let (router, api) = OpenApiRouter::with_openapi(doc)
        .merge(router)
        .split_for_parts();
	router.merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", api.clone()))
}