use std::env;
use std::fs;
use std::io::{Error, Write};
use std::iter::Iterator;
use std::path::Path;
use std::result::Result;

#[derive(Debug)]
enum Command {
  Arithmetic(Arithmetic),
  MemoryAccess(MemoryAccess),
  Noop,
}

#[derive(Debug)]
enum Arithmetic {
  Add,
  Sub,
}

impl Arithmetic {
  fn to_asm(&self) -> String {
    match self {
      Arithmetic::Add => "// add\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP\n\
                          D=M\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = D + *SP\n\
                          A=M\n\
                          D=D+A\n\
                          @SP // *SP = D\n\
                          A=M\n\
                          M=D\n\
                          @SP // SP++\n\
                          M=M+1\n"
        .to_string(),
      Arithmetic::Sub => "// sub\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP\n\
                          D=M\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP - D\n\
                          A=M\n\
                          D=A-D\n\
                          @SP // *SP = D\n\
                          A=M\n\
                          M=D\n\
                          @SP // SP++\n\
                          M=M+1\n"
        .to_string(),
    }
  }
}

#[derive(Debug)]
struct MemoryAccess {
  command: AccessCommand,
  segment: Segment,
  index: i32,
  original: String,
  filename: String,
}

impl MemoryAccess {
  fn to_asm(&self) -> String {
    match &self {
      // Pop commands
      MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::Local,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::This,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::That,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::Argument,
        index,
        original,
        ..
      } => format!(
        "// {original}\n\
         @{i} // D = i\n\
         D=A\n\
         @{label} // A = {label} + D\n\
         A=M\n\
         D=A+D\n\
         @SP // SP--\n\
         M=M-1\n\
         @SP // *D = *SP\n\
         A=M\n\
         A=M\n\
         D=D+A\n\
         A=D-A\n\
         D=D-A\n\
         M=D\n",
        i = index,
        label = self.segment.asm_label(),
        original = original,
      ),
      MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::Temp,
        index,
        original,
        ..
      } => format!(
        "// {original}\n\
         @{loc} // D = i\n\
         D=A\n\
         @SP // SP--\n\
         M=M-1\n\
         @SP // *D = *SP\n\
         A=M\n\
         A=M\n\
         D=D+A\n\
         A=D-A\n\
         D=D-A\n\
         M=D\n",
        loc = index + 5,
        original = original,
      ),
      MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::Pointer,
        index,
        original,
        ..
      } if *index == 0 || *index == 1 => format!(
        "// {original}\n\
        @SP // SP--\n\
        M=M-1\n\
        @SP // {label} = *SP\n\
        A=M\n\
        D=M\n\
        @{label}\n\
        M=D\n",
        original = original,
        label = if *index == 0 { "THIS" } else { "THAT" },
      ),
      MemoryAccess {
        command: AccessCommand::Pop,
        segment: Segment::Static,
        index,
        original,
        filename,
      } => format!(
        "// {original}\n\
        @SP // SP--\n\
        M=M-1\n\
        @SP // {filename}.{index} = *SP\n\
        A=M\n\
        D=M\n\
        @{filename}.{index}\n\
        M=D\n",
        original = original,
        filename = filename,
        index = index,
      ),
      // Push Commands
      MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Constant,
        index,
        original,
        ..
      } => format!(
        "// {original}\n\
         @{i} // D = 17\n\
         D=A\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        i = index,
        original = original
      ),
      MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Local,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::This,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::That,
        index,
        original,
        ..
      }
      | MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Argument,
        index,
        original,
        ..
      } => format!(
        "// {original}\n\
         @{i} // D = i\n\
         D=A\n\
         @{label} // D = {label} + D\n\
         A=M\n\
         A=A+D\n\
         D=M // D = *A\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        i = index,
        original = original,
        label = self.segment.asm_label(),
      ),
      MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Temp,
        index,
        original,
        ..
      } => format!(
        "// {original}\n\
         @{loc} // D = temp\n\
         D=M\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        loc = index + 5,
        original = original,
      ),
      MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Pointer,
        index,
        original,
        ..
      } if *index == 0 || *index == 1 => format!(
        "// {original}\n\
        @{label} // *SP = {label}\n\
        D=M\n\
        @SP\n\
        A=M\n\
        M=D\n\
        @SP // SP++\n\
        M=M+1\n",
        original = original,
        label = if *index == 0 { "THIS" } else { "THAT" },
      ),
      MemoryAccess {
        command: AccessCommand::Push,
        segment: Segment::Static,
        index,
        original,
        filename,
      } => format!(
        "// {original}\n\
        @{filename}.{index} // *SP = {filename}.{index}\n\
        D=M\n\
        @SP\n\
        A=M\n\
        M=D\n\
        @SP // SP++\n\
        M=M+1\n",
        original = original,
        filename = filename,
        index = index,
      ),
      _ => panic!(format!("Unhandled memory access command: {:?}", self)),
    }
  }
}

