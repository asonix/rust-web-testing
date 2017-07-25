/*
 * This file is part of Authentication.
 *
 * Copyright © 2017 Riley Trautman
 *
 * Authentication is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Authentication is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Authentication.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;
use std::thread;
use std::sync::{Arc, mpsc};
use std::any::Any;
use std::fmt;
use std::error::Error as StdError;

pub struct Message<T: Send + Sync> {
    name: String,
    message: Option<T>,
    retries: i32,
}

impl<T: Send + Sync> Message<T> {
    pub fn new(name: String, message: Option<T>) -> Message<T> {
        Message::<T> {
            name: name,
            message: message,
            retries: 10,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn message(&self) -> &Option<T> {
        &self.message
    }

    pub fn retries(&self) -> i32 {
        self.retries
    }
}

#[derive(Debug)]
pub enum Error {
    ProcessingError(String),
    DuplicateHandler(String),
    ExitHandler,
    SendError,
    JoinError,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ProcessingError(_) => "Error processing job",
            Error::DuplicateHandler(_) => "Handler with that name already exists",
            Error::ExitHandler => "Cannot register handler with reserved anme 'exit'",
            Error::SendError => "Could not send data",
            Error::JoinError => "Could not join thread",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ProcessingError(ref s) => write!(f, "Error processing data: '{}'", s),
            Error::DuplicateHandler(ref s) => write!(f, "Handler already exists for '{}'", s),
            Error::ExitHandler => write!(f, "Cannot register handler with reserved name 'exit'"),
            Error::SendError => write!(f, "Could not send data to thread"),
            Error::JoinError => write!(f, "Could not join thread"),
        }
    }
}

impl<T: Send + Sync> From<mpsc::SendError<Message<T>>> for Error {
    fn from(_err: mpsc::SendError<Message<T>>) -> Error {
        Error::SendError
    }
}

impl From<Box<Any + Send>> for Error {
    fn from(_err: Box<Any + Send>) -> Error {
        Error::JoinError
    }
}

type Handler<'a, T> = Fn(&Option<T>) -> Result<(), Error> + Send + Sync + 'a;
type SafeHandler<'a, T> = Arc<Handler<'a, T>>;

#[derive(Clone)]
pub struct InitialConfig<'a, T: Send + Sync>
where
    T: 'a,
{
    handlers: HashMap<String, SafeHandler<'a, T>>,
}

impl<'a, T: Send + Sync> InitialConfig<'a, T> {
    pub fn new() -> Self {
        InitialConfig { handlers: HashMap::new() }
    }

    pub fn register_handler(
        &mut self,
        name: String,
        handler: SafeHandler<'a, T>,
    ) -> Result<(), Error> {
        if &name == "exit" {
            return Err(Error::ExitHandler);
        }

        if self.handlers.contains_key(&name) {
            return Err(Error::DuplicateHandler(name));
        };

        self.handlers.insert(name, handler);

        Ok(())
    }
}

pub struct Config<T: Send + Sync> {
    hook: mpsc::Sender<Message<T>>,
    handle: thread::JoinHandle<()>,
}

impl<T: Send + Sync> Config<T> {
    pub fn hook(&self) -> mpsc::Sender<Message<T>> {
        self.hook.clone()
    }
}

pub fn run<'a, T: Send + Sync + Clone>(config: InitialConfig<'static, T>) -> Config<T> {
    let (hook, receiver) = mpsc::channel::<Message<T>>();
    let thread_hook = hook.clone();

    let thread = thread::spawn(move || {
        let InitialConfig { handlers } = config.clone();

        for msg in receiver {
            if msg.name() == "exit" {
                break;
            }

            let handler = match handlers.get(msg.name()) {
                Some(ref handler) => *handler,
                None => {
                    println!("No handler for message '{}'", msg.name());
                    continue;
                }
            };

            if let Err(err) = handler(msg.message()) {
                if msg.retries > 0 {
                    println!(
                        "Task for '{}' failed with error: '{}', retrying",
                        msg.name(),
                        err
                    );
                    thread_hook
                        .send(Message {
                            name: msg.name,
                            message: msg.message,
                            retries: msg.retries - 1,
                        })
                        .expect("Failed to requeue task");
                } else {
                    println!(
                        "Task for '{}' failed permanently with error: '{}'",
                        msg.name(),
                        err
                    );
                }
            };
        }

        ()
    });

    Config::<T> {
        hook: hook,
        handle: thread,
    }
}

pub fn cleanup<T: Send + Sync>(config: Config<T>) -> Result<(), Error> {
    let Config { handle, hook } = config;

    hook.send(Message {
        name: "exit".to_owned(),
        message: None,
        retries: 0,
    })?;

    handle.join()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_works() {
        let config: InitialConfig<i32> = InitialConfig::new();

        let config = run(config);

        let result = cleanup(config);

        assert!(result.is_ok(), "Failed to perform job lifecycle");
    }
}