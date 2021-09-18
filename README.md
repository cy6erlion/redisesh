``` text
                   .___.__                     .__
_______   ____   __| _/|__| ______ ____   _____|  |__
\_  __ \_/ __ \ / __ | |  |/  ___// __ \ /  ___/  |  \ 
 |  | \/\  ___// /_/ | |  |\___ \\  ___/ \___ \|   Y  \
 |__|    \___  >____ | |__/____  >\___  >____  >___|  /
             \/     \/         \/     \/     \/     \/
                Redis based session manager
```

Sessions are stored with redis hashes. The session key is 128 random
bits (16-bytes) which is encoded with base64. Sessions can be 
configured to expire.

``` rust
let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
let session_data = Some(String::from("{username: John}"));
let base64_token = redisesh.insert(session_data).unwrap();
```

⧉
