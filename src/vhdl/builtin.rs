// Copyright (c) 2018 Fabian Schuiki

//! Builtin libraries, packages, types, and functions.

use std::collections::HashSet;

use num::BigInt;

use common::score::NodeRef;
use common::source::*;
use common::name::*;

use score::{ResolvableName, ScoreBoard, ScopeRef, LibRef, BuiltinPkgRef, Def, TypeMarkRef, TypeDeclRef, EnumRef, UnitRef, BuiltinOpRef};
use scope::Scope;
use ty::*;
use op::*;

// Define some global references for the builtins.
lazy_static! {
	/// A reference to the root scope where all builtins are declared.
	pub static ref ROOT_SCOPE_REF: ScopeRef = LibRef::alloc().into();
	/// A reference to the library `STD`.
	pub static ref STD_LIB_REF: LibRef = LibRef::alloc();
	/// A reference to the package `STANDARD`.
	pub static ref STANDARD_PKG_REF: BuiltinPkgRef = BuiltinPkgRef::alloc();
	/// A reference to the package `TEXTIO`.
	pub static ref TEXTIO_PKG_REF: BuiltinPkgRef = BuiltinPkgRef::alloc();
	/// A reference to the package `ENV`.
	pub static ref ENV_PKG_REF: BuiltinPkgRef = BuiltinPkgRef::alloc();

	/// The builtin `BOOLEAN` type.
	pub static ref BOOLEAN_TYPE: BuiltinType = BuiltinType::new_enum("BOOLEAN");
	/// The builtin `BIT` type.
	pub static ref BIT_TYPE: BuiltinType = BuiltinType::new_enum("BIT");
	/// The builtin `SEVERITY_LEVEL` type.
	pub static ref SEVERITY_LEVEL_TYPE: BuiltinType = BuiltinType::new_enum("SEVERITY_LEVEL");
	/// A reference to the type `INTEGER`.
	pub static ref INTEGER_TYPE: BuiltinType = BuiltinType::new("INTEGER", IntTy::new(
		Dir::To,
		i32::min_value().into(),
		i32::max_value().into()
	));
	/// The builtin `TIME` type.
	pub static ref TIME_TYPE: BuiltinType = {
		let id = TypeDeclRef::alloc();
		BuiltinType::with_id(id, "TIME", make_time_type(
			id,
			IntTy::new(
				Dir::To,
				i64::min_value().into(),
				i64::max_value().into(),
			)
		))
	};
	pub static ref TIME_TYPE_REF: TypeDeclRef = TypeDeclRef::alloc();
	/// The builtin `DELAY_LENGTH` type.
	pub static ref DELAY_LENGTH_TYPE: BuiltinType = {
		let id = TypeDeclRef::alloc();
		BuiltinType::with_id(id, "DELAY_LENGTH", make_time_type(
			id,
			IntTy::new(
				Dir::To,
				0.into(),
				i64::max_value().into(),
			)
		))
	};
	/// The builtin `NATURAL` type.
	pub static ref NATURAL_TYPE: BuiltinType = BuiltinType::new("NATURAL", IntTy::new(
		Dir::To,
		0.into(),
		i32::max_value().into()
	));
	/// The builtin `POSITIVE` type.
	pub static ref POSITIVE_TYPE: BuiltinType = BuiltinType::new("POSITIVE", IntTy::new(
		Dir::To,
		1.into(),
		i32::max_value().into()
	));
	/// The builtin `BOOLEAN_VECTOR` type.
	pub static ref BOOLEAN_VECTOR_TYPE: BuiltinType = BuiltinType::new("BOOLEAN_VECTOR", ArrayTy::new(
		vec![ArrayIndex::Unbounded(Box::new(NATURAL_TYPE.named_ty()))],
		Box::new(BOOLEAN_TYPE.named_ty())
	));
	/// The builtin `BIT_VECTOR` type.
	pub static ref BIT_VECTOR_TYPE: BuiltinType = BuiltinType::new("BIT_VECTOR", ArrayTy::new(
		vec![ArrayIndex::Unbounded(Box::new(NATURAL_TYPE.named_ty()))],
		Box::new(BIT_TYPE.named_ty())
	));
	/// The builtin `INTEGER_VECTOR` type.
	pub static ref INTEGER_VECTOR_TYPE: BuiltinType = BuiltinType::new("INTEGER_VECTOR", ArrayTy::new(
		vec![ArrayIndex::Unbounded(Box::new(NATURAL_TYPE.named_ty()))],
		Box::new(INTEGER_TYPE.named_ty())
	));
	/// The builtin `TIME_VECTOR` type.
	pub static ref TIME_VECTOR_TYPE: BuiltinType = BuiltinType::new("TIME_VECTOR", ArrayTy::new(
		vec![ArrayIndex::Unbounded(Box::new(NATURAL_TYPE.named_ty()))],
		Box::new(named_builtin_type("TIME", *TIME_TYPE_REF))
	));
	/// The builtin `FILE_OPEN_KIND` type.
	pub static ref FILE_OPEN_KIND_TYPE: BuiltinType = BuiltinType::new_enum("FILE_OPEN_KIND");
	/// The builtin `FILE_OPEN_STATUS` type.
	pub static ref FILE_OPEN_STATUS_TYPE: BuiltinType = BuiltinType::new_enum("FILE_OPEN_STATUS");

	// A list of builtin unary operators.
	static ref BUILTIN_UNARY_OPS: Vec<BuiltinUnaryOp> = vec![
		BuiltinUnaryOp::new(UnaryOp::Pos),
		BuiltinUnaryOp::new(UnaryOp::Neg),
		BuiltinUnaryOp::new(UnaryOp::Abs),
		BuiltinUnaryOp::new(UnaryOp::Cond),
		BuiltinUnaryOp::new(UnaryOp::Not),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::And)),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::Or)),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::Nand)),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::Nor)),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::Xor)),
		BuiltinUnaryOp::new(UnaryOp::Logical(LogicalOp::Xnor)),
	];

	// A list of builtin binary operators.
	static ref BUILTIN_BINARY_OPS: Vec<BuiltinBinaryOp> = vec![
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::And)),
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::Or)),
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::Nand)),
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::Nor)),
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::Xor)),
		BuiltinBinaryOp::new(BinaryOp::Logical(LogicalOp::Xnor)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Eq)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Neq)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Lt)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Leq)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Gt)),
		BuiltinBinaryOp::new(BinaryOp::Rel(RelationalOp::Geq)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Eq)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Neq)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Lt)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Leq)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Gt)),
		BuiltinBinaryOp::new(BinaryOp::Match(RelationalOp::Geq)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Sll)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Srl)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Sla)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Sra)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Rol)),
		BuiltinBinaryOp::new(BinaryOp::Shift(ShiftOp::Ror)),
		BuiltinBinaryOp::new(BinaryOp::Add),
		BuiltinBinaryOp::new(BinaryOp::Sub),
		BuiltinBinaryOp::new(BinaryOp::Concat),
		BuiltinBinaryOp::new(BinaryOp::Mul),
		BuiltinBinaryOp::new(BinaryOp::Div),
		BuiltinBinaryOp::new(BinaryOp::Mod),
		BuiltinBinaryOp::new(BinaryOp::Rem),
		BuiltinBinaryOp::new(BinaryOp::Pow),
	];
}

