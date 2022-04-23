# Get Started

Learn Rust by doing.

- to do application
- TDD
- fullstack (all in rust)

# Stack

web

- actix-web

web-assembly

- yew

deployment

- postgres
- supabase
- heroku

# Design

## Entity

```postgresql
CREATE TABLE todos
(
    namespace  VARCHAR(36) NOT NULL,
    id         SERIAL      NOT NULL,
    content    TEXT        NOT NULL,
    status     VARCHAR(32) NOT NULl,
    created_at TIMESTAMP   NOT NULL,
    updated_at TIMESTAMP   NOT NULL,
    PRIMARY KEY (namespace, id)
);
```

## API Endpoints

### Common Header

short for `todo namespace`
default to: default

```text
t-NS: {NS}
```

### GET /todos

get a list of todos

query

```
?status=todo
```

body

```json
[
  {
    "id": 1,
    "namespace": "default",
    "content": "first thing",
    "status": "todo",
    "create_at": 1647151812778
  }
]
```

### POST /todos

create a todo

body

```json
{
  "content": "second thing"
}
```

### PATCH /todos/{id}

update content of a todo

```json
{
  "content": "updated thing"
}
```

### PATCH /todos/{id}/{status}

available status:

- todo
- done
- archive

update status of a todo

[no body]

### DELETE /todos/{id}

delete a **archive** todo

## Status Transform

![img.png](doc/status_transform.png)

# Backend

## Mods

```text
src/
- main.rs
- lib.rs
- handlers/
    - todo_handler.rs
- domains/
    - todo_domain.rs
- infra/
    - utils.rs
tests/
```

unit test e2e test integration test

# Ideas

* CRUD derive
* REST Repository

# Road map

* [x] business error design
    * [x] repo error
    * [] service error
    * [] api error
* [] domain service mock repo test
* [] domain service status machine
* [] application service mock domain test
* [] handler hooks into application
* [] handler mock application test
* [] api error design and handling
* [] e2e test
* [] repo `check` like test (data driven test)
* [] more docs
* [] more tests
* [] abstraction Repository
* [] abstraction CRUD

* [x] design
* [x] ui + tailwindCSS

* [x] backend: add field to identify each client
* [] frontend: split components to files
* [] frontend: integration with backend
* [] deploy: backend to heroku
    * [] backend: add api for heath check
    * [] frontend: add switch to backend
* [] deploy: frontend

* [] blog: write tutorials
