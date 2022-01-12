use std::{env, fs::File, io::Read, path::PathBuf};

use memmap::Mmap;
use pinot::{FontRef, TableProvider};

static HELP: &str = "
print information about the first 100 glyphs in the font.

USAGE:
    mmap_test PATH [--mmap]

If --mmap is passed, we will use mmap to read the font, otherwise
we will read it into memory.
";


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::get_from_env_or_exit();
    let mut file = File::open(&args.path)?;
    if args.mmap {
        let mmap = unsafe { Mmap::map(&file).expect("failed to map the file") };
        read_glyphs(&mmap);
    } else {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        read_glyphs(&bytes);
    }
    Ok(())
}

fn read_glyphs(font_bytes: &[u8]) {
    let font = FontRef::from_index(font_bytes, 0).expect("could not load font");
    if let Some(gsub) = font.gsub() {
        println!("GSUB contains {} lookups", gsub.num_lookups());
    }
    if let Some(gpos) = font.gpos() {
        println!("GPOS contains {} lookups", gpos.num_lookups());
    }
}

macro_rules! exit_err {
    ($($arg:tt)*) => ({
        eprintln!($($arg)*);
        eprintln!("{}", HELP);
        std::process::exit(1);
    })
}

struct Args {
    path: PathBuf,
    mmap: bool,
}

impl Args {
    fn get_from_env_or_exit() -> Self {
        let mut args = env::args().skip(1);
        let path = match args.next().map(PathBuf::from) {
            Some(ref p) if p.exists()  => p.to_owned(),
            Some(ref p) => exit_err!("path {:?} does not exist", p),
            None => exit_err!("path is required"),
        };

        let mmap = match args.next().as_deref() {
            Some("--mmap") => true,
            Some(thing) => exit_err!("unexpected argument '{}'", thing),
            None => false,
        };

        Args { path, mmap }
    }
}