#[derive(Debug)]
enum AccessCommand {
  Pop,
  Push,
}

#[derive(Debug)]
enum Segment {
  Constant,
  Local,
  Argument,
  This,
  That,
  Temp,
  Pointer,
  Static,
}

impl Segment {
  fn asm_label(&self) -> &'static str {
    match self {
      Segment::Local => "LCL",
      Segment::This => "THIS",
      Segment::That => "THAT",
      Segment::Argument => "ARG",
      _ => panic!(format!("Segment {:?}, does not have a label", self)),
    }
  }
}

impl Command {
  fn to_asm(&self) -> String {
    match self {
      Self::Arithmetic(command) => command.to_asm(),
      Self::MemoryAccess(command) => command.to_asm(),
      Self::Noop => "".to_string(),
    }
  }

  fn arithmetic(command: &str) -> Result<Self, String> {
    match command {
      "add" => Ok(Self::Arithmetic(Arithmetic::Add)),
      "sub" => Ok(Self::Arithmetic(Arithmetic::Sub)),
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

  fn parse_from_str(str: &str, filename: &str) -> Result<Self, String> {
    match str.split(" ").collect::<Vec<&str>>().as_slice() {
      [""] => Ok(Command::Noop),
      command if command[0] == "//" => Ok(Command::Noop),
      [command, memory, id] => Ok(Command::memory_access(command, memory, id, filename)?),
      [command] => Ok(Command::arithmetic(command)?),
      _ => Err(format!("couldn't parse command {}", str)),
    }
  }
}

impl std::str::FromStr for AccessCommand {
  type Err = String;
  fn from_str(str: &str) -> Result<Self, Self::Err> {
    match str {
      "pop" => Ok(Self::Pop),
      "push" => Ok(Self::Push),
      _ => Err(format!("couldn't parse command {}", str)),
    }
  }
}

impl std::str::FromStr for Segment {
  type Err = String;
  fn from_str(str: &str) -> Result<Self, Self::Err> {
    match str {
      "constant" => Ok(Self::Constant),
      "local" => Ok(Self::Local),
      "argument" => Ok(Self::Argument),
      "this" => Ok(Self::This),
      "that" => Ok(Self::That),
      "temp" => Ok(Self::Temp),
      "pointer" => Ok(Self::Pointer),
      "static" => Ok(Self::Static),
      _ => Err(format!("can't parse segment {}", str)),
    }
  }
}

#[derive(Debug)]
struct VmParser {
  vm_file: Vec<String>,
  filename: String,
}

impl VmParser {
  fn new(file_name: &str) -> Result<Self, Error> {
    let vm_string = fs::read_to_string(file_name)?;
    let mut lines: Vec<String> = vm_string.lines().map(String::from).collect();
    lines.reverse();
    Ok(VmParser { vm_file: lines, filename: file_name.to_string() })
  }
}

impl Iterator for VmParser {
  type Item = Command;
  fn next(&mut self) -> Option<Self::Item> {
    let next_item = self.vm_file.pop()?;
    let path = Path::new(&self.filename).file_stem().unwrap();
    let command = Command::parse_from_str(&next_item, path.to_str().unwrap())
      .expect(&format!("Error parsing command {}", next_item));
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
