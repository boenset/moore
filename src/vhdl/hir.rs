// Copyright (c) 2017 Fabian Schuiki

//! The High-level Intermediate Representation of a VHDL design.

use std::collections::HashMap;
use moore_common::source::*;
use moore_common::name::*;
use moore_common::util::HasSpan;
use score::*;
use typed_arena::Arena;
use syntax::ast;
use konst::*;
pub use syntax::ast::Dir;


/// A collection of arenas where HIR nodes may be allocated.
pub struct Arenas {
	pub lib: Arena<Lib>,
	pub entity: Arena<Entity>,
	pub arch: Arena<Arch>,
	pub intf_sig: Arena<IntfSignal>,
	pub subtype_ind: Arena<SubtypeInd>,
	pub package: Arena<Package>,
	pub type_decl: Arena<TypeDecl>,
	pub subtype_decl: Arena<SubtypeDecl>,
	pub expr: Arena<Expr>,
	pub const_decl: Arena<ConstDecl>,
	pub signal_decl: Arena<SignalDecl>,
	pub variable_decl: Arena<VarDecl>,
	pub file_decl: Arena<FileDecl>,
	pub process_stmt: Arena<ProcessStmt>,
	pub sig_assign_stmt: Arena<SigAssignStmt>,
	pub array_type_index: Arena<Spanned<ArrayTypeIndex>>,
}


impl Arenas {
	/// Create a new set of arenas.
	pub fn new() -> Arenas {
		Arenas {
			lib: Arena::new(),
			entity: Arena::new(),
			arch: Arena::new(),
			intf_sig: Arena::new(),
			subtype_ind: Arena::new(),
			package: Arena::new(),
			type_decl: Arena::new(),
			subtype_decl: Arena::new(),
			expr: Arena::new(),
			const_decl: Arena::new(),
			signal_decl: Arena::new(),
			variable_decl: Arena::new(),
			file_decl: Arena::new(),
			process_stmt: Arena::new(),
			sig_assign_stmt: Arena::new(),
			array_type_index: Arena::new(),
		}
	}
}


#[derive(Debug)]
pub struct Lib {
	pub entities: Vec<EntityRef>,
	pub cfgs: Vec<CfgRef>,
	pub pkg_decls: Vec<PkgDeclRef>,
	pub pkg_insts: Vec<PkgInstRef>,
	pub ctxs: Vec<CtxRef>,
	pub archs: Vec<ArchRef>,
	pub pkg_bodies: Vec<PkgBodyRef>,
}

impl Lib {
	pub fn new() -> Lib {
		Lib {
			entities: Vec::new(),
			cfgs: Vec::new(),
			pkg_decls: Vec::new(),
			pkg_insts: Vec::new(),
			ctxs: Vec::new(),
			archs: Vec::new(),
			pkg_bodies: Vec::new(),
		}
	}
}


#[derive(Debug)]
pub struct Entity {
	/// The context items associated with the entity.
	pub ctx_items: CtxItemsRef,
	/// The library in which the entity is defined.
	pub lib: LibRef,
	/// The entity name.
	pub name: Spanned<Name>,
	/// The list of generics that the entity declares.
	pub generics: Vec<GenericRef>,
	/// The list of ports that the entity declares.
	pub ports: Vec<IntfSignalRef>,
}


#[derive(Debug)]
pub struct Arch {
	/// The context items associated with the entity.
	pub ctx_items: CtxItemsRef,
	/// The entity of the architecture.
	pub entity: EntityRef,
	/// The architecture name.
	pub name: Spanned<Name>,
	/// The list of declarations in the architecture.
	pub decls: Vec<DeclInBlockRef>,
	/// The list of statements in the architecture.
	pub stmts: Vec<ConcStmtRef>,
}


#[derive(Debug)]
pub struct IntfSignal {
	/// The name of this signal.
	pub name: Spanned<Name>,
	/// The mode of this signal.
	pub mode: IntfSignalMode,
	/// The type of this signal.
	pub ty: SubtypeIndRef,
	/// Whether this signal was declared with the `bus` keyword.
	pub bus: bool,
	/// The expression determining the initial value of this signals.
	pub init: Option<ExprRef>,
}


#[derive(Debug, Clone, Copy)]
pub enum IntfSignalMode {
	In,
	Out,
	Inout,
	Buffer,
	Linkage,
}


