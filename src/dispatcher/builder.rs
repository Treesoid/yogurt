use super::{ExecContext, NodeType};
use crate::argument::parser::ArgumentParser;
use crate::argument::Argument;
use crate::{Command, Dispatcher, Error, Result};
use std::fmt::Debug;

#[allow(clippy::type_complexity)]
pub struct CommandBuilder<C: Debug> {
    children: Vec<Command<C>>,
    node: NodeType,
    exec: Option<Box<dyn Fn(&mut ExecContext<C>) -> Result<()>>>,
}

impl<C: Debug> CommandBuilder<C> {
    pub fn literal(name: impl Into<String>) -> Self {
        Self {
            children: vec![],
            node: NodeType::Literal(name.into()),
            exec: None,
        }
    }

    pub fn argument(parser: impl ArgumentParser, name: impl Into<String>, required: bool) -> Self {
        Self {
            children: vec![],
            exec: None,
            node: NodeType::Argument(Argument::new(parser.validator(), name.into(), required)),
        }
    }

    pub fn exec(mut self, exec: Box<dyn Fn(&mut ExecContext<C>) -> Result<()>>) -> Self {
        self.exec = Some(exec);
        self
    }

    pub fn child(mut self, child: impl Into<Command<C>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn build(self) -> Command<C> {
        let (mut literals, arguments): (Vec<_>, Vec<_>) =
            self.children.into_iter().partition(|c| c.is_literal());
        literals.extend(arguments);
        Command {
            children: literals,
            node: self.node,
            exec: self.exec,
        }
    }
}

pub struct DispatcherBuilder<C: Debug> {
    root: CommandBuilder<C>,
    prefix: String,
    context_factory: Option<Box<dyn Fn() -> C>>,
}

impl<C: Debug> DispatcherBuilder<C> {
    pub fn new() -> Self {
        Self {
            root: CommandBuilder::literal(""),
            prefix: "".to_string(),
            context_factory: None,
        }
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    pub fn context(mut self, factory: Box<dyn Fn() -> C>) -> Self {
        self.context_factory = Some(factory);
        self
    }

    pub fn child(mut self, child: impl Into<Command<C>>) -> Self {
        self.root.children.push(child.into());
        self
    }

    pub fn build(self) -> Result<Dispatcher<C>> {
        Ok(Dispatcher {
            root: self.root.build(),
            prefix: self.prefix,
            context_factory: self.context_factory.ok_or(Error::IncompleteBuilder)?,
        })
    }
}

impl<C: Debug> Default for DispatcherBuilder<C> {
    fn default() -> Self {
        Self {
            root: CommandBuilder::literal(""),
            prefix: "".to_string(),
            context_factory: None,
        }
    }
}
