//! # Redisesh
//! Redis based session management
mod error;
use error::Error;
use sodiumoxide::{base64, randombytes::randombytes};

/// The session token is represented as Base64 encoded string
pub type SessionToken = String;

/// Session Configuration
pub struct Config {
    /// Expiration for all sessions that will be stored
    expiration: Option<std::time::Duration>,
}
/// Session manager
pub struct Redisesh {
    /// Connection to redis server
    conn: redis::Connection,
    config: Option<Config>,
}
impl Redisesh {
    /// Connect to redis
    pub fn new(redis_url: &str) -> Result<Self, Error> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_connection()?;
        Ok(Redisesh { conn, config: None })
    }
    /// Configure sessions
    pub fn configure(&mut self, configuration: Config) {
        self.config = Some(configuration);
    }
    /// Insert session into redis store
    pub fn insert(&mut self, session_data: Option<String>) -> Result<SessionToken, Error> {
        let token = Redisesh::generate_token();
        let base64_token = base64::encode(token, base64::Variant::Original);
        let active = self.is_active(&base64_token)?;

        // XXX: This check is propably not required, because of the
        //      cryptographically random generated token
        if !active {
            // Store token with redis
            let _: () = redis::cmd("HSETNX")
                .arg(&base64_token)
                .arg("session_data")
                .arg(session_data)
                .query(&mut self.conn)?;

            // Check configuration
            match &self.config {
                Some(configuration) => {
                    // Handle expiration
                    match configuration.expiration {
                        Some(exp) => self.set_expiration(&base64_token, exp.as_secs())?,
                        None => (),
                    }
                }
                None => (),
            }
            Ok(base64_token)
        } else {
            Err(Error::ActiveSessionError)
        }
    }
    /// Set session expiration
    fn set_expiration(&mut self, base64_token: &str, duration: u64) -> Result<(), Error> {
        let _: () = redis::cmd("EXPIRE")
            .arg(&base64_token)
            .arg(duration)
            .query(&mut self.conn)?;
        Ok(())
    }
    /// Checks if a session is active
    pub fn is_active(&mut self, base64_token: &str) -> Result<bool, Error> {
        let exists: bool = redis::cmd("HEXISTS")
            .arg(base64_token)
            .arg("session_data")
            .query(&mut self.conn)?;

        if exists {
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Remove session from redis store
    pub fn remove(&mut self, base64_token: &str) -> Result<(), Error> {
        let _: () = redis::cmd("HDEL")
            .arg(base64_token)
            .arg("session_data")
            .query(&mut self.conn)?;
        Ok(())
    }
    /// Generate random token of 64 bytes
    fn generate_token() -> Vec<u8> {
        randombytes(64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_redisesh() {
        let client1 = Redisesh::new("redis://127.0.0.1/");
        let client2 = Redisesh::new("Invalid address");

        match client1 {
            Ok(_r) => {
                assert!(true)
            }
            Err(err) => panic!("{}", err.to_string()),
        }

        match client2 {
            Ok(_r) => {
                panic!("Should faild")
            }
            Err(_err) => assert!(true),
        }
    }

    #[test]
    fn test_insert_session() {
        let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
        let session_data = String::from("{username: john_smith}");
        let res = redisesh.insert(Some(session_data));

        match res {
            Ok(key) => {
                let client = redis::Client::open("redis://127.0.0.1/").unwrap();
                let mut con = client.get_connection().unwrap();
                let sesh_data: String = redis::cmd("HGET")
                    .arg(key)
                    .arg("session_data")
                    .query(&mut con)
                    .unwrap();

                assert_eq!(sesh_data, String::from("{username: john_smith}"));
            }
            Err(err) => panic!("{}", err),
        }
    }

    #[test]
    fn test_is_active() {
        let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
        let base64_token = "bQaJNiB01CFAJrv/jRdUwFQbLI9LoqiIfJWxH1t3oiyWuL0gio3CVTUgWwdWUPdm/FslH3n3gAEMQerfvkwtPQ==";
        let exists = redisesh.is_active(base64_token).unwrap();

        // Token does not exist
        if exists {
            assert!(false);
        } else {
            assert!(true)
        }

        let session_data = Some(String::from("{username: John}"));
        let base64_token = redisesh.insert(session_data).unwrap();
        let exists = redisesh.is_active(&base64_token).unwrap();

        // Token exists because we inserted it
        if exists {
            assert!(true);
        } else {
            assert!(false)
        }
    }

    #[test]
    fn test_remove() {
        let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
        let session_data = Some(String::from("{username: John}"));
        let base64_token = redisesh.insert(session_data).unwrap();
        let exists = redisesh.is_active(&base64_token).unwrap();

        // Token currently exists because we just inserted it
        if exists {
            assert!(true);
        } else {
            assert!(false)
        }

        // Remove the token
        redisesh.remove(&base64_token).unwrap();

        let exists = redisesh.is_active(&base64_token).unwrap();

        // Token does not exist because we just removed it
        if exists {
            assert!(false);
        } else {
            assert!(true);
        }
    }

    #[test]
    fn test_generate_token() {
        let token1 = Redisesh::generate_token();
        let token2 = Redisesh::generate_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_expiration() {
        let expiration = std::time::Duration::from_secs(1);
        let mut redisesh = Redisesh::new("redis://127.0.0.1/").unwrap();
        let session_data = Some(String::from("{username: John}"));
        let base64_token = redisesh.insert(session_data).unwrap();

        std::thread::sleep(expiration);

        let exists = redisesh.is_active(&base64_token).unwrap();

        // Token still exists because no expiration was set after it
        // was inserted
        if exists {
            assert!(true);
        } else {
            assert!(false)
        }

        // Set session expiration
        redisesh.configure(Config {
            expiration: Some(expiration),
        });

        // Insert new session
        let session_data = Some(String::from("{username: Smith}"));
        let base64_token = redisesh.insert(session_data).unwrap();

        std::thread::sleep(expiration);

        let exists = redisesh.is_active(&base64_token).unwrap();

        // Token does not exist because expiration was set
        if exists {
            assert!(false);
        } else {
            assert!(true)
        }
    }
}
