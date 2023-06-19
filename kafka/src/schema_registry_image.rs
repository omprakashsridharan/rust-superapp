use std::collections::HashMap;

use testcontainers::{Image, ImageArgs};

const NAME: &str = "confluentinc/cp-kafka";
const TAG: &str = "6.1.1";
pub const SCHEMA_REGISTRY_PORT: u16 = 8081;

#[derive(Debug, Default, Clone)]
pub struct SchemaRegistryArgs;

impl ImageArgs for SchemaRegistryArgs {
    fn into_iterator(self) -> Box<dyn Iterator<Item = String>> {
        return Box::new(vec![].into_iter());
    }
}

pub struct SchemaRegistry {
    env_vars: HashMap<String, String>,
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert(
            "SCHEMA_REGISTRY_LISTENERS".to_owned(),
            "http://0.0.0.0:8081".to_owned(),
        );
        Self { env_vars }
    }
}

impl Image for SchemaRegistry {
    type Args = SchemaRegistryArgs;

    fn name(&self) -> String {
        NAME.to_owned()
    }

    fn tag(&self) -> String {
        TAG.to_owned()
    }

    fn ready_conditions(&self) -> Vec<testcontainers::core::WaitFor> {
        vec![]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![SCHEMA_REGISTRY_PORT]
    }


}
