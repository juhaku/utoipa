use std::net::Ipv4Addr;

use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify,
};
use utoipa_swagger_ui::Config;
use utoipa_warp::{openapi_json_filter, router::OpenApiRouter, routes, serving::swagger_ui_filter};
use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

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

    // Build the todo API router using utoipa-warp's OpenApiRouter
    let todo_router = OpenApiRouter::new()
        .routes(routes!(
            todo::list_todos, todo::create_todo, todo::delete_todo;
            filter = todo::handlers()
        ));

    // Nest under /api and extract the OpenApi spec
    let (api_filter, mut openapi) = OpenApiRouter::new()
        .nest("/api", todo_router)
        .split_for_parts();

    // Apply security modifier
    SecurityAddon.modify(&mut openapi);

    // Serve the OpenAPI JSON and Swagger UI using convenience filters
    let api_doc = openapi_json_filter("api-doc.json", openapi);
    let swagger_ui = swagger_ui_filter("swagger-ui", Config::from("/api-doc.json"));

    let all_routes = api_doc
        .or(swagger_ui)
        .unify()
        .or(api_filter)
        .unify();

    warp::serve(all_routes)
        .run((Ipv4Addr::UNSPECIFIED, 8080))
        .await
}

mod todo {
    use std::{
        convert::Infallible,
        sync::{Arc, Mutex},
    };

    use serde::{Deserialize, Serialize};
    use utoipa::{IntoParams, ToSchema};
    use warp::{hyper::StatusCode, Filter, Rejection, Reply};

    pub type Store = Arc<Mutex<Vec<Todo>>>;

    /// Item to complete.
    #[derive(Serialize, Deserialize, ToSchema, Clone)]
    pub struct Todo {
        /// Unique database id.
        #[schema(example = 1)]
        id: i64,
        /// Description of what need to be done.
        #[schema(example = "Buy movie tickets")]
        value: String,
    }

    #[derive(Debug, Deserialize, ToSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum Order {
        AscendingId,
        DescendingId,
    }

    #[derive(Debug, Deserialize, IntoParams)]
    #[into_params(parameter_in = Query)]
    pub struct ListQueryParams {
        /// Filters the returned `Todo` items according to whether they contain the specified string.
        #[param(style = Form, example = json!("task"))]
        contains: Option<String>,
        /// Order the returned `Todo` items.
        #[param(inline)]
        order: Option<Order>,
    }

    pub fn handlers() -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
        let store = Store::default();

        let list = warp::path("todo")
            .and(warp::get())
            .and(warp::path::end())
            .and(with_store(store.clone()))
            .and(warp::query::<ListQueryParams>())
            .and_then(list_todos);

        let create = warp::path("todo")
            .and(warp::post())
            .and(warp::path::end())
            .and(warp::body::json())
            .and(with_store(store.clone()))
            .and_then(create_todo);

        let delete = warp::path!("todo" / i64)
            .and(warp::delete())
            .and(warp::path::end())
            .and(with_store(store))
            .and(warp::header::header("todo_apikey"))
            .and_then(delete_todo);

        list.or(create).or(delete)
    }

    fn with_store(store: Store) -> impl Filter<Extract = (Store,), Error = Infallible> + Clone {
        warp::any().map(move || store.clone())
    }

    /// List todos from in-memory storage.
    ///
    /// List all todos from in-memory storage.
    #[utoipa::path(
        get,
        path = "/todo",
        params(ListQueryParams),
        responses(
            (status = 200, description = "List todos successfully", body = [Todo])
        )
    )]
    pub async fn list_todos(
        store: Store,
        query: ListQueryParams,
    ) -> Result<impl Reply, Infallible> {
        let todos = store.lock().unwrap();

        let mut todos: Vec<Todo> = if let Some(contains) = query.contains {
            todos
                .iter()
                .filter(|todo| todo.value.contains(&contains))
                .cloned()
                .collect()
        } else {
            todos.clone()
        };

        if let Some(order) = query.order {
            match order {
                Order::AscendingId => {
                    todos.sort_by_key(|todo| todo.id);
                }
                Order::DescendingId => {
                    todos.sort_by_key(|todo| todo.id);
                    todos.reverse();
                }
            }
        }

        Ok(warp::reply::json(&todos))
    }

    /// Create new todo item.
    ///
    /// Creates new todo item to in-memory storage if it is unique by id.
    #[utoipa::path(
        post,
        path = "/todo",
        request_body = Todo,
        responses(
            (status = 200, description = "Todo created successfully", body = Todo),
            (status = 409, description = "Todo already exists")
        )
    )]
    pub async fn create_todo(todo: Todo, store: Store) -> Result<Box<dyn Reply>, Infallible> {
        let mut todos = store.lock().unwrap();

        if todos
            .iter()
            .any(|existing_todo| existing_todo.id == todo.id)
        {
            Ok(Box::new(StatusCode::CONFLICT))
        } else {
            todos.push(todo.clone());

            Ok(Box::new(warp::reply::with_status(
                warp::reply::json(&todo),
                StatusCode::CREATED,
            )))
        }
    }

    /// Delete todo item by id.
    ///
    /// Delete todo item by id from in-memory storage.
    #[utoipa::path(
        delete,
        path = "/todo/{id}",
        responses(
            (status = 200, description = "Delete successful"),
            (status = 400, description = "Missing todo_apikey request header"),
            (status = 401, description = "Unauthorized to delete todo"),
            (status = 404, description = "Todo not found to delete"),
        ),
        params(
            ("id" = i64, Path, description = "Todo's unique id")
        ),
        security(
            ("api_key" = [])
        )
    )]
    pub async fn delete_todo(
        id: i64,
        store: Store,
        api_key: String,
    ) -> Result<impl Reply, Infallible> {
        if api_key != "utoipa-rocks" {
            return Ok(StatusCode::UNAUTHORIZED);
        }

        let mut todos = store.lock().unwrap();

        let size = todos.len();

        todos.retain(|existing| existing.id != id);

        if size == todos.len() {
            Ok(StatusCode::NOT_FOUND)
        } else {
            Ok(StatusCode::OK)
        }
    }
}
