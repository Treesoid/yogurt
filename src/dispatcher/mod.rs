use crate::argument::parser::ArgumentParser;
use crate::argument::Argument;
use crate::parsers::tokenize::{tokenize, Token};
use crate::{Error, InvalidCommandReason, Result};
pub use builder::*;
pub use exec_context::ExecContext;
use nom::bytes::complete::tag;
use nom::character::complete::multispace0;
use std::collections::{HashMap};
use std::fmt::Debug;

mod builder;
mod exec_context;

pub enum NodeType {
    Argument(Argument),
    Literal(String),
}

enum ExecState {
    Working,
    Done(Result<()>),
}

pub struct Dispatcher<C: Debug> {
    root: Command<C>,
    prefix: String,
    context_factory: Box<dyn Fn() -> C>,
}

#[allow(clippy::type_complexity)]
pub struct Command<C: Debug> {
    children: Vec<Command<C>>,
    node: NodeType,
    exec: Option<Box<dyn Fn(&mut ExecContext<C>) -> Result<()>>>,
}

impl<C: Debug> Command<C> {
    pub fn literal(name: impl Into<String>) -> CommandBuilder<C> {
        CommandBuilder::literal(name)
    }

    pub fn argument(
        name: impl Into<String>,
        parser: impl ArgumentParser,
        required: bool,
    ) -> CommandBuilder<C> {
        CommandBuilder::argument(parser, name, required)
    }

    fn execute(
        &self,
        mut offset: usize,
        tokens: &[String],
        named_arguments: &mut HashMap<String, String>,
        context: &mut ExecContext<C>,
    ) -> ExecState {
        if offset <= tokens.len() {
            return ExecState::Done(if let Some(exec) = &self.exec {
                exec(context)
            } else {
                Err(Error::InvalidCommand(InvalidCommandReason::UnknownCommand))
            });
        }

        for child in &self.children {
            if child.process(&mut offset, tokens, named_arguments, context) {
                match child.execute(offset, tokens, named_arguments, context) {
                    ExecState::Working => continue,
                    ExecState::Done(res) => return ExecState::Done(res),
                }
            }
        }

        ExecState::Working
    }

    fn process(
        &self,
        offset: &mut usize,
        tokens: &[String],
        named_arguments: &mut HashMap<String, String>,
        context: &mut ExecContext<C>,
    ) -> bool {
        match &self.node {
            NodeType::Literal(name) => {
                if let Some(token) = tokens.get(*offset) {
                    if name == token {
                        *offset += 1;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            NodeType::Argument(argument) => {
                if let Some(named) = named_arguments.get(&argument.name) {
                    if argument.matches(named) {
                        context.insert_argument(argument.name.clone(), named.clone());
                        true
                    } else {
                        false
                    }
                } else if let Some(token) = tokens.get(*offset) {
                    if argument.matches(token) {
                        *offset += 1;
                        context.insert_argument(argument.name.clone(), token.clone());
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.node, NodeType::Literal(_))
    }

    pub fn is_argument(&self) -> bool {
        matches!(self.node, NodeType::Argument(_))
    }
}

impl<C: Debug> Dispatcher<C> {
    pub fn builder() -> DispatcherBuilder<C> {
        DispatcherBuilder::new()
    }

    pub fn run_command(&self, command: &str) -> Result<()> {
        // remove leading whitespace and prefix
        let (command, _) = multispace0(command)?;
        let (command, _) = tag(self.prefix.as_str())(command)?;

        let (_, mut tokens) = tokenize(command)?;
        tokens.push(Token::End);

        let mut cmd_tokens = vec![];
        for token in tokens {
            if token != Token::End {
                cmd_tokens.push(token);
            } else if !cmd_tokens.is_empty() {
                self.execute_command(cmd_tokens)?;
                cmd_tokens = vec![];
            }
        }
        Ok(())
    }

    fn execute_command(&self, tokens: Vec<Token>) -> Result<()> {
        println!("{tokens:#?}");
        let (named_arguments, tokens): (Vec<_>, _) = tokens
            .into_iter()
            .partition(|token| matches!(token, &Token::Named(_, _)));
        let tokens = unwrap_tokens(tokens);
        let mut named_args = map_named_arguments(named_arguments);

        match self.root.execute(
            0,
            tokens.as_slice(),
            &mut named_args,
            &mut ExecContext::new((self.context_factory)()),
        ) {
            ExecState::Working => Err(Error::InvalidCommand(InvalidCommandReason::UnknownCommand)),
            ExecState::Done(res) => res,
        }
    }
}

fn unwrap_tokens(tokens: Vec<Token>) -> Vec<String> {
    let mut output = vec![];
    for token in tokens {
        if let Token::Simple(content) = token {
            output.push(content);
        }
    }
    output
}

fn map_named_arguments(tokens: Vec<Token>) -> HashMap<String, String> {
    let mut output = HashMap::new();
    for token in tokens {
        if let Token::Named(key, value) = token {
            output.insert(key, value);
        }
    }
    output
}

impl<C: Debug> From<CommandBuilder<C>> for Command<C> {
    fn from(builder: CommandBuilder<C>) -> Self {
        builder.build()
    }
}
