#[derive(Debug)]
pub struct MemoryAccess {
  pub command: AccessCommand,
  pub segment: Segment,
  pub index: i32,
  pub original: String,
  pub filename: String,
}

impl MemoryAccess {
  pub fn to_asm(&self) -> String {
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
pub enum AccessCommand {
  Pop,
  Push,
}

#[derive(Debug)]
pub enum Segment {
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
