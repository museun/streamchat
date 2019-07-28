this consists of two components.

* a daemon, _streamchatd_
* a rust client, _streamchatc_ that connects via **tcp**, and prints to its **tty**

the daemon, `streamchatd` connects to twitch and buffers messages. once a client connects, it sends these buffered messages to the client. this allows multiple clients to connect and get a *broadcast* style output, and allows clients to reconnect and *resume* with a back log.

the client, `streamchatc` is a rust client that connects to `streamchatd` over a tcp socket. its meant to be used in a terminal that supports **ANSI** colors. it formats each message into a 4 column table

```
|fringe| |nick| |message| |fringe|
```

on a single line:
```
         |nick| |message|        
```

when wrapping, fringes are applied to the start/end of the leading/trailing lines
```
         |nick| |message a| |fringe|
|fringe| |nick| |message b| |fringe|
|fringe| |nick| |message c| |fringe|
|fringe| |nick| |message d|         `
```

the rust library allow one to write their own client, it provides the types used by the daemon, and some other utilities.

## streamchatd
```
usage: streamchatd
    -l <int>
    -c <string>
    -n <string>
```
| flag | description |
--- | ---
-l | backlog limit to store, number of messages to keep.
-c | channel to join
-n | nickname to use
---
the configuration file is `streamchatd.toml`
os | location
--- | ---
linux-ish | `$XDG_CONFIG_HOME/museun/streamchat`
windows | `%APPDATA%/museun/config/streamchat` 
*example:*
```
address = 'localhost:51002'
oauth_token = 'oauth:some_long_token'
limit = 32
channel = 'museun'
nick = 'museun'
```
key | value
--- | ---
address |  the address that to listen on (tcp socket)
oauth_token | twitch oauth token. **be sure** to include the preceeding `oauth:`
limit  | how many messages to store, overridden by the `-l` flag
channel | the twitch channel to join. overridden by the `-c` flag. **note** its `museun` (twitch naming) not `#museun` (irc naming)
nick | the nick to authenticate with. overridden by the `n` flag
---
## streamchatc
```
Optional arguments:
  -h, --help                show this help message
  -l, --left STRING         left fringe to use
  --left-color #RRGGBB      left fringe color
  -r, --right STRING        right fringe to use
  --right-color #RRGGBB     right fringe color
  -a, --address ADDR        address of the streamchatd instance
  -n, --buffer-max NUMBER   maximum number of messages to buffer
  -m, --nick-max NUMBER     maximum width of nicknames
  --print-config            print the configuration path
  --config BOOL             use the config file (default: true)
  --standalone              run the client without the server
  --nick TWITCH_NAME        your twitch name
  --channel TWITCH_CHANNEL  the channel to join
```

### standalone mode
* if you use `--standalone` you don't need a `streamchatd` instance running, but a backlog won't be preserved.
* if you use `--config false` then it'll require you to have `--nick`, `--channel` and `--address` 
* if you use `--standalone` and `--config false` then you'll be required to have `--nick`, `--channel` and an ENV variable of `STREAMCHAT_TWITCH_OAUTH_TOKEN` set to your Twitch OAUTH token


the configuration file is `streamchatc.toml`
os | location
--- | ---
linux-ish | `$XDG_CONFIG_HOME/museun/streamchat`
windows | `%APPDATA%/museun/config/streamchat` 
*example:*
```
address = 'localhost:51002'
default_line_max = 60
nick_max = 10

[left_fringe]
fringe = '⤷'
color = '#0000FF'

[right_fringe]
fringe = '⤶'
color = '#FF0000'
```
key | value
--- | ---
address |  the address that `streamchatd` is listening on (tcp socket)
default_line_max |  how wide the lines will be before wrapping, if it can't be determined automatically
nick_max | how long a nick can be before truncation
left_fringe.fringe | the fringe string, which can be override by the `-l` flag
left_fringe.color | `#RRGGBB` color string of the fringe
right_fringe.fringe | the fringe string, which can be override by the `-r` flag
right_fringe.color | `#RRGGBB` color string of the fringe
---
## color config
* custom user colors can be done via twitch chat. using `!color #RRGGBB | RRGGBB`.
* users can reset their colors simply by doing `!color`
* the color format for this command is `#RRGGBB` or `RRGGBB` or one of Twitch's named colors. See this enum [twitchchat](https://github.com/museun/twitchchat/blob/9cda6169f3460714ec97db250b9e10124d455e07/src/twitch/color.rs#L89).

its stored in `color_config.json`
os | location
--- | ---
linux-ish | `$XDG_DATA_HOME/museun/streamchat`
windows | `%APPDATA%/museun/data/streamchat` 

it looks like this:
```json
{  
    "23196011": [
        0,
        255,
        0
    ] 
}
```
where each object is `userid` : `[R, G, B]`

the **userid** is the twitch user id, and the array is an array of **u8s in base 10**
## response json
```json
{
  "version": 1,
  "userid": "23196011",
  "timestamp": "1552369599356",
  "name": "museun",
  "data": "need a test example Kappa",
  "color": "OrangeRed",
  "custom_color": {
    "Turbo": [
      221,
      160,
      221
    ]
  },
  "is_action": false,
  "badges": [
    {
      "kind": "Broadcaster",
      "data": "1"
    }
  ],
  "emotes": [
    {
      "id": 25,
      "ranges": [
        {
          "start": 20,
          "end": 24
        }
      ]
    }
  ],
  "tags": {
    "user-id": "23196011",
    "turbo": "0",
    "flags": "",
    "emotes": "25:20-24",
    "mod": "0",
    "room-id": "23196011",
    "tmi-sent-ts": "1552369599175",
    "id": "74f2fde6-6ab7-40fb-b7fa-3f2cb44577a8",
    "badges": "broadcaster/1",
    "color": "#FF4500",
    "display-name": "museun",
    "user-type": "",
    "subscriber": "0"
  }
}
```
refer to [Message](src/message.rs) for the struct definition, it uses some types from [twitchchat](https://docs.rs/twitchchat/0.1.0/twitchchat/twitch/index.html)

to write your own clients, just open a tcp connection to `$addr:port` and read newline (**\n**) separated json (listed above) until end of stream, or you're done.

when you connect, you may get up to `$backlog` of messages, so reconnecting can be considered cheap -- you'll always receive the backlog you've not seen before.

to write a different transport, look at [Socket](src/transports/socket.rs). they can be added into the daemon by adding their trait object into the vec on creation.
