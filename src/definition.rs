use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::char as cchar;
use nom::combinator::{map, opt};
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::basic::{
    Comment, CommentRef, Identifier, IdentifierRef, Linefeed, ListSeparator, Separator, Space,
};
use crate::constant::{parse_list_separator, ConstValue, ConstValueRef, IntConstant};
use crate::field::{Field, FieldRef};
use crate::functions::{Function, FunctionRef};
use crate::types::{FieldType, FieldTypeRef};
use crate::Parser;

// Const           ::=  'const' FieldType Identifier '=' ConstValue ListSeparator?
#[derive(Debug, Clone, PartialEq)]
pub struct ConstRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub type_: FieldTypeRef<'a>,
    pub value: ConstValueRef<'a>,
}

impl<'a> Parser<'a> for ConstRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                tag("const"),
                preceded(Separator::parse, FieldTypeRef::parse),
                preceded(Separator::parse, IdentifierRef::parse),
                preceded(opt(Separator::parse), cchar('=')),
                preceded(opt(Separator::parse), ConstValueRef::parse),
                opt(pair(opt(Separator::parse), ListSeparator::parse)),
            )),
            |(doc_comment, _, type_, name, _, value, _)| Self {
                doc_comment,
                name,
                type_,
                value,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Const {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub type_: FieldType,
    pub value: ConstValue,
}

impl<'a> From<ConstRef<'a>> for Const {
    fn from(r: ConstRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            name: r.name.into(),
            type_: r.type_.into(),
            value: r.value.into(),
        }
    }
}

impl<'a> Parser<'a> for Const {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ConstRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Typedef         ::=  'typedef' DefinitionType Identifier
// DefinitionType  ::=  BaseType | ContainerType
// BaseType        ::=  'bool' | 'byte' | 'i8' | 'i16' | 'i32' | 'i64' | 'double' | 'string' | 'binary'
// ContainerType   ::=  MapType | SetType | ListType
#[derive(Debug, Clone, PartialEq)]
pub struct TypedefRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub old: FieldTypeRef<'a>,
    pub alias: IdentifierRef<'a>,
}

impl<'a> Parser<'a> for TypedefRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                tag("typedef"),
                preceded(
                    Separator::parse,
                    alt((
                        FieldTypeRef::parse_base_type,
                        FieldTypeRef::parse_container_type,
                    )),
                ),
                preceded(Separator::parse, IdentifierRef::parse),
            )),
            |(doc_comment, _, old, alias)| TypedefRef {
                doc_comment,
                old,
                alias,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Typedef {
    pub doc_comment: Option<Comment>,
    pub old: FieldType,
    pub alias: Identifier,
}

impl<'a> From<TypedefRef<'a>> for Typedef {
    fn from(r: TypedefRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            old: r.old.into(),
            alias: r.alias.into(),
        }
    }
}

impl<'a> Parser<'a> for Typedef {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        TypedefRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Enum            ::=  'enum' Identifier '{' (Identifier ('=' IntConstant)? ListSeparator?)* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct EnumRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub children: Vec<EnumValueRef<'a>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValueRef<'a> {
    pub name: IdentifierRef<'a>,
    pub value: Option<IntConstant>,
}

impl<'a> Parser<'a> for EnumRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                tag("enum"),
                preceded(Separator::parse, IdentifierRef::parse),
                tuple((opt(Separator::parse), cchar('{'), opt(Separator::parse))),
                separated_list0(parse_list_separator, EnumValueRef::parse),
                opt(parse_list_separator),
                preceded(opt(Separator::parse), cchar('}')),
            )),
            |(doc_comment, _, name, _, children, _, _)| Self {
                doc_comment,
                name,
                children,
            },
        )(input)
    }
}

impl<'a> Parser<'a> for EnumValueRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                IdentifierRef::parse,
                opt(map(
                    tuple((
                        opt(Separator::parse),
                        cchar('='),
                        opt(Separator::parse),
                        IntConstant::parse,
                    )),
                    |(_, _, _, i)| (i),
                )),
            )),
            |(name, value)| Self { name, value },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub children: Vec<EnumValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub name: Identifier,
    pub value: Option<IntConstant>,
}