/// Add the definition for a builtin resolvable name to a scope.
fn define_builtin(scope: &mut Scope, name: ResolvableName, def: Def) {
	scope.defs.insert(name, vec![Spanned::new(def, INVALID_SPAN)]);
}

/// Add the definition for a builtin identifier to a scope.
fn define_builtin_ident(scope: &mut Scope, name: &str, def: Def) {
	let name = get_name_table().intern(name, false);
	define_builtin(scope, name.into(), def)
}

/// Add the definition for a builtin bit literal to a scope.
fn define_builtin_bit(scope: &mut Scope, bit: char, def: Def) {
	define_builtin(scope, bit.into(), def)
}

/// Create a named type that refers to a builtin type.
fn named_builtin_type<T: Into<TypeMarkRef>>(name: &str, type_ref: T) -> Ty {
	let name = get_name_table().intern(name, false);
	Ty::Named(name.into(), type_ref.into())
}

/// Create a named physical unit.
fn named_unit(name: &str, abs: usize, rel: Option<(usize, usize)>) -> PhysicalUnit {
	let name = get_name_table().intern(name, false);
	let abs = BigInt::from(abs);
	let rel = rel.map(|(scale, index)| (BigInt::from(scale), index));
	PhysicalUnit::new(name, abs, rel)
}

