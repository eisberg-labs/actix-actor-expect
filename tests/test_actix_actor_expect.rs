use std::io::Error;

use actix::prelude::*;
use actix::Message;

use actix_actor_expect::ActorExpect;

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
        "Miss".to_string(),
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

#[actix::test]
async fn sends_no_message() {
    let actor_expect = ActorExpect::<TestActor, Error>::expect_send(
        TestActorCommand::Hello,
        "Message".to_string(),
        "Miss".to_string(),
    );

    assert_eq!(actor_expect.total_calls(), 0_usize);
}

// TODO: how to test it?
// use futures::FutureExt;
// #[actix::test]
// async fn placeholder_actor_doesnt_accept_incoming() {
//     let process = async {
//         let actor_expect = ActorExpect::<TestActor>::placeholder::<TestActorCommand>();
//         let addr = actor_expect.addr;
//         let req = addr.send(TestActorCommand::Echo("Message".to_string()));
//         req.await;
//         panic!("Should panic before!")
//     };
//
//     let result = process.catch_unwind().await;
//
//     assert!(result.is_err());
//     assert_eq!(
//         &format!("{:?}", result.err().unwrap().downcast_ref::<&str>()),
//         ""
//     )
// }