impl<'a> From<EnumRef<'a>> for Enum {
    fn from(r: EnumRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(e) => Some(e.into()),
                None => None,
            },
            name: r.name.into(),
            children: r.children.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> From<EnumValueRef<'a>> for EnumValue {
    fn from(r: EnumValueRef<'a>) -> Self {
        Self {
            name: r.name.into(),
            value: r.value,
        }
    }
}

impl<'a> Parser<'a> for Enum {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        EnumRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

impl<'a> Parser<'a> for EnumValue {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        EnumValueRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Struct          ::=  'struct' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct StructRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub fields: Vec<FieldRef<'a>>,
}

impl<'a> Parser<'a> for StructRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                pair(tag("struct"), Separator::parse),
                IdentifierRef::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, FieldRef::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(doc_comment, _, name, _, fields, _)| Self {
                doc_comment,
                name,
                fields,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Struct {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub fields: Vec<Field>,
}

impl<'a> From<StructRef<'a>> for Struct {
    fn from(r: StructRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            name: r.name.into(),
            fields: r.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Parser<'a> for Struct {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        StructRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Union          ::=  'union' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct UnionRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub fields: Vec<FieldRef<'a>>,
}

impl<'a> Parser<'a> for UnionRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                pair(tag("union"), Separator::parse),
                IdentifierRef::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, FieldRef::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(doc_comment, _, name, _, fields, _)| Self {
                doc_comment,
                name,
                fields,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Union {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub fields: Vec<Field>,
}

impl<'a> From<UnionRef<'a>> for Union {
    fn from(r: UnionRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            name: r.name.into(),
            fields: r.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Parser<'a> for Union {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        UnionRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Exception       ::=  'exception' Identifier '{' Field* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct ExceptionRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub fields: Vec<FieldRef<'a>>,
}

impl<'a> Parser<'a> for ExceptionRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                pair(tag("exception"), Separator::parse),
                IdentifierRef::parse,
                delimited(opt(Separator::parse), cchar('{'), opt(Separator::parse)),
                separated_list0(Separator::parse, FieldRef::parse),
                pair(opt(Separator::parse), cchar('}')),
            )),
            |(doc_comment, _, name, _, fields, _)| Self {
                doc_comment,
                name,
                fields,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exception {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub fields: Vec<Field>,
}

impl<'a> From<ExceptionRef<'a>> for Exception {
    fn from(r: ExceptionRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            name: r.name.into(),
            fields: r.fields.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Parser<'a> for Exception {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ExceptionRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

// Service         ::=  'service' Identifier ( 'extends' Identifier )? '{' Function* '}'
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceRef<'a> {
    pub doc_comment: Option<CommentRef<'a>>,
    pub name: IdentifierRef<'a>,
    pub extension: Option<IdentifierRef<'a>>,
    pub functions: Vec<FunctionRef<'a>>,
}

impl<'a> Parser<'a> for ServiceRef<'a> {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        map(
            tuple((
                opt(terminated(
                    CommentRef::parse,
                    terminated(Linefeed::parse, opt(Space::parse)),
                )),
                delimited(
                    pair(tag("service"), Separator::parse),
                    IdentifierRef::parse,
                    opt(Separator::parse),
                ),
                opt(map(
                    tuple((
                        tag("extends"),
                        Separator::parse,
                        IdentifierRef::parse,
                        opt(Separator::parse),
                    )),
                    |(_, _, ext, _)| ext,
                )),
                delimited(
                    pair(cchar('{'), opt(Separator::parse)),
                    separated_list0(Separator::parse, FunctionRef::parse),
                    pair(opt(Separator::parse), cchar('}')),
                ),
            )),
            |(doc_comment, name, extension, functions)| Self {
                doc_comment,
                name,
                extension,
                functions,
            },
        )(input)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Service {
    pub doc_comment: Option<Comment>,
    pub name: Identifier,
    pub extension: Option<Identifier>,
    pub functions: Vec<Function>,
}

impl<'a> From<ServiceRef<'a>> for Service {
    fn from(r: ServiceRef<'a>) -> Self {
        Self {
            doc_comment: match r.doc_comment {
                Some(d) => Some(d.into()),
                None => None,
            },
            name: r.name.into(),
            extension: r.extension.map(Into::into),
            functions: r.functions.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Parser<'a> for Service {
    fn parse(input: &'a str) -> IResult<&'a str, Self> {
        ServiceRef::parse(input).map(|(remains, parsed)| (remains, parsed.into()))
    }
}

#[cfg(test)]
mod test {
    use crate::basic::LiteralRef;

    use super::*;

    #[test]
    fn test_const() {
        assert_eq!(
            ConstRef::parse("const bool is_rust_easy = 'yes!';")
                .unwrap()
                .1,
            ConstRef {
                doc_comment: None,
                name: IdentifierRef::from("is_rust_easy"),
                type_: FieldTypeRef::Bool,
                value: ConstValueRef::Literal(LiteralRef::from("yes!"))
            }
        );
    }

    #[test]
    fn test_typedef() {
        assert_eq!(
            TypedefRef::parse("typedef i32 MyI32").unwrap().1,
            TypedefRef {
                doc_comment: None,
                old: FieldTypeRef::I32,
                alias: IdentifierRef::from("MyI32")
            }
        );
    }

    #[test]
    fn test_enum() {
        let expected = EnumRef {
            doc_comment: None,
            name: IdentifierRef::from("PL"),
            children: vec![
                EnumValueRef {
                    name: IdentifierRef::from("Rust"),
                    value: None,
                },
                EnumValueRef {
                    name: IdentifierRef::from("Go"),
                    value: Some(IntConstant::from(2)),
                },
                EnumValueRef {
                    name: IdentifierRef::from("Cpp"),
                    value: Some(IntConstant::from(3)),
                },
            ],
        };
        assert_eq!(
            EnumRef::parse("enum PL { Rust Go=2 , Cpp = 3 }").unwrap().1,
            expected
        );
        assert_eq!(
            EnumRef::parse("enum PL{Rust Go=2,Cpp=3}").unwrap().1,
            expected
        );
    }

    #[test]
    fn test_struct() {
        let expected = StructRef {
            doc_comment: None,
            name: IdentifierRef::from("user"),
            fields: vec![
                FieldRef {
                    id: Some(IntConstant::from(1)),
                    required: Some(false),
                    type_: FieldTypeRef::String,
                    name: IdentifierRef::from("name"),
                    default: None,
                },
                FieldRef {
                    id: Some(IntConstant::from(2)),
                    required: None,
                    type_: FieldTypeRef::I32,
                    name: IdentifierRef::from("age"),
                    default: Some(ConstValueRef::Int(IntConstant::from(18))),
                },
            ],
        };
        assert_eq!(
            StructRef::parse("struct user{1:optional string name; 2:i32 age=18}")
                .unwrap()
                .1,
            expected
        );
        assert_eq!(
            StructRef::parse("struct user { 1 : optional string name ; 2 : i32 age = 18 }")
                .unwrap()
                .1,
            expected
        );
    }

    #[test]
    fn test_service() {
        let function = FunctionRef {
            oneway: false,
            returns: Some(FieldTypeRef::String),
            name: IdentifierRef::from("GetUser"),
            parameters: vec![FieldRef {
                id: None,
                required: Some(true),
                type_: FieldTypeRef::String,
                name: IdentifierRef::from("name"),
                default: None,
            }],
            exceptions: None,
        };
        let expected = ServiceRef {
            doc_comment: None,
            name: IdentifierRef::from("DemoService"),
            extension: Some(IdentifierRef::from("BaseService")),
            functions: vec![function.clone(), function],
        };
        assert_eq!(
            ServiceRef::parse(
                "service DemoService extends BaseService { \
         string GetUser(required string name),
         string GetUser(required string name) }"
            )
            .unwrap()
            .1,
            expected
        );
    }
}
