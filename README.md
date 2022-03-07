# Actix Actor Expect [![Continuous Integration](https://github.com/eisberg-labs/actix-actor-expect/actions/workflows/ci.yml/badge.svg)](https://github.com/eisberg-labs/actix-actor-expect/actions/workflows/ci.yml) [![cargo-badge][]][cargo] [![license-badge][]][license]

Utility for unit testing actix actors, extension for `Mocker`. I wrote a blog post [Mocking Actix Actor without getting a gray hair](https://amarjanica.com/mocking-actix-actor-without-getting-a-gray-hair/) a
while ago, you might find it useful.

# Usage
Dependencies:

```toml
[dev-dependencies]
actix-actor-expect = "0.1.0"
```

Code:
```rust

#[derive(Clone, Debug, PartialEq, Message)]
#[rtype(result = "Result<String, Error>")]
pub enum TestActorCommand {
    Hello,
    Dunno,
    Echo(String),
}

#[derive(Debug, Default)]
pub struct TestActor {}

impl Actor for TestActor {
    type Context = Context<Self>;
}

impl Handler<TestActorCommand> for TestActor {
    type Result = Result<String, Error>;

    fn handle(&mut self, msg: TestActorCommand, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TestActorCommand::Echo(message) => Ok(message),
            rest => Ok(format!("{:?}", rest)),
        }
    }
}

#[actix::test]
async fn sends_hello_back() {
    let actor_expect = ActorExpect::<TestActor, Error>::expect_send(
        TestActorCommand::Echo("Message".to_string()),
        "Message".to_string(),
        None,
    );
    let actor = &actor_expect.addr;

    let _ = actor
        .send(TestActorCommand::Echo("Message".to_string()))
        .await
        .expect("Not able to process Echo");
    let _ = actor
        .send(TestActorCommand::Hello)
        .await
        .expect("Not able to process Hello");
    let _ = actor
        .send(TestActorCommand::Dunno)
        .await
        .expect("Not able to process Dunno");

    assert_eq!(actor_expect.total_calls(), 3_usize);
    assert_eq!(
        actor_expect.calls_of_variant(TestActorCommand::Echo("Message".to_string())),
        1_usize
    );
}
```

Also take a look at [tests](./tests).

# License

Distributed under the terms of [MIT license](./LICENSE-MIT) and [Apache license](./LICENSE-APACHE).


[cargo-badge]: https://img.shields.io/crates/v/actix-actor-expect.svg?style=flat-square
[cargo]: https://crates.io/crates/actix-actor-expect
[license-badge]: https://img.shields.io/badge/license-MIT/Apache--2.0-lightgray.svg?style=flat-square
[license]: #license
