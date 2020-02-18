//! The UI functionality for the task formats.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::SystemTime;

use failure::Error;
use serde::{Deserialize, Serialize};

pub use json::JsonUI;
pub use print::PrintUI;
pub use raw::RawUI;
pub use silent::SilentUI;
use task_maker_dag::{ExecutionResult, ExecutionStatus, WorkerUuid};
use task_maker_exec::ExecutorStatus;

use crate::ioi::{SubtaskId, TestcaseId};
use crate::terry::{Seed, SolutionOutcome};
use crate::{ioi, terry};

mod json;
mod print;
mod raw;
mod silent;

/// Channel type for sending `UIMessage`s.
pub type UIChannelSender = Sender<UIMessage>;
/// Channel type for receiving `UIMessage`s.
pub type UIChannelReceiver = Receiver<UIMessage>;

/// The status of an execution.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UIExecutionStatus {
    /// The `Execution` is known to the DAG and when all its dependencies are ready it will
    /// started.
    Pending,
    /// The `Execution` has been started on a worker.
    Started {
        /// The UUID of the worker.
        worker: WorkerUuid,
    },
    /// The `Execution` has been completed.
    Done {
        /// The result of the execution.
        result: ExecutionResult,
    },
    /// At least one of its dependencies have failed, the `Execution` has been skipped.
    Skipped,
}

/// A message sent to the UI.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UIMessage {
    /// A message asking the UI to exit.
    StopUI,

    /// An update on the status of the executor.
    ServerStatus {
        /// The status of the executor.
        status: ExecutorStatus<SystemTime>,
    },

    /// An update on the compilation status.
    Compilation {
        /// The compilation of this file.
        file: PathBuf,
        /// The status of the compilation.
        status: UIExecutionStatus,
    },

    /// An update on the stdout of a compilation.
    CompilationStdout {
        /// The compilation of this file.
        file: PathBuf,
        /// The prefix of the stdout of the compilation.
        content: String,
    },

    /// An update on the stderr of a compilation.
    CompilationStderr {
        /// The compilation of this file.
        file: PathBuf,
        /// The prefix of the stderr of the compilation.
        content: String,
    },

    /// The information about the task which is being run.
    IOITask {
        /// The task information.
        task: Box<ioi::Task>,
    },

    /// The generation of a testcase in a IOI task.
    IOIGeneration {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The status of the generation.
        status: UIExecutionStatus,
    },

    /// An update on the stderr of the generation of a testcase.
    IOIGenerationStderr {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The prefix of the stderr of the generation.
        content: String,
    },

    /// The validation of a testcase in a IOI task.
    IOIValidation {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The status of the validation.
        status: UIExecutionStatus,
    },

    /// An update on the stderr of the validation of a testcase.
    IOIValidationStderr {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The prefix of the stderr of the validator.
        content: String,
    },

    /// The solution of a testcase in a IOI task.
    IOISolution {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The status of the solution.
        status: UIExecutionStatus,
    },

    /// The evaluation of a solution in a IOI task.
    IOIEvaluation {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The path of the solution.
        solution: PathBuf,
        /// The status of the solution.
        status: UIExecutionStatus,
    },

    /// The checking of a solution in a IOI task.
    IOIChecker {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The path of the solution.
        solution: PathBuf,
        /// The status of the solution. Note that a failure of this execution
        /// may not mean that the checker failed.
        status: UIExecutionStatus,
    },

    /// The score of a testcase is ready.
    IOITestcaseScore {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The id of the testcase.
        testcase: TestcaseId,
        /// The path of the solution.
        solution: PathBuf,
        /// The score of the testcase.
        score: f64,
        /// The message associated with the score.
        message: String,
    },

    /// The score of a subtask is ready.
    IOISubtaskScore {
        /// The id of the subtask.
        subtask: SubtaskId,
        /// The path of the solution.
        solution: PathBuf,
        /// The normalized score, a value between 0 and 1
        normalized_score: f64,
        /// The score of the subtask.
        score: f64,
    },

    /// The score of a task is ready.
    IOITaskScore {
        /// The path of the solution.
        solution: PathBuf,
        /// The score of the task.
        score: f64,
    },

    /// The compilation of a booklet.
    IOIBooklet {
        /// The name of the booklet.
        name: String,
        /// The status of the compilation.
        status: UIExecutionStatus,
    },

    /// The compilation of a dependency of a booklet. It can be processed many times, for example an
    /// asy file is compiled first, and then cropped.
    IOIBookletDependency {
        /// The name of the booklet.
        booklet: String,
        /// The name of the dependency.
        name: String,
        /// The index (0-based) of the step of this compilation.
        step: usize,
        /// The number of steps of the compilation of this dependency.
        num_steps: usize,
        /// The status of this step.
        status: UIExecutionStatus,
    },

    /// The information about the task which is being run.
    TerryTask {
        /// The task information.
        task: Box<terry::Task>,
    },

    /// The generation of a testcase in a Terry task.
    TerryGeneration {
        /// The path of the solution.
        solution: PathBuf,
        /// The seed used to generate the input file.
        seed: Seed,
        /// The status of the generation.
        status: UIExecutionStatus,
    },

    /// The validation of a testcase in a Terry task.
    TerryValidation {
        /// The path of the solution.
        solution: PathBuf,
        /// The status of the validation.
        status: UIExecutionStatus,
    },

    /// The solution of a testcase in a Terry task.
    TerrySolution {
        /// The path of the solution.
        solution: PathBuf,
        /// The status of the solution.
        status: UIExecutionStatus,
    },

    /// The checking of a solution in a Terry task.
    TerryChecker {
        /// The path of the solution.
        solution: PathBuf,
        /// The status of the checker.
        status: UIExecutionStatus,
    },

    /// The outcome of a solution in a Terry task.
    TerrySolutionOutcome {
        /// The path of the solution.
        solution: PathBuf,
        /// The outcome of the solution. `Err` is caused by an invalid response from the checker.
        outcome: Result<SolutionOutcome, String>,
    },

    /// A warning has been emitted.
    Warning {
        /// The message of the warning.
        message: String,
    },
}

