use std::sync::Arc;

use serde_json::json;
use tide::{http::Mime, Response};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::Config;

use crate::todo::Store;

#[async_std::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    let config = Arc::new(Config::from("/api-doc/openapi.json"));
    let mut app = tide::with_state(config);

    #[derive(OpenApi)]
    #[openapi(
        handlers(
            todo::list_todos,
            todo::create_todo,
            todo::delete_todo,
            todo::mark_done
        ),
        components(
            schemas(todo::Todo, todo::TodoError)
        ),
        modifiers(&SecurityAddon),
        tags(
            (name = "todo", description = "Todo items management endpoints.")
        )
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

    // serve OpenApi json
    app.at("/api-doc/openapi.json")
        .get(|_| async move { Ok(Response::builder(200).body(json!(ApiDoc::openapi()))) });

    // serve Swagger UI
    app.at("/swagger-ui/*").get(serve_swagger);

    app.at("/api").nest({
        let mut todos = tide::with_state(Store::default());

        todos.at("/todo").get(todo::list_todos);
        todos.at("/todo").post(todo::create_todo);
        todos.at("/todo/:id").delete(todo::delete_todo);
        todos.at("/todo/:id").put(todo::mark_done);

        todos
    });

    app.listen("0.0.0.0:8080").await
}

async fn serve_swagger(request: tide::Request<Arc<Config<'_>>>) -> tide::Result<Response> {
    let config = request.state().clone();
    let path = request.url().path().to_string();
    let tail = path.strip_prefix("/swagger-ui/").unwrap();

    match utoipa_swagger_ui::serve(tail, config) {
        Ok(swagger_file) => swagger_file
            .map(|file| {
                Ok(Response::builder(200)
                    .body(file.bytes.to_vec())
                    .content_type(file.content_type.parse::<Mime>()?)
                    .build())
            })
            .unwrap_or_else(|| Ok(Response::builder(404).build())),
        Err(error) => Ok(Response::builder(500).body(error.to_string()).build()),
    }
}

mod todo {
    use std::sync::{Arc, Mutex};

    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use tide::{Request, Response};
    use utoipa::ToSchema;

    /// Item to complete
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub(super) struct Todo {
        /// Unique database id for `Todo`
        #[schema(example = 1)]
        id: i32,
        /// Description of task to complete
        #[schema(example = "Buy coffee")]
        value: String,
        /// Indicates whether task is done or not
        done: bool,
    }

    /// Error that might occur when managing `Todo` items
    #[derive(Serialize, Deserialize, ToSchema)]
    pub(super) enum TodoError {
        /// Happens when Todo item alredy exists
        Config(String),
        /// Todo not found from storage
        NotFound(String),
    }

    pub(super) type Store = Arc<Mutex<Vec<Todo>>>;

    /// List todos from in-memory stoarge.
    ///
    /// List all todos from in memory storage.
    #[utoipa::path(
        get,
        path = "/api/todo",
        responses(
            (status = 200, description = "List all todos successfully", body = [Todo])
        )
    )]
    pub(super) async fn list_todos(req: Request<Store>) -> tide::Result<Response> {
        let todos = req.state().lock().unwrap().clone();

        Ok(Response::builder(200).body(json!(todos)).build())
    }

    /// Create new todo
    ///
    /// Create new todo to in-memory storage if not exists.
    #[utoipa::path(
        post,
        path = "/api/todo",
        request_body = Todo,
        responses(
            (status = 201, description = "Todo created successfully", body = Todo),
            (status = 409, description = "Todo already exists", body = TodoError, example = json!(TodoError::Config(String::from("id = 1"))))
        )
    )]
    pub(super) async fn create_todo(mut req: Request<Store>) -> tide::Result<Response> {
        let new_todo = req.body_json::<Todo>().await?;
        let mut todos = req.state().lock().unwrap();

        todos
            .iter()
            .find(|existing| existing.id == new_todo.id)
            .map(|existing| {
                Ok(Response::builder(409)
                    .body(json!(TodoError::Config(format!("id = {}", existing.id))))
                    .build())
            })
            .unwrap_or_else(|| {
                todos.push(new_todo.clone());

                Ok(Response::builder(200).body(json!(new_todo)).build())
            })
    }

    /// Delete todo by id.
    ///
    /// Delete todo from in-memory storage.
    #[utoipa::path(
        delete,
        path = "/api/todo/{id}",
        responses(
            (status = 200, description = "Todo deleted successfully"),
            (status = 401, description = "Unauthorized to delete Todo"),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Id of todo item to delete")
        ),
        security(
            ("api_key" = [])
        )
    )]
    pub(super) async fn delete_todo(req: Request<Store>) -> tide::Result<Response> {
        let id = req.param("id")?.parse::<i32>()?;
        let api_key = req
            .header("todo_apikey")
            .map(|header| header.as_str().to_string())
            .unwrap_or_default();

        if api_key != "utoipa-rocks" {
            return Ok(Response::new(401));
        }

        let mut todos = req.state().lock().unwrap();

        let old_size = todos.len();

        todos.retain(|todo| todo.id != id);

        if old_size == todos.len() {
            Ok(Response::builder(404)
                .body(json!(TodoError::NotFound(format!("id = {id}"))))
                .build())
        } else {
            Ok(Response::new(200))
        }
    }

    /// Mark todo done by id
    #[utoipa::path(
        put,
        path = "/api/todo/{id}",
        responses(
            (status = 200, description = "Todo marked done successfully"),
            (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
        ),
        params(
            ("id" = i32, Path, description = "Id of todo item to mark done")
        )
    )]
    pub(super) async fn mark_done(req: Request<Store>) -> tide::Result<Response> {
        let id = req.param("id")?.parse::<i32>()?;
        let mut todos = req.state().lock().unwrap();

        todos
            .iter_mut()
            .find(|todo| todo.id == id)
            .map(|todo| {
                todo.done = true;
                Ok(Response::new(200))
            })
            .unwrap_or_else(|| {
                Ok(Response::builder(404)
                    .body(json!(TodoError::NotFound(format!("id = {id}"))))
                    .build())
            })
    }
}
