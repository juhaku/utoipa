use rocket::{catch, catchers, routes, Build, Request, Rocket};
use serde_json::json;
use todo::RequireApiKey;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as ScalarServable};
use utoipa_swagger_ui::SwaggerUi;

use crate::todo::TodoStore;

#[rocket::launch]
fn rocket() -> Rocket<Build> {
    env_logger::init();

    #[derive(OpenApi)]
    #[openapi(
        paths(
            todo::get_tasks,
            todo::create_todo,
            todo::mark_done,
            todo::delete_todo,
            todo::search_todos,
        ),
        components(
            schemas(todo::Todo, todo::TodoError)
        ),
        tags(
            (name = "todo", description = "Todo management endpoints.")
        ),
        modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            )
        }
    }

    rocket::build()
        .manage(TodoStore::default())
        .register("/todo", catchers![unauthorized])
        .mount(
            "/",
            SwaggerUi::new("/swagger-ui/<_..>").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        // There is no need to create RapiDoc::with_openapi because the OpenApi is served
        // via SwaggerUi instead we only make rapidoc to point to the existing doc.
        .mount("/", RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
        // Alternative to above
        // .mount(
        //     "/",
        //     RapiDoc::with_openapi("/api-docs/openapi2.json", ApiDoc::openapi()).path("/rapidoc")
        // )
        .mount("/", Redoc::with_url("/redoc", ApiDoc::openapi()))
        .mount("/", Scalar::with_url("/scalar", ApiDoc::openapi()))
        .mount(
            "/todo",
            routes![
                todo::get_tasks,
                todo::create_todo,
                todo::mark_done,
                todo::delete_todo,
                todo::search_todos
            ],
        )
}

#[catch(401)]
async fn unauthorized(req: &Request<'_>) -> serde_json::Value {
    let (_, todo_error) = req.guard::<RequireApiKey>().await.failed().unwrap();

    json!(todo_error)
}

mod todo {
    use std::sync::{Arc, Mutex};

    use rocket::{
        delete, get,
        http::Status,
        outcome::Outcome,
        post, put,
        request::{self, FromRequest},
        response::{status::Custom, Responder},
        serde::json::Json,
        FromForm, Request, State,
    };
    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};

    pub(super) type TodoStore = Arc<Mutex<Vec<Todo>>>;

    /// Todo operation error.
    #[derive(Serialize, ToSchema, Responder, Debug)]
    pub(super) enum TodoError {
        /// When there is conflict creating a new todo.
        #[response(status = 409)]
        Conflict(String),

        /// When todo item is not found from storage.
        #[response(status = 404)]
        NotFound(String),

        /// When unauthorized to complete operation
        #[response(status = 401)]
        Unauthorized(String),
    }

    pub(super) struct RequireApiKey;

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for RequireApiKey {
        type Error = TodoError;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            match request.headers().get("todo_apikey").next() {
                Some("utoipa-rocks") => Outcome::Success(RequireApiKey),
                None => Outcome::Error((
                    Status::Unauthorized,
                    TodoError::Unauthorized(String::from("missing api key")),
                )),
                _ => Outcome::Error((
                    Status::Unauthorized,
                    TodoError::Unauthorized(String::from("invalid api key")),
                )),
            }
        }
    }

    pub(super) struct LogApiKey;

    #[rocket::async_trait]
    impl<'r> FromRequest<'r> for LogApiKey {
        type Error = TodoError;

        async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
            match request.headers().get("todo_apikey").next() {
                Some("utoipa-rocks") => {
                    log::info!("authenticated");
                    Outcome::Success(LogApiKey)
                }
                _ => {
                    log::info!("no api key");
                    Outcome::Forward(Status::Unauthorized)
                }
            }
        }
    }

    /// Task to do.
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub(super) struct Todo {
        /// Unique todo id.
        #[schema(example = 1)]
        id: i32,
        /// Description of a tasks.
        #[schema(example = "Buy groceries")]
        value: String,
        /// Indication whether task is done or not.
        done: bool,
    }

    /// List all available todo items.
    #[utoipa::path(
        context_path = "/todo",
        responses(
            (status = 200, description = "Get all todos", body = [Todo])
        )
    )]
    #[get("/")]
    pub(super) async fn get_tasks(store: &State<TodoStore>) -> Json<Vec<Todo>> {
        Json(store.lock().unwrap().clone())
    }

    /// Create new todo item.
    ///
    /// Create new todo item and add it to the storage.
    #[utoipa::path(
        context_path = "/todo",
        request_body = Todo,
        responses(
            (status = 201, description = "Todo item created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError, example = json!(TodoError::Conflict(String::from("id = 1"))))
        )
    )]
    #[post("/", data = "<todo>")]
    pub(super) async fn create_todo(
        todo: Json<Todo>,
        store: &State<TodoStore>,
    ) -> Result<Custom<Json<Todo>>, TodoError> {
        let mut todos = store.lock().unwrap();
        todos
            .iter()
            .find(|existing| existing.id == todo.id)
            .map(|todo| Err(TodoError::Conflict(format!("id = {}", todo.id))))
            .unwrap_or_else(|| {
                todos.push(todo.0.clone());

                Ok(Custom(Status::Created, Json(todo.0)))
            })
    }

    /// Mark Todo item done by given id
    ///
    /// Tries to find todo item by given id and mark it done if found. Will return not found in case todo
    /// item does not exists.
    #[utoipa::path(
        context_path = "/todo",
        responses(
            (status = 200, description = "Todo item marked done successfully"),
            (status = 404, description = "Todo item not found from storage", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id", description = "Todo item unique id")
        ),
        security(
            (),
            ("api_key" = [])
        )
    )]
    #[put("/<id>")]
    pub(super) async fn mark_done(
        id: i32,
        _api_key: LogApiKey,
        store: &State<TodoStore>,
    ) -> Result<Status, TodoError> {
        store
            .lock()
            .unwrap()
            .iter_mut()
            .find(|todo| todo.id == id)
            .map(|todo| {
                todo.done = true;

                Ok(Status::Ok)
            })
            .unwrap_or_else(|| Err(TodoError::NotFound(format!("id = {id}"))))
    }

    /// Delete Todo by given id.
    ///
    /// Delete Todo from storage by Todo id if found.
    #[utoipa::path(
        context_path = "/todo",
        responses(
            (status = 200, description = "Todo deleted successfully"),
            (status = 401, description = "Unauthorized to delete Todos", body = TodoError, example = json!(TodoError::Unauthorized(String::from("id = 1")))),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id", description = "Todo item id")
        ),
        security(
            ("api_key" = [])
        )
    )]
    #[delete("/<id>")]
    pub(super) async fn delete_todo(
        id: i32,
        _api_key: RequireApiKey,
        store: &State<TodoStore>,
    ) -> Result<Status, TodoError> {
        let mut todos = store.lock().unwrap();
        let len = todos.len();
        todos.retain(|todo| todo.id != id);

        if len == todos.len() {
            Err(TodoError::NotFound(format!("id = {id}")))
        } else {
            Ok(Status::Ok)
        }
    }

    #[derive(Deserialize, FromForm, IntoParams)]
    pub(super) struct SearchParams {
        /// Value to be search form `Todo`s
        value: String,
        /// Search whether todo is done
        done: Option<bool>,
    }

    /// Search Todo items by their value.
    ///
    /// Search is performed in case sensitive manner from value of Todo.
    #[utoipa::path(
        context_path = "/todo",
        params(
            SearchParams
        ),
        responses(
            (status = 200, description = "Found Todo items", body = [Todo])
        )
    )]
    #[get("/search?<search..>")]
    pub(super) async fn search_todos(
        search: SearchParams,
        store: &State<TodoStore>,
    ) -> Json<Vec<Todo>> {
        let SearchParams { value, done } = search;

        Json(
            store
                .lock()
                .unwrap()
                .iter()
                .filter(|todo| {
                    todo.value.to_lowercase().contains(&value.to_lowercase())
                        && done.map(|done| done == todo.done).unwrap_or(true)
                })
                .cloned()
                .collect(),
        )
    }
}