/// The status of the compilation of a file.
#[derive(Debug, Clone, PartialEq)]
pub enum CompilationStatus {
    /// The compilation is known but it has not started yet.
    Pending,
    /// The compilation is running on a worker.
    Running,
    /// The compilation has completed.
    Done {
        /// The result of the compilation.
        result: ExecutionResult,
        /// The standard output of the compilation.
        stdout: Option<String>,
        /// The standard error of the compilation.
        stderr: Option<String>,
    },
    /// The compilation has failed.
    Failed {
        /// The result of the compilation.
        result: ExecutionResult,
        /// The standard output of the compilation.
        stdout: Option<String>,
        /// The standard error of the compilation.
        stderr: Option<String>,
    },
    /// The compilation has been skipped.
    Skipped,
}

impl CompilationStatus {
    /// Apply to this `CompilationStatus` a new `UIExecutionStatus`.
    pub fn apply_status(&mut self, status: UIExecutionStatus) {
        match status {
            UIExecutionStatus::Pending => *self = CompilationStatus::Pending,
            UIExecutionStatus::Started { .. } => *self = CompilationStatus::Running,
            UIExecutionStatus::Done { result } => {
                if let ExecutionStatus::Success = result.status {
                    *self = CompilationStatus::Done {
                        result,
                        stdout: None,
                        stderr: None,
                    };
                } else {
                    *self = CompilationStatus::Failed {
                        result,
                        stdout: None,
                        stderr: None,
                    };
                }
            }
            UIExecutionStatus::Skipped => *self = CompilationStatus::Skipped,
        }
    }

    /// Set the standard output of the compilation.
    pub fn apply_stdout(&mut self, content: String) {
        // FIXME: if the stdout is sent before the status of the execution this breaks
        match self {
            CompilationStatus::Done { stdout, .. } | CompilationStatus::Failed { stdout, .. } => {
                stdout.replace(content);
            }
            _ => {}
        }
    }

