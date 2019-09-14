use std::env;
use std::fs;
use std::io::{Error, Write};
use std::iter::Iterator;
use std::path::Path;
use std::result::Result;

mod arithmetic;
mod branching;
mod function;
mod memory_access;

use arithmetic::Arithmetic;
use branching::Branching;
use function::Function;
use memory_access::MemoryAccess;

#[derive(Debug)]
enum Command {
  Arithmetic(Arithmetic),
  MemoryAccess(MemoryAccess),
  Branching(Branching),
  Function(Function),
  Noop,
}

impl Command {
  fn to_asm(&self) -> String {
    match self {
      Self::Arithmetic(command) => command.to_asm(),
      Self::Branching(command) => command.to_asm(),
      Self::MemoryAccess(command) => command.to_asm(),
      Self::Function(command) => command.to_asm(),
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

  fn branching(command: &str, label: &str) -> Result<Self, String> {
    match command {
      "goto" => Ok(Self::Branching(Branching::Goto(label.to_string()))),
      "if-goto" => Ok(Self::Branching(Branching::IfGoto(label.to_string()))),
      "label" => Ok(Self::Branching(Branching::Label(label.to_string()))),
      _ => Err(format!("unkown branching command {}", command)),
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

  fn fn_decl(name: &str, nlocals: &str) -> Result<Self, String> {
    Ok(Self::Function(Function::Decl {
      name: name.to_string(),
      nlocals: nlocals
        .parse()
        .map_err(|_| format!("Error parsing {}", nlocals))?,
    }))
  }

  fn fn_call(name: &str, nargs: &str, index: usize) -> Result<Self, String> {
    Ok(Self::Function(Function::Call {
      name: name.to_string(),
      nargs: nargs
        .parse()
        .map_err(|_| format!("Error parsing {}", nargs))?,
      index,
    }))
  }

  fn parse_from_str((i, str): &(usize, String), filename: &str) -> Result<Self, String> {
    match str
      .split("//")
      .nth(0)
      .unwrap()
      .split_whitespace()
      .collect::<Vec<&str>>()
      .as_slice()
    {
      [] => Ok(Command::Noop),
      command if command[0] == "//" => Ok(Command::Noop),
      ["function", name, nlocals] => Ok(Command::fn_decl(name, nlocals)?),
      ["call", name, nargs] => Ok(Command::fn_call(name, nargs, *i)?),
      ["return"] => Ok(Command::Function(Function::Return)),
      [command, memory, id] => Ok(Command::memory_access(command, memory, id, filename)?),
      [command, label] => Ok(Command::branching(command, label)?),
      [command] => Ok(Command::arithmetic(command, *i)?),
      rest => {
        println!("{:?}", rest);
        Err(format!("couldn't parse command {}", str))
      }
    }
  }
}

#[derive(Debug)]
struct VmParser {
  vm_file: Vec<(usize, String)>,
  filename: String,
}

impl VmParser {
  fn new(path: &std::path::PathBuf) -> Result<Self, Error> {
    let vm_string = fs::read_to_string(path)?;
    let mut lines: Vec<(usize, String)> = vm_string.lines().map(String::from).enumerate().collect();
    lines.reverse();
    Ok(VmParser {
      vm_file: lines,
      filename: path.to_str().unwrap().to_string(),
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
  fn new(path: &Path) -> Result<Self, Error> {
    let path = if path.is_dir() {
      let name = path.file_name().expect("invalid direcotry name");
      path.join(name).with_extension("asm")
    } else {
      path.with_extension("asm")
    };
    let file = fs::File::create(path)?;
    Ok(AsmWriter { file: file })
  }

  fn write(&mut self, parsers: Vec<VmParser>) -> Result<(), Error> {
    if parsers.len() > 0 {
      write!(
        self.file,
        "//Initialize\n\
         @256 // SP = 256\n\
         D=A\n\
         @SP\n\
         M=D\n"
      )?;
      write!(
        self.file,
        "{}",
        Command::Function(Function::Call {
          name: "Sys.init".to_string(),
          index: 0,
          nargs: 0
        })
        .to_asm()
      )?;
    }
    for mut parser in parsers {
      loop {
        match parser.next() {
          Some(command) => write!(self.file, "{}", command.to_asm())?,
          None => break,
        }
      }
    }
    Ok(())
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let path = &args.get(1).expect("USAGE: vm <filename|directory>");
  let path = Path::new(path);
  let parsers: Vec<VmParser> = if path.is_dir() {
    fs::read_dir(path)
      .expect("Not a directory")
      .filter_map(|path| {
        let path = &path.expect("Invalid path").path();
        if let Some("vm") = path.extension().and_then(|str| str.to_str()) {
          return Some(
            VmParser::new(path).expect(&format!("Cannot open file {}", path.to_str().unwrap())),
          );
        }
        None
      })
      .collect()
  } else {
    vec![VmParser::new(&path.to_path_buf()).expect(&format!(
      "Cannot open file named {}",
      path.to_str().unwrap()
    ))]
  };
  let mut asm_writer = AsmWriter::new(path).expect("Cannot open asm file for writing");
  asm_writer.write(parsers).expect("Error writing file.");
}
