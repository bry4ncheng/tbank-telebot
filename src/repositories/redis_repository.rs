use anyhow::anyhow;
use bb8_redis::RedisConnectionManager;
use bb8_redis::bb8::Pool;
use bb8_redis::redis::{AsyncCommands, RedisError};
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
    pub async fn get_data_from_redis(self, key: &String) -> anyhow::Result<String> {
        let mut redis_conn = self.redis_client.get().await.unwrap();
        let redis_key = format!("{}:{}", REDIS_PREFIX, key);
        let res = redis_conn.get(redis_key.clone()).await;
        return match res {
            Ok(data) => Ok(data),
            Err(e) => {
                warn!("Something went wrong populating redis: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
    }

    pub async fn set_data_in_redis(self, key: &String, value: String, to_expire: bool) -> anyhow::Result<()> {
        let mut redis_conn = self.redis_client.get().await?;
        if to_expire {
            //2 min TTL
            let res : Result<(), RedisError> = redis_conn.set_ex(format!("{}:{}", REDIS_PREFIX, key.clone()), value, 120).await;
            return match res {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("Something went wrong populating redis: {:?}", e);
                    Err(anyhow!("Something went wrong!"))
                }
            };
        }else{
            // No ttl
            let res : Result<(), RedisError> = redis_conn.set(format!("{}:{}", REDIS_PREFIX, key.clone()), value).await;
            return match res {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("Something went wrong populating redis: {:?}", e);
                    Err(anyhow!("Something went wrong!"))
                }
            };
        }
    }


    pub async fn remove_data_in_redis(self, key: &String) -> anyhow::Result<()> {
        let mut redis_conn = self.redis_client.get().await?;
        //1 day ttl
        let res : Result<(), RedisError> = redis_conn.del(format!("{}:{}", REDIS_PREFIX, key.clone())).await;
        return match res {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Something went wrong populating redis: {:?}", e);
                Err(anyhow!("Something went wrong!"))
            }
        };
    }
}