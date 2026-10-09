use axum::{extract::Request, middleware::Next, response::Response};

use crate::app::{Action, AppError, Resource, StaffContext};

/// Lets a request through only when the signed-in staff member may do
/// `action` on `resource`. It reads the `StaffContext` that the `staff_auth`
/// middleware put on the request, so it must sit inside that layer.
///
/// Put it on each dashboard route, so that the permission a route needs is
/// written next to the route:
///
/// ```ignore
/// .route("/dams", get(list_dams).route_layer(require(Resource::Dams, Action::Read)))
/// ```
pub async fn check_permission(
    resource: Resource,
    action: Action,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let Some(staff) = req.extensions().get::<StaffContext>() else {
        // No staff on the request means the route was mounted outside the
        // staff sign-in layer: refuse instead of letting it through.
        return Err(AppError::Unauthorized(
            "No signed-in staff member on the request".to_string(),
        ));
    };

    if let Err(error) = staff.require(resource, action) {
        tracing::info!(
            staff_id = *staff.staff_id(),
            resource = %String::from(resource),
            action = %String::from(action),
            "dashboard request refused: permission missing"
        );

        return Err(error);
    }

    Ok(next.run(req).await)
}

/// The layer form of [`check_permission`] for one resource and action.
#[macro_export]
macro_rules! require {
    ($resource:expr, $action:expr) => {
        axum::middleware::from_fn(
            move |req: axum::extract::Request, next: axum::middleware::Next| {
                $crate::infra::http::check_permission($resource, $action, req, next)
            },
        )
    };
}
