use std::env;
use std::fs;
use std::io::{Error, Write};
use std::iter::Iterator;
use std::path::Path;
use std::result::Result;

mod arithmetic;
mod memory_access;
use arithmetic::Arithmetic;
use memory_access::MemoryAccess;

#[derive(Debug)]
enum Command {
  Arithmetic(Arithmetic),
  MemoryAccess(MemoryAccess),
  Noop,
}

impl Command {
  fn to_asm(&self) -> String {
    match self {
      Self::Arithmetic(command) => command.to_asm(),
      Self::MemoryAccess(command) => command.to_asm(),
      Self::Noop => "".to_string(),
    }
  }

  fn arithmetic(command: &str, i: usize) -> Result<Self, String> {
    match command {
      "add" => Ok(Self::Arithmetic(Arithmetic::Add)),
      "sub" => Ok(Self::Arithmetic(Arithmetic::Sub)),
      "neg" => Ok(Self::Arithmetic(Arithmetic::Neg)),
      "and" => Ok(Self::Arithmetic(Arithmetic::And)),
      "or" => Ok(Self::Arithmetic(Arithmetic::Or)),
      "not" => Ok(Self::Arithmetic(Arithmetic::Not)),
      "eq" => Ok(Self::Arithmetic(Arithmetic::Eq(i))),
      "lt" => Ok(Self::Arithmetic(Arithmetic::Lt(i))),
      "gt" => Ok(Self::Arithmetic(Arithmetic::Gt(i))),
      _ => Err(format!("unknown arithmetic command {}", command)),
    }
  }

  fn memory_access(command: &str, segment: &str, id: &str, filename: &str) -> Result<Self, String> {
    Ok(Self::MemoryAccess(MemoryAccess {
      command: command.parse()?,
      segment: segment.parse()?,
      index: id.parse().map_err(|_| format!("Error parsing {}", id))?,
      original: format!("{} {} {}", command, segment, id),
      filename: filename.to_string(),
    }))
  }

  fn parse_from_str((i, str): &(usize, String), filename: &str) -> Result<Self, String> {
    match str.split(" ").collect::<Vec<&str>>().as_slice() {
      [""] => Ok(Command::Noop),
      command if command[0] == "//" => Ok(Command::Noop),
      [command, memory, id] => Ok(Command::memory_access(command, memory, id, filename)?),
      [command] => Ok(Command::arithmetic(command, *i)?),
      _ => Err(format!("couldn't parse command {}", str)),
    }
  }
}

#[derive(Debug)]
struct VmParser {
  vm_file: Vec<(usize, String)>,
  filename: String,
}

impl VmParser {
  fn new(file_name: &str) -> Result<Self, Error> {
    let vm_string = fs::read_to_string(file_name)?;
    let mut lines: Vec<(usize, String)> = vm_string.lines().map(String::from).enumerate().collect();
    lines.reverse();
    Ok(VmParser {
      vm_file: lines,
      filename: file_name.to_string(),
    })
  }
}

impl Iterator for VmParser {
  type Item = Command;
  fn next(&mut self) -> Option<Self::Item> {
    let next_item = self.vm_file.pop()?;
    let path = Path::new(&self.filename).file_stem().unwrap();
    let command = Command::parse_from_str(&next_item, path.to_str().unwrap())
      .expect(&format!("Error parsing command {}", next_item.1));
    Some(command)
  }
}

struct AsmWriter {
  file: fs::File,
}

impl AsmWriter {
  fn new(file_name: &str) -> Result<Self, Error> {
    let path = Path::new(file_name).with_extension("asm");
    let file = fs::File::create(path)?;
    Ok(AsmWriter { file: file })
  }

  fn write(&mut self, parser: &mut VmParser) -> Result<(), Error> {
    loop {
      match parser.next() {
        Some(command) => write!(self.file, "{}", command.to_asm())?,
        None => break,
      }
    }
    Ok(())
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let file_name = &args.get(1).expect("USAGE: vm <filename>");
  let mut parser =
    VmParser::new(file_name).expect(&format!("Cannot open file named {}", file_name));
  let mut asm_writer = AsmWriter::new(file_name).expect("Cannot open asm file for writing");
  asm_writer.write(&mut parser).expect("Error writing file.");
}
