use chrono::{Duration, NaiveDateTime};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use git2::{BranchType, Repository};

use console::style;

fn main() -> Result<()> {
    let repo = Repository::open_from_env()?;

    loop {
        let local_branches = get_branches(&repo, BranchType::Local)?;

        let branch_names = get_branch_names(&local_branches);

        if branch_names.is_empty() {
            println!("No local branches found");
        } else {
            let selected_branch = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Select a branch (Press 'Esc or q' to exit):")
                .default(0)
                .items(&branch_names)
                .interact_opt()?;

            match selected_branch {
                Some(branch_index) => {
                    let branch_info = &local_branches[branch_index];

                    if branch_info.name == "master" || branch_info.is_head() {
                        println!("Cannot delete master or current branch\n");
                        continue;
                    }

                    println!(
                        "Last commit: {} - {} - {}",
                        &branch_info.last_commit.id.to_string()[..7],
                        branch_info.last_commit.time,
                        branch_info.last_commit.message
                    );
                    let delete_branch_confirmation = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!(
                            "Do you want to delete branch {} ?",
                            style(&branch_info.name).cyan()
                        ))
                        .default(false)
                        .show_default(true)
                        .wait_for_newline(true)
                        .interact()?;

                    match delete_branch_confirmation {
                        true => {
                            let mut branch_to_delete =
                                repo.find_branch(&branch_info.name, BranchType::Local)?;

                            branch_to_delete.delete()?;

                            println!(
                                "Branch {} deleted.\n\rTo undo this action, run: git checkout -b {} {}",
                                style(&branch_info.name).cyan(), branch_info.name, branch_info.last_commit.id
                            );
                        }
                        false => {
                            println!("Branch {} not deleted", style(&branch_info.name).cyan());
                        }
                    }
                }
                None => {
                    println!("No branch selected, exiting");
                    break;
                }
            }
        }
    }

    Ok(())
}

fn get_branches(repo: &Repository, branch_type: BranchType) -> Result<Vec<Branch>> {
    let mut branches = Vec::new();

    for branch in repo.branches(Some(branch_type))? {
        let (branch, _) = branch?;
        let branch_name = branch.name_bytes()?;

        let commit_raw = branch.get().peel_to_commit()?;

        let last_commit_time = commit_raw.time();
        let offset = Duration::minutes(i64::from(last_commit_time.offset_minutes()));
        let last_commit_time =
            NaiveDateTime::from_timestamp(last_commit_time.seconds(), 0) + offset;

        let last_commit_message = commit_raw.message_bytes();

        let last_commit = Commit {
            id: commit_raw.id(),
            message: String::from_utf8(last_commit_message.to_vec())?,
            time: last_commit_time,
        };

        let branch = Branch {
            name: String::from_utf8(branch_name.to_vec())?,
            last_commit,
            branch,
        };

        branches.push(branch);
    }

    branches.sort_by_key(|branch| branch.name.clone());

    Ok(branches)
}

fn get_branch_names<'a>(branches: &'a Vec<Branch>) -> Vec<String> {
    let mut out_branches = vec![];

    for branch in branches {
        if branch.is_head() {
            out_branches.push(format!("* {}", style(&branch.name).green()));
        } else if branch.name == "master" {
            out_branches.push(format!("{}", style(&branch.name).green()));
        } else {
            out_branches.push(branch.name.clone());
        }
    }

    out_branches
}

struct Commit {
    id: git2::Oid,
    message: String,
    time: NaiveDateTime,
}
struct Branch<'repo> {
    name: String,
    last_commit: Commit,
    branch: git2::Branch<'repo>,
}

impl<'repo> Branch<'repo> {
    fn is_head(&self) -> bool {
        self.branch.is_head()
    }
}

impl std::fmt::Debug for Branch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Branch {{ name: {}, time: {}, id: {} }}",
            self.name, self.last_commit.time, self.last_commit.id
        )
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    GitError(#[from] git2::Error),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
