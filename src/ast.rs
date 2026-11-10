#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Char,
    Named(String),
    Parameterized(String, Vec<Type>),
    Void,
    Inferred,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Function(Function),
    Struct(Struct),
    Impl(ImplBlock),
    Enum(Enum),
    Trait(Trait),
    Use(UseDecl),
    Const(Const),
    TypeAlias(TypeAlias),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub is_self: bool,
    pub is_mut_self: bool,
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub generic_params: Vec<GenericParam>,
    pub trait_name: Option<String>,
    pub type_name: String,
    pub type_args: Vec<Type>,
    pub methods: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Type,
    pub body: Option<Vec<Statement>>,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<String>,
    pub glob: bool,
    pub group: Option<Vec<String>>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Const {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub generic_params: Vec<GenericParam>,
    pub target: Type,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Var {
        name: String,
        ty: Type,
        value: Expr,
    },
    Mut {
        name: String,
        ty: Type,
        value: Expr,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    CompoundAssign {
        target: Expr,
        op: BinOp,
        value: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        body: Vec<Statement>,
        else_if_branches: Vec<(Expr, Vec<Statement>)>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Statement>,
    },
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
    },
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expr),
    Wildcard,
    Ident(String),
    EnumVariant {
        name: String,
        fields: Vec<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    CharLit(char),
    Ident(String),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    ArrayLit(Vec<Expr>),
    Range {
        start: Option<Box<Expr>>,
        end: Box<Expr>,
        inclusive: bool,
    },
    Lambda {
        params: Vec<String>,
        body: Box<Expr>,
    },
    Cast {
        expr: Box<Expr>,
        ty: Type,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Power,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}
