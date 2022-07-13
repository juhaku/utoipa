use std::sync::Mutex;

use actix_web::{
    delete, get, post, put,
    web::{Data, Json, Path, Query, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};
use utoipa::{Component, IntoParams};

use crate::{LogApiKey, RequireApiKey};

#[derive(Default)]
pub(super) struct TodoStore {
    todos: Mutex<Vec<Todo>>,
}

pub(super) fn configure(store: Data<TodoStore>) -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .app_data(store)
            .service(search_todos)
            .service(get_todos)
            .service(create_todo)
            .service(delete_todo)
            .service(get_todo_by_id)
            .service(update_todo);
    }
}

/// Task to do.
#[derive(Serialize, Deserialize, Component, Clone, Debug)]
pub(super) struct Todo {
    /// Unique id for the todo item.
    #[component(example = 1)]
    id: i32,
    /// Description of the taks to do.
    #[component(example = "Remember to buy groceries")]
    value: String,
    /// Mark is the task done or not
    checked: bool,
}

/// Request to update existing `Todo` item.
#[derive(Serialize, Deserialize, Component, Clone, Debug)]
pub(super) struct TodoUpdateRequest {
    /// Optional new value for the `Todo` task.
    #[component(example = "Dentist at 14.00")]
    value: Option<String>,
    /// Optional check status to mark is the task done or not.
    checked: Option<bool>,
}

/// Todo endpoint error responses
#[derive(Serialize, Deserialize, Clone, Component)]
pub(super) enum ErrorResponse {
    /// When Todo is not found by search term.
    NotFound(String),
    /// When there is a conflict storing a new todo.
    Conflict(String),
    /// When todo enpoint was called without correct credentials
    Unauthorized(String),
}

/// Get list of todos.
///
/// List todos from in-memory todo store.
///
/// One could call the api endpoit with following curl.
/// ```text
/// curl localhost:8080/todo
/// ```
#[utoipa::path(
    responses(
        (status = 200, description = "List current todo items", body = [Todo])
    )
)]
#[get("/todo")]
pub(super) async fn get_todos(todo_store: Data<TodoStore>) -> impl Responder {
    let todos = todo_store.todos.lock().unwrap();

    HttpResponse::Ok().json(todos.clone())
}

/// Create new Todo to shared in-memory storage.
///
/// Post a new `Todo` in request body as json to store it. Api will return
/// created `Todo` on success or `ErrorResponse::Conflict` if todo with same id already exists.
///
/// One could call the api with.
/// ```text
/// curl localhost:8080/todo -d '{"id": 1, "value": "Buy movie ticket", "checked": false}'
/// ```
#[utoipa::path(
    request_body = Todo,
    responses(
        (status = 201, description = "Todo created successfully", body = Todo),
        (status = 409, description = "Todo with id already exists", body = ErrorResponse, example = json!(ErrorResponse::Conflict(String::from("id = 1"))))
    )
)]
#[post("/todo")]
pub(super) async fn create_todo(todo: Json<Todo>, todo_store: Data<TodoStore>) -> impl Responder {
    let mut todos = todo_store.todos.lock().unwrap();
    let todo = &todo.into_inner();

    todos
        .iter()
        .find(|existing| existing.id == todo.id)
        .map(|existing| {
            HttpResponse::Conflict().json(ErrorResponse::Conflict(format!("id = {}", existing.id)))
        })
        .unwrap_or_else(|| {
            todos.push(todo.clone());

            HttpResponse::Ok().json(todo)
        })
}

