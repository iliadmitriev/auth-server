use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use uuid::Uuid;

pub async fn create_session(
    mut redis: ConnectionManager,
    user_id: Uuid,
    duration_days: u64,
) -> Result<String, redis::RedisError> {
    let session_id = Uuid::new_v4().to_string();
    let key = format!("session:{}", session_id);

    let ttl_seconds = duration_days * 24 * 60 * 60;
    let _: () = redis.set_ex(&key, user_id.to_string(), ttl_seconds).await?;

    Ok(session_id)
}
