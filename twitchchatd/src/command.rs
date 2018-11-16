#[derive(Debug, PartialEq, Clone)]
pub enum Command<'a> {
    Ping {
        data: &'a str,
    },
    Privmsg {
        target: &'a str,
        sender: &'a str,
        data: &'a str,
    },
    Unknown {
        cmd: &'a str,
        args: Vec<&'a str>,
        data: &'a str,
    },
}
