use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image, ImageArgs};

const NAME: &str = "confluentinc/cp-schema-registry";
const TAG: &str = "7.4.0";
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

impl SchemaRegistry {
    pub fn new(env_vars: HashMap<String, String>) -> Self {
        let mut default_env_vars = HashMap::new();
        default_env_vars.insert(
            "SCHEMA_REGISTRY_LISTENERS".to_owned(),
            "http://0.0.0.0:8081".to_owned(),
        );
        default_env_vars.insert(
            "SCHEMA_REGISTRY_HOST_NAME".to_owned(),
            "schema-registry".to_owned(),
        );
        default_env_vars.extend(env_vars.into_iter());
        Self {
            env_vars: default_env_vars,
        }
    }
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
        vec![WaitFor::message_on_stderr("Server started, listening for requests... (io.confluent.kafka.schemaregistry.rest.SchemaRegistryMain)")]
    }

    fn expose_ports(&self) -> Vec<u16> {
        vec![SCHEMA_REGISTRY_PORT]
    }

    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }
}