/// Add the definition for a builtin operator to a scope.
fn define_builtin_op<O>(scope: &mut Scope, op: O, id: BuiltinOpRef)
	where O: Into<Operator>
{
	scope.defs
		.entry(ResolvableName::Operator(op.into()))
		.or_insert_with(|| Vec::new())
		.push(Spanned::new(Def::BuiltinOp(id), INVALID_SPAN));
}

// Define the scopes of the builtins.
lazy_static! {
	/// The root scope.
	///
	/// It contains definitions equal to `library std; use std.standard.all;`
	pub static ref ROOT_SCOPE: Scope = {
		let mut scope = Scope::new(None);
		define_builtin_ident(&mut scope, "STD", Def::Lib(*STD_LIB_REF));
		scope.imported_scopes.insert((*STANDARD_PKG_REF).into());

		// Define the default operator implementations.
		for op in BUILTIN_UNARY_OPS.iter() {
			define_builtin_op(&mut scope, op.op, op.id);
		}
		for op in BUILTIN_BINARY_OPS.iter() {
			define_builtin_op(&mut scope, op.op, op.id);
		}

		scope
	};

	/// The scope of the library `STD`.
	pub static ref STD_LIB_SCOPE: Scope = {
		let mut scope = Scope::new(Some(*ROOT_SCOPE_REF));
		define_builtin_ident(&mut scope, "STANDARD", Def::BuiltinPkg(*STANDARD_PKG_REF));
		define_builtin_ident(&mut scope, "TEXTIO", Def::BuiltinPkg(*TEXTIO_PKG_REF));
		define_builtin_ident(&mut scope, "ENV", Def::BuiltinPkg(*ENV_PKG_REF));
		scope
	};

	/// The scope of the package `STANDARD`.
	pub static ref STANDARD_PKG_SCOPE: Scope = {
		let mut scope = Scope::new(Some((*STD_LIB_REF).into()));

		// `type BOOLEAN is (FALSE, TRUE)`
		define_builtin_ident(&mut scope, "BOOLEAN", Def::Type(BOOLEAN_TYPE.id));
		define_builtin_ident(&mut scope, "FALSE", Def::Enum(EnumRef(BOOLEAN_TYPE.id, 0)));
		define_builtin_ident(&mut scope, "TRUE", Def::Enum(EnumRef(BOOLEAN_TYPE.id, 1)));

		// `type BIT is ('0', '1')`
		define_builtin_ident(&mut scope, "BIT", Def::Type(BIT_TYPE.id));
		define_builtin_bit(&mut scope, '0', Def::Enum(EnumRef(BIT_TYPE.id, 0)));
		define_builtin_bit(&mut scope, '1', Def::Enum(EnumRef(BIT_TYPE.id, 1)));

		// `type SEVERITY_LEVEL is (NOTE, WARNING, ERROR, FAILURE)`
		define_builtin_ident(&mut scope, "SEVERITY_LEVEL", Def::Type(SEVERITY_LEVEL_TYPE.id));
		define_builtin_ident(&mut scope, "NOTE", Def::Enum(EnumRef(SEVERITY_LEVEL_TYPE.id, 0)));
		define_builtin_ident(&mut scope, "WARNING", Def::Enum(EnumRef(SEVERITY_LEVEL_TYPE.id, 1)));
		define_builtin_ident(&mut scope, "ERROR", Def::Enum(EnumRef(SEVERITY_LEVEL_TYPE.id, 2)));
		define_builtin_ident(&mut scope, "FAILURE", Def::Enum(EnumRef(SEVERITY_LEVEL_TYPE.id, 3)));

		// `type INTEGER is range ... to ...`
		define_builtin_ident(&mut scope, "INTEGER", Def::Type(INTEGER_TYPE.id));

		// `type TIME is range ... to ... units ... end units`
		define_builtin_ident(&mut scope, "TIME", Def::Type(TIME_TYPE.id));
		define_builtin_ident(&mut scope, "fs", Def::Unit(UnitRef(TIME_TYPE.id, 0)));
		define_builtin_ident(&mut scope, "ps", Def::Unit(UnitRef(TIME_TYPE.id, 1)));
		define_builtin_ident(&mut scope, "ns", Def::Unit(UnitRef(TIME_TYPE.id, 2)));
		define_builtin_ident(&mut scope, "us", Def::Unit(UnitRef(TIME_TYPE.id, 3)));
		define_builtin_ident(&mut scope, "ms", Def::Unit(UnitRef(TIME_TYPE.id, 4)));
		define_builtin_ident(&mut scope, "sec", Def::Unit(UnitRef(TIME_TYPE.id, 5)));
		define_builtin_ident(&mut scope, "min", Def::Unit(UnitRef(TIME_TYPE.id, 6)));
		define_builtin_ident(&mut scope, "hr", Def::Unit(UnitRef(TIME_TYPE.id, 7)));

		// `subtype DELAY_LENGTH is TIME range 0 to TIME'HIGH`
		define_builtin_ident(&mut scope, "DELAY_LENGTH", Def::Type(DELAY_LENGTH_TYPE.id));

		// `subtype NATURAL is INTEGER range 0 to INTEGER'HIGH`
		define_builtin_ident(&mut scope, "NATURAL", Def::Type(NATURAL_TYPE.id));

		// `subtype POSITIVE is INTEGER range 1 to INTEGER'HIGH`
		define_builtin_ident(&mut scope, "POSITIVE", Def::Type(POSITIVE_TYPE.id));

		// `type BOOLEAN_VECTOR is array (NATURAL range <>) of BOOLEAN`
		define_builtin_ident(&mut scope, "BOOLEAN_VECTOR", Def::Type(BOOLEAN_VECTOR_TYPE.id));

		// `type BIT_VECTOR is array (NATURAL range <>) of BIT`
		define_builtin_ident(&mut scope, "BIT_VECTOR", Def::Type(BIT_VECTOR_TYPE.id));

		// `type INTEGER_VECTOR is array (NATURAL range <>) of INTEGER`
		define_builtin_ident(&mut scope, "INTEGER_VECTOR", Def::Type(INTEGER_VECTOR_TYPE.id));

		// `type TIME_VECTOR is array (NATURAL range <>) of TIME`
		define_builtin_ident(&mut scope, "TIME_VECTOR", Def::Type(TIME_VECTOR_TYPE.id));

		// `type FILE_OPEN_KIND is (READ_MODE, WRITE_MODE, APPEND_MODE)`
		define_builtin_ident(&mut scope, "FILE_OPEN_KIND", Def::Type(FILE_OPEN_KIND_TYPE.id));
		define_builtin_ident(&mut scope, "READ_MODE", Def::Enum(EnumRef(FILE_OPEN_KIND_TYPE.id, 0)));
		define_builtin_ident(&mut scope, "WRITE_MODE", Def::Enum(EnumRef(FILE_OPEN_KIND_TYPE.id, 1)));
		define_builtin_ident(&mut scope, "APPEND_MODE", Def::Enum(EnumRef(FILE_OPEN_KIND_TYPE.id, 2)));

		// `type FILE_OPEN_STATUS is (OPEN_OK, STATUS_ERROR, NAME_ERROR, MODE_ERROR)`
		define_builtin_ident(&mut scope, "FILE_OPEN_STATUS", Def::Type(FILE_OPEN_STATUS_TYPE.id));
		define_builtin_ident(&mut scope, "OPEN_OK", Def::Enum(EnumRef(FILE_OPEN_STATUS_TYPE.id, 0)));
		define_builtin_ident(&mut scope, "STATUS_ERROR", Def::Enum(EnumRef(FILE_OPEN_STATUS_TYPE.id, 1)));
		define_builtin_ident(&mut scope, "NAME_ERROR", Def::Enum(EnumRef(FILE_OPEN_STATUS_TYPE.id, 2)));
		define_builtin_ident(&mut scope, "MODE_ERROR", Def::Enum(EnumRef(FILE_OPEN_STATUS_TYPE.id, 3)));

		scope
	};

	/// All builtin scopes.
	///
	/// These are added to the scoreboard upon construction.
	pub static ref BUILTIN_SCOPES: Vec<(ScopeRef, &'static Scope)> = vec![
		(*ROOT_SCOPE_REF, &*ROOT_SCOPE),
		((*STD_LIB_REF).into(), &*STD_LIB_SCOPE),
		((*STANDARD_PKG_REF).into(), &*STANDARD_PKG_SCOPE),
	];

	/// All builtin types.
	///
	/// These are added to the scoreboard upon construction.
	pub static ref BUILTIN_TYPES: Vec<(TypeDeclRef, Ty)> = vec![
		(BOOLEAN_TYPE.id, BOOLEAN_TYPE.ty.clone()),
		(BIT_TYPE.id, BIT_TYPE.ty.clone()),
		(SEVERITY_LEVEL_TYPE.id, SEVERITY_LEVEL_TYPE.ty.clone()),
		(INTEGER_TYPE.id, INTEGER_TYPE.ty.clone()),
		(TIME_TYPE.id, TIME_TYPE.ty.clone()),
		(DELAY_LENGTH_TYPE.id, DELAY_LENGTH_TYPE.ty.clone()),
		(NATURAL_TYPE.id, NATURAL_TYPE.ty.clone()),
		(POSITIVE_TYPE.id, POSITIVE_TYPE.ty.clone()),
		(BOOLEAN_VECTOR_TYPE.id, BOOLEAN_VECTOR_TYPE.ty.clone()),
		(BIT_VECTOR_TYPE.id, BIT_VECTOR_TYPE.ty.clone()),
		(INTEGER_VECTOR_TYPE.id, INTEGER_VECTOR_TYPE.ty.clone()),
		(TIME_VECTOR_TYPE.id, TIME_VECTOR_TYPE.ty.clone()),
		(FILE_OPEN_KIND_TYPE.id, FILE_OPEN_KIND_TYPE.ty.clone()),
		(FILE_OPEN_STATUS_TYPE.id, FILE_OPEN_STATUS_TYPE.ty.clone()),
	];

	/// All builtin scope references.
	pub static ref BUILTIN_SCOPE_REFS: HashSet<ScopeRef> = (*BUILTIN_SCOPES)
		.iter()
		.map(|&(id,_)| id)
		.collect();
}

