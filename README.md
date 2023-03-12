# Note taking app in Rust

This is a small demo / learning project to build a Note-taking app in `Rust` with `axum`. It is inspired by <https://github.com/thermondo/backend-coding-challenge-django> which I came across recently. Instead of using Django and existing frameworks, I wanted to build something similar in Rust and it should not take me more than 1 day.

## Goal
- Build a REST API to
    - create notes
    - view notes
    - edit notes
    - delete notes
    - tag notes
    - search notes
- Develop a protype data storage layer
- Develop an interface that could allow plugging in other data storage engines or ORM
- Keep in mind to use user-permission based filtering

## Non goals
I'm building this PoC for learning purposes, not to develop an actual maintainable application. Because of this, I decided to not use
existing frameworks that should be used as best-practice normally.
- I try to not use existing ORM frameworks (e.g. `diesel`, `sea_orm`)
- I am not building a UI client
- The app will be far from Production-grade safety
- I will not over-optimize, e.g. extra Multithreading
- I will not prepare the app for fullstack deployments, with database, load-balancers, secrets
- I will not implement actual authentication functionality. This is a topic that is too difficult to do right, so I will instead rely on existing middleware for this. But for now, I'm skipping that part and only implement some groundwork to plug in an auth service later.

## Design plan
- Use `axum` for the REST API, provide basic CRUD-like endpoints
- Have a datastorage backend layer that handles data persistence
    - initially use an in-memory store
    - allow simple adaption to use database backends, MySQL, SQLite etc
        - use a trait as translation from API to DB
    - allow simple text (or JSON) based data storage

### Limitations
The in-memory storage system does not have a proper index system for foreign keys and instead uses `Iterator` and `Filter` a lot. This was
done intentionally, because I did not want to add too much extra complexity and Rust iterators are fast enough for my PoC to not need indicies.
For now, I did not want to use a proper SQL or document-based database backend for this PoC, but tried to develop a somewhat flexible
API that would allow to switch to another backend storage by implementing the `Persister` trait.


## Disclaimer
I developed this app on a weekend on the side as a proof of concept for me to have a look at `axum`. I did not want to
go and copy&paste code snippets to build an app with existing frameworks and middleware, but instead build things from scratch.
This is not a very smart approach in general, but helped me in understanding several design choices of axum and address some more complex issues
with the trait system.

Please don't use this code as an inspiration for proper backend service architecture.

## Test it yourself
- Clone the repo: `git clone https://github.com/anergictcell/note-demo.git`
- Run it (you must have the Rust toolchain installed): `cd note-demo/ && cargo run`
    - If you want some debug-logging, run with `NOTE_VERBOSITY=4 cargo run`

### Add notes:
```bash
curl \
-X POST \
-H "Content-Type: application/json" \
--data-raw '{"title": "My note", "body": "I have to prepare a UI", "tags": ["todo", "ui"],  "visibility": "Public"}' \
127.0.0.1:3000/note
```

### Modify a note
```bash
curl \
-X PUT \
-H "Content-Type: application/json" \
--data-raw '{"title": "Build UI", "body": "I __really__ have to prepare a UI", "tags": ["todo", "ui", "urgent"],  "visibility": "Public"}' \
127.0.0.1:3000/note/0
```
The `0` is a placeholder for the Id of the note.

### Query notes:
- All notes: `http://127.0.0.1:3000/notes`
- A single note: `http://127.0.0.1:3000/note/0`
- Filter by tags: `http://127.0.0.1:3000/notes/tag/urgent`
- Show all tags: `http://127.0.0.1:3000/tags`

### Delete a note
```bash
curl -X DELETE 127.0.0.1:3000/note/0
```
