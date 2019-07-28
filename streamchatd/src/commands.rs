use std::collections::HashMap;

pub(crate) enum Response {
    Message(String),
    Nothing,
    Missing,
}

type Func = Box<Fn(u64, &str) -> Option<String>>;

#[derive(Default)]
pub(crate) struct CommandProcessor(HashMap<String, Func>);

impl CommandProcessor {
    pub(crate) fn add<S, F>(&mut self, command: S, func: F)
    where
        S: ToString,
        F: Fn(u64, &str) -> Option<String> + 'static,
    {
        self.0
            .insert(format!("!{}", command.to_string()), Box::new(func));
    }

    pub(crate) fn handle(&self, user: u64, command: &str, rest: &str) -> Response {
        let func = match self.0.get(command) {
            Some(func) => func,
            None => return Response::Missing,
        };

        match (func)(user, rest) {
            Some(msg) => Response::Message(msg),
            None => Response::Nothing,
        }
    }
}