/// Add the builtins to a scoreboard.
pub fn register_builtins<'ast, 'ctx>(sb: &ScoreBoard<'ast, 'ctx>) {
	debugln!("registering builtins");

	// Add the builtin scopes.
	sb.scope2_table.borrow_mut().extend((*BUILTIN_SCOPES)
		.iter()
		.map(|&(id, scope)| (id, scope.clone()))
	);

	// Add the builtin types.
	sb.ty_table.borrow_mut().extend((*BUILTIN_TYPES)
		.iter()
		.map(|&(id, ref ty)| (id.into(), sb.intern_ty(ty.clone())))
	);
}

/// Create a physical type with time units.
fn make_time_type(decl: TypeDeclRef, base: IntTy) -> PhysicalTy {
	PhysicalTy::new(
		decl,
		base,
		vec![
			named_unit("fs",  1,                        None            ),
			named_unit("ps",  1_000,                    Some((1000, 0)) ),
			named_unit("ns",  1_000_000,                Some((1000, 1)) ),
			named_unit("us",  1_000_000_000,            Some((1000, 2)) ),
			named_unit("ms",  1_000_000_000_000,        Some((1000, 3)) ),
			named_unit("sec", 1_000_000_000_000_000,    Some((1000, 4)) ),
			named_unit("min", 60_000_000_000_000_000,   Some((60, 5))   ),
			named_unit("hr",  3600_000_000_000_000_000, Some((60, 6))   ),
		],
		0
	)
}

