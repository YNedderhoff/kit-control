use clap::{Parser, Subcommand, ValueEnum};
use fs2::FileExt;
use kitcontrol_core::{
    Request, Response, dispatch, dispatch_json,
    error::{Error, Result},
    model::{Document, Snapshot},
    storage,
    transfer::{ChangePlan, Fault},
};
use serde_json::{Value, json};
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "kitcontrol",
    version,
    about = "Kit editor with native TD-27 MIDI and a parameter emulator."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List native input/output port names without sending MIDI messages.
    Ports,
    /// Read, plan and apply documented TD-27 parameters.
    Td27 {
        #[command(subcommand)]
        command: Td27Command,
    },
    /// List the offline workspace simulator.
    Devices,
    /// Create a synthetic workspace. Optionally save an initial device snapshot.
    Demo {
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        device: Option<PathBuf>,
    },
    /// Read a workspace as a complete document.
    Inspect {
        workspace: PathBuf,
    },
    /// Validate a workspace and its profile.
    Validate {
        workspace: PathBuf,
    },
    /// Apply one JSON EditOperation locally; this never changes a device.
    Edit {
        workspace: PathBuf,
        #[arg(long)]
        operation: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Copy one pad's complete synthetic sound settings.
    CopyPad {
        workspace: PathBuf,
        #[arg(long)]
        source_kit: String,
        #[arg(long)]
        source_pad: String,
        #[arg(long)]
        target_kit: String,
        #[arg(long)]
        target_pad: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Copy an entire synthetic kit, preserving effects, layers and opaque data.
    CopyKit {
        workspace: PathBuf,
        #[arg(long)]
        source_kit: String,
        #[arg(long)]
        target_slot: usize,
        #[arg(long)]
        new_kit_id: String,
        #[arg(long)]
        output: PathBuf,
    },
    /// Reorder occupied kit IDs and retain setlist targets.
    Reorder {
        workspace: PathBuf,
        #[arg(long, value_delimiter = ',')]
        order: Vec<String>,
        #[arg(long)]
        output: PathBuf,
    },
    Diff {
        before: PathBuf,
        after: PathBuf,
    },
    /// Create a plan from an explicit device snapshot and desired workspace.
    Plan {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        workspace: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Apply an explicit plan to a synthetic device state file and read it back.
    Apply {
        #[arg(long)]
        device: PathBuf,
        #[arg(long)]
        plan: PathBuf,
        #[arg(long, value_enum, default_value_t = InjectFault::None)]
        fault: InjectFault,
    },
    Verify {
        #[arg(long)]
        device: PathBuf,
        #[arg(long)]
        workspace: PathBuf,
    },
    /// Finish an interrupted workspace save from its transaction journal.
    Recover {
        workspace: PathBuf,
    },
    /// Execute the same versioned JSON request used by the desktop app.
    Request {
        /// JSON request file, or '-' for standard input.
        file: PathBuf,
    },
}

#[derive(Subcommand)]
enum Td27Command {
    /// Read an explicit MIDI target (JSON target file), saving a scoped snapshot.
    Read {
        #[arg(long)]
        target: PathBuf,
        #[arg(long)]
        kit: u8,
        #[arg(long)]
        output: PathBuf,
    },
    /// Identify an explicit target. This does not read kit parameters.
    Identify {
        #[arg(long)]
        target: PathBuf,
    },
    /// Create an immutable plan from a snapshot and a JSON parameter/value map.
    Plan {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        changes: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Apply a specific plan after a fresh read and verify each parameter.
    Apply {
        #[arg(long)]
        target: PathBuf,
        #[arg(long)]
        plan: PathBuf,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum InjectFault {
    None,
    Disconnect,
    ReadbackMismatch,
}

fn to_value<T: serde::Serialize>(value: T) -> Result<Value> {
    Ok(serde_json::to_value(value)?)
}
fn require(response: Response) -> Result<Value> {
    response.result.ok_or_else(|| {
        response
            .error
            .unwrap_or_else(|| Error::new("internal_error", "Missing dispatcher result"))
    })
}
fn load(path: &Path) -> Result<Document> {
    storage::load(path)
}
fn path_text(path: &Path) -> Result<String> {
    path.to_str()
        .map(String::from)
        .ok_or_else(|| Error::new("invalid_path", "Path must be UTF-8"))
}

fn save_edit(workspace: &Path, output: &Path, operation: Value) -> Result<Value> {
    let response = dispatch_json(&serde_json::to_string(
        &json!({"action":"edit","document":load(workspace)?,"operation":operation}),
    )?);
    let document: Document = serde_json::from_value(require(response)?)?;
    require(dispatch(Request::Save {
        path: path_text(output)?,
        document,
    }))
}

fn execute(command: Command) -> Result<Response> {
    let value = match command {
        Command::Request { file } => {
            let input = if file == Path::new("-") {
                let mut text = String::new();
                io::stdin()
                    .take(16 * 1024 * 1024 + 1)
                    .read_to_string(&mut text)?;
                if text.len() > 16 * 1024 * 1024 {
                    return Err(Error::new("file_too_large", "Request exceeds 16 MiB"));
                }
                text
            } else {
                serde_json::to_string(&storage::read_json::<Value>(&file)?)?
            };
            return Ok(dispatch_json(&input));
        }
        Command::Ports => return Ok(dispatch(Request::MidiPorts {})),
        Command::Td27 { command } => return execute_td27(command),
        Command::Devices => return Ok(dispatch(Request::Devices {})),
        Command::Demo { output, device } => {
            let result = require(dispatch(Request::Demo {}))?;
            let document: Document = serde_json::from_value(result["document"].clone())?;
            let files = storage::save(&output, &document)?;
            if let Some(device) = device {
                storage::create_json(&device, &result["snapshot"])?;
            }
            json!({"path":output,"files":files,"profile_id":"kitcontrol.synthetic.v1"})
        }
        Command::Inspect { workspace } => to_value(load(&workspace)?)?,
        Command::Validate { workspace } => {
            return Ok(dispatch(Request::Validate {
                document: load(&workspace)?,
            }));
        }
        Command::Edit {
            workspace,
            operation,
            output,
        } => save_edit(&workspace, &output, storage::read_json(&operation)?)?,
        Command::CopyPad {
            workspace,
            source_kit,
            source_pad,
            target_kit,
            target_pad,
            output,
        } => save_edit(
            &workspace,
            &output,
            json!({"kind":"copy_pad","source_kit_id":source_kit,"source_pad":source_pad,"target_kit_id":target_kit,"target_pad":target_pad}),
        )?,
        Command::CopyKit {
            workspace,
            source_kit,
            target_slot,
            new_kit_id,
            output,
        } => save_edit(
            &workspace,
            &output,
            json!({"kind":"copy_kit","source_kit_id":source_kit,"target_slot":target_slot,"new_kit_id":new_kit_id}),
        )?,
        Command::Reorder {
            workspace,
            order,
            output,
        } => save_edit(
            &workspace,
            &output,
            json!({"kind":"reorder_kits","order":order}),
        )?,
        Command::Diff { before, after } => {
            return Ok(dispatch(Request::Diff {
                before: load(&before)?,
                after: load(&after)?,
            }));
        }
        Command::Plan {
            baseline,
            workspace,
            output,
        } => {
            let value = require(dispatch(Request::Plan {
                baseline: storage::read_json(&baseline)?,
                desired: load(&workspace)?,
            }))?;
            storage::create_json(&output, &value)?;
            value
        }
        Command::Apply {
            device,
            plan,
            fault,
        } => {
            let lock_path = device.with_extension("kitcontrol-lock");
            if std::fs::symlink_metadata(&lock_path).is_ok_and(|m| m.file_type().is_symlink()) {
                return Err(Error::new(
                    "unsafe_path",
                    "Device lock cannot be a symbolic link",
                ));
            }
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(lock_path)?;
            lock.try_lock_exclusive()
                .map_err(|_| Error::new("workspace_busy", "The simulated device is in use"))?;
            let snapshot: Snapshot = storage::read_json(&device)?;
            let plan: ChangePlan = storage::read_json(&plan)?;
            let fault = match fault {
                InjectFault::None => Fault::None,
                InjectFault::Disconnect => Fault::Disconnect,
                InjectFault::ReadbackMismatch => Fault::ReadbackMismatch,
            };
            let result = dispatch(Request::SimulateApply {
                device: snapshot,
                plan: Box::new(plan),
                fault,
            });
            if let Some(report) = &result.result
                && matches!(report["status"].as_str(), Some("verified" | "partial"))
            {
                storage::write_json(&device, &report["snapshot"])?;
            }
            return Ok(result);
        }
        Command::Verify { device, workspace } => {
            return Ok(dispatch(Request::Verify {
                device: storage::read_json(&device)?,
                expected: load(&workspace)?,
            }));
        }
        Command::Recover { workspace } => {
            return Ok(dispatch(Request::Recover {
                path: path_text(&workspace)?,
            }));
        }
    };
    Ok(Response {
        api_version: 1,
        ok: true,
        result: Some(value),
        error: None,
    })
}

fn execute_td27(command: Td27Command) -> Result<Response> {
    match command {
        Td27Command::Identify { target } => Ok(dispatch(Request::Td27Identify {
            target: storage::read_json(&target)?,
        })),
        Td27Command::Read {
            target,
            kit,
            output,
        } => {
            if output.exists() {
                return Err(Error::new(
                    "destination_exists",
                    "Snapshot destination already exists",
                ));
            }
            let response = dispatch(Request::Td27Read {
                target: storage::read_json(&target)?,
                kit,
            });
            if response.ok {
                storage::create_json(
                    &output,
                    response
                        .result
                        .as_ref()
                        .ok_or_else(|| Error::new("internal_error", "Missing snapshot"))?,
                )?;
            }
            Ok(response)
        }
        Td27Command::Plan {
            baseline,
            changes,
            output,
        } => {
            let response = dispatch(Request::Td27Plan {
                baseline: storage::read_json(&baseline)?,
                changes: storage::read_json(&changes)?,
            });
            if response.ok {
                storage::create_json(
                    &output,
                    response
                        .result
                        .as_ref()
                        .ok_or_else(|| Error::new("internal_error", "Missing plan"))?,
                )?;
            }
            Ok(response)
        }
        Td27Command::Apply { target, plan } => Ok(dispatch(Request::Td27Apply {
            target: storage::read_json(&target)?,
            plan: Box::new(storage::read_json(&plan)?),
        })),
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let response = execute(cli.command).unwrap_or_else(|error| Response {
        api_version: 1,
        ok: false,
        result: None,
        error: Some(error),
    });
    let exit_code = response.exit_code();
    if let Some(error) = &response.error {
        let _ = writeln!(io::stderr(), "{}: {}", error.code, error.message);
    }
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if serde_json::to_writer_pretty(&mut output, &response).is_err() || writeln!(output).is_err() {
        return ExitCode::from(3);
    }
    ExitCode::from(exit_code as u8)
}
