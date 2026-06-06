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

pub async fn get_session_user_id(
    mut redis: ConnectionManager,
    session_id: &str,
) -> Result<Option<Uuid>, redis::RedisError> {
    let key = format!("session:{}", session_id);
    let user_id: Option<String> = redis.get(&key).await?;

    Ok(user_id.and_then(|s| Uuid::parse_str(&s).ok()))
}

pub async fn delete_session(
    mut redis: ConnectionManager,
    session_id: &str,
) -> Result<(), redis::RedisError> {
    let key = format!("session:{}", session_id);
    let _: () = redis.del(&key).await?;

    Ok(())
}