/// A builtin unary operator.
struct BuiltinUnaryOp {
	/// The unique ID.
	id: BuiltinOpRef,
	/// The operator symbol.
	op: UnaryOp,
}

impl BuiltinUnaryOp {
	/// Create a new unart operator.
	fn new(op: UnaryOp) -> BuiltinUnaryOp {
		BuiltinUnaryOp {
			id: BuiltinOpRef::alloc(),
			op: op,
		}
	}
}

/// A builtin binary operator.
struct BuiltinBinaryOp {
	/// The unique ID.
	id: BuiltinOpRef,
	/// The operator symbol.
	op: BinaryOp,
}

impl BuiltinBinaryOp {
	/// Create a new unart operator.
	fn new(op: BinaryOp) -> BuiltinBinaryOp {
		BuiltinBinaryOp {
			id: BuiltinOpRef::alloc(),
			op: op,
		}
	}
}

/// A builtin type, function, or operator.
pub struct Builtin {
	/// The definition of this builtin.
	pub def: Def,
	/// The name of this builtin.
	pub name: ResolvableName,
	/// The type of this builtin.
	pub ty: Option<Ty>,
}

impl Builtin {
	/// Create a new builtin with a definition and a name.
	pub fn new<N: Into<ResolvableName>>(def: Def, name: N) -> Builtin {
		Builtin {
			def: def,
			name: name.into(),
			ty: None,
		}
	}

