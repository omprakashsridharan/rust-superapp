use opentelemetry::propagation::{Extractor, Injector};
use rdkafka::message::{BorrowedHeaders, Headers, OwnedHeaders};
pub struct HeaderInjector<'a>(pub &'a mut OwnedHeaders);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        let mut new = OwnedHeaders::new().insert(rdkafka::message::Header {
            key,
            value: Some(&value),
        });

        for header in self.0.iter() {
            let s = String::from_utf8(header.value.unwrap().to_vec()).unwrap();
            new = new.insert(rdkafka::message::Header {
                key: header.key,
                value: Some(&s),
            });
        }

        self.0.clone_from(&new);
    }
}

pub struct HeaderExtractor<'a>(pub &'a BorrowedHeaders);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        for i in 0..self.0.count() {
            if let Ok(val) = self.0.get_as::<str>(i) {
                if val.key == key {
                    return val.value;
                }
            }
        }
        None
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|kv| kv.key).collect::<Vec<_>>()
    }
}