/// Delete Todo by given path variable id.
///
/// This ednpoint needs `api_key` authentication in order to call. Api key can be found from README.md.
///
/// Api will delete todo from shared in-memory storage by the provided id and return success 200.
/// If storage does not contain `Todo` with given id 404 not found will be returned.
#[utoipa::path(
    responses(
        (status = 200, description = "Todo deleted successfully"),
        (status = 401, description = "Unauthorized to delete Todo", body = ErrorResponse, example = json!(ErrorResponse::Unauthorized(String::from("missing api key")))),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    ),
    security(
        ("api_key" = [])
    )
)]
#[delete("/todo/{id}", wrap = "RequireApiKey")]
pub(super) async fn delete_todo(id: Path<i32>, todo_store: Data<TodoStore>) -> impl Responder {
    let mut todos = todo_store.todos.lock().unwrap();
    let id = id.into_inner();

    let new_todos = todos
        .iter()
        .filter(|todo| todo.id != id)
        .cloned()
        .collect::<Vec<_>>();

    if new_todos.len() == todos.len() {
        HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
    } else {
        *todos = new_todos;
        HttpResponse::Ok().finish()
    }
}

/// Get Todo by given todo id.
///
/// Return found `Todo` with status 200 or 404 not found if `Todo` is not found from shared in-memory storage.
#[utoipa::path(
    responses(
        (status = 200, description = "Todo found from storage", body = Todo),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    )
)]
#[get("/todo/{id}")]
pub(super) async fn get_todo_by_id(id: Path<i32>, todo_store: Data<TodoStore>) -> impl Responder {
    let todos = todo_store.todos.lock().unwrap();
    let id = id.into_inner();

    todos
        .iter()
        .find(|todo| todo.id == id)
        .map(|todo| HttpResponse::Ok().json(todo))
        .unwrap_or_else(|| {
            HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
        })
}

/// Update Todo with given id.
///
/// This endpoint supports optional authentication.
///
/// Tries to update `Todo` by given id as path variable. If todo is found by id values are
/// updated according `TodoUpdateRequest` and updated `Todo` is returned with status 200.
/// If todo is not found then 404 not found is returned.
#[utoipa::path(
    request_body = TodoUpdateRequest,
    responses(
        (status = 200, description = "Todo updated successfully", body = Todo),
        (status = 404, description = "Todo not found by id", body = ErrorResponse, example = json!(ErrorResponse::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id", description = "Unique storage id of Todo")
    ),
    security(
        (),
        ("api_key" = [])
    )
)]
#[put("/todo/{id}", wrap = "LogApiKey")]
pub(super) async fn update_todo(
    id: Path<i32>,
    todo: Json<TodoUpdateRequest>,
    todo_store: Data<TodoStore>,
) -> impl Responder {
    let mut todos = todo_store.todos.lock().unwrap();
    let id = id.into_inner();
    let todo = todo.into_inner();

    todos
        .iter_mut()
        .find_map(|todo| if todo.id == id { Some(todo) } else { None })
        .map(|existing_todo| {
            if let Some(checked) = todo.checked {
                existing_todo.checked = checked;
            }
            if let Some(value) = todo.value {
                existing_todo.value = value;
            }

            HttpResponse::Ok().json(existing_todo)
        })
        .unwrap_or_else(|| {
            HttpResponse::NotFound().json(ErrorResponse::NotFound(format!("id = {id}")))
        })
}

/// Search todos Query
#[derive(Deserialize, Debug, IntoParams)]
pub(super) struct SearchTodos {
    /// Content that should be found from Todo's value field
    value: String,
}

/// Search Todos with by value
///
/// Perform search from `Todo`s present in in-memory storage by matching Todo's value to
/// value provided as query paramter. Returns 200 and matching `Todo` items.
#[utoipa::path(
    params(
        SearchTodos
    ),
    responses(
        (status = 200, description = "Search Todos did not result error", body = [Todo]),
    )
)]
#[get("/todo/search")]
pub(super) async fn search_todos(
    query: Query<SearchTodos>,
    todo_store: Data<TodoStore>,
) -> impl Responder {
    let todos = todo_store.todos.lock().unwrap();

    HttpResponse::Ok().json(
        todos
            .iter()
            .filter(|todo| {
                todo.value
                    .to_lowercase()
                    .contains(&query.value.to_lowercase())
            })
            .cloned()
            .collect::<Vec<_>>(),
    )
}
