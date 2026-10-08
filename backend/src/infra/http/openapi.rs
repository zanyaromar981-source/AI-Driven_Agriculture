use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    features::farmers::web::{
        EditProfileParams, Language, ProfileResponse, SendSignInCodeParams, SignInCodeSentResponse,
        SignedInResponse, VerifySignInCodeParams, handlers as farmer_handlers,
    },
    features::farms::web::{
        CellParams, CellResponse, CentroidResponse, CreateFarmParams, Crop, CropAreaResponse,
        FarmResponse, FarmSummaryResponse, FarmsResponse, GridCellResponse, OneFarmResponse,
        OutlinePointResponse, PointParams, RepaintFarmCellsParams, SavedFarmResponse,
        handlers as farm_handlers,
    },
    infra::http::{ErrorBody, health},
};

struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        farm_handlers::get_farms,
        farm_handlers::create_farm,
        farm_handlers::get_farm,
        farm_handlers::repaint_farm_cells,
        farm_handlers::delete_farm,
        farmer_handlers::send_sign_in_code,
        farmer_handlers::verify_sign_in_code,
        farmer_handlers::get_profile,
        farmer_handlers::update_profile,
        health::liveness,
        health::readiness,
    ),
    components(schemas(
        CreateFarmParams,
        RepaintFarmCellsParams,
        PointParams,
        CellParams,
        FarmSummaryResponse,
        FarmResponse,
        SavedFarmResponse,
        CropAreaResponse,
        CentroidResponse,
        OutlinePointResponse,
        CellResponse,
        GridCellResponse,
        Crop,
        FarmsResponse,
        OneFarmResponse,
        SendSignInCodeParams,
        SignInCodeSentResponse,
        VerifySignInCodeParams,
        SignedInResponse,
        EditProfileParams,
        ProfileResponse,
        Language,
        ErrorBody
    )),
    modifiers(&BearerAuth),
    tags(
        (name = "farm-doctor-api", description = "Farm Doctor API"),
        (name = "farmers", description = "Sign in with a phone and a code, and the farmer's profile"),
        (name = "farms", description = "A farmer's farms: outline, cell grid and crops")
    ),
    info(
        title = "farm-doctor-api",
        version = "1.0.0",
        description = "API behind the farmer app and the Ministry dashboard."
    ),
    servers((url = "/", description = "API"))
)]
pub struct ApiDoc;

pub fn swagger_ui() -> SwaggerUi {
    SwaggerUi::new("/api-docs").url("/api-docs/openapi.json", ApiDoc::openapi())
}
