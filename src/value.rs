use std::fmt;
use std::rc::Rc;

use crate::syntax::{Level, Name, Tm};

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

/// A semantic closure: a function from `Val` to `Val`.
///
/// Either a Coquand-style $\langle \rho, t \rangle$ (environment + body)
/// or one of the synthetic closures produced by OTT-specific reduction
/// rules.
#[derive(Clone)]
pub enum Closure {
    /// $\langle \rho, t \rangle$ -- the standard NbE closure: capture the
    /// environment $\rho$ now, evaluate body $t$ when applied.
    Body(Env, Rc<Tm>),
    /// $\lambda x.\, \mathsf{Eq}\, (B\, x)\, (f\, x)\, (g\, x)$ -- the
    /// pointwise-equality codomain produced when $\mathsf{Eq}\, (\Pi (x:A).
    /// B)\, f\, g$ reduces (Pujet-Tabareau Figure 4, Eq-Pi rule). Captures
    /// the codomain closure $B$ and the two function values $f$, $g$.
    EqPi(Rc<Closure>, Rc<Val>, Rc<Val>),
    /// $\lambda x.\, \mathsf{refl}$ -- the constantly-$\mathsf{refl}$
    /// closure used as the equality proof inside coe-Pi reductions.
    ReflPi(Rc<Closure>),
    /// $\lambda \_.\, v$ -- a non-dependent closure ($\Pi/\Sigma$ codomain
    /// that ignores its bound variable).
    Const(Rc<Val>),
    /// Codomain of $\mathsf{Eq}\, \mathcal{U}\, (\Pi (x:A).\, B)\, (\Pi
    /// (x:A').\, B')$ once it has reduced to a $\Sigma$
    /// (Pujet-Tabareau Figure 4 Eq-$\Pi$-at-$\mathcal{U}$). Applied to a
    /// proof $e_0 : \mathsf{Eq}\, \mathcal{U}\, A'\, A$ it yields the inner
    /// $\Pi$ value $\Pi (a' : A').\, \mathsf{Eq}\, \mathcal{U}\, (B\,
    /// (\mathsf{coe}\, A'\, A\, e_0\, a'))\, (B'\, a')$.
    EqUPiCod {
        dom_l: Rc<Val>,
        cod_l: Rc<Closure>,
        dom_r: Rc<Val>,
        cod_r: Rc<Closure>,
    },
    /// Body of the inner $\Pi$ produced by [`Closure::EqUPiCod`]. Applied
    /// to $a' : A'$ it yields $\mathsf{Eq}\, \mathcal{U}\, (B\,
    /// (\mathsf{coe}\, A'\, A\, e_0\, a'))\, (B'\, a')$.
    EqUPiBody {
        dom_l: Rc<Val>,
        cod_l: Rc<Closure>,
        dom_r: Rc<Val>,
        cod_r: Rc<Closure>,
        eq_dom: Rc<Val>,
    },
    /// Codomain of $\mathsf{Eq}\, \mathcal{U}\, (\Sigma (x:A).\, B)\,
    /// (\Sigma (x:A').\, B')$ (Pujet-Tabareau Figure 4
    /// Eq-$\Sigma$-at-$\mathcal{U}$). Applied to $e_0 : \mathsf{Eq}\,
    /// \mathcal{U}\, A\, A'$ it yields $\Pi (a : A).\, \mathsf{Eq}\,
    /// \mathcal{U}\, (B\, a)\, (B'\, (\mathsf{coe}\, A\, A'\, e_0\, a))$.
    EqUSigmaCod {
        dom_l: Rc<Val>,
        cod_l: Rc<Closure>,
        dom_r: Rc<Val>,
        cod_r: Rc<Closure>,
    },
    /// Body of the inner $\Pi$ produced by [`Closure::EqUSigmaCod`].
    EqUSigmaBody {
        dom_l: Rc<Val>,
        cod_l: Rc<Closure>,
        dom_r: Rc<Val>,
        cod_r: Rc<Closure>,
        eq_dom: Rc<Val>,
    },
    /// Body of $\mathsf{coe}\, (\Pi (x:A).\, B)\, (\Pi (x:A').\, B')\, p\,
    /// f$ (Pujet-Tabareau Figure 5 coe-$\Pi$). Applied to $a' : A'$ it
    /// transports across the codomain: with $a = \mathsf{coe}\, A'\, A\,
    /// (\pi_1\, p)\, a'$, returns $\mathsf{coe}\, (B\, a)\, (B'\, a')\,
    /// (\pi_2\, p\, a')\, (f\, a)$.
    CoePiBody {
        dom_l: Rc<Val>,
        cod_l: Rc<Closure>,
        dom_r: Rc<Val>,
        cod_r: Rc<Closure>,
        proof: Rc<Val>,
        fun: Rc<Val>,
    },
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Body(_, t) => write!(f, "Body({t:?})"),
            Self::EqPi(_, _, _) => write!(f, "EqPi"),
            Self::ReflPi(_) => write!(f, "ReflPi"),
            Self::Const(v) => write!(f, "Const({v:?})"),
            Self::EqUPiCod { .. } => write!(f, "EqUPiCod"),
            Self::EqUPiBody { .. } => write!(f, "EqUPiBody"),
            Self::EqUSigmaCod { .. } => write!(f, "EqUSigmaCod"),
            Self::EqUSigmaBody { .. } => write!(f, "EqUSigmaBody"),
            Self::CoePiBody { .. } => write!(f, "CoePiBody"),
        }
    }
}

