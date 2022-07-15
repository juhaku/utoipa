use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing, Extension, Router, Server};
use hyper::Error;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::todo::Store;

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[derive(OpenApi)]
    #[openapi(
        handlers(
            todo::list_todos,
            todo::search_todos,
            todo::create_todo,
            todo::mark_done,
            todo::delete_todo,
        ),
        components(todo::Todo, todo::TodoError),
        modifiers(&SecurityAddon),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            if let Some(components) = openapi.components.as_mut() {
                components.add_security_scheme(
                    "api_key",
                    SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
                )
            }
        }
    }

    let store = Arc::new(Store::default());
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui/*tail").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route(
            "/todo",
            routing::get(todo::list_todos).post(todo::create_todo),
        )
        .route("/todo/search", routing::get(todo::search_todos))
        .route(
            "/todo/:id",
            routing::put(todo::mark_done).delete(todo::delete_todo),
        )
        .layer(Extension(store));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    Server::bind(&address).serve(app.into_make_service()).await
}

mod todo {
    use std::sync::Arc;

    use axum::{
        extract::{Path, Query},
        response::IntoResponse,
        Extension, Json,
    };
    use hyper::{HeaderMap, StatusCode};
    use serde::{Deserialize, Serialize};
    use tokio::sync::Mutex;
    use utoipa::{Component, IntoParams};

    /// In-memonry todo store
    pub(super) type Store = Mutex<Vec<Todo>>;

    /// Item to do.
    #[derive(Serialize, Deserialize, Component, Clone)]
    pub(super) struct Todo {
        id: i32,
        #[component(example = "Buy groceries")]
        value: String,
        done: bool,
    }

    /// Todo operation errors
    #[derive(Serialize, Deserialize, Component)]
    pub(super) enum TodoError {
        /// Todo already exists conflict.
        #[component(example = "Todo already exists")]
        Conflict(String),
        /// Todo not found by id.
        #[component(example = "id = 1")]
        NotFound(String),
        /// Todo operation unauthorized
        #[component(example = "missing api key")]
        Unauthorized(String),
    }

    /// List all Todo items
    ///
    /// List all Todo items from in-memory storage.
    #[utoipa::path(
        get,
        path = "/todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    pub(super) async fn list_todos(Extension(store): Extension<Arc<Store>>) -> Json<Vec<Todo>> {
        let todos = store.lock().await.clone();

        Json(todos)
    }

    /// Todo search query
    #[derive(Deserialize, IntoParams)]
    pub(super) struct TodoSearchQuery {
        /// Search by value. Search is incase sensitive.
        value: String,
        /// Search by `done` status.
        done: bool,
    }

    /// Search Todos by query params.
    ///
    /// Search `Todo`s by query parmas and return matching `Todo`s.
    #[utoipa::path(
        get,
        path = "/todo/search",
        params(
            TodoSearchQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = [Todo])
        )
    )]
    pub(super) async fn search_todos(
        Extension(store): Extension<Arc<Store>>,
        query: Query<TodoSearchQuery>,
    ) -> Json<Vec<Todo>> {
        Json(
            store
                .lock()
                .await
                .iter()
                .filter(|todo| {
                    todo.value.to_lowercase() == query.value.to_lowercase()
                        && todo.done == query.done
                })
                .cloned()
                .collect(),
        )
    }

    /// Create new Todo
    ///
    /// Tries to create a new Todo item to in-memory storage or fails with 409 conflict if already exists.
    #[utoipa::path(
        post,
        path = "/todo",
        request_body = Todo,
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError)
        )
    )]
    pub(super) async fn create_todo(
        Json(todo): Json<Todo>,
        Extension(store): Extension<Arc<Store>>,
    ) -> impl IntoResponse {
        let mut todos = store.lock().await;

        todos
            .iter_mut()
            .find(|existing_todo| existing_todo.id == todo.id)
            .map(|found| {
                (
                    StatusCode::CONFLICT,
                    Json(TodoError::Conflict(format!(
                        "todo already exists: {}",
                        found.id
                    ))),
                )
                    .into_response()
            })
            .unwrap_or_else(|| {
                todos.push(todo.clone());

                (StatusCode::CREATED, Json(todo)).into_response()
            })
    }

    /// Mark Todo item done by id
    ///
    /// Mark Todo item done by given id. Return only status 200 on success or 404 if Todo is not found.
    #[utoipa::path(
        put,
        path = "/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found")
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            (), // <-- make optional authentication
            ("api_key" = [])
        )
    )]
    pub(super) async fn mark_done(
        Path(id): Path<i32>,
        Extension(store): Extension<Arc<Store>>,
        headers: HeaderMap,
    ) -> StatusCode {
        match check_api_key(false, headers) {
            Ok(_) => (),
            Err(_) => return StatusCode::UNAUTHORIZED,
        }

        let mut todos = store.lock().await;

        todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .map(|todo| {
                todo.done = true;
                StatusCode::OK
            })
            .unwrap_or(StatusCode::NOT_FOUND)
    }

    /// Delete Todo item by id
    ///
    /// Delete Todo item from in-memory storage by id. Returns either 200 success of 404 with TodoError if Todo is not found.
    #[utoipa::path(
        delete,
        path = "/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 401, description = "Unauthorized to delete Todo", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Todo database id")
        ),
        security(
            ("api_key" = [])
        )
    )]
    pub(super) async fn delete_todo(
        Path(id): Path<i32>,
        Extension(store): Extension<Arc<Store>>,
        headers: HeaderMap,
    ) -> impl IntoResponse {
        match check_api_key(true, headers) {
            Ok(_) => (),
            Err(error) => return error.into_response(),
        }

        let mut todos = store.lock().await;

        let len = todos.len();

        todos.retain(|todo| todo.id != id);

        if todos.len() != len {
            StatusCode::OK.into_response()
        } else {
            (
                StatusCode::NOT_FOUND,
                Json(TodoError::NotFound(format!("id = {id}"))),
            )
                .into_response()
        }
    }

    // normally you should create a midleware for this but this is sufficient for sake of example.
    fn check_api_key(require_api_key: bool, headers: HeaderMap) -> Result<(), impl IntoResponse> {
        match headers.get("todo_apikey") {
            Some(header) if header != "utoipa-rocks" => Err((
                StatusCode::UNAUTHORIZED,
                Json(TodoError::Unauthorized(String::from("incorrect api key"))),
            )
                .into_response()),
            None if require_api_key => Err((
                StatusCode::UNAUTHORIZED,
                Json(TodoError::Unauthorized(String::from("missing api key"))),
            )
                .into_response()),
            _ => Ok(()),
        }
    }
}
