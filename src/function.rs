#[derive(Debug)]
pub enum Function {
  Decl {
    name: String,
    nlocals: usize,
  },
  Call {
    name: String,
    nargs: usize,
    index: usize,
  },
  Return,
}

impl Function {
  pub fn to_asm(&self) -> String {
    match self {
      Function::Decl { name, nlocals } => format!(
        "// function {name} {nlocals}\n\
         ({name})\n\
         @{nlocals} // D = n_locals\n\
         D=A\n\
         @{name}.decl.skiplocals\n\
         D;JEQ\n\
         ({name}.decl.locals)\n\
         @SP // *SP = 0\n\
         A=M\n\
         M=0\n\
         @SP // SP++\n\
         M=M+1\n\
         D=D-1 // D--\n\
         @{name}.decl.locals\n\
         D;JNE\n\
         ({name}.decl.skiplocals)\n",
        name = name,
        nlocals = nlocals
      ),
      Function::Call { name, nargs, index } => format!(
        "// call {name} {nargs}\n\
         @{name}.ret.{index} // *SP = return_address\n\
         D=A\n\
         @SP\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n\
         @LCL // *SP = LCL\n\
         D=M\n\
         @SP\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n\
         @ARG // *SP = ARG\n\
         D=M\n\
         @SP\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n\
         @THIS // *SP = THIS\n\
         D=M\n\
         @SP\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n\
         @THAT // *SP = THAT\n\
         D=M\n\
         @SP\n\
         A=M\n\
         M=D\n\
         @SP // SP++\n\
         M=M+1\n\
         @SP // ARG = SP - 5 - {nargs}\n\
         A=M\n\
         D=A\n\
         @{arg_offset}\n\
         D=D-A\n\
         @ARG\n\
         M=D\n\
         @SP // LCL = SP\n\
         D=M\n\
         @LCL\n\
         M=D\n\
         @{name}\n\
         0;JMP\n\
         ({name}.ret.{index})\n",
        name = name,
        nargs = nargs,
        index = index,
        arg_offset = 5 + nargs,
      ),
      Function::Return => "// return \n\
                           @LCL // endFrame = LCL\n\
                           D=M\n\
                           @R13\n\
                           M=D\n\
                           @5 // retAddr = *(endFrame - 5)\n\
                           A=D-A\n\
                           D=M\n\
                           @R14\n\
                           M=D\n\
                           @SP // SP--\n\
                           M=M-1\n\
                           @SP // *ARG=pop\n\
                           A=M\n\
                           D=M\n\
                           @ARG\n\
                           A=M\n\
                           M=D\n\
                           @ARG // SP = ARG + 1\n\
                           D=M\n\
                           D=D+1\n\
                           @SP\n\
                           M=D\n\
                           @R13 // THAT = *(enfFrame -1)\n\
                           D=M\n\
                           @1\n\
                           D=D-A\n\
                           A=D\n\
                           D=M\n\
                           @THAT\n\
                           M=D\n\
                           @R13 // THIS = *(enfFrame - 2)\n\
                           D=M\n\
                           @2\n\
                           D=D-A\n\
                           A=D\n\
                           D=M\n\
                           @THIS\n\
                           M=D\n\
                           @R13 // ARG= *(enfFrame - 3)\n\
                           D=M\n\
                           @3\n\
                           D=D-A\n\
                           A=D\n\
                           D=M\n\
                           @ARG\n\
                           M=D\n\
                           @R13 // LCL= *(enfFrame - 4)\n\
                           D=M\n\
                           @4\n\
                           D=D-A\n\
                           A=D\n\
                           D=M\n\
                           @LCL\n\
                           M=D\n\
                           @R14 // goto retAddr\n\
                           A=M\n\
                           0;JMP\n"
        .to_string(),
    }
  }
}
