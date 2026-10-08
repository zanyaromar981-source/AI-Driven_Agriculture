use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    features::farms::web::{
        CellParams, CellResponse, CentroidResponse, CreateFarmParams, Crop, CropAreaResponse,
        FarmResponse, FarmSummaryResponse, GridCellResponse, OutlinePointResponse, PointParams,
        RepaintFarmCellsParams, SavedFarmResponse, handlers as farm_handlers,
    },
    infra::http::{ErrorWrapper, FieldError, Meta, ResponseWrapper, health},
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
        farm_handlers::repaint_farm_cells,
        farm_handlers::delete_farm,
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
        ResponseWrapper<SavedFarmResponse>,
        ResponseWrapper<Vec<FarmSummaryResponse>>,
        ErrorWrapper,
        FieldError,
        Meta
    )),
    modifiers(&BearerAuth),
    tags(
        (name = "farm-doctor-api", description = "Farm Doctor API"),
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