    /// Set the standard error of the compilation.
    pub fn apply_stderr(&mut self, content: String) {
        match self {
            CompilationStatus::Done { stderr, .. } | CompilationStatus::Failed { stderr, .. } => {
                stderr.replace(content);
            }
            _ => {}
        }
    }
}

/// The sender of the UIMessage
pub struct UIMessageSender {
    sender: UIChannelSender,
}

impl UIMessageSender {
    /// Make a new pair of UIMessageSender and ChannelReceiver.
    pub fn new() -> (UIMessageSender, UIChannelReceiver) {
        let (sender, receiver) = channel();
        (UIMessageSender { sender }, receiver)
    }

    /// Send a message to the channel.
    pub fn send(&self, message: UIMessage) -> Result<(), Error> {
        self.sender.send(message).map_err(|e| e.into())
    }
}

/// The trait that describes the UI functionalities.
pub trait UI: Send {
    /// Process a new UI message.
    fn on_message(&mut self, message: UIMessage);
    /// Make the UI print the ending results.
    fn finish(&mut self);
}

/// The type of the UI to use, it enumerates all the known UI interfaces.
#[derive(Debug)]
pub enum UIType {
    /// The `PrintUI`.
    Print,
    /// The `RawUI`.
    Raw,
    /// The `CursesUI`.
    Curses,
    /// The `JsonUI`.
    Json,
    /// The `SilentUI`.
    Silent,
}

impl std::str::FromStr for UIType {
    type Err = String;

    fn from_str(s: &str) -> Result<UIType, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "print" => Ok(UIType::Print),
            "raw" => Ok(UIType::Raw),
            "curses" => Ok(UIType::Curses),
            "json" => Ok(UIType::Json),
            "silent" => Ok(UIType::Silent),
            _ => Err(format!("Unknown ui: {}", s)),
        }
    }
}

/// Write to `$self.stream`, in the color specified as second parameter. The arguments that follow
/// will be passed to `write!`.
///
/// ```
/// #[macro_use]
/// extern crate task_maker_format;
///
/// use termcolor::{StandardStream, ColorSpec, ColorChoice};
/// use task_maker_format::cwrite;
///
/// # fn main() {
/// struct Printer { stream: StandardStream }
/// let mut color = ColorSpec::new();
/// color.set_bold(true);
///
/// let mut printer = Printer { stream: StandardStream::stdout(ColorChoice::Auto) };
/// cwrite!(printer, color, "The output is {}", 42);
/// # }
/// ```
#[macro_export]
macro_rules! cwrite {
    ($self:expr, $color:expr, $($arg:tt)*) => {{
        use termcolor::WriteColor;
        use std::io::Write;
        $self.stream.set_color(&$color).unwrap();
        write!(&mut $self.stream, $($arg)*).unwrap();
        $self.stream.reset().unwrap();
    }};
}

/// Write to `$self.stream`, in the color specified as second parameter. The arguments that follow
/// will be passed to `writeln!`.
///
/// ```
/// #[macro_use]
/// extern crate task_maker_format;
///
/// use termcolor::{StandardStream, ColorSpec, ColorChoice};
/// use task_maker_format::cwriteln;
///
/// # fn main() {
/// struct Printer { stream: StandardStream }
/// let mut color = ColorSpec::new();
/// color.set_bold(true);
///
/// let mut printer = Printer { stream: StandardStream::stdout(ColorChoice::Auto) };
/// cwriteln!(printer, color, "The output is {}", 42);
/// # }
/// ```
#[macro_export]
macro_rules! cwriteln {
    ($self:expr, $color:expr, $($arg:tt)*) => {{
        use termcolor::WriteColor;
        use std::io::Write;
        $self.stream.set_color(&$color).unwrap();
        writeln!(&mut $self.stream, $($arg)*).unwrap();
        $self.stream.reset().unwrap();
    }};
}
