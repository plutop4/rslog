
use serde::Serialize;

#[derive(Debug, Default, PartialEq, Serialize)]
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

#[cfg(test)]
mod test {
    use super::*;
    use redis::FromRedisValue;
    use redis::Value as v;
    #[test]
    fn from_valid() {
        let val = v::Bulk(vec![
            v::Int(1),
            v::Int(2),
            v::Int(3),
            v::Bulk(vec![
                v::Data("command".as_bytes().to_vec()),
                v::Data("arg1".as_bytes().to_vec()),
                v::Data("arg2".as_bytes().to_vec()),
            ]),
            v::Data("127.0.0.1:10000".as_bytes().to_vec()),
            v::Data("my cool client".as_bytes().to_vec()),
        ]);
        assert_eq!(
            SlowlogRecord::from_redis_value(&val).unwrap(),
            SlowlogRecord {
                id: 1,
                time: 2,
                duration: 3,
                command: vec!["command".to_owned(), "arg1".to_owned(), "arg2".to_owned()],
                client_socket: "127.0.0.1:10000".to_owned(),
                client_name: "my cool client".to_owned(),
            }
        )
    }
}
