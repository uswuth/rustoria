---
name: rust-diesel
description: Use when working with databases in Rust using Diesel ORM.
---

# Diesel — ORM and Query Builder for Rust

## Overview

Diesel is a safe, extensible ORM and query builder for Rust. It eliminates boilerplate for database interaction and
catches query errors at compile time by leveraging Rust's type system. Backends: **PostgreSQL**, **MySQL** (MariaDB
works via the MySQL backend — it is not a separate backend), and **SQLite**.

## Installation

```toml
[dependencies]
diesel = { version = "2.3", features = ["postgres"] }  # or "mysql", "sqlite"
dotenvy = "0.15"
```

Install Diesel CLI (same feature flags either way — match your backend):

```bash
cargo install diesel_cli --no-default-features --features postgres
# or the prebuilt binary via cargo-binstall:
cargo binstall diesel_cli --no-default-features --features postgres
```

## Setup

```bash
# Set database URL
echo DATABASE_URL=postgres://user:pass@localhost/mydb > .env

# Initialize
diesel setup

# Create migration
diesel migration generate create_posts
# Edit migrations/<timestamp>_create_posts/up.sql and down.sql

# Run pending migrations
diesel migration run

# Roll back the last migration batch
diesel migration revert

# Revert the last batch, then immediately re-run it (revert + run)
diesel migration redo
```

## Core Concepts

### 1. Schema

Diesel CLI auto-generates `src/schema.rs`:

```rust
diesel::table! {
    users (id) {
        id -> Int4,
        name -> Varchar,
    }
}

diesel::table! {
    posts (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Varchar,
        body -> Text,
        published -> Bool,
    }
}

diesel::joinable!(posts -> users (user_id));
diesel::allow_tables_to_appear_in_same_query!(users, posts);
```

### 2. Models

```rust
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Post {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: String,
    pub published: bool,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::posts)]
pub struct NewPost<'a> {
    pub user_id: i32,
    pub title: &'a str,
    pub body: &'a str,
}

#[derive(AsChangeset)]
#[diesel(table_name = crate::schema::posts)]
pub struct PostChangeset<'a> {
    pub title: Option<&'a str>,
    pub body: Option<&'a str>,
    pub published: Option<bool>,
}
```

### 3. CRUD Operations

Return `QueryResult<T>` and let the caller handle errors — no `expect` in library code.

```rust
use diesel::prelude::*;
use crate::schema::posts::dsl::*;

// CREATE — `returning` works on PostgreSQL, and on SQLite with the
// `returning_clauses_for_sqlite_3_35` feature. MySQL does NOT support
// RETURNING: use `.execute(conn)` there and re-select the row.
pub fn create_post(conn: &mut PgConnection, user_id: i32, title: &str, body: &str) -> QueryResult<Post> {
    let new_post = NewPost { user_id, title, body };
    diesel::insert_into(posts)
        .values(&new_post)
        .returning(Post::as_returning())
        .get_result(conn)
}

// READ (all)
pub fn get_all_posts(conn: &mut PgConnection) -> QueryResult<Vec<Post>> {
    posts.select(Post::as_select()).load(conn)
}

// READ (filtered)
pub fn get_published_posts(conn: &mut PgConnection) -> QueryResult<Vec<Post>> {
    posts
        .filter(published.eq(true))
        .limit(5)
        .select(Post::as_select())
        .load(conn)
}

// READ (single) — Ok(None) when not found
pub fn get_post(conn: &mut PgConnection, post_id: i32) -> QueryResult<Option<Post>> {
    posts
        .find(post_id)
        .select(Post::as_select())
        .first(conn)
        .optional()
}

// UPDATE
pub fn update_post(
    conn: &mut PgConnection,
    post_id: i32,
    changes: &PostChangeset<'_>,
) -> QueryResult<Post> {
    diesel::update(posts.find(post_id))
        .set(changes)
        .returning(Post::as_returning())
        .get_result(conn)
}

// DELETE — returns the number of rows deleted
pub fn delete_post(conn: &mut PgConnection, post_id: i32) -> QueryResult<usize> {
    diesel::delete(posts.find(post_id)).execute(conn)
}
```

### 4. Complex Queries

Self-contained examples against the §1 schema; note how `dsl::*` imports stay inside each function (see Best Practices).

