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

__basic__

``` rust
// Establish redis connection
let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();

// Data to be store in session hash map value
let session_data = Some(String::from("{username: John}"));

// Create a new session by inserting the session data in redis.
// Also generates and returns a random Token which is the key of the
// session in the redis hashmap.
let base64_token = redisesh.insert(session_data).unwrap();
```

__expiration__

``` rust
let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
let session_data = Some(String::from("{username: John}"));

// Session Expiration date (seconds)
let expiration = std::time::Duration::from_secs(10);

// Set session expiration
redisesh.configure(Config {
  expiration: Some(expiration),
});

// This session expires after 10 seconds
let base64_token = redisesh.insert(session_data).unwrap();

// Sleep for 2 seconds to make sure the session has expired
std::thread::sleep(std::time::Duration::from_secs(2));

// Check if session exists
let exists = redisesh.is_active(&base64_token).unwrap();

// Session does not exist
assert_eq!(exists, false);
```

⧉
