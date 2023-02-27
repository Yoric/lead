use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::{DateTime, Utc};
use clap::Parser;
use data::CompanyName;

mod data;
mod time;

#[derive(clap::Parser, Debug)]
#[command(author, version, about = "A tool for tracking job leads")]
struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(long, value_parser=time::parse_utc)]
    /// The date and time at which this happened. Defaults to now.
    on: Option<DateTime<Utc>>,

    /// The path in which to store the database.
    #[arg(long, default_value_t={dotenv::var("LEADS_ROOT").unwrap_or_else(|_| ".".to_string())})]
    path: String,

    /// A file name for the leads db, relative to `path`.
    #[arg(long, default_value_t={dotenv::var("LEADS_DB").unwrap_or_else(|_| "leads.yml".to_string())})]
    file: String,

    /// A file name for the archived leads db, relative to `path`.
    #[arg(long, default_value_t={dotenv::var("LEADS_ARCHIVE").unwrap_or_else(|_| "archive.yml".to_string())})]
    archive: String,
}

#[derive(clap::Args, Clone, Debug)]
struct LeadName {
    #[arg(long)]
    /// The name of the company.
    company: CompanyName,

    #[arg(long)]
    /// If there is more than one position at the company, the index of the position.
    index: Option<usize>,
}

#[derive(clap::Args, Clone, Debug)]
struct OptionalLeadName {
    #[arg(long)]
    /// The name of the company.
    company: Option<CompanyName>,

    #[arg(long)]
    /// If there is more than one position at the company, the index of the position.
    index: Option<usize>,
}

#[derive(clap::Subcommand, Debug)]
enum Command {
    /// Create a new lead.
    New {
        #[command(flatten)]
        lead: LeadName,

        /// The name of the position.
        #[arg(long)]
        position: String,

        /// The source for this lead, typically a URL.
        #[arg(long)]
        source: String,
    },

    /// Close a lead.
    Close {
        #[command(flatten)]
        lead: LeadName,

        #[arg(long)]
        reason: String,
    },

    /// Things the candidate needs to do.
    Todo {
        #[command(flatten)]
        lead: LeadName,

        #[command(subcommand)]
        command: TaskCommand
    },

    /// Things the candidate is waiting for.
    Wait {
        #[command(flatten)]
        lead: LeadName,

        #[command(subcommand)]
        command: TaskCommand,
    },

    /// Misc note.
    Note {
        #[command(flatten)]
        lead: LeadName,

        #[command(subcommand)]
        command: NoteCommand
    },

    /// Add a status update.
    Status {
        #[command(flatten)]
        lead: LeadName,

        status: String,
    },

    Show {
        #[command(flatten)]
        lead: OptionalLeadName,
    },

    #[command(hide = true)]
    SelfCheck,
}

#[derive(clap::Subcommand, Debug)]
enum NoteCommand {
    /// Create a new note.
    Add {
        name: String,
        note: String,
    },
    Replace {
        name: String,
        note: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum TaskCommand {
    Add {
        action: String,
        deadline: Option<DateTime<Utc>>,
    },
    Done {
        #[arg(default_value_t = 0)]
        index: usize,
    },
}

enum ShouldWrite {
    Commit,
    Discard,
}

impl Args {
    pub fn execute(
        self,
        db_archive_path: &Path,
        db: &mut data::Leads,
    ) -> Result<ShouldWrite, anyhow::Error> {
        use ShouldWrite::*;
        let updated_on = self.on.unwrap_or_else(Utc::now);
        match self.command {
            Command::SelfCheck => {
                println!("Self check passed");
                Ok(Discard)
            }
            Command::New {
                lead,
                position,
                source
            } => {
                if lead.index.is_some() {
                    return Err(anyhow::anyhow!(
                        "Cannot specify index when creating a new lead"
                    ));
                }
                let index = db.new_lead(lead.company, position, source);
                if index > 0 {
                    println!("Created lead {}", index);
                }
                Ok(Commit)
            }
            Command::Close {
                lead,
                reason
            } => {
                let details = db
                    .close_lead(updated_on, &lead.company, lead.index, reason)
                    .context("Failed to remove lead")?;
                let mut db_archive = data::Leads::from_path(db_archive_path)
                    .context("Failed to load or create archive")?;
                db_archive.push_lead(lead.company, details);
                serde_yaml::to_writer(std::fs::File::create(db_archive_path)?, &db_archive)
                    .context("Failed to write archive")?;
                Ok(Commit)
            }
            Command::Note {
                lead,
                command: NoteCommand::Add { name, note }
            } => {
                let details = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                details.add_note(name, note);
                Ok(Commit)
            }
            Command::Status {
                lead,
                status
            } => {
                let details = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                details.add_status(updated_on, status);
                Ok(Commit)
            }

            // Todos
            Command::Todo {
                lead,
                command: TaskCommand::Add {
                    action,
                    deadline
            }} => {
                let lead = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                let deadline = match deadline {
                    None => {
                        eprintln!("No deadline specified, defaulting to 7 days from now");
                        Utc::now() + chrono::Duration::days(7)
                    }
                    Some(d) => d,
                };
                lead.add_todo(
                    updated_on,
                    action,
                    deadline,
                );
                Ok(Commit)
            }
            Command::Todo {
                lead,
                command: TaskCommand::Done { index }
             } => {
                let details = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                details.complete_todo(
                    updated_on,
                    index,
                )?;
                Ok(Commit)
            }

            // Waits
            Command::Wait {
                lead,
                command: TaskCommand::Add { action, deadline } } => {
                let details = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                details.add_wait(
                    updated_on,
                    action,
                    deadline,
                );
                Ok(Commit)
            }
            Command::Wait {
                lead,
                command: TaskCommand::Done { index }
            } => {
                let details = db
                    .get_mut(&lead.company, lead.index)
                    .context("Failed to get lead")?;
                details.complete_wait(
                    updated_on,
                    index,
                )?;
                Ok(Commit)
            }

            Command::Show {
                lead: OptionalLeadName { company: None, .. }
            } => {
                println!("Active leads:");
                for (company, _) in db {
                    println!("* {company}");
                }
                Ok(Discard)
            }

            Command::Show {
                lead: OptionalLeadName { company: Some(lead), index }
            } => {
                let position = db.get(&lead, index)?;
                serde_yaml::to_writer(std::io::stdout(), &position)?;
                Ok(Discard)
            }

            _ => unimplemented!()
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let args = Args::parse();


    // Load db.
    let mut db_path = PathBuf::from(&args.path);
    db_path.push(&args.file);
    let db_path = db_path;

    let mut db_archive_path = PathBuf::from(&args.path);
    db_archive_path.push(&args.archive);
    let db_archive_path = db_archive_path;

    let mut db = data::Leads::from_path(&db_path)?;

    // Execute command.
    args.execute(&db_archive_path, &mut db)?;

    // Write back to disk.
    serde_yaml::to_writer(std::fs::File::create(&db_path)?, &db)?;
    Ok(())
}