	/// Create a new builtin operator.
	pub fn operator<O: Into<Operator>>(op: O) -> Builtin {
		Builtin::new(Def::BuiltinOp(BuiltinOpRef::alloc()), op.into())
	}

	/// Assign a type to the builtin.
	///
	/// Panics if the builtin already has a type.
	pub fn ty<T: Into<Ty>>(self, ty: T) -> Builtin {
		assert!(self.ty.is_none());
		Builtin {
			ty: Some(ty.into()),
			..self
		}
	}
}

fn integer_type_builtins(ty: &Ty, into: &mut Vec<Builtin>) {
	numerical_type_builtins(ty, into);
	equality_builtins(ty, into);
	ordering_builtins(ty, into);
}

fn enum_type_builtins(ty: &Ty, into: &mut Vec<Builtin>) {
	equality_builtins(ty, into);
	ordering_builtins(ty, into);
}

fn equality_builtins(ty: &Ty, into: &mut Vec<Builtin>) {
	// The type of the operator `(T, T) return BOOLEAN`.
	let op_ty = SubprogTy::new(vec![
		SubprogTyArg::positional(ty.clone()),
		SubprogTyArg::positional(ty.clone()),
	], Some(BOOLEAN_TYPE.named_ty()));

	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Eq)).ty(op_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Neq)).ty(op_ty.clone()));
}

fn ordering_builtins(ty: &Ty, into: &mut Vec<Builtin>) {
	// The type of the operator `(T, T) return BOOLEAN`.
	let op_ty = SubprogTy::new(vec![
		SubprogTyArg::positional(ty.clone()),
		SubprogTyArg::positional(ty.clone()),
	], Some(BOOLEAN_TYPE.named_ty()));

	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Lt)).ty(op_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Leq)).ty(op_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Gt)).ty(op_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Rel(RelationalOp::Geq)).ty(op_ty.clone()));
}

