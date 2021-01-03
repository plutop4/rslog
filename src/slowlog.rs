#[derive(Debug, Default)]
pub struct SlowlogRecord {
    pub id: u64,
    pub time: u64,
    pub duration: u32,
    pub command: Vec<String>,
    pub client_socket: String,
    pub client_name: String,
}

fn next_value<T: redis::FromRedisValue>(
    i: &mut std::slice::Iter<redis::Value>,
) -> redis::RedisResult<T> {
    match i.next() {
        Some(v) => redis::FromRedisValue::from_redis_value(v),
        None => Err(redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "The field is not found in the response",
        ))),
    }
}

impl redis::FromRedisValue for SlowlogRecord {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<SlowlogRecord> {
        let rows = &mut v.as_sequence().unwrap().iter();
        Ok(SlowlogRecord {
            id: next_value(rows)?,
            time: next_value(rows)?,
            duration: next_value(rows)?,
            command: next_value(rows)?,
            client_socket: next_value(rows)?,
            client_name: next_value(rows)?,
        })
    }
}
