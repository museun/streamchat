## 'api' response json
```json
{
    "userid": "23196011",
    "timestamp": 1542340383311,
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

---

### twitchchatd
`TWITCH_CHAT_OAUTH_TOKEN` must be set to your oauth:token from twitch

```
usage: twitchchatd
  -n int            number of messages to buffer
  -m fd             mock stream for testing clients (file.txt | stdin | -)
  -a addr:port      address to host on
  -c string         channel to join
  -n string         nick to use (required)
```

flag | description
--- | ---
-n | this controls how many messages are stored in the backlog<br>defaults to 16
-m | this allows you to provide chat replay from a text file, or stdin
-a | tcp address to host the server on.<br>defaults to _localhost:51002_
-c | which channel to join, note # shouldn't be preprended.
-n | which nick to use, it should match the name of the oauth token<br>**and is required**

---

### twitchchatc
```
usage: twitchchatc
  -l char           left fringe character. defaults to ⤷
  -r char           right fringe character. defaults to ⤶
  -a addr:port      which address to connect to
  -m int            max width of lines
  -n int            max width of names
```

flag | description
--- | ---
-l | this **UTF-8** character appears in the _left-most_ column when wrapping.<br>setting it to " " will remove it
-r | this **UTF-8** character appears in the _right-most_ column when wrapping.<br>setting it to " " will remove it
-a | tcp address to connect to the daemon<br>defaults to _localhost:51002_
-m | this will hardwrap lines at _int_ - 2 (the fringes).<br>without this specified, it'll try to pick the widest width of your terminal upon start<br>if it cannot find the terminal size, it'll default to 60 (minus 2)
-n | this will truncate (and append …) names over _int_ length<br>defaults to 10
