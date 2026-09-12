use std::any::Any;
use std::sync::Arc;

use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::dispatch::Dispatcher;
use crate::lifecycle::{PluginLifecycle, PluginService};
use crate::logging;
use crate::manifest;
use crate::protocol::{
    RegistrationArguments, StreamDeckConnection, TokioWebSocketTransport, Transport,
};
use crate::registry::Registry;
use crate::Action;
use crate::Result;

/// Host that owns shared state, the registry, and the connection loop.
pub struct Plugin;

impl Plugin {
    /// Start a builder from process arguments.
    ///
    /// Pass [`std::env::args()`](std::env::args) as-is. The program name is ignored.
    pub fn builder(args: impl IntoIterator<Item = impl AsRef<str>>) -> PluginBuilder<()> {
        PluginBuilder {
            arguments: RegistrationArguments::parse(args)
                .expect("invalid Stream Deck launch arguments"),
            state: Some(()),
            registry: Registry::new(),
            lifecycle: Vec::new(),
            services: Vec::new(),
            transport: None,
            skip_manifest: false,
        }
    }
}

/// Fluent builder for [`Plugin`].
pub struct PluginBuilder<S> {
    arguments: RegistrationArguments,
    state: Option<S>,
    registry: Registry,
    lifecycle: Vec<Box<dyn PluginLifecycle>>,
    services: Vec<Box<dyn PluginService>>,
    transport: Option<Box<dyn Transport>>,
    skip_manifest: bool,
}

impl PluginBuilder<()> {
    /// Set the shared plugin state.
    pub fn state<S: Send + Sync + 'static>(self, state: S) -> PluginBuilder<S> {
        PluginBuilder {
            arguments: self.arguments,
            state: Some(state),
            registry: self.registry,
            lifecycle: self.lifecycle,
            services: self.services,
            transport: self.transport,
            skip_manifest: self.skip_manifest,
        }
    }
}

impl<S: Send + Sync + 'static> PluginBuilder<S> {
    /// Explicitly register an action type.
    pub fn add_action<A: Action<State = S>>(mut self) -> Self {
        self.registry.add_action::<A, S>();
        self
    }

    /// Register every `#[streamdeck_action]` discovered by `inventory`.
    pub fn add_registered_actions(mut self) -> Self {
        self.registry.add_registered();
        self
    }

    /// Add a plugin-wide lifecycle hook.
    pub fn lifecycle(mut self, hook: impl PluginLifecycle) -> Self {
        self.lifecycle.push(Box::new(hook));
        self
    }

    /// Add a start/stop service.
    pub fn service(mut self, service: impl PluginService) -> Self {
        self.services.push(Box::new(service));
        self
    }

    /// Inject a transport. Used by tests.
    pub fn transport(mut self, transport: impl Transport + 'static) -> Self {
        self.transport = Some(Box::new(transport));
        self
    }

    /// Skip the debug manifest UUID check.
    pub fn skip_manifest_check(mut self) -> Self {
        self.skip_manifest = true;
        self
    }
}

impl<S: PluginService + Send + Sync + 'static> PluginBuilder<S> {
    /// Connect, dispatch, and shut down.
    pub async fn run(mut self) -> Result<()> {
        let state = self.state.take().ok_or_else(|| {
            crate::Error::State("Plugin::builder().state(...) is required before run()".into())
        })?;
        if !self.skip_manifest {
            manifest::check(&self.registry, &self.arguments.plugin_uuid)?;
        }

        let transport = self
            .transport
            .unwrap_or_else(|| Box::new(TokioWebSocketTransport::new()));
        let connection =
            StreamDeckConnection::with_transport(self.arguments.clone(), BoxedTransport(transport));
        let sender = connection.sender();
        logging::init_tracing(sender.clone(), &self.arguments.plugin_uuid);

        let state = Arc::new(state);
        state.start(sender.clone()).await?;
        for service in &self.services {
            service.start(sender.clone()).await?;
        }

        let dispatcher = Arc::new(Mutex::new(Dispatcher::new(
            self.registry,
            Arc::clone(&state) as Arc<dyn Any + Send + Sync>,
            sender,
            self.lifecycle,
        )));

        let token = CancellationToken::new();
        let shutdown = token.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            shutdown.cancel();
        });

        let run_result = connection
            .run(
                {
                    let dispatcher = Arc::clone(&dispatcher);
                    move |message| {
                        let dispatcher = Arc::clone(&dispatcher);
                        async move {
                            dispatcher.lock().await.dispatch(message).await;
                        }
                    }
                },
                token,
            )
            .await;

        for service in self.services.iter().rev() {
            if let Err(error) = service.stop().await {
                tracing::error!(error = %error, "failed to stop plugin service");
            }
        }
        if let Err(error) = state.stop().await {
            tracing::error!(error = %error, "failed to stop plugin state service");
        }
        run_result.map_err(Into::into)
    }
}

struct BoxedTransport(Box<dyn Transport>);

#[async_trait::async_trait]
impl Transport for BoxedTransport {
    async fn connect(&mut self, url: &str) -> streamdeck_plugin_protocol::Result<()> {
        self.0.connect(url).await
    }

    async fn send(&mut self, json: &str) -> streamdeck_plugin_protocol::Result<()> {
        self.0.send(json).await
    }

    async fn receive(&mut self) -> streamdeck_plugin_protocol::Result<Option<String>> {
        self.0.receive().await
    }
}
