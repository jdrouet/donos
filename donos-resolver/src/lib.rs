#[cfg(feature = "mock")]
pub mod mock;
pub mod prelude;

use donos_proto::packet::{DnsPacket, QueryType};

#[derive(Clone, Debug)]
pub enum ManagerBuilderError {
    NoResolver,
}

#[derive(Debug, Default)]
pub struct ManagerBuilder {
    resolvers: Vec<Box<dyn prelude::Resolver>>,
}

impl ManagerBuilder {
    pub fn add_resolver(&mut self, value: Box<dyn prelude::Resolver>) {
        self.resolvers.push(value);
    }

    pub fn with_resolver(mut self, value: Box<dyn prelude::Resolver>) -> Self {
        self.resolvers.push(value);
        self
    }

    pub fn build(self) -> Result<Manager, ManagerBuilderError> {
        if self.resolvers.is_empty() {
            return Err(ManagerBuilderError::NoResolver);
        }
        Ok(Manager {
            resolvers: self.resolvers,
        })
    }
}

#[derive(Clone, Debug)]
pub enum ManagerError {
    Failed(Vec<prelude::ResolverError>),
}

#[derive(Debug)]
pub struct Manager {
    resolvers: Vec<Box<dyn prelude::Resolver>>,
}

impl Manager {
    pub async fn resolve(
        &self,
        kind: QueryType,
        hostname: &str,
    ) -> Result<(DnsPacket, Vec<prelude::ResolverError>), ManagerError> {
        let mut errors = Vec::new();
        for resolver in self.resolvers.iter() {
            match resolver.resolve(kind, hostname).await {
                Ok(found) => return Ok((found, errors)),
                Err(err) => errors.push(err),
            };
        }
        Err(ManagerError::Failed(errors))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn manager_builder_should_error_if_no_resolver() {
        let builder = super::ManagerBuilder::default().build();
        assert!(builder.is_err());
    }

    #[tokio::test]
    async fn manager_should_call_resolvers() {
        let manager = super::ManagerBuilder::default()
            .with_resolver(Box::new(crate::mock::MockResolver::new("first")))
            .build()
            .unwrap();
        let _ = manager
            .resolve(super::QueryType::A, "foo.bar")
            .await
            .unwrap_err();
    }
}