#[derive(Debug)]
pub struct SubtypeInd {
	/// The location within the source code.
	pub span: Span,
	/// The type mark.
	pub type_mark: Spanned<TypeMarkRef>,
	/// The optional constraint.
	pub constraint: Option<Spanned<Constraint>>,
}


/// A constraint.
///
/// See IEEE 1076-2008 section 6.3.
///
/// ```ignore
/// constraint := range_constraint | array_constraint | record_constraint
/// ```
#[derive(Debug)]
pub enum Constraint {
	/// A range constraint.
	Range(Range),
	/// An array constraint.
	Array(ArrayConstraint),
	/// A record constraint.
	Record(RecordConstraint),
}

impl From<ArrayConstraint> for Constraint {
	fn from(value: ArrayConstraint) -> Constraint {
		Constraint::Array(value)
	}
}

impl From<RecordConstraint> for Constraint {
	fn from(value: RecordConstraint) -> Constraint {
		Constraint::Record(value)
	}
}

/// An element constraint.
///
/// See IEEE 1076-2008 section 6.3.
///
/// ```ignore
/// element_constraint := array_constraint | record_constraint
/// ```
#[derive(Debug)]
pub enum ElementConstraint {
	Array(ArrayConstraint),
	Record(RecordConstraint),
}

impl HasSpan for ElementConstraint {
	fn span(&self) -> Span {
		match *self {
			ElementConstraint::Array(ref n) => n.span(),
			ElementConstraint::Record(ref n) => n.span(),
		}
	}
}

impl From<ArrayConstraint> for ElementConstraint {
	fn from(value: ArrayConstraint) -> ElementConstraint {
		ElementConstraint::Array(value)
	}
}

impl From<RecordConstraint> for ElementConstraint {
	fn from(value: RecordConstraint) -> ElementConstraint {
		ElementConstraint::Record(value)
	}
}

/// An array constraint.
///
/// See IEEE 1076-2008 section 5.3.2.
///
/// ```ignore
/// array_constraint :=
///     index_constraint [array.element_constraint] |
///     "(" "open" ")" [array.element_constraint]
/// ```
#[derive(Debug)]
pub struct ArrayConstraint {
	/// The span this constraint covers.
	pub span: Span,
	/// The index constraint. An empty vector corresponds to the `open`
	/// constraint.
	pub index: Vec<Spanned<DiscreteRange>>,
	/// The optional element constraint.
	pub elem: Option<Box<Spanned<ElementConstraint>>>,
}

impl HasSpan for ArrayConstraint {
	fn span(&self) -> Span {
		self.span
	}
}

/// A discrete range.
///
/// See IEEE 1076-2008 section 5.3.2.1.
///
/// ```ignore
/// discrete_range := discrete.subtype_indication | range
/// ```
#[derive(Debug)]
pub enum DiscreteRange {
	/// A discrete range specified by a discrete subtype.
	Subtype(SubtypeIndRef),
	/// A discrete range specified by a range.
	Range(Range),
}

impl From<SubtypeIndRef> for DiscreteRange {
	fn from(value: SubtypeIndRef) -> DiscreteRange {
		DiscreteRange::Subtype(value)
	}
}

impl From<Range> for DiscreteRange {
	fn from(value: Range) -> DiscreteRange {
		DiscreteRange::Range(value)
	}
}

/// A range.
///
/// See IEEE 1076-2008 section 5.2.1.
///
/// ```ignore
/// range := range.attribute_name | simple_expression direction simple_expression
/// ```
#[derive(Debug)]
pub enum Range {
	// Attr(AttrRef),
	Immediate(Dir, ExprRef, ExprRef),
}


/// A record constraint as per IEEE 1076-2008 section 5.3.3.
#[derive(Debug)]
pub struct RecordConstraint {
	/// The span this constraint covers.
	pub span: Span,
	/// Constraints for individual elements.
	pub elems: HashMap<Name, Box<ElementConstraint>>,
}

impl HasSpan for RecordConstraint {
	fn span(&self) -> Span {
		self.span
	}
}


#[derive(Debug)]
pub struct Package {
	/// The parent scope.
	pub parent: ScopeRef,
	/// The package name.
	pub name: Spanned<Name>,
	/// The list of generics.
	pub generics: Vec<GenericRef>,
	/// The list of declarations in the package.
	pub decls: Vec<DeclInPkgRef>,
}