```rust
use diesel::prelude::*;

// JOIN
fn titles_with_authors(conn: &mut PgConnection) -> QueryResult<Vec<(String, String)>> {
    use crate::schema::{posts, users};
    posts::table
        .inner_join(users::table)
        .select((posts::title, users::name))
        .load(conn)
}

// Filtering, ordering, pagination
fn recent_rust_posts(conn: &mut PgConnection) -> QueryResult<Vec<Post>> {
    use crate::schema::posts::dsl::*;
    posts
        .filter(published.eq(true))
        .filter(title.like("%rust%"))
        .order(id.desc())
        .limit(10)
        .offset(0)
        .select(Post::as_select())
        .load(conn)
}

// Aggregation
fn post_count(conn: &mut PgConnection) -> QueryResult<i64> {
    use crate::schema::posts::dsl::*;
    use diesel::dsl::count;
    posts.select(count(id)).first(conn)
}

// Subquery: posts whose author is named "alice"
fn alice_posts(conn: &mut PgConnection) -> QueryResult<Vec<Post>> {
    use crate::schema::{posts, users};
    let alice_ids = users::table
        .filter(users::name.eq("alice"))
        .select(users::id);
    posts::table
        .filter(posts::user_id.eq_any(alice_ids))
        .select(Post::as_select())
        .load(conn)
}
```

### 5. Raw SQL

```rust
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Integer;
use crate::schema::users;

#[derive(QueryableByName)]
#[diesel(table_name = users)]
struct UserRow {
    id: i32,
    name: String,
}

// `$1` is PostgreSQL bind syntax; MySQL/SQLite use `?`
fn find_user(conn: &mut PgConnection, user_id: i32) -> QueryResult<Vec<UserRow>> {
    sql_query("SELECT id, name FROM users WHERE id = $1")
        .bind::<Integer, _>(user_id)
        .load(conn)
}
```

### 6. Associations

```rust
use diesel::prelude::*;
use crate::schema::{posts, users};

// One-to-many
#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Queryable, Selectable, Identifiable, Associations)]
#[diesel(belongs_to(User))]
#[diesel(table_name = posts)]
pub struct PostSummary {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
}

fn posts_of_user(conn: &mut PgConnection, user_id: i32) -> QueryResult<Vec<PostSummary>> {
    let user = users::table.find(user_id).first::<User>(conn)?;
    PostSummary::belonging_to(&user)
        .select(PostSummary::as_select())
        .load(conn)
}
```

### 7. Transactions

```rust
use diesel::prelude::*;
use crate::schema::posts;

fn insert_and_publish(conn: &mut PgConnection, new_post: &NewPost<'_>) -> QueryResult<()> {
    conn.transaction(|conn| {
        diesel::insert_into(posts::table)
            .values(new_post)
            .execute(conn)?;

        diesel::update(posts::table)
            .filter(posts::id.eq(1))
            .set(posts::published.eq(true))
            .execute(conn)?;

        Ok(())
    })
}
```

### 8. Connection Pooling

```toml
[dependencies]
diesel = { version = "2.3", features = ["postgres", "r2d2"] }
```

```rust
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

type DbPool = Pool<ConnectionManager<PgConnection>>;

// Failing fast at startup is fine — the pool must exist before serving.
fn create_pool(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("failed to create database pool")
}
```

Using the pool from axum: **Diesel is synchronous** — never run queries directly in an async handler (that blocks the
tokio executor). Move the work off the runtime with `spawn_blocking` (see the rust-tokio skill), and map failures into
an error response instead of unwrapping:

```rust
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;

async fn get_posts(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<Post>>, (StatusCode, String)> {
    let all_posts = tokio::task::spawn_blocking(move || -> Result<Vec<Post>, String> {
        let mut conn = pool.get().map_err(|e| e.to_string())?;
        crate::schema::posts::table
            .select(Post::as_select())
            .load::<Post>(&mut conn)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))? // JoinError
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;            // query error

    Ok(Json(all_posts))
}
```

For fully-async database access, look at the `diesel-async` crate (`AsyncPgConnection`, with deadpool/bb8 pools) instead
of r2d2 + `spawn_blocking`.

### 9. Best Practices

1. **Use `get_result`/`get_results` with `.returning(...)`** where supported — PostgreSQL always, SQLite with the
   `returning_clauses_for_sqlite_3_35` feature; **MySQL has no RETURNING**
2. **Use `optional()`** for queries that might return nothing
3. **Use transactions** for multi-step operations
4. **Use connection pooling** (r2d2) in production
5. **Keep `dsl::*` imports inside functions** to avoid namespace pollution
6. **Use `check_for_backend`** for better compile-time error messages
7. **Use `AsChangeset`** for partial updates
8. **Use `belonging_to`** for associations
9. **Return `QueryResult`** from data-access helpers; leave error policy to the caller

## When to Use Diesel

- PostgreSQL/MySQL/SQLite databases
- Type-safe query building
- Complex joins and subqueries
- Applications needing compile-time query validation