fn numerical_type_builtins(ty: &Ty, into: &mut Vec<Builtin>) {
	// The type of unary operators `(T) return T`.
	let unary_ty = SubprogTy::new(vec![
		SubprogTyArg::positional(ty.clone()),
	], Some(ty.clone()));

	// The type of binary operators `(T, T) return T`.
	let binary_ty = SubprogTy::new(vec![
		SubprogTyArg::positional(ty.clone()),
		SubprogTyArg::positional(ty.clone()),
	], Some(ty.clone()));

	into.push(Builtin::operator(UnaryOp::Pos).ty(unary_ty.clone()));
	into.push(Builtin::operator(UnaryOp::Neg).ty(unary_ty.clone()));
	into.push(Builtin::operator(UnaryOp::Abs).ty(unary_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Add).ty(binary_ty.clone()));
	into.push(Builtin::operator(BinaryOp::Sub).ty(binary_ty.clone()));
}

/// A builtin type.
pub struct BuiltinType {
	/// The ID of this type.
	pub id: TypeDeclRef,
	/// The name of this type.
	pub name: Name,
	/// The actual type.
	pub ty: Ty,
	/// Auxiliary definitions.
	pub aux: Vec<Builtin>,
}

impl BuiltinType {
	/// Create a new builtin type.
	pub fn new<T: Into<Ty>>(name: &str, ty: T) -> BuiltinType {
		BuiltinType {
			id: TypeDeclRef::alloc(),
			name: get_name_table().intern(name, false),
			ty: ty.into(),
			aux: Vec::new(),
		}
	}

	/// Create a new builtin type with predefined ID.
	pub fn with_id<T: Into<Ty>>(id: TypeDeclRef, name: &str, ty: T) -> BuiltinType {
		BuiltinType {
			id: id,
			name: get_name_table().intern(name, false),
			ty: ty.into(),
			aux: Vec::new(),
		}
	}

	/// Create a new builtin enum type.
	pub fn new_enum(name: &str) -> BuiltinType {
		let id = TypeDeclRef::alloc();
		BuiltinType {
			id: id,
			name: get_name_table().intern(name, false),
			ty: EnumTy::new(id).into(),
			aux: Vec::new(),
		}
	}

	/// Get a named type that refers to this builtin type.
	pub fn named_ty(&self) -> Ty {
		Ty::Named(self.name.into(), self.id.into())
	}
}

/// A helper to build an enum.
struct EnumBuilder {
	id: TypeDeclRef,
	name: Name,
	vars: Vec<ResolvableName>,
}

impl EnumBuilder {
	/// Create a new enum builder.
	fn new(name: &str) -> EnumBuilder {
		EnumBuilder::with_id(name, TypeDeclRef::alloc())
	}

	/// Create a new enum builder with a given ID.
	fn with_id(name: &str, id: TypeDeclRef) -> EnumBuilder {
		EnumBuilder {
			id: id,
			name: get_name_table().intern(name, false),
			vars: Vec::new()
		}
	}

	/// Add an identifier enum variant.
	fn ident(mut self, name: &str) -> EnumBuilder {
		self.vars.push(get_name_table().intern(name, false).into());
		self
	}

	/// Add a bit enum variant.
	fn bit(mut self, bit: char) -> EnumBuilder {
		self.vars.push(bit.into());
		self
	}

	/// Build the enum.
	fn build(self) -> BuiltinType {
		let ty = EnumTy::new(self.id).into();
		let mut aux = Vec::new();
		for (i, var) in self.vars.into_iter().enumerate() {
			aux.push(Builtin::new(EnumRef(self.id, i).into(), var));
		}
		enum_type_builtins(&ty, &mut aux);
		BuiltinType {
			id: self.id,
			name: self.name,
			ty: ty,
			aux: aux,
		}
	}
}