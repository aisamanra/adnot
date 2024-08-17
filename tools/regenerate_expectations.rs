use adnot::Value;

use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for exp in std::fs::read_dir("test/expectations")? {
        let exp = exp?.path().canonicalize()?;
        let original = exp.clone();
        let fname = exp.file_name().ok_or("bad file name")?.to_string_lossy();
        if let Some(prefix) = fname.strip_suffix(".adnot") {
            println!("regenerating {}.adnot.exp", prefix);
            let contents = std::fs::read_to_string(original)?;
            let value = Value::parse_str(&contents);

            let mut f = std::fs::File::create(format!("test/expectations/{}.exp", fname))?;
            writeln!(f, "{:#?}", value)?;
        }
    }

    Ok(())
}
