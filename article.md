# Getting Started with Axum and Tokio: Building a REST API in Rust

Welcome! If you're curious about building web APIs in Rust, you've come to the right place. In this tutorial, we'll build a simple REST API together using two powerful tools: **Tokio** and **Axum**.

**Tokio** is an async runtime that enables Rust applications to handle thousands of concurrent connections efficiently. Think of it as the engine that powers asynchronous operations in Rust.

**Axum** is a modern web framework built by the Tokio team. It's designed to be ergonomic, type-safe, and blazingly fast. Together, they make building web services in Rust a pleasant experience.

We'll build a simple task management API with full CRUD operations (Create, Read, Update, Delete). By the end, you'll understand how to handle routes, manage shared state, and work with JSON data.

## Project Setup

Let's start by creating a new Rust project and adding our dependencies. Here's what our `Cargo.toml` needs:

```toml
[package]
name = "tasks-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tower-http = { version = "0.6", features = ["trace"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

Here's what each dependency does:
- **axum**: Our web framework
- **tokio**: The async runtime (with "full" features for all functionality)
- **serde**: Serialization/deserialization for JSON
- **tower-http**: Middleware utilities (we'll use tracing)
- **tracing**: Logging and observability

## Building Our Data Model

Let's define what a task looks like. We'll keep it simple with an ID, title, and completion status:

```rust
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    id: u64,
    title: String,
    completed: bool,
}

// Our shared application state
type AppState = Arc<RwLock<Vec<Task>>>;
```

The `AppState` type might look complex, but it's a common pattern in Rust web applications:
- `Vec<Task>`: Our in-memory collection of tasks
- `RwLock`: Allows multiple readers or one writer (safe concurrent access)
- `Arc`: Allows sharing the state across multiple handlers safely

## Creating Our First Handler

Let's build our first endpoint to list all tasks:

```rust
use axum::{extract::State, Json};

async fn list_tasks(State(state): State<AppState>) -> Json<Vec<Task>> {
    let tasks = state.read().await;
    Json(tasks.clone())
}
```

This is an async function that:
1. Uses the `State` extractor to access our shared data
2. Acquires a read lock on the task list
3. Returns the tasks as JSON automatically

Axum handles all the serialization for us. The `Json` wrapper tells Axum to convert our data to JSON and set the correct `Content-Type` header.

## Building the CRUD Operations

Now let's implement the remaining operations. First, creating a new task:

```rust
#[derive(Deserialize)]
struct CreateTask {
    title: String,
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let mut tasks = state.write().await;
    
    let new_task = Task {
        id: tasks.len() as u64 + 1,
        title: payload.title,
        completed: false,
    };
    
    tasks.push(new_task.clone());
    (StatusCode::CREATED, Json(new_task))
}
```

Notice how the `Json` extractor automatically deserializes the request body into our `CreateTask` struct. Axum validates the JSON and handles errors for us.

Next, getting a specific task by ID:

```rust
use axum::extract::Path;

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Task>, StatusCode> {
    let tasks = state.read().await;
    
    tasks
        .iter()
        .find(|task| task.id == id)
        .cloned()
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}
```

The `Path` extractor pulls the ID from the URL path and parses it into a `u64`. If the task isn't found, we return a 404 status code.

Finally, deleting a task:

```rust
async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> StatusCode {
    let mut tasks = state.write().await;
    
    if let Some(pos) = tasks.iter().position(|task| task.id == id) {
        tasks.remove(pos);
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}
```

## Setting Up Routes and Server

Now let's wire everything together with a router and add some middleware:

```rust
use axum::{routing::get, routing::post, routing::delete, Router};
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create shared state
    let state = Arc::new(RwLock::new(Vec::<Task>::new()));
    
    // Build our router
    let app = Router::new()
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/:id", get(get_task).delete(delete_task))
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    
    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    
    println!("Server running on http://127.0.0.1:3000");
    
    axum::serve(listener, app).await.unwrap();
}
```

The `#[tokio::main]` macro sets up the Tokio runtime for us. The `TraceLayer` middleware automatically logs all incoming requests and responses - very helpful for debugging!

Notice how routes can handle multiple HTTP methods. `/tasks` responds to both GET (list) and POST (create), while `/tasks/:id` handles GET (get one) and DELETE.

## Error Handling

For production applications, you'll want better error handling. Here's a pattern using custom error types:

```rust
use axum::response::{IntoResponse, Response};

enum AppError {
    NotFound,
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Task not found"),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal error"),
        };
        
        (status, message).into_response()
    }
}
```

By implementing `IntoResponse`, you can return your custom error type directly from handlers, and Axum will convert it to the appropriate HTTP response.

## Testing Your API

Let's test our API using curl:

```bash
# Create a task
curl -X POST http://127.0.0.1:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title":"Learn Axum"}'

# List all tasks
curl http://127.0.0.1:3000/tasks

# Get a specific task
curl http://127.0.0.1:3000/tasks/1

# Delete a task
curl -X DELETE http://127.0.0.1:3000/tasks/1
```

Run your server with `cargo run` and try these commands. You should see JSON responses and logging output in your terminal!

## Conclusion

Congratulations! You've built a working REST API with Axum and Tokio. Let's recap what makes this combination powerful:

- **Type Safety**: Axum's extractors catch errors at compile time, not runtime
- **Performance**: Tokio's async runtime handles thousands of concurrent connections efficiently
- **Ergonomics**: Minimal boilerplate, automatic JSON serialization, and great error messages
- **Ecosystem**: Seamless integration with Tower middleware and the broader Rust ecosystem

### Next Steps

To take your API further, consider:
- Adding a real database (PostgreSQL with SQLx, for example)
- Implementing authentication and authorization
- Adding validation with the `validator` crate
- Exploring more Tower middleware (CORS, rate limiting, compression)
- Writing tests for your handlers

The [Axum documentation](https://docs.rs/axum) and [Tokio documentation](https://docs.rs/tokio) are excellent resources for diving deeper. The Axum GitHub repository also has many example projects to learn from.

Happy coding!
