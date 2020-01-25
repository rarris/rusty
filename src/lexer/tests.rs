use pretty_assertions::{assert_eq, assert_ne};
#[test]
fn pou_tokens() {
    let mut lexer = super::lex("PROGRAM END_PROGRAM");
    assert_eq!(lexer.token, super::Token::KeywordProgram);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndProgram);
}

#[test]
fn var_tokens() {
    let mut lexer = super::lex("VAR END_VAR");
    assert_eq!(lexer.token, super::Token::KeywordVar);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndVar);
}

#[test]
fn hello_is_an_identifier() {
    let mut lexer = super::lex("hello a12 _a12");
    assert_eq!(lexer.token, super::Token::Identifier, "{}", lexer.slice());
    lexer.advance();
    assert_eq!(lexer.token, super::Token::Identifier, "{}", lexer.slice());
    lexer.advance();
    assert_eq!(lexer.token, super::Token::Identifier, "{}", lexer.slice());
    lexer.advance();
}

#[test]
fn an_identifier_cannot_start_with_a_number() {
    let lexer = super::lex("2g12");
    assert_ne!(lexer.token, super::Token::Identifier);
}

#[test]
fn punctuations() {
    let lexer = super::lex(":");
    assert_eq!(lexer.token, super::Token::KeywordColon, "{}", lexer.slice());
    let lexer = super::lex(";");
    assert_eq!(
        lexer.token,
        super::Token::KeywordSemicolon,
        "{}",
        lexer.slice()
    );
}

#[test]
fn parens() {
    let mut lexer = super::lex("( )");
    assert_eq!(lexer.token, super::Token::KeywordParensOpen);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordParensClose);
}

#[test]
fn a_assignment_is_keword_assignment() {
    let lexer = super::lex(":=");
    assert_eq!(lexer.token, super::Token::KeywordAssignment);
}

#[test]
fn operator_test() {
    let mut lexer = super::lex("+ - * / MOD = <> < > <= >=");
    assert_eq!(lexer.token, super::Token::OperatorPlus);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorMinus);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorMultiplication);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorDivision);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorModulo);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorEqual);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorNotEqual);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorLess);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorGreater);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorLessOrEqual);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorGreaterOrEqual);
}

#[test]
fn boolean_expression_test() {
    let mut lexer = super::lex("AND XOR OR NOT");
    assert_eq!(lexer.token, super::Token::OperatorAnd);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorXor);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorOr);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::OperatorNot);
}

#[test]
fn literals_test() {
    let mut lexer = super::lex("1 2.2 0123.0123 321");

    for x in 0..5 {
        print!("{}", x);
        assert_eq!(lexer.token, super::Token::LiteralNumber);
        lexer.advance();
    }
}

#[test]
fn a_full_program_generates_correct_token_sequence() {
    let mut lexer = super::lex(
        r"
        PROGRAM hello
        VAR
          a : INT;
        END_VAR
        END_PROGRAM
        ",
    );

    assert_eq!(lexer.token, super::Token::KeywordProgram);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::Identifier);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordVar);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::Identifier);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordColon);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::Identifier);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordSemicolon);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndVar);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndProgram);
}

#[test]
fn boolean_literals() {
    let mut lexer = super::lex(r" TRUE FALSE");
    assert_eq!(lexer.token, super::Token::LiteralTrue);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::LiteralFalse);
}

#[test]
fn if_expression() {
    let mut lexer = super::lex(
        r"
        IF THEN ELSIF ELSE END_IF
        ",
    );

    assert_eq!(lexer.token, super::Token::KeywordIf);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordThen);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordElseIf);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordElse);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndIf);
}

#[test]
fn for_statement() {
    let mut lexer = super::lex(
        r"
        FOR TO BY DO END_FOR
        ",
    );

    assert_eq!(lexer.token, super::Token::KeywordFor);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordTo);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordBy);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordDo);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndFor);
}

#[test]
fn while_statement() {
    let mut lexer = super::lex(
        r"
        WHILE DO END_WHILE
        ",
    );

    assert_eq!(lexer.token, super::Token::KeywordWhile);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordDo);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndWhile);
}

#[test]
fn repeat_statement() {
    let mut lexer = super::lex(
        r"
        REPEAT UNTIL END_REPEAT
        ",
    );

    assert_eq!(lexer.token, super::Token::KeywordRepeat);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordUntil);
    lexer.advance();
    assert_eq!(lexer.token, super::Token::KeywordEndRepeat);
}
