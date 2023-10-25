use anyhow::anyhow;
use bb8_redis::{RedisConnectionManager};
use bb8_redis::bb8::Pool;
use bb8_redis::redis::{AsyncCommands, RedisError};
use serde_json::Value;
use tracing::warn;

const REDIS_PREFIX: &str = "usr";
#[derive(Clone)]
pub struct RedisRepository {
    redis_client: Pool<RedisConnectionManager>
}

impl RedisRepository {
    pub async fn new(redis_url: String) -> Self {
        let redis_manager = RedisConnectionManager::new(redis_url.clone()).unwrap();
        let redis_pool = Pool::builder().build(redis_manager).await.unwrap();
        Self {
            redis_client: redis_pool,
        }
    }

    //REDIS
    pub async fn get_credentials_from_redis(self, key: &String) -> anyhow::Result<Value> {
        let mut redis_conn = self.redis_client.get().await.unwrap();
        let redis_key = format!("{}:{}", REDIS_PREFIX, key);
        let res: String = redis_conn.get(redis_key.clone()).await?;
        //Up to u if u need this in use case @zaki
        //delete entry after use
        //let _del_res = redis_conn.del(redis_key).await?;
        let data = match serde_json::from_str::<Value>(&res) {
            Ok(data) => Ok(data),
            Err(e) => {
                warn!("Something went wrong while deserializing: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
        data
    }

    pub async fn set_user_in_redis(self, key: &String, value: String) -> anyhow::Result<()> {
        let mut redis_conn = self.redis_client.get().await?;
        //1 day ttl
        let res : Result<(), RedisError> = redis_conn.set_ex(format!("{}:{}", REDIS_PREFIX, key.clone()), value, 86400).await;
        return match res {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Something went wrong populating redis: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
    }
}