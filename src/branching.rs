#[derive(Debug)]
pub enum Branching {
  Goto(String),
  IfGoto(String),
  Label(String),
}

impl Branching {
  pub fn to_asm(&self) -> String {
    match self {
      Branching::Goto(label) => format!(
        "// goto {label}\n\
         @{label}\n\
         0;JMP\n",
        label = label,
      ),
      Branching::IfGoto(label) => format!(
        "// if-goto {label}\n\
         @SP // SP--\n\
         M=M-1\n\
         @SP // D = *SP\n\
         A=M\n\
         D=M\n\
         @{label}\n\
         D;JNE\n",
        label = label,
      ),
      Branching::Label(label) => format!(
        "// label {label}\n\
         ({label})\n",
        label = label,
      ),
    }
  }
}
