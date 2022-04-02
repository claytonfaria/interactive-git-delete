use chrono::{Duration, NaiveDateTime};
use dialoguer::{theme::ColorfulTheme, Select};
use git2::{BranchType, Repository};

fn main() -> Result<()> {
    let repo = Repository::open_from_env()?;

    let branches = get_local_branches(&repo)?;

    let branch_names = &branches
        .iter()
        .map(|branch| &branch.name)
        .collect::<Vec<_>>();

    if branch_names.is_empty() {
        println!("No local branches found");
    } else {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select branch")
            .default(0)
            .items(branch_names)
            .interact()
            .unwrap();

        println!("You selected {}!", branch_names[selection]);

        println!("{:?}", branches[selection]);
    }

    Ok(())
}

fn get_local_branches(repo: &Repository) -> Result<Vec<Branch>> {
    let mut branches = Vec::new();

    for branch in repo.branches(Some(BranchType::Local))? {
        let (branch, _) = branch?;
        let branch_name = branch.name_bytes()?;

        let commit = branch.get().peel_to_commit()?;

        let time = commit.time();
        let offset = Duration::minutes(i64::from(time.offset_minutes()));
        let time = NaiveDateTime::from_timestamp(time.seconds(), 0) + offset;

        let branch = Branch {
            name: String::from_utf8_lossy(branch_name).to_string(),
            time,
            id: commit.id(),
            branch,
        };

        branches.push(branch);
    }

    Ok(branches)
}

struct Branch<'repo> {
    time: NaiveDateTime,
    id: git2::Oid,
    name: String,
    branch: git2::Branch<'repo>,
}

impl std::fmt::Debug for Branch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Branch {{ name: {}, time: {}, id: {} }}",
            self.name, self.time, self.id
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
}
