

# Embedded Rust for Real-Time Systems
By Albert Werner Laursen

---

## Agenda
1. State of RTOS in Rust
2. Embassy vs. RTIC
3. Async/Await
4. Examples

---

## What is an RTOS?

- What is an Operating System?
+ General Purpose OS vs. RTOS
+ Traditional RTOS
+ How does Rust fit in?

---

## State of RTOS in Rust

- No real RTOS in Rust
+ [arewertosyet.com](https://arewertosyet.com)
  - No...
+ So what do we do?
  - Asynchronous Execution
  - "pseudo" threading

---

## Embassy

- [embassy.dev](https://embassy.dev)
- Full framework for async programming
  - Batteries included
  - HAL, drivers, timing, etc.

---

## RTIC

- [rtic.rs](https://rtic.rs)
- Real-Time Interrupt-driven Concurrency
  - "Bring your own HAL"
  - [Stack Resource Policy (SRP)](https://link.springer.com/article/10.1007/BF00365393)
  - preemptive deadlock and race-free scheduling

---

## Async / Await

- Read the [Async Book](https://rust-lang.github.io/async-book/)
- Rust core feature
- Asynchronous interface
  - `async fn` and `await`
  - compatible with any executor

---

`async fn` is syntactic sugar
    
```rust []
async fn foo() -> u32 {
    42
}

fn foo() -> impl Future<Output = u32> {
    async { 42 }
}
```

---

So what is a Future?

```rust []
trait Future {
    type Output;
    fn poll(
      self: Pin<&mut Self>, cx: &mut Context<'_>
    ) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

---

A Future is a state machine

```mermaid
graph LR
    A[Start] --> B{Poll}
    B -->|Ready| C[Output]
    B -->|Pending| D[Wait]
    D -->|Wake| B
``` 

---

`async`/`await` is an ergonomic way to work with Futures

```rust []
async fn get_data() -> Result<Data, Error> {
    let response = fetch("https://data.com").await?;
    let data = parse(response).await?;
    Ok(data)
}
```

---

Nested Futures are tree like state machines

```mermaid
graph TD
    A[Get Data] -->|poll| B{Fetch}
    A -->|poll| C{Parse}

    B -->|poll| D[send]
    B -->|poll| E[recieve]
    C -->|poll| F[logic]
```

---

An executor is responsible for polling the Future

```rust []
fn unary_executor() -> ! {
    let root_future = get_data();
    loop {
        match my_future.poll() {
            Poll::Ready(output) => do_something(output);
            Poll::Pending => {}
        }
    }
}
```

---

Concurrency is achieved by running multiple Futures

```rust
fn executor() -> ! {
    let futures: Vec<Future> = get_futures();
    loop {
        for future in futures.iter_mut() {
            match future.poll() {
                Poll::Ready(output) => do_something(output);
                Poll::Pending => {}
            }
        }
    }
}
```

---

- Executor logic is what differentiates async frameworks
  - `tokio`, `async-std`, etc.
  - `embassy`, `rtic`, etc.
- This is cooperative multitasking
  - No real preemption
  - shorter futures -> real concurrency
  - longer futures -> blocking

---

## Examples

- We will focus on the `rtic` framework
  - `embassy` is very similar
  - `rtic` is race-free
  
```bash
git clone https://github.com/Awlaursen/rtic-examples.git
```