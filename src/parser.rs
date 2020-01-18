use super::lexer;
use logos::Lexer;

use super::ast::CompilationUnit;
use super::ast::Operator;
use super::ast::PrimitiveType;
use super::ast::Program;
use super::ast::Statement;
use super::ast::Type;
use super::ast::Variable;
use super::ast::VariableBlock;
use super::ast::ConditionalBlock;
use super::lexer::Token::*;

macro_rules! expect {
    ( $token:expr, $lexer:expr) => {
        if $lexer.token != $token {
            return Err(format!("expected {:?}, but found {:?}", $token, $lexer.token).to_string());
        }
    };
}

type RustyLexer<'a> = Lexer<lexer::Token, &'a str>;

fn create_program() -> Program {
    Program {
        name: "".to_string(),
        variable_blocks: Vec::new(),
        statements: Vec::new(),
    }
}

///
/// returns an error for an uidientified token
///  
fn unidentified_token(lexer: &RustyLexer) -> String {
    format!(
        "unidentified token: {t:?} at {location:?}",
        t = lexer.slice(),
        location = lexer.range()
    )
}

///
/// returns an error for an unexpected token
///  
fn unexpected_token(lexer: &RustyLexer) -> String {
    format!(
        "unexpected token: {t:?} at {location:?}",
        t = lexer.token,
        location = lexer.range()
    )
}

fn slice_and_advance(lexer: &mut RustyLexer) -> String {
    let slice = lexer.slice().to_string();
    lexer.advance();
    slice
}

pub fn parse(mut lexer: RustyLexer) -> Result<CompilationUnit, String> {
    let mut unit = CompilationUnit { units: Vec::new() };

    loop {
        match lexer.token {
            KeywordProgram => {
                let program = parse_program(&mut lexer);
                match program {
                    Ok(p) => unit.units.push(p),

                    Err(msg) => return Err(msg),
                };
            }
            End => return Ok(unit),
            Error => return Err(unidentified_token(&lexer)),
            _ => return Err(unexpected_token(&lexer)),
        };

        lexer.advance();
    }
    //the match in the loop will always return
}

fn parse_program(lexer: &mut RustyLexer) -> Result<Program, String> {
    lexer.advance(); //Consume ProgramKeyword
    let mut result = create_program();
    expect!(Identifier, lexer);

    //Parse Identifier
    result.name = slice_and_advance(lexer);

    //Parse variable declarations
    while lexer.token == KeywordVar {
        let block = parse_variable_block(lexer);
        match block {
            Ok(b) => result.variable_blocks.push(b),
            Err(msg) => return Err(msg),
        };
    }

    //Parse the statemetns
    let mut body = parse_body(lexer, &|it| *it == KeywordEndProgram)?;
    result.statements.append(&mut body);

    Ok(result)
}

fn parse_body(lexer: &mut RustyLexer, until: &dyn Fn(&lexer::Token) -> bool) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();
    while !until(&lexer.token) && lexer.token != End && lexer.token != Error {
        let statement = parse_control_statement(lexer)?;
        statements.push(statement);
    }
    if !until(&lexer.token) {
        return Err(format!("unexpected end of body {:?}", lexer.token).to_string());
    }
    Ok(statements)
}

fn parse_control_statement(lexer: &mut RustyLexer) -> Result<Statement, String> {
    if lexer.token == KeywordIf {
        return parse_if_statement(lexer);
    }
    parse_statement(lexer)
}

fn parse_statement(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let result = parse_primary_expression(lexer);
    expect!(KeywordSemicolon, lexer);
    lexer.advance();
    result
}

fn parse_primary_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    parse_equality_expression(lexer)
}

fn parse_equality_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_compare_expression(lexer)?;
    let operator = match lexer.token {
        OperatorEqual => Operator::Equal,
        OperatorNotEqual => Operator::NotEqual,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_equality_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_compare_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_additive_expression(lexer)?;
    let operator = match lexer.token {
        OperatorLess => Operator::Less,
        OperatorGreater => Operator::Greater,
        OperatorLessOrEqual => Operator::LessOrEqual,
        OperatorGreaterOrEqual => Operator::GreaterOrEqual,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_compare_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_additive_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_multiplication_expression(lexer)?;
    let operator = match lexer.token {
        OperatorPlus => Operator::Plus,
        OperatorMinus => Operator::Minus,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_additive_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_multiplication_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let left = parse_boolean_expression(lexer)?;
    let operator = match lexer.token {
        OperatorMultiplication => Operator::Multiplication,
        OperatorDivision => Operator::Division,
        OperatorModulo => Operator::Modulo,
        _ => return Ok(left),
    };
    lexer.advance();
    let right = parse_multiplication_expression(lexer)?;
    Ok(Statement::BinaryExpression {
        operator,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn parse_boolean_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let current = parse_parenthesized_expression(lexer);
    let operator = match lexer.token {
        OperatorAnd => Some(Operator::And),
        OperatorOr => Some(Operator::Or),
        OperatorXor => Some(Operator::Xor),
        _ => None,
    };

    if let Some(operator) = operator {
        lexer.advance();
        return Ok(Statement::BinaryExpression {
            operator,
            left: Box::new(current?),
            right: Box::new(parse_primary_expression(lexer)?),
        });
    }
    current
}

fn parse_parenthesized_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    match lexer.token {
        KeywordParensOpen => {
            lexer.advance();
            let result = parse_primary_expression(lexer);
            expect!(KeywordParensClose, lexer);
            lexer.advance();
            result
        }
        _ => parse_unary_expression(lexer),
    }
}

fn parse_unary_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let operator = match lexer.token {
        OperatorNot => Some(Operator::Not),
        OperatorMinus => Some(Operator::Minus),
        _ => None,
    };
    if let Some(operator) = operator {
        lexer.advance();
        Ok(Statement::UnaryExpression {
            operator: operator,
            value: Box::new(parse_parenthesized_expression(lexer)?),
        })
    } else {
        parse_leaf_expression(lexer)
    }
}

fn parse_leaf_expression(lexer: &mut RustyLexer) -> Result<Statement, String> {
    let current = match lexer.token {
        Identifier => parse_reference(lexer),
        LiteralNumber => parse_literal_number(lexer),
        LiteralTrue => parse_bool_literal(lexer, true),
        LiteralFalse => parse_bool_literal(lexer, false),
        _ => Err(unexpected_token(lexer)),
    };

    if current.is_ok() && lexer.token == KeywordAssignment {
        lexer.advance();
        return Ok(Statement::Assignment {
            left: Box::new(current?),
            right: Box::new(parse_primary_expression(lexer)?),
        });
    };
    current
}

fn parse_if_statement(lexer: &mut RustyLexer) -> Result<Statement, String> {
    
    let end_of_body = | it : &lexer::Token | 
                                *it == KeywordElseIf
                            || *it == KeywordElse
                            || *it == KeywordEndIf;

    
    let mut conditional_blocks = vec![];

    while lexer.token == KeywordElseIf || lexer.token == KeywordIf{
        lexer.advance();//If//ElseIf
        let condition = parse_primary_expression(lexer);
        expect!(KeywordThen, lexer);
        lexer.advance();
        let body = parse_body(lexer, &end_of_body);

        let condition_block = ConditionalBlock {
            condition: Box::new(condition?),
            body: body?,
        };

        conditional_blocks.push(condition_block);
    }
    
    let mut else_block = Vec::new();

    if lexer.token == KeywordElse {
        lexer.advance(); // else
        else_block.append(&mut parse_body(lexer, &|it| *it == KeywordEndIf)?)
    }
    lexer.advance();
    
    

    Ok(Statement::IfStatement{blocks: conditional_blocks, else_block: else_block})
    
    // while lexer.token == KeywordElseIf {
    //     let condition = parse_primary_expression(lexer);
    //     expect!(KeywordThen, lexer);
    //     let body = parse_body(lexer, &end_of_body);
    //     else_bodies.push()
    // }

    // while until(&lexer.token) && lexer.token != End && lexer.token != Error {
    //     let statement = parse_control_statement(lexer)?;
    //     statements.push(statement);
    // }
    // let body = parse_body(lexer, &| it |    
    //                                        *it == KeywordElseIf
    //                                     || *it == KeywordElse
    //                                     || *it == KeywordEndIf);

    

}

fn parse_bool_literal(lexer: &mut RustyLexer, value: bool) -> Result<Statement, String> {
    lexer.advance();
    Ok(Statement::LiteralBool { value })
}

fn parse_reference(lexer: &mut RustyLexer) -> Result<Statement, String> {
    Ok(Statement::Reference {
        name: slice_and_advance(lexer).to_string(),
    })
}

fn parse_literal_number(lexer: &mut RustyLexer) -> Result<Statement, String> {
    Ok(Statement::LiteralNumber {
        value: slice_and_advance(lexer),
    })
}

fn parse_variable_block(lexer: &mut RustyLexer) -> Result<VariableBlock, String> {
    lexer.advance(); //Consume VarBlock
    let mut result = VariableBlock {
        variables: Vec::new(),
    };

    while lexer.token == Identifier {
        result = parse_variable(lexer, result)?;
    }

    expect!(KeywordEndVar, lexer);

    lexer.advance();
    Ok(result)
}

fn parse_variable(
    lexer: &mut RustyLexer,
    mut owner: VariableBlock,
) -> Result<VariableBlock, String> {
    let name = slice_and_advance(lexer);

    expect!(KeywordColon, lexer);
    lexer.advance();

    expect!(Identifier, lexer);
    let data_type = slice_and_advance(lexer);
    //Convert to real datatype

    expect!(KeywordSemicolon, lexer);
    lexer.advance();

    owner.variables.push(Variable {
        name,
        data_type: get_data_type(data_type),
    });
    Ok(owner)
}

fn get_data_type(name: String) -> Type {
    let prim_type = match name.to_lowercase().as_str() {
        "int" => Some(PrimitiveType::Int),
        "bool" => Some(PrimitiveType::Bool),
        _ => None,
    };

    if let Some(prim_type) = prim_type {
        Type::Primitive(prim_type)
    } else {
        Type::Custom
    }
}

#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};
mod tests {
    use super::super::ast::PrimitiveType;
    use super::super::ast::Type;
    use super::super::lexer;
    use super::Statement;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_returns_empty_compilation_unit() {
        let result = super::parse(lexer::lex("")).unwrap();
        assert_eq!(result.units.len(), 0);
    }

    #[test]
    fn simple_foo_program_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM foo END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        assert_eq!(prg.name, "foo");
    }

    #[test]
    fn two_programs_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM foo END_PROGRAM  PROGRAM bar END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        assert_eq!(prg.name, "foo");
        let prg2 = &result.units[1];
        assert_eq!(prg2.name, "bar");
    }

    #[test]
    fn simple_program_with_varblock_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM buz VAR END_VAR END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];

        assert_eq!(prg.variable_blocks.len(), 1);
    }

    #[test]
    fn simple_program_with_two_varblocks_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM buz VAR END_VAR VAR END_VAR END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];

        assert_eq!(prg.variable_blocks.len(), 2);
    }

    #[test]
    fn a_program_needs_to_end_with_end_program() {
        let lexer = lexer::lex("PROGRAM buz ");
        let result = super::parse(lexer);
        assert_eq!(
            result,
            Err("unexpected end of body End".to_string())
        );
    }

    #[test]
    fn a_variable_declaration_block_needs_to_end_with_endvar() {
        let lexer = lexer::lex("PROGRAM buz VAR END_PROGRAM ");
        let result = super::parse(lexer);
        assert_eq!(
            result,
            Err("expected KeywordEndVar, but found KeywordEndProgram".to_string())
        );
    }

    #[test]
    fn simple_program_with_variable_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM buz VAR x : INT; END_VAR END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let variable = &prg.variable_blocks[0].variables[0];

        assert_eq!(variable.name, "x");
        assert_eq!(variable.data_type, Type::Primitive(PrimitiveType::Int));
    }

    #[test]
    fn single_statement_parsed() {
        let lexer = lexer::lex("PROGRAM exp x; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        if let Statement::Reference { name } = statement {
            assert_eq!(name, "x");
        } else {
            panic!("Expected Reference but found {:?}", statement);
        }
    }

    #[test]
    fn literal_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM exp 7; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        if let Statement::LiteralNumber { value } = statement {
            assert_eq!(value, "7");
        } else {
            panic!("Expected LiteralNumber but found {:?}", statement);
        }
    }

    #[test]
    fn boolean_literals_can_be_parsed() {
        let lexer = lexer::lex("PROGRAM exp TRUE OR FALSE; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Or,
    left: LiteralBool {
        value: true,
    },
    right: LiteralBool {
        value: false,
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn additon_of_two_variables_parsed() {
        let lexer = lexer::lex("PROGRAM exp x+y; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        if let Statement::BinaryExpression {
            operator,
            left,  //Box<Reference> {name : left}),
            right, //Box<Reference> {name : right}),
        } = statement
        {
            if let Statement::Reference { name } = &**left {
                assert_eq!(name, "x");
            }
            if let Statement::Reference { name } = &**right {
                assert_eq!(name, "y");
            }
            assert_eq!(operator, &super::Operator::Plus);
        } else {
            panic!("Expected Reference but found {:?}", statement);
        }
    }

    #[test]
    fn additon_of_three_variables_parsed() {
        let lexer = lexer::lex("PROGRAM exp x+y-z; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        if let Statement::BinaryExpression {
            operator,
            left,  //Box<Reference> {name : left}),
            right, //Box<Reference> {name : right}),
        } = statement
        {
            assert_eq!(operator, &super::Operator::Plus);
            if let Statement::Reference { name } = &**left {
                assert_eq!(name, "x");
            }
            if let Statement::BinaryExpression {
                operator,
                left,
                right,
            } = &**right
            {
                if let Statement::Reference { name } = &**left {
                    assert_eq!(name, "y");
                }
                if let Statement::Reference { name } = &**right {
                    assert_eq!(name, "z");
                }
                assert_eq!(operator, &super::Operator::Minus);
            } else {
                panic!("Expected Reference but found {:?}", statement);
            }
        } else {
            panic!("Expected Reference but found {:?}", statement);
        }
    }

    #[test]
    fn parenthesis_expressions_should_not_change_the_ast() {
        let lexer = lexer::lex("PROGRAM exp (x+y); END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        if let Statement::BinaryExpression {
            operator,
            left,
            right,
        } = statement
        {
            if let Statement::Reference { name } = &**left {
                assert_eq!(name, "x");
            }
            if let Statement::Reference { name } = &**right {
                assert_eq!(name, "y");
            }
            assert_eq!(operator, &super::Operator::Plus);
        } else {
            panic!("Expected Reference but found {:?}", statement);
        }
    }

    #[test]
    fn multiplication_expressions_parse() {
        let lexer = lexer::lex("PROGRAM exp 1*2/7; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Multiplication,
    left: LiteralNumber {
        value: "1",
    },
    right: BinaryExpression {
        operator: Division,
        left: LiteralNumber {
            value: "2",
        },
        right: LiteralNumber {
            value: "7",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn addition_ast_test() {
        let lexer = lexer::lex("PROGRAM exp 1+2; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Plus,
    left: LiteralNumber {
        value: "1",
    },
    right: LiteralNumber {
        value: "2",
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn multiplication_ast_test() {
        let lexer = lexer::lex("PROGRAM exp 1+2*3; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Plus,
    left: LiteralNumber {
        value: "1",
    },
    right: BinaryExpression {
        operator: Multiplication,
        left: LiteralNumber {
            value: "2",
        },
        right: LiteralNumber {
            value: "3",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn term_ast_test() {
        let lexer = lexer::lex("PROGRAM exp 1+2*3+4; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Plus,
    left: LiteralNumber {
        value: "1",
    },
    right: BinaryExpression {
        operator: Plus,
        left: BinaryExpression {
            operator: Multiplication,
            left: LiteralNumber {
                value: "2",
            },
            right: LiteralNumber {
                value: "3",
            },
        },
        right: LiteralNumber {
            value: "4",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn module_expression_test() {
        let lexer = lexer::lex("PROGRAM exp 5 MOD 2; END_PROGRAM");

        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Modulo,
    left: LiteralNumber {
        value: "5",
    },
    right: LiteralNumber {
        value: "2",
    },
}"#;

        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn parenthesized_term_ast_test() {
        let lexer = lexer::lex("PROGRAM exp (1+2)*(3+4); END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Multiplication,
    left: BinaryExpression {
        operator: Plus,
        left: LiteralNumber {
            value: "1",
        },
        right: LiteralNumber {
            value: "2",
        },
    },
    right: BinaryExpression {
        operator: Plus,
        left: LiteralNumber {
            value: "3",
        },
        right: LiteralNumber {
            value: "4",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn assignment_test() {
        let lexer = lexer::lex("PROGRAM exp x := 3; x := 1 + 2; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        {
            let statement = &prg.statements[0];
            let ast_string = format!("{:#?}", statement);
            let expected_ast = r#"Assignment {
    left: Reference {
        name: "x",
    },
    right: LiteralNumber {
        value: "3",
    },
}"#;
            assert_eq!(ast_string, expected_ast);
        }

        {
            let statement = &prg.statements[1];
            let ast_string = format!("{:#?}", statement);
            let expected_ast = r#"Assignment {
    left: Reference {
        name: "x",
    },
    right: BinaryExpression {
        operator: Plus,
        left: LiteralNumber {
            value: "1",
        },
        right: LiteralNumber {
            value: "2",
        },
    },
}"#;
            assert_eq!(ast_string, expected_ast);
        }
    }

    #[test]
    fn equality_expression_test() {
        let lexer = lexer::lex("PROGRAM exp x = 3; x - 0 <> 1 + 2; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        {
            let statement = &prg.statements[0];
            let ast_string = format!("{:#?}", statement);
            let expected_ast = r#"BinaryExpression {
    operator: Equal,
    left: Reference {
        name: "x",
    },
    right: LiteralNumber {
        value: "3",
    },
}"#;
            assert_eq!(ast_string, expected_ast);
        }

        {
            let statement = &prg.statements[1];
            let ast_string = format!("{:#?}", statement);
            let expected_ast = r#"BinaryExpression {
    operator: NotEqual,
    left: BinaryExpression {
        operator: Minus,
        left: Reference {
            name: "x",
        },
        right: LiteralNumber {
            value: "0",
        },
    },
    right: BinaryExpression {
        operator: Plus,
        left: LiteralNumber {
            value: "1",
        },
        right: LiteralNumber {
            value: "2",
        },
    },
}"#;
            assert_eq!(ast_string, expected_ast);
        }
    }
    #[test]
    fn comparison_expression_test() {
        let lexer = lexer::lex(
            "PROGRAM exp 
                                    a < 3; 
                                    b > 0;
                                    c <= 7;
                                    d >= 4;
                                    e := 2 + 1 > 3 + 1;
                                    END_PROGRAM",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        {
            let statement = &prg.statements[0];
            let expected_ast = r#"BinaryExpression {
    operator: Less,
    left: Reference {
        name: "a",
    },
    right: LiteralNumber {
        value: "3",
    },
}"#;
            assert_eq!(format!("{:#?}", statement), expected_ast);
        }
        {
            let statement = &prg.statements[1]; // b > 0
            let expected_ast = r#"BinaryExpression {
    operator: Greater,
    left: Reference {
        name: "b",
    },
    right: LiteralNumber {
        value: "0",
    },
}"#;
            assert_eq!(format!("{:#?}", statement), expected_ast);
        }
        {
            let statement = &prg.statements[2]; // c <= 7
            let expected_ast = r#"BinaryExpression {
    operator: LessOrEqual,
    left: Reference {
        name: "c",
    },
    right: LiteralNumber {
        value: "7",
    },
}"#;
            assert_eq!(format!("{:#?}", statement), expected_ast);
        }
        {
            let statement = &prg.statements[3]; // d >= 4
            let expected_ast = r#"BinaryExpression {
    operator: GreaterOrEqual,
    left: Reference {
        name: "d",
    },
    right: LiteralNumber {
        value: "4",
    },
}"#;
            assert_eq!(format!("{:#?}", statement), expected_ast);
        }
        {
            //e := 2 + 1 > 3 + 1;
            let statement = &prg.statements[4];
            let expected_ast = r#"Assignment {
    left: Reference {
        name: "e",
    },
    right: BinaryExpression {
        operator: Greater,
        left: BinaryExpression {
            operator: Plus,
            left: LiteralNumber {
                value: "2",
            },
            right: LiteralNumber {
                value: "1",
            },
        },
        right: BinaryExpression {
            operator: Plus,
            left: LiteralNumber {
                value: "3",
            },
            right: LiteralNumber {
                value: "1",
            },
        },
    },
}"#;
            assert_eq!(format!("{:#?}", statement), expected_ast);
        }
    }

    #[test]
    fn boolean_expression_ast_test() {
        let lexer = lexer::lex("PROGRAM exp a AND NOT b OR c XOR d; END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: And,
    left: Reference {
        name: "a",
    },
    right: BinaryExpression {
        operator: Or,
        left: UnaryExpression {
            operator: Not,
            value: Reference {
                name: "b",
            },
        },
        right: BinaryExpression {
            operator: Xor,
            left: Reference {
                name: "c",
            },
            right: Reference {
                name: "d",
            },
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn boolean_expression_paran_ast_test() {
        let lexer = lexer::lex("PROGRAM exp a AND (NOT (b OR c) XOR d); END_PROGRAM");
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: And,
    left: Reference {
        name: "a",
    },
    right: BinaryExpression {
        operator: Xor,
        left: UnaryExpression {
            operator: Not,
            value: BinaryExpression {
                operator: Or,
                left: Reference {
                    name: "b",
                },
                right: Reference {
                    name: "c",
                },
            },
        },
        right: Reference {
            name: "d",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn signed_literal_minus_test() {
        let lexer = lexer::lex(
            "
        PROGRAM exp 
        -1;
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"UnaryExpression {
    operator: Minus,
    value: LiteralNumber {
        value: "1",
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn signed_literal_expression_test() {
        let lexer = lexer::lex(
            "
        PROGRAM exp 
        2 +-x;
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Plus,
    left: LiteralNumber {
        value: "2",
    },
    right: UnaryExpression {
        operator: Minus,
        value: Reference {
            name: "x",
        },
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn signed_literal_expression_reversed_test() {
        let lexer = lexer::lex(
            "
        PROGRAM exp 
        -4 + 5;
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = r#"BinaryExpression {
    operator: Plus,
    left: UnaryExpression {
        operator: Minus,
        value: LiteralNumber {
            value: "4",
        },
    },
    right: LiteralNumber {
        value: "5",
    },
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn if_statement() {
        let lexer = lexer::lex(
        "
        PROGRAM exp 
        IF TRUE THEN
        END_IF
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = 
r#"IfStatement {
    blocks: [
        ConditionalBlock {
            condition: LiteralBool {
                value: true,
            },
            body: [],
        },
    ],
    else_block: [],
}"#;
        assert_eq!(ast_string, expected_ast);
    }

    #[test]
    fn if_else_statement_with_expressions() {
        let lexer = lexer::lex(
        "
        PROGRAM exp 
        IF TRUE THEN
            x;
        ELSE
            y;
        END_IF
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = 
r#"IfStatement {
    blocks: [
        ConditionalBlock {
            condition: LiteralBool {
                value: true,
            },
            body: [
                Reference {
                    name: "x",
                },
            ],
        },
    ],
    else_block: [
        Reference {
            name: "y",
        },
    ],
}"#;
        assert_eq!(ast_string, expected_ast);
    }


    #[test]
    fn if_elsif_elsif_else_statement_with_expressions() {
        let lexer = lexer::lex(
        "
        PROGRAM exp 
        IF TRUE THEN
            x;
        ELSIF y THEN
            z;
        ELSIF w THEN
            v;
        ELSE
            u;
        END_IF
        END_PROGRAM
        ",
        );
        let result = super::parse(lexer).unwrap();

        let prg = &result.units[0];
        let statement = &prg.statements[0];

        let ast_string = format!("{:#?}", statement);
        let expected_ast = 
r#"IfStatement {
    blocks: [
        ConditionalBlock {
            condition: LiteralBool {
                value: true,
            },
            body: [
                Reference {
                    name: "x",
                },
            ],
        },
        ConditionalBlock {
            condition: Reference {
                name: "y",
            },
            body: [
                Reference {
                    name: "z",
                },
            ],
        },
        ConditionalBlock {
            condition: Reference {
                name: "w",
            },
            body: [
                Reference {
                    name: "v",
                },
            ],
        },
    ],
    else_block: [
        Reference {
            name: "u",
        },
    ],
}"#;
        assert_eq!(ast_string, expected_ast);
    }

}
