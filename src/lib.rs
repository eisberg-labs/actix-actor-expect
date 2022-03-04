use std::any::Any;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use actix::actors::mocker::Mocker;
use actix::{Actor, Addr, Context, Message};

type ReceivedCallsLog = Arc<Mutex<Vec<Box<dyn Any>>>>;

/// Utility for unit testing actix actors.
/// Helper for reducing the boilerplate when unit testing actix actors.
/// Configures a  mocker actor to expect a particular incoming command `I` and to respond with provided outgoing response `O`.
pub struct ActorExpect<T: Sized + Unpin + 'static, Error: 'static> {
    pub addr: Addr<Mocker<T>>,
    received_calls: ReceivedCallsLog,
    phantom_error_data: PhantomData<Error>,
}

impl<T: Sized + Unpin + 'static, Error: 'static> ActorExpect<T, Error> {
    /// Creates a mocker that accepts incoming and returns outgoing message.
    /// If other messages are received, default_outgoing message is returned.
    pub fn expect_send<I, O>(incoming: I, outgoing: O, default_outgoing: O) -> Self
    where
        I: 'static + Clone + PartialEq + Message + Send + Debug,
        I::Result: Send,
        O: 'static + Clone + PartialEq + Debug,
    {
        let log: ReceivedCallsLog = Arc::new(Mutex::new(vec![]));
        let cloned_log = log.clone(); // cloned right away to avoid error borrow of moved value
        let mocker = Mocker::<T>::mock(Box::new(move |msg, ctx| {
            let result: O = ActorExpect::<T, Error>::process_messaging(
                &cloned_log,
                msg,
                incoming.clone(),
                outgoing.clone(),
                default_outgoing.clone(),
                ctx,
            );

            let boxed_result: Box<Option<Result<O, Error>>> = Box::new(Some(Ok(result)));
            boxed_result
        }));

        let addr = mocker.start();

        Self {
            addr,
            received_calls: log.clone(),
            phantom_error_data: PhantomData,
        }
    }

    /// Creates an actor that is a placeholder:
    /// - it doesn't accept sent messages.
    /// - if message is received, inbox closes right away.
    pub fn placeholder<O: 'static + Clone + PartialEq>() -> Self {
        let mocker = Mocker::<T>::mock(Box::new(move |_msg, _ctx| {
            let result: Option<Result<O, Error>> = None;
            Box::new(result)
        }));
        let addr = mocker.start();
        Self {
            addr,
            received_calls: Arc::new(Mutex::new(vec![])),
            phantom_error_data: PhantomData,
        }
    }

    /// Returns a total number of calls that the mocker received.
    pub fn total_calls(&self) -> usize {
        let received_calls = self
            .received_calls
            .lock()
            .expect("Received calls log error!");
        received_calls.len()
    }

    /// Returns a total number of calls that the mocker received for msg type or variant.
    ///
    /// # Arguments
    /// * `msg` - message for actor
    pub fn calls_of_variant<MSG: Any + 'static + PartialEq>(&self, msg: MSG) -> usize {
        let mut count = 0;
        for item in self
            .received_calls
            .lock()
            .unwrap_or_else(|_| panic!("Received calls log error!"))
            .iter()
        {
            let it = item
                .as_ref()
                .downcast_ref::<MSG>()
                .unwrap_or_else(|| panic!("Invalid item {:?}!", item));
            if msg == *it {
                count += 1
            }
        }
        count
    }

    fn process_messaging<
        I: 'static + Clone + PartialEq + Debug,
        O: 'static + Clone + PartialEq + Debug,
    >(
        log: &ReceivedCallsLog,
        msg: Box<dyn Any>,
        incoming: I,
        outgoing: O,
        default_outgoing: O,
        _ctx: &mut Context<Mocker<T>>,
    ) -> O {
        let command: &I = msg
            .downcast_ref::<I>()
            .unwrap_or_else(|| panic!("Cannot downcast command {:?}!", msg));
        let _ = log
            .lock()
            .unwrap_or_else(|_| panic!("Received calls log error!"))
            .push(Box::new(command.clone()));
        if command.clone() == incoming {
            outgoing
        } else {
            default_outgoing
        }
    }
}
