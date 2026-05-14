use crate::syntax::{Level, Name, Tm};
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct Env(pub Vec<Val>);

impl Env {
    #[must_use]
    pub fn extend(&self, v: Val) -> Self {
        let mut new = self.clone();
        new.0.push(v);
        new
    }

    pub fn lookup(&self, ix: usize) -> Val {
        self.0[self.0.len() - 1 - ix].clone()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone)]
pub enum Closure {
    Body(Env, Rc<Tm>),
    EqPi(Rc<Closure>, Rc<Val>, Rc<Val>),
    ReflPi(Rc<Closure>),
    Const(Rc<Val>),
}

impl std::fmt::Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Body(_, t) => write!(f, "Body({t:?})"),
            Self::EqPi(_, _, _) => write!(f, "EqPi"),
            Self::ReflPi(_) => write!(f, "ReflPi"),
            Self::Const(v) => write!(f, "Const({v:?})"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Val {
    Lam(Name, Closure),
    Pi(Name, Rc<Val>, Closure),
    Sigma(Name, Rc<Val>, Closure),
    Pair(Rc<Val>, Rc<Val>),
    U,
    Nat,
    Zero,
    Suc(Rc<Val>),
    Bool,
    BTrue,
    BFalse,
    Unit,
    TT,
    Empty,
    Eq(Rc<Val>, Rc<Val>, Rc<Val>),
    Refl,
    Coe(Rc<Val>, Rc<Val>, Rc<Val>, Rc<Val>),
    Stuck(Head, Spine),
}

#[derive(Clone, Debug)]
pub enum Head {
    Var(Level),
}

pub type Spine = Vec<Elim>;

#[derive(Clone, Debug)]
pub enum Elim {
    App(Val),
    Fst,
    Snd,
    NatRec(Val, Val, Val),
    BoolRec(Val, Val, Val),
    EmptyRec(Val),
}

impl Val {
    pub fn var(lvl: Level) -> Self {
        Self::Stuck(Head::Var(lvl), Vec::new())
    }
}
