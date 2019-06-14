use super::*;

pub(crate) struct Service<R> {
    client: Client<R>,
    transports: Vec<Box<dyn Transport>>,
    processor: CommandProcessor,
}

impl<R: ReadAdapter> Service<R> {
    pub(crate) fn new(
        client: Client<R>,
        transports: Vec<Box<dyn Transport>>,
        processor: CommandProcessor,
    ) -> Self {
        Self {
            client,
            transports,
            processor,
        }
    }

    pub(crate) fn run(mut self) -> Result<(), Error> {
        while let Some(msg) = self.read_message() {
            log::trace!("got a privmsg");

            let user_id = match msg.user_id() {
                None => {
                    log::warn!("no user-id attached to that message");
                    continue;
                }
                Some(user_id) => user_id,
            };
            let (data, action) = if msg.message().starts_with('\x01') {
                (&msg.message()[8..msg.message().len() - 1], true)
            } else {
                (msg.message(), false)
            };

            if data.starts_with('!') {
                let mut s = data.splitn(2, ' ');
                if let (false, Some(cmd), Some(args)) = (action, s.next(), s.next()) {
                    self.handle_command(user_id, &msg.channel(), cmd, args)
                }
            }

            let data = data.to_string();
            self.dispatch(Self::new_local_msg(msg, data, action));
        }

        Ok(())
    }

    fn new_local_msg(msg: PrivMsg, data: String, is_action: bool) -> Message {
        let colors = ColorConfig::load();
        let name = msg.display_name().unwrap_or_else(|| msg.user()).to_string();

        let user_id = msg.user_id().expect("user-id");
        let timestamp = crate::make_timestamp().to_string();

        Message {
            version: Version::default(),
            userid: user_id.to_string(),
            color: msg.color().unwrap_or_default(),
            custom_color: colors.get(user_id).map(Into::into),
            badges: msg.badges(),
            emotes: msg.emotes(),
            tags: msg.tags().clone(),

            timestamp,
            name,
            data,
            is_action,
        }
    }

    fn dispatch(&mut self, msg: Message) {
        for transport in self.transports.iter_mut() {
            log::trace!("sending to a transport");

            if let Err(err) = transport.send(msg.clone()) {
                log::error!("cannot write to transport: {}", err);
            }
        }
    }

    fn read_message(&mut self) -> Option<PrivMsg> {
        log::trace!("waiting for a message");
        match self.client.read_message() {
            Ok(TwitchMsg::PrivMsg(msg)) => Some(msg),
            Err(err) => {
                log::error!("could not read message, quitting: {}", err);
                std::process::exit(1);
            }
            msg => {
                log::trace!("{:?}", msg);
                None
            }
        }
    }

    fn handle_command(&mut self, user_id: u64, channel: &str, cmd: &str, args: &str) {
        match self.processor.handle(user_id, cmd, args) {
            Response::Nothing | Response::Missing => {}
            Response::Message(resp) => {
                self.client
                    .writer()
                    .send(channel, &resp)
                    .expect("send to client");
            }
        };
    }
}