/// Semantic values in weak-head normal form.
///
/// NbE-style: introductions are concrete, eliminations on neutrals
/// accumulate in [`Stuck`]. The OTT rules of Pujet-Tabareau Figure 4
/// reduce $\mathsf{Eq}$ and $\mathsf{coe}$ down to these constructors
/// whenever the type argument is concrete.
#[derive(Clone, Debug)]
pub enum Val {
    /// $\lambda x.\, t$ -- closed under env $\rho$ via [`Closure`].
    Lam(Name, Closure),
    /// $\Pi (x : A).\, B$.
    Pi(Name, Rc<Val>, Closure),
    /// $\Sigma (x : A).\, B$.
    Sigma(Name, Rc<Val>, Closure),
    /// $(a, b)$.
    Pair(Rc<Val>, Rc<Val>),
    /// $\mathcal{U}$.
    U,
    /// $\mathbb{N}$.
    Nat,
    /// $0$.
    Zero,
    /// $\mathsf{S}\, n$.
    Suc(Rc<Val>),
    /// $\{| \ell_1, \ldots, \ell_n |\}$.
    TagType(Vec<Name>),
    /// `` `\ell ``.
    Tag(Name),
    /// $\mathsf{Eq}\, A\, x\, y$ stuck because $A$ is a neutral type; otherwise
    /// the OTT reductions of Figure 4 would have already fired.
    Eq(Rc<Val>, Rc<Val>, Rc<Val>),
    /// $\mathsf{refl}$.
    Refl,
    /// $\mathsf{coe}\, A\, B\, e\, t$ stuck because $A$ or $B$ is neutral.
    Coe(Rc<Val>, Rc<Val>, Rc<Val>, Rc<Val>),
    /// A neutral term: a variable [`Head`] under a spine of eliminations.
    /// Written $\mathbf{ne}$ or $h\, \overline{e}$ in the literature.
    Stuck(Head, Spine),
}

/// Head of a neutral term. With no top-level signature in [`Val`] only
/// rigid de Bruijn *levels* appear here.
#[derive(Clone, Debug)]
pub enum Head {
    /// $x_\ell$ -- a rigid variable at de Bruijn level $\ell$.
    Var(Level),
}

/// Spine $\overline{e}$ of eliminations applied to a neutral head, in
/// outermost-first order so that quoting walks them naturally.
pub type Spine = Vec<Elim>;

/// A single elimination frame, the dual of a [`Val`] introduction.
#[derive(Clone, Debug)]
pub enum Elim {
    /// $\square\, v$ -- function application.
    App(Val),
    /// $\pi_1\, \square$.
    Fst,
    /// $\pi_2\, \square$.
    Snd,
    /// $\mathsf{rec}_\mathbb{N}\, P\, z\, s\, \square$.
    NatRec(Val, Val, Val),
    /// $\mathsf{rec}_{\{|\ldots|\}}\, P\, \{\ell_i \Rightarrow e_i\}\, \square$.
    TagRec(Val, Vec<(Name, Val)>),
}

impl Val {
    pub fn var(lvl: Level) -> Self {
        Self::Stuck(Head::Var(lvl), Vec::new())
    }
}
