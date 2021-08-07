use git2::Repository;
use std::env;
use structopt::StructOpt;
use termion::color;

#[derive(StructOpt, Debug)]
struct Opts {
    // The name of the remote
    #[structopt(long)]
    remote: Option<String>,

    // The name of the default branch (e.g. main or master)
    #[structopt(long)]
    default_branch: Option<String>,

    #[structopt(subcommand)]
    // Note that we mark a field as a subcommand
    cmd: Option<Command>,
}

#[derive(StructOpt, Debug)]
enum Command {
    Show,
    Prev,
    Next,
}

struct Context {
    repo: Repository,
}

fn main() {
    let opts = Opts::from_args();

    let path = env::current_dir().expect("Cannot find current directory. Giving up.");
    let repo = match Repository::open(&path) {
        Ok(repo) => repo,
        Err(e) => panic!("No git repo found at {:?}: {}", path, e),
    };

    let ctx = Context { repo };

    match opts.cmd.unwrap_or(Command::Show) {
        Command::Show => cmd_show(ctx).unwrap(),
        _ => println!("Not implemented"),
    }
}

fn cmd_show(ctx: Context) -> Result<(), git2::Error> {
    println!(
        "{}State:{}   {:?}",
        color::Fg(color::Green),
        color::Fg(color::Reset),
        ctx.repo.state(),
    );

    let remotes: Vec<String> = ctx
        .repo
        .remotes()
        .unwrap()
        .iter()
        .map(|r| r.unwrap().to_string())
        .collect();
    println!(
        "{}Remotes:{} {}",
        color::Fg(color::Green),
        color::Fg(color::Reset),
        remotes.join(",")
    );

    // Prepare the revwalk
    let mut revwalk = ctx.repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::NONE | git2::Sort::TIME)?;
    revwalk.push_head()?;

    // lookup commits from Oids
    let revwalk = revwalk.filter_map(|id| {
        let id = if let Ok(id) = id { id } else { return None };
        let commit = ctx.repo.find_commit(id).map_err(Some);
        Some(commit)
    });

    // print!
    for commit in revwalk {
        if let Ok(commit) = commit {
            println!(
                "{}{:}{} {}",
                color::Fg(color::Yellow),
                &commit.id().to_string()[..7],
                color::Fg(color::Reset),
                String::from_utf8_lossy(commit.message_bytes())
                    .lines()
                    .next()
                    .unwrap()
            );
        } else {
            println!("{:?}", commit);
        }
    }

    Ok(())
}
