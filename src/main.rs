use derivative::Derivative;
use git2::Repository;
use std::env;
use structopt::StructOpt;
use termion::color;

#[derive(StructOpt, Debug)]
struct Opts {
    // The name of the branch to compare against
    #[structopt(long)]
    origin_branch: Option<String>,

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

#[derive(Derivative)]
#[derivative(Debug)]
struct Context {
    #[derivative(Debug = "ignore")]
    repo: Repository,
    origin_branch: String,
}

fn main() {
    let opts = Opts::from_args();

    let path = env::current_dir().expect("Cannot find current directory. Giving up.");
    let repo = match Repository::open(&path) {
        Ok(repo) => repo,
        Err(e) => panic!("No git repo found at {:?}: {}", path, e),
    };

    let origin_branch = opts
        .origin_branch
        .unwrap_or_else(|| guess_origin_branch(&repo).expect("Cannot find reference branch"));
    let ctx = Context {
        repo,
        origin_branch,
    };

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

    print_status(&ctx);

    // Prepare the revwalk
    let mut revwalk = ctx.repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::NONE | git2::Sort::TIME)?;

    let start_revspec = ctx.repo.revparse_single(&ctx.origin_branch)?;
    revwalk.hide(start_revspec.id())?;
    revwalk.push(
        ctx.repo
            .head()?
            .resolve()?
            .target()
            .expect("Resolved target should always get a valid Oid"),
    )?;

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

fn guess_origin_branch(repo: &Repository) -> Result<String, git2::Error> {
    for remote in &["origin", "upstream"] {
        let reference = repo.find_reference(&format!("refs/remotes/{}/HEAD", remote));
        if let Ok(reference) = reference {
            return Ok(reference
                .shorthand()
                .expect("Cannot get a valid shorthand")
                .into());
        }
    }

    for head in &["master", "main"] {
        if repo.find_reference(&format!("refs/heads/{}", head)).is_ok() {
            return Ok(head.to_string());
        }
    }

    // TODO find a better way to give up
    Ok("master".into())
}

fn print_status(ctx: &Context) {
    let head = match ctx.repo.head() {
        Ok(head) => Some(head),
        Err(ref e)
            if e.code() == git2::ErrorCode::UnbornBranch
                || e.code() == git2::ErrorCode::NotFound =>
        {
            None
        }
        Err(_) => {
            return;
        }
    };
    let head = head.as_ref().and_then(|h| h.shorthand());
    println!("On {}", head.unwrap_or("HEAD"));
}
