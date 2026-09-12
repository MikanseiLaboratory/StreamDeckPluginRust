use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::action::ActionInstance;
use crate::protocol::Controller;
use crate::Action;

/// Compile-time registration record emitted by `#[streamdeck_action]`.
pub struct ActionRegistration {
    /// Action UUID.
    pub uuid: &'static str,
    /// Expected controller for manifest checks.
    pub controller: Controller,
    /// Builds an instance from type-erased plugin state.
    pub factory: fn(Arc<dyn Any + Send + Sync>) -> Box<dyn ActionInstance>,
}

/// UUID → factory map.
#[derive(Default)]
pub struct Registry {
    factories: HashMap<String, ActionRegistration>,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register one action type that shares plugin state `S`.
    pub fn add_action<A, S>(&mut self)
    where
        A: Action<State = S>,
        S: Send + Sync + 'static,
    {
        self.add(ActionRegistration {
            uuid: A::UUID,
            controller: Controller::Keypad,
            factory: |state| {
                let state = state
                    .downcast::<S>()
                    .expect("plugin state type does not match this action's State");
                Box::new(crate::TypedInstance::<A>::new(state))
            },
        });
    }

    /// Insert a registration record.
    pub fn add(&mut self, registration: ActionRegistration) {
        self.factories
            .insert(registration.uuid.to_ascii_lowercase(), registration);
    }

    /// Collect every `inventory` submission.
    pub fn add_registered(&mut self) {
        for registration in inventory::iter::<ActionRegistration> {
            self.add(ActionRegistration {
                uuid: registration.uuid,
                controller: registration.controller,
                factory: registration.factory,
            });
        }
    }

    /// Look up a factory by UUID (case-insensitive).
    pub fn get(&self, uuid: &str) -> Option<&ActionRegistration> {
        self.factories.get(&uuid.to_ascii_lowercase())
    }

    /// Registered UUIDs in insertion order (sorted for stable checks).
    pub fn uuids(&self) -> Vec<&'static str> {
        let mut uuids: Vec<_> = self.factories.values().map(|item| item.uuid).collect();
        uuids.sort_unstable();
        uuids
    }
}
