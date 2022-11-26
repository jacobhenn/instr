use crate::parse::Token;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Reg {
    X,
    Acc,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Val {
    Num(i64),
    X,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    /// Move the cursor left. If the cursor is already at the beginning of the tape, do nothing.
    Gol,

    /// Move the cursor right.
    Gor,

    /// Copy the contents of the given register into the value at the cursor.
    Get(Reg),

    /// Copy the value of the cursor into the given register.
    Put(Reg),

    /// Unconditionally move the instruction pointer to the given label.
    Jmp(Token),

    /// Jump to the given label if the value in the given register is not 0.
    Jnz(Reg, Token),

    /// Jump to the given label if the vaule in the given register is less than 0.
    Jlz(Reg, Token),

    /// Store the current value of the instruction pointer in the value at the cursor
    Sav,

    /// Set the instruction pointer to the value at the cursor.
    Ret,

    /// Reads a single character of user input, and sets the value at the cursor to that character's
    /// representation as a Unicode Scalar Value.
    Inp,

    /// If the value at the cursor is a unicode scalar value, print it to stdout as a char and set
    /// `acc` to 0. Else, set `acc` to 1.
    Out,

    /// Set the value at the cursor to the given value.
    Set(Val),

    /// Add the given value to the value at the cursor.
    Add(Val),

    /// Multiply the value at the cursor by the given value.
    Mul(Val),

    /// Perform integer division of the value at the cursor by the given value.
    Div(Val),

    /// Decrement `acc`.
    Dec,
}
