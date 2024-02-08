use crate::{handlers::admin, AppState};
use axum::{
    middleware,
    routing::{delete, get, patch, post},
    Router,
};

pub fn admin_router(app_state: AppState) -> Router {
    Router::new()
        .route("/list", get(admin::handle_admin_list))
        .route("/add", post(admin::handle_admin_add))
        .route("/delete", delete(admin::handle_admin_delete))
        .route("/update", patch(admin::handle_admin_update))
        .with_state(app_state.clone())
        .route_layer(middleware::from_fn_with_state(
            app_state,
            crate::middleware::mw_auth_admin::admin_auth,
        ))
}