#[derive(Debug)]
pub struct TypeDecl {
	/// The parent scope.
	pub parent: ScopeRef,
	/// The type name.
	pub name: Spanned<Name>,
	/// The type data.
	pub data: Option<Spanned<TypeData>>,
}

/// The meat of a type declaration.
#[derive(Debug)]
pub enum TypeData {
	/// An enumeration type.
	Enum(Vec<EnumLit>),
	/// An integer, float, or physical type with optional units.
	Range(Dir, ExprRef, ExprRef),
	/// An access type.
	Access(SubtypeIndRef),
	/// An array type.
	Array(Vec<ArrayTypeIndexRef>, SubtypeIndRef),
	/// A file type.
	File(TypeMarkRef),
}

/// An enumeration literal as listed in a type declaration.
#[derive(Debug)]
pub enum EnumLit {
	Ident(Spanned<Name>),
	Char(Spanned<char>),
}

/// An index of an array type.
#[derive(Debug)]
pub enum ArrayTypeIndex {
	/// An unbounded array index of the form `... range <>`.
	Unbounded(Spanned<TypeMarkRef>),
	/// A constrained array index of the form of a subtype indication.
	Subtype(SubtypeIndRef),
	/// A constrained array index of the form `... to/downto ...`.
	Range(Dir, ExprRef, ExprRef),
}


/// A subtype declaration as per IEEE 1076-2008 section 6.3.
#[derive(Debug)]
pub struct SubtypeDecl {
	/// The parent scope.
	pub parent: ScopeRef,
	/// The subtype name.
	pub name: Spanned<Name>,
	/// The actualy subtype.
	pub subty: SubtypeIndRef,
}


#[derive(Debug)]
pub struct Expr {
	/// The parent scope.
	pub parent: ScopeRef,
	/// The range in the source file that this expression covers.
	pub span: Span,
	/// The expression data.
	pub data: ExprData,
}


#[derive(Debug)]
pub enum ExprData {
	/// A resolved name. Consists of the definition and the definition's span.
	Name(Def, Span),
	/// A selection, e.g. `a.b`.
	Select(ExprRef, Spanned<ResolvableName>),
	/// An attribute selection, e.g. `a'b`.
	Attr(ExprRef, Spanned<ResolvableName>),
	/// An integer literal.
	IntegerLiteral(ConstInt),
	/// A float literal.
	FloatLiteral(ConstFloat),
	/// A unary operator expression.
	Unary(UnaryOp, ExprRef),
	/// A binary operator expression.
	Binary(Operator, ExprRef, ExprRef),
	// A range expression.
	Range(Dir, ExprRef, ExprRef),
}


#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
	Not,
	Abs,
	Pos,
	Neg,
	Logical(ast::LogicalOp),
}


#[derive(Debug)]
pub struct ConstDecl {
	/// The scope within which the constant is declared.
	pub parent: ScopeRef,
	/// The name of the constant.
	pub name: Spanned<Name>,
	/// The subtype of the constant.
	pub subty: SubtypeIndRef,
	/// The optional initial value for the constant.
	pub init: Option<ExprRef>,
}


