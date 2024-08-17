use std::env;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const TEST_PREFIX: &str = "
#[cfg(test)]

use pretty_assertions::assert_eq;

// to let us use pretty_assertions with strings, we write a newtype
// that delegates Debug to Display
#[derive(PartialEq, Eq)]
struct StringWrapper<'a> {
  wrapped: &'a str,
}

impl<'a> core::fmt::Debug for StringWrapper<'a> {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    core::fmt::Display::fmt(self.wrapped, f)
  }
}

fn assert_eq(x: &str, y: &str) {
  assert_eq!(StringWrapper {wrapped: x}, StringWrapper {wrapped: y});
}

use crate::Value;
";

const TEST_TEMPLATE: &str = "
#[test]
fn test_%PREFIX%() {
  let source = include_str!(\"%ROOT%/test/expectations/%PREFIX%.adnot\");
  let expectation = include_str!(\"%ROOT%/test/expectations/%PREFIX%.adnot.exp\");
  let parsed = Value::parse_str(source);
  assert_eq(expectation, &format!(\"{:#?}\\n\", parsed));
}
";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR")?;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let dest = Path::new(&out_dir).join("exp_tests.rs");
    let mut test_file = File::create(&dest)?;
    writeln!(test_file, "{}", TEST_PREFIX)?;

    let exp_dir = Path::new(&manifest_dir).join("test").join("expectations");
    println!("got: {:?}", exp_dir);
    for exp in std::fs::read_dir(exp_dir)? {
        let exp = exp?;
        println!("cargo:rerun-if-changed={}", exp.path().display());
        let exp = exp.path().canonicalize()?;
        let fname = exp.file_name().ok_or("bad file name")?.to_string_lossy();
        let prefix = if let Some(p) = fname.strip_suffix(".adnot") {
            p
        } else {
            continue;
        };

        let test = TEST_TEMPLATE
            .replace("%PREFIX%", prefix)
            .replace("%ROOT%", &manifest_dir);
        writeln!(test_file, "{}", test)?;
    }
    Ok(())
}
