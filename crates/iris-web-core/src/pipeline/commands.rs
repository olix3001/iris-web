use std::sync::{Mutex, Arc};

use super::{request_pipeline::PipelineData, controller::ControllerParam};

pub(crate) struct CommandQueue {
    commands: Arc<Mutex<Vec<Box<dyn CommandAction>>>>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn execute(&self, pipeline_data: &mut PipelineData) {
        let mut commands = self.commands.lock().unwrap();
        for command in commands.iter_mut() {
            command.execute(pipeline_data);
        }
        commands.clear();
    }
}

#[derive(Clone)]
pub struct Commands {
    queue: Arc<CommandQueue>,
}

impl Commands {
    pub(crate) fn from_queue(queue: Arc<CommandQueue>) -> Self {
        Self {
            queue,
        }
    }

    pub fn add_command<T: CommandAction + 'static>(&self, command: T) {
        let mut commands = self.queue.commands.lock().unwrap();
        commands.push(Box::new(command));
    }

    pub fn add_data<T: Send + Sync + 'static>(&self, data: T) {
        self.add_command(AddData {
            data: Some(data),
        });
    }
}

pub trait CommandAction {
    fn execute(&mut self, pipeline_data: &mut PipelineData);
}

pub struct AddData<T> {
    data: Option<T>,
}
impl<T: Send + Sync + 'static> CommandAction for AddData<T> {
    fn execute(&mut self, pipeline_data: &mut PipelineData) {
        pipeline_data.add_data(self.data.take().unwrap());
    }
}

impl ControllerParam for Commands {
    type Item<'new> = Self;

    fn fetch<'r>(pipeline: &'r PipelineData) -> Option<Self::Item<'r>> {
        Some(Commands::from_queue(pipeline.command_queue.clone()))
    }
}

