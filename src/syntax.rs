use std::rc::Rc;

pub type Name = Rc<str>;

pub fn name(s: &str) -> Name {
    Rc::from(s)
}

pub type Index = usize;
pub type Level = usize;

#[derive(Clone, Debug)]
pub enum Raw {
    Var(Name),
    Lam(Vec<Name>, Box<Raw>),
    App(Box<Raw>, Box<Raw>),
    Pi(Vec<Name>, Box<Raw>, Box<Raw>),
    Arrow(Box<Raw>, Box<Raw>),
    Sigma(Vec<Name>, Box<Raw>, Box<Raw>),
    Prod(Box<Raw>, Box<Raw>),
    Pair(Box<Raw>, Box<Raw>),
    Fst(Box<Raw>),
    Snd(Box<Raw>),
    U,
    Nat,
    Zero,
    NumLit(u64),
    Suc(Box<Raw>),
    NatRec(Box<Raw>, Box<Raw>, Box<Raw>, Box<Raw>),
    Bool,
    BTrue,
    BFalse,
    BoolRec(Box<Raw>, Box<Raw>, Box<Raw>, Box<Raw>),
    Unit,
    TT,
    Empty,
    EmptyRec(Box<Raw>, Box<Raw>),
    Eq(Box<Raw>, Box<Raw>, Box<Raw>),
    Refl,
    Coe(Box<Raw>, Box<Raw>, Box<Raw>, Box<Raw>),
    Let(Name, Box<Raw>, Box<Raw>, Box<Raw>),
    Ann(Box<Raw>, Box<Raw>),
}

#[derive(Clone, Debug)]
pub enum Tm {
    Var(Index),
    Lam(Name, Rc<Tm>),
    App(Rc<Tm>, Rc<Tm>),
    Pi(Name, Rc<Tm>, Rc<Tm>),
    Sigma(Name, Rc<Tm>, Rc<Tm>),
    Pair(Rc<Tm>, Rc<Tm>),
    Fst(Rc<Tm>),
    Snd(Rc<Tm>),
    U,
    Nat,
    Zero,
    Suc(Rc<Tm>),
    NatRec(Rc<Tm>, Rc<Tm>, Rc<Tm>, Rc<Tm>),
    Bool,
    BTrue,
    BFalse,
    BoolRec(Rc<Tm>, Rc<Tm>, Rc<Tm>, Rc<Tm>),
    Unit,
    TT,
    Empty,
    EmptyRec(Rc<Tm>, Rc<Tm>),
    Eq(Rc<Tm>, Rc<Tm>, Rc<Tm>),
    Refl,
    Coe(Rc<Tm>, Rc<Tm>, Rc<Tm>, Rc<Tm>),
    Let(Name, Rc<Tm>, Rc<Tm>, Rc<Tm>),
}

#[derive(Clone, Debug)]
pub enum Decl {
    Def(Name, Raw, Raw),
    Eval(Raw),
    Check(Raw, Raw),
}
