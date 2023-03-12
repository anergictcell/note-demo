use axum::routing::post;
use axum::Json;
use axum::Router;
use models::Tag;
use models::note::Draft;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use axum::extract;
use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;

use models::note::Note;

use persistence::memory::InMemoryStorage;
use persistence::Persister;

use crate::models::User;

mod models;
mod persistence;

struct AppState<P>
where
    P: for<'a> Persister<'a>,
{
    // Using the std::sync::Mutex here instead of axum's async Mutex because
    // this PoC does not use IO-heavy operations.
    // https://docs.rs/tokio/1.25.0/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use
    data: Arc<Mutex<P>>,
}

// Clone is manually implemented because Derive does not work with the trait
impl<P: for<'a> Persister<'a>> Clone for AppState<P> {
    fn clone(&self) -> Self {
        AppState {
            data: self.data.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    // Activate logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("NOTE_VERBOSITY"))
        .init();

    // state is the data backend - here it is InMemoryStorage
    let state = AppState {
        data: Arc::new(Mutex::new(InMemoryStorage::default())),
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/notes", get(notes))
        .route("/notes/tag/:tag_label", get(tagged_notes))
        .route(
            "/note/:id",
            get(get_note).put(edit_note).delete(delete_note),
        )
        .route("/note", post(add_note))
        .route("/tags", get(tags))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Used for debugging => Returns all notes
async fn root<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    info!("GET /");
    let data = state.data.lock().expect("mutex was poisoned");
    let res = data.notes().cloned().collect::<Vec<Note>>();
    info!("--> 200 [{} notes]", res.len());
    Ok(Json(res))
}

/// Returns all notes from the user sending the request
async fn notes<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    info!("GET /notes/");
    // TODO: Implement actual user handling
    let user = User::default();
    let data = state.data.lock().expect("mutex was poisoned");
    let res = data.user_notes(&user).cloned().collect::<Vec<Note>>();
    info!("--> 200 [{} notes]", res.len());
    Ok(Json(res))
}

/// Returns a single note from the user sending the request
async fn get_note<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
    Path(id): Path<usize>,
) -> Result<Json<Note>, (StatusCode, String)> {
    info!("GET /note/{}", id);
    // TODO: Implement actual user handling
    let user = User::default();
    let data = state.data.lock().expect("mutex was poisoned");
    let Some(note) = data.note(id.into()) else {
        info!("--> 404");
        return Err((StatusCode::NOT_FOUND, "Note does not exist".to_string())) 
    };
    if note.user() == user.id() {
        info!("--> 200");
        Ok(Json(note.clone()))
    } else {
        info!("--> 401");
        Err((
            StatusCode::UNAUTHORIZED,
            "Note belongs to other user".to_string(),
        ))
    }
}

/// Creates a new note and stores it
async fn add_note<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
    extract::Json(draft): extract::Json<Draft>,
) -> Result<Json<Note>, (StatusCode, String)> {
    // TODO: Implement actual user handling
    let user = User::default();
    info!("POST /note/{}", draft.title());
    let mut data = state.data.lock().expect("mutex was poisoned");
    info!("--> 200");
    Ok(Json(data.add_note(draft, &user).clone()))
}

/// Modifies an existing note of the user sending the request
async fn edit_note<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
    Path(id): Path<usize>,
    extract::Json(draft): extract::Json<Draft>,
) -> Result<Json<Note>, (StatusCode, String)> {
    // TODO: Implement actual user handling
    let user = User::default();
    info!("PUT /note/{}", id);
    let mut data = state.data.lock().expect("mutex was poisoned");
    let Some(note) = data.note(id.into()) else {
        info!("--> 404");
        return Err((StatusCode::NOT_FOUND, "Note does not exist".to_string()))
    };
    if note.user() != user.id() {
        info!("--> 401");
        return Err((
            StatusCode::UNAUTHORIZED,
            "Note belongs to other user".to_string(),
        ));
    }
    info!("--> 200");
    Ok(Json(data.update_note(draft, id.into()).clone()))
}

/// Deletes an existing note of the user sending the request
async fn delete_note<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
    Path(id): Path<usize>,
) -> Result<Json<()>, (StatusCode, String)> {
    // TODO: Implement actual user handling
    let user = User::default();
    info!("DELETE /note/{}", id);
    let mut data = state.data.lock().expect("mutex was poisoned");
    let Some(note) = data.note(id.into()) else {
        info!("--> 404");
        return Err((StatusCode::NOT_FOUND, "Note does not exist".to_string()))
    };
    if note.user() != user.id() {
        info!("--> 401");
        return Err((
            StatusCode::UNAUTHORIZED,
            "Note belongs to other user".to_string(),
        ));
    }
    data.delete_note(id.into());
    info!("--> 200");
    Ok(Json(()))
}

/// Returns all notes from the user sending the request with the provided tag
async fn tagged_notes<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
    Path(tag_label): Path<String>,
) -> Result<Json<Vec<Note>>, (StatusCode, String)> {
    info!("GET /notes/tag/{}", tag_label);
    // TODO: Implement actual user handling
    let user = User::default();
    let data = state.data.lock().expect("mutex was poisoned");
    let Some(tag) = data.tag(&tag_label) else {
        info!("--> 400");
        return Err((StatusCode::BAD_REQUEST, "Tag does not exist".to_string()))
    };

    let res = data
        .tagged_notes(tag)
        .filter(|note| note.user() == user.id())
        .cloned()
        .collect::<Vec<Note>>();
    info!("--> 200 [{} notes]", res.len());
    Ok(Json(res))
}

/// Returns all notes from the user sending the request
async fn tags<P: for<'a> persistence::Persister<'a>>(
    State(state): State<AppState<P>>,
) -> Result<Json<Vec<Tag>>, (StatusCode, String)> {
    info!("GET /tags/");
    let data = state.data.lock().expect("mutex was poisoned");
    let res = data.tags().cloned().collect::<Vec<Tag>>();
    info!("--> 200 [{} tags]", res.len());
    Ok(Json(res))
}
