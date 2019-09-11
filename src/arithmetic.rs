#[derive(Debug)]
pub enum Arithmetic {
  Add,
  Sub,
  Neg,
  And,
  Or,
  Not,
  Eq(usize),
  Lt(usize),
  Gt(usize),
}

impl Arithmetic {
  pub fn to_asm(&self) -> String {
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
      Arithmetic::Neg => "// neg\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP\n\
                          D=M\n\
                          D=-D\n\
                          @SP // *SP = D\n\
                          A=M\n\
                          M=D\n\
                          @SP // SP++\n\
                          M=M+1\n"
        .to_string(),
      Arithmetic::And => "// and\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP\n\
                          D=M\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP - D\n\
                          A=M\n\
                          D=D&A\n\
                          @SP // *SP = D\n\
                          A=M\n\
                          M=D\n\
                          @SP // SP++\n\
                          M=M+1\n"
        .to_string(),
      Arithmetic::Or => "// or\n\
                         @SP // SP--\n\
                         M=M-1\n\
                         A=M // D = *SP\n\
                         D=M\n\
                         @SP // SP--\n\
                         M=M-1\n\
                         A=M // D = *SP - D\n\
                         A=M\n\
                         D=D|A\n\
                         @SP // *SP = D\n\
                         A=M\n\
                         M=D\n\
                         @SP // SP++\n\
                         M=M+1\n"
        .to_string(),
      Arithmetic::Not => "// not\n\
                          @SP // SP--\n\
                          M=M-1\n\
                          A=M // D = *SP\n\
                          D=M\n\
                          D=!D\n\
                          @SP // *SP = D\n\
                          A=M\n\
                          M=D\n\
                          @SP // SP++\n\
                          M=M+1\n"
        .to_string(),
      Arithmetic::Eq(i) => format!(
        "// eq\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // D = *SP\n\
         D=M\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // A = *SP\n\
         A=M\n\
         D=A-D\n\
         @IfEq{i}\n\
         D;JEQ\n\
         D=0\n\
         @Else{i}\n\
         0;JMP\n\
         (IfEq{i})\n\
         D=-1\n\
         (Else{i})\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        i = i,
      ),
      Arithmetic::Lt(i) => format!(
        "// lt\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // D = *SP\n\
         D=M\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // A = *SP\n\
         A=M\n\
         D=D-A\n\
         @IfEq{i}\n\
         D;JGT\n\
         D=0\n\
         @Else{i}\n\
         0;JMP\n\
         (IfEq{i})\n\
         D=-1\n\
         (Else{i})\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        i = i,
      ),
      Arithmetic::Gt(i) => format!(
        "// gt\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // D = *SP\n\
         D=M\n\
         @SP // SP--\n\
         M=M-1\n\
         A=M // A = *SP\n\
         A=M\n\
         D=D-A\n\
         @IfEq{i}\n\
         D;JLT\n\
         D=0\n\
         @Else{i}\n\
         0;JMP\n\
         (IfEq{i})\n\
         D=-1\n\
         (Else{i})\n\
         @SP // *SP = D\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n",
        i = i,
      ),
    }
  }
}
