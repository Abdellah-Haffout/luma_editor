use crate::descriptor::{CommandId, CommandRegistry};
use crate::journal::CrashJournal;
use luma_core::graph::GraphStore;
use luma_core::history::HistoryStack;
use luma_domain::evaluation::DomainPayloadStore;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum CommandExecutionError {
    #[error("Command not found in dispatcher: {0}")]
    NotFound(CommandId),

    #[error("Failed to execute command '{0}': {1}")]
    ExecutionFailed(CommandId, String),

    #[error("Command condition not satisfied: {0}")]
    ConditionFailed(String),
}

/// Context provided to every command during execution.
pub struct CommandContext<'a> {
    pub graph: &'a mut GraphStore,
    pub history: &'a mut HistoryStack,
    pub payloads: &'a mut DomainPayloadStore,
    pub dataflow: &'a mut luma_dataflow::engine::DataflowEngine,
}

pub type CommandFn = Box<dyn FnMut(&mut CommandContext) -> Result<(), String> + Send + Sync>;

pub struct CommandDispatcher {
    registry: CommandRegistry,
    handlers: HashMap<CommandId, CommandFn>,
    journal: Option<CrashJournal>,
}

impl Default for CommandDispatcher {
    fn default() -> Self {
        Self::new(CommandRegistry::new())
    }
}

impl CommandDispatcher {
    pub fn new(registry: CommandRegistry) -> Self {
        Self {
            registry,
            handlers: HashMap::new(),
            journal: None,
        }
    }

    pub fn with_journal(mut self, journal: CrashJournal) -> Self {
        self.journal = Some(journal);
        self
    }

    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut CommandRegistry {
        &mut self.registry
    }

    pub fn register_handler(
        &mut self,
        id: impl Into<CommandId>,
        handler: impl FnMut(&mut CommandContext) -> Result<(), String> + Send + Sync + 'static,
    ) {
        self.handlers.insert(id.into(), Box::new(handler));
    }

    pub fn dispatch(
        &mut self,
        id: &CommandId,
        ctx: &mut CommandContext,
    ) -> Result<(), CommandExecutionError> {
        let handler = self
            .handlers
            .get_mut(id)
            .ok_or_else(|| CommandExecutionError::NotFound(id.clone()))?;

        handler(ctx)
            .map_err(|err| CommandExecutionError::ExecutionFailed(id.clone(), err))?;

        if let Some(journal) = &mut self.journal {
            journal.record(id);
        }

        Ok(())
    }
}
