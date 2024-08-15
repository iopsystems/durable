# Durable

Durable is durable execution engine. You write a workflow and durable takes care
of running it to completion, regardless of whether your application is restarted,
updated, or otherwise becomes unavailable while it is running.

## Features
- Reliably runs workflows to completion, even if the host running the workflow
  is interrupted while it is running.
- Supports running the worker embedded within an existing application.
- Provides at-least-once semantics for external effects by workflows and
  exactly-once semantics for changes made to the database that is being used
  by the worker.

## Getting started
To try it out locally, run

```bash
# First, start up a postgresql database for durable to use.
# Note that the setup provided here is insecure.
docker run -p 5432:5432 -e POSTGRES_HOST_AUTH_METHOD=trust -d --rm postgres:15

cargo run --bin durable-worker --features cli -- \
    --database-url postgres://postgres@localhost:5432/postgres \
    --migrate
```

Now, in another terminal, build the example workflows. 

> To do this you will need to have installed `cargo-component`.
>
> ```bash
> cargo install --locked cargo-component
> ```

```bash
cargo component build --examples --features sqlx,http --profile wasm
```

Now, you can run it on the worker we created above.
```bash
cargo run --bin durable -- launch \
    --database-url postgres://postgres@localhost:5432/postgres \
    --tail \
    'my first task' target/wasm32-wasip1/wasm/examples/hello-world.wasm
```

This should then output
```
Hello, my first task!
```

You can see more of what can be done with durable by looking at the examples in
the [`examples`](examples) crate.

## Writing a Workflow
Durable workflows are wasm components that use WASI to interact with the durable
runtime. In practice, this means that you write a small rust program that uses
the [`durable`] crate and then compile that to a wasm component by using
`cargo-component`.

Let's do just that!

Here is our workflow:
```rust
use std::net::IpAddr;
use serde::Deserialize;

#[derive(Deserialize)]
struct Ip {
    origin: IpAddr
}

fn main() {
    let response = durable::http::get("https://httpbin.org/ip")
        .send()
        .expect("failed to send an HTTP request");
    let response: Ip = response
        .json()
        .expect("response contained unexpected JSON");

    println!("This workflow was run at {}", response.origin);
}
```

All this workflow does is make a request to <https://httpbin.org/ip>, which
returns the IP address of the host that made the request.

Here's the `Cargo.toml` we'll need. Note that we need the `http` feature of
durable.

```toml
[package]
name = "example-http"
version = "0.0.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }

[dependencies.durable]
git = "https://github.com/iopsystems/durable"
features = ["http"]
```

Now we can build our crate by running `cargo-component`:
```bash
cargo component --bin example-http
```

This will give us a wasm binary that we can then run on a durable worker using
the `durable` command:
```bash
durable launch --tail 'http example' target/wasm32-wasip1/example-http.wasm \
    --database-url postgres://your/database
```

Note that you will need to have a worker running on the same postgres database
in order for the task to run.

## How it works
At its basic level, durable works by recording the outcomes of any external
effects performed by a workflow. If you make an HTTP request, or a database
transaction, then the results of that operation gets saved as an event in the
database. If the workflow ever gets interrupted it will be restarted on a
different worker. However, this time, instead of redoing the external effects
that have already been done it will return the saved results from the first
execution within the event log.

To illustrate this, lets consider a workflow that looks like this:
```rust
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let response = durable::http::get("https://example.com").send()?;
    
    std::thread::sleep(Duration::from_secs(10));

    println!("Got response:\n{}", response.text());

    Ok(())
}
```

If this runs normally, without being interrupted, then the event log at the
end might look something like this:
```text
idx | event         | data
0   | http/send     | HTTP/1.1 200 OK ...
1   | now           | 10293101412
2   | sleep_until   |
3   | write         | 2031
```

We can see the task making the HTTP request, getting the current time, sleeping,
and then writing to the log. At each point, it records what the result of that
operation to the event log.

Suppose, now, that instead of running until the end, the worker running the
workflow dies during the call to `sleep_until`. When this happens, the event
log looks like this:
```text
idx | event         | data
0   | http/send     | HTTP/1.1 200 OK ...
1   | now           | 10293101412
```

Some other worker picks up the workflow and starts running it from the top.
However, something is different this time around, when the workflow makes the
HTTP request it looks at the event log and sees the previous entry recorded
within. Instead of doing the HTTP request again, it just reads the response
from last time and returns that. The same happens with the call to `now`.

Now that the workflow has reached where it was interrupted before we can
continue where we left off before. Since the workflow is sandboxed, and
external function calls are recorded, workflow execution is
deterministic[^1]. This means that this new execution of the workflow should
be in the same state as the previous one was killed within.

[^1]: By using interior mutability and some more advanced features of the
      durable crate it is possible to break this, which would result in
      executions that differ from each other. This is not easy to do
      accidentally, though, so is usually not an issue.