#[derive(Debug)]
pub struct SignalDecl {
	/// The scope within which the signal is declared.
	pub parent: ScopeRef,
	/// The name of the signal.
	pub name: Spanned<Name>,
	/// The subtype of the signal.
	pub subty: SubtypeIndRef,
	/// The signal kind.
	pub kind: SignalKind,
	/// The optional initial value for the signals.
	pub init: Option<ExprRef>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalKind {
	Normal,
	Register,
	Bus,
}


#[derive(Debug)]
pub struct VarDecl {
	/// The scope within which the variable is declared.
	pub parent: ScopeRef,
	/// Whether the variable was declared as shared or not.
	pub shared: bool,
	/// The name of the variable.
	pub name: Spanned<Name>,
	/// The subtype of the variable.
	pub subty: SubtypeIndRef,
	/// The optional initial value for the variable.
	pub init: Option<ExprRef>,
}


#[derive(Debug)]
pub struct FileDecl {
	/// The scope within which the file is declared.
	pub parent: ScopeRef,
	/// The name of the file.
	pub name: Spanned<Name>,
	/// The subtype of the file.
	pub subty: SubtypeIndRef,
	/// Additional file opening information. The first expression evaluates to a
	/// string containing the file name. The second expression evaluates to a
	/// file open kind.
	pub open: Option<(ExprRef, Option<ExprRef>)>,
}

/// A process statement.
///
/// See IEEE 1076-2008 section 11.3.
#[derive(Debug)]
pub struct ProcessStmt {
	/// The scope within which the process is declared.
	pub parent: ScopeRef,
	/// The optional process label.
	pub label: Option<Spanned<Name>>,
	/// Whether this is a postponed process. See language reference.
	pub postponed: bool,
	/// The sensitivity list.
	pub sensitivity: ProcessSensitivity,
	/// The declarations made before the `begin` keyword.
	pub decls: Vec<DeclInProcRef>,
	/// The statements inside the process.
	pub stmts: Vec<SeqStmtRef>,
}

/// A process sensitivity specification.
///
/// See IEEE 1076-2008 section 11.3.
#[derive(Debug)]
pub enum ProcessSensitivity {
	/// No sensitivity list provided.
	None,
	/// The `all` sensitivity list.
	All,
	/// Explicitly enumerated signals.
	List(Vec<Def>),
}

/// A sequential signal assignment.
///
/// See IEEE 1076-2008 section 10.5.
#[derive(Debug)]
pub struct SigAssignStmt {
	/// The scope within which the statement has been made.
	pub parent: ScopeRef,
	/// The location of the entire statement in the source file.
	pub span: Span,
	/// The optional statement label.
	pub label: Option<Spanned<Name>>,
	/// The target of the assignment.
	pub target: SigAssignTarget,
	/// The location of the right hand side in the source file.
	pub target_span: Span,
	/// The kind of the assignment.
	pub kind: SigAssignKind,
	/// The location of the right hand side in the source file.
	pub kind_span: Span,
}

/// A signal assignment target.
#[derive(Debug)]
pub enum SigAssignTarget {
	Name(SignalRef),
	Aggregate,
}

/// A signal assignment kind.
#[derive(Debug)]
pub enum SigAssignKind {
	/// A simple waveform assignment.
	SimpleWave(DelayMechanism, Waveform),
	/// A simple force assignment.
	SimpleForce(ForceMode, ExprRef),
	/// A simple release assignment.
	SimpleRelease(ForceMode),
	/// A conditional waveform assignment.
	CondWave(DelayMechanism, Cond<Waveform>),
	/// A conditional force assignment.
	CondForce(ForceMode, Cond<ExprRef>),
	/// A selected waveform assignment.
	SelWave(DelayMechanism, Sel<Waveform>),
	/// A selected force assignment.
	SelForce(ForceMode, Sel<ExprRef>),
}

/// A conditional waveform or expression.
#[derive(Debug)]
pub struct Cond<T> {
	/// The conditional values, represented as (value, cond) tuples.
	pub when: Vec<(T, ExprRef)>,
	/// The optional `else` value.
	pub other: Option<T>,
}

/// A selected waveform or expression.
#[derive(Debug)]
pub struct Sel<T> {
	/// The discriminant expression that is used to select among the choices.
	pub disc: ExprRef,
	/// The selected values, represented as (value, choices) tuples.
	pub when: Vec<(T, Choices)>,
}

/// The mode of a signal force/release statement.
///
/// See IEEE 1076-2008 section 10.5.2.1.
#[derive(Copy, Clone, Debug)]
pub enum ForceMode {
	/// Specifies an effective-value force/release. This is the default if the
	/// assignment target is a in port/signal, or no port/signal at all.
	In,
	/// Specifies a driving-value force/release. This is the default if the
	/// assignment target is a out/inout/buffer port/signal.
	Out,
}

/// The delay mechanism of a normal signal assignment.
#[derive(Copy, Clone, Debug)]
pub enum DelayMechanism {
	/// A `transport` delay mechanism.
	Transport,
	/// A `inertial` delay mechanism.
	Inertial,
	/// A `reject <time_expr> inertial` delay mechanism.
	RejectInertial(ExprRef),
}

/// A signal assignment waveform.
///
/// An empty vector corresponds to the `unaffected` waveform.
pub type Waveform = Vec<WaveElem>;

/// An element of a signal assignment waveform.
#[derive(Debug)]
pub struct WaveElem {
	/// The value expression of the element. Corresponds to `null` if `None`.
	pub value: Option<ExprRef>,
	/// The optional `after` time expression.
	pub after: Option<ExprRef>,
}

/// A list of choices used in aggregates, selected assignments, and case
/// statements.
pub type Choices = Vec<ExprRef>;
