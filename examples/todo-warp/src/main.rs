use std::{net::Ipv4Addr, sync::Arc};

use utoipa::OpenApi;
use utoipa_swagger_ui::Config;
use warp::{
    hyper::{Response, StatusCode},
    path::Tail,
    Filter, Rejection, Reply,
};

use crate::todo::Todo;

#[tokio::main]
async fn main() {
    env_logger::init();

    let config = Arc::new(Config::from("/api-doc.json"));

    #[derive(OpenApi)]
    #[openapi(
        handlers(todo::list_todos, todo::create_todo, todo::delete_todo),
        components(Todo),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
    struct ApiDoc;

    let api_doc = warp::path("api-doc.json")
        .and(warp::get())
        .map(|| warp::reply::json(&ApiDoc::openapi()));

    let swagger_ui = warp::path("swagger-ui")
        .and(warp::get())
        .and(warp::path::tail())
        .and(warp::any().map(move || config.clone()))
        .and_then(serve_swagger);

    warp::serve(api_doc.or(swagger_ui).or(todo::handlers()))
        .run((Ipv4Addr::UNSPECIFIED, 8080))
        .await
}

async fn serve_swagger(
    tail: Tail,
    config: Arc<Config<'static>>,
) -> Result<Box<dyn Reply + 'static>, Rejection> {
    let path = tail.as_str();
    match utoipa_swagger_ui::serve(path, config) {
        Ok(file) => {
            if let Some(file) = file {
                Ok(Box::new(
                    Response::builder()
                        .header("Content-Type", file.content_type)
                        .body(file.bytes),
                ))
            } else {
                Ok(Box::new(StatusCode::NOT_FOUND))
            }
        }
        Err(error) => Ok(Box::new(
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string()),
        )),
    }
}

mod todo {
    use std::{
        convert::Infallible,
        sync::{Arc, Mutex},
    };

    use serde::{Deserialize, Serialize};
    use utoipa::Component;
    use warp::{hyper::StatusCode, Filter, Reply};

    pub type Store = Arc<Mutex<Vec<Todo>>>;

    /// Item to complete.
    #[derive(Serialize, Deserialize, Component, Clone)]
    pub struct Todo {
        /// Unique database id.
        #[component(example = 1)]
        id: i64,
        /// Description of what need to be done.
        #[component(example = "Buy movie tickets")]
        value: String,
    }

    pub fn handlers() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let store = Store::default();

        let list = warp::path("todo")
            .and(warp::get())
            .and(warp::path::end())
            .and(with_store(store.clone()))
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
        responses(
            (status = 200, description = "List todos successfully", body = [Todo])
        )
    )]
    pub async fn list_todos(store: Store) -> Result<impl Reply, Infallible> {
        let todos = store.lock().unwrap();

        Ok(warp::reply::json(&todos.clone()))
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
            (status = 404, description = "Todo not found to delete")
        ),
        params(
            ("id" = i64, path, description = "Todo's unique id")
        )
    )]
    pub async fn delete_todo(id: i64, store: Store) -> Result<impl Reply, Infallible> {
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
