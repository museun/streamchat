this consists of three components.
a daemon, a rust client and a rust library.

the daemon, `twitchchatd` connects to twitch and buffers messages. once a client connects, it sends these buffered messages to the client. this allows multiple clients to connect and get a *broadcast* style output, and allows clients to reconnect and *resume* with a back log.

the client, `twitchchatc` is a rust client that connects to `twitchchatd` over a tcp socket. its meant to be used in a terminal that supports **ANSI** colors. it formats each message into a 4 column table

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

## twitchchatd
```
usage: twitchchatd
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
the configuration file is `twitchchatd.toml`
os | location
--- | ---
linux-ish | `$XDG_CONFIG_HOME/museun/twitchchat`
windows | `%APPDATA%/museun/config/twitchchat` 
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
## twitchchatc
```
usage: twitchchatc
    -l <string>
    -r <string>
```
| flag | description |
--- | ---
-l | string that appears in the left most column
-r | string that appears in the right most column
---
the configuration file is `twitchchatc.toml`
os | location
--- | ---
linux-ish | `$XDG_CONFIG_HOME/museun/twitchchat`
windows | `%APPDATA%/museun/config/twitchchat` 
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
address |  the address that `twitchchatd` is listening on (tcp socket)
default_line_max |  how wide the lines will be before wrapping, if it can't be determined automatically
nick_max | how long a nick can be before truncation
left_fringe.fringe | the fringe string, which can be override by the `-l` flag
left_fringe.color | `#RRGGBB` color string of the fringe
right_fringe.fringe | the fringe string, which can be override by the `-r` flag
right_fringe.color | `#RRGGBB` color string of the fringe
---
## color config
custom user colors can be done via twitch chat. using `!color #RRGGBB | RRGGBB`.

its stored in `color_config.json`
os | location
--- | ---
linux-ish | `$XDG_DATA_HOME/museun/twitchchat`
windows | `%APPDATA%/museun/data/twitchchat` 

it looks like this:
```json
{
  "map": {
    "23196011": [
      0,
      255,
      0
    ]
  }
}
```
where the `map` contains `userid` : `[R, G, B]`

the **userid** is the twitch user id, and the array is an array of **u8s in base 10**
## response json
```json
{
    "version": 1,
    "userid": "23196011",
    "timestamp": "1542340383311",
    "name": "museun",
    "data": "Kappa Kappa VoHiYo",
    "color": {
        "r": 255,
        "g": 69,
        "b": 0
    },
    "is_action": false,
    "badges": [
        "Broadcaster"
    ],
    "emotes": [
        {
            "ranges": [
                {
                    "start": 0,
                    "end": 4
                },
                {
                    "start": 6,
                    "end": 10
                }
            ],
            "id": 25
        },
        {
            "ranges": [
                {
                    "start": 12,
                    "end": 17
                }
            ],
            "id": 81274
        }
    ],
    "tags": {
        "room-id": "23196011",
        "flags": "",
        "emotes": "25:0-4,6-10/81274:12-17",
        "mod": "0",
        "badges": "broadcaster/1",
        "subscriber": "0",
        "emote-only": "1",
        "display-name": "museun",
        "user-type": "",
        "id": "1b446a16-696d-4603-9f55-f67acbf3021e",
        "user-id": "23196011",
        "turbo": "0",
        "color": "#FF4500",
        "tmi-sent-ts": "1542340383153"
    }
}
```
refer to [Message](twitchchat/src/message.rs) for the struct definition

to write your own clients, just open a tcp connection to `$addr:port` and read newline (**\n**) separated json (listed above) until end of stream, or you're done.<br>
when you connect, you may get up to `$backlog` of messages, so reconnecting can be considered cheap -- you'll always receive the backlog you've not seen before.

to write a different transport, look at [Socket](twitchchat/src/transports/socket.rs). they can be added into the daemon by adding their trait object into the vec on creation.
