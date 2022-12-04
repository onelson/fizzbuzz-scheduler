# Tasks Scheduler

## Overview

This project is split into 3 areas:

- a `server` crate for the HTTP service used to manage tasks.
- a `worker` crate for the worker process which consumes queued tasks.
- a `common` crate which makes an attempt to isolate the database access and hold types needed by both processes.

All three of these crates are members of a cargo workspace defined in the repo root.

## Running and Configuration

This project requires `rust` and a `postgres` database.

Both rust processes rely on a `DATABASE_URL` environment variable which should
contain the database connection string.

### HowTo


#### Database setup

To run a postgres database using docker:

```
$ docker run --rm -e POSTGRES_PASSWORD=1234 -p 5432:5432 postgres:15
```

The above command will block your shell. Shut down the database with `CTRL+C`.

#### Rust Processes

Launch the HTTP service before the worker, configuring the `DATABASE_URL` variable:
```
$ DATABASE_URL='postgresql://postgres:1234@localhost/postgres' cargo run -p server
```

N.b. the HTTP service will attempt to **bootstrap the database schema on startup**.
As such, the database needs to be ready to accept connections before you launch the server.

To run the worker:
```
$ DATABASE_URL='postgresql://postgres:1234@localhost/postgres' cargo run -p worker
```

You can adjust logging for both processes using the `RUST_LOG` environment variable.

Both processes can be launched as many times concurrently as desired.


## HTTP API

| Method | Path            | Description          |
|--------|-----------------|----------------------|
| GET    | /tasks          | List tasks           |
| POST   | /tasks          | Create a new task    |
| GET    | /tasks/:task_id | View a single task   |
| DELETE | /tasks/:task_id | Remove a single task |

### Listing

The task list can be filtered by using query params:
- `type`, accepted values are `Fizz`, `Buzz` and `FizzBuzz`
- `state`, accepted values are `Pending` and `Completed`

Example: 
```
$ curl -sX GET http://localhost:3000/tasks?type=Buzz | jq                                                                                                                                                                                 
{                                                 
  "results": [
    {
      "created_at": "2022-12-04T21:02:55.357504Z",
      "execution_time": "2022-01-01T01:00:03Z",
      "id": 38,
      "type": "Buzz",
      "state": "Completed",
      "updated_at": "2022-12-04T21:03:57.946094Z"
    },
    {
      "created_at": "2022-12-04T21:03:03.477633Z",
      "execution_time": "2022-01-01T01:00:03Z",
      "id": 45,
      "type": "Buzz",
      "state": "Completed",
      "updated_at": "2022-12-04T21:04:46.722710Z"
    }
  ]
}
```

### Creating

This endpoint expects to receive a json payload with an object containing two keys:

- `type`, accepted values are`Fizz`, `Buzz`, or `FizzBuzz`
- `execution_time`, accepted values are **ISO 8601** timestamp strings.

A successful response will contain a json object with an `id` key (the newly created task's id).

> N.b. the `content-type` header should be set to `application/json`.

Example:

```
$ curl -sX POST \
  -H 'content-type: application/json' \
  -d '{"type": "Fizz", "execution_time": "2022-12-08T00:00:00Z"}' \
  http://localhost:3000/tasks | jq
{
  "id": 57
}
```


### Reading

An individual task can be viewed by its id.
The body will be a json object containing all the task's details.

```
$ curl -sX GET http://localhost:3000/tasks/34 | jq
{
  "id": 34,
  "type": "Buzz",
  "execution_time": "2022-01-01T01:00:03Z",
  "state": "Completed",
  "created_at": "2022-12-04T20:59:27.228606Z",
  "updated_at": "2022-12-04T21:01:34.388050Z"
}
```

### Deleting

Making a `DELETE` request to a task's URI will permanently destroy the record of it.

A successful response will have a `204` "No Content" status.

```
$ curl -vsX DELETE http://localhost:3000/tasks/34
*   Trying 127.0.0.1:3000...
* TCP_NODELAY set
* Connected to localhost (127.0.0.1) port 3000 (#0)
> DELETE /tasks/34 HTTP/1.1
> Host: localhost:3000
> User-Agent: curl/7.68.0
> Accept: */*
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 204 No Content
< date: Sun, 04 Dec 2022 22:24:16 GMT
<
* Connection #0 to host localhost left intact
```
