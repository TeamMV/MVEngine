use crate::ui::geometry::shape::msfx::ast::{BinaryExpr, DeclStmt, ExportAdaptiveStmt, ExportShapeStmt, FnExpr, ForStmt, IfStmt, InputStmt, MSFXAST, MSFXExpr, MSFXStmt, ShapeExpr, UnaryExpr, WhileStmt, Function, TyExpr};
use crate::ui::geometry::shape::msfx::lexer::{MSFXKeyword, MSFXLexer, MSFXOperator, MSFXToken};
use crate::ui::geometry::shape::msfx::ty::MSFXType;
use hashbrown::HashMap;
use mvutils::lazy;
use mvutils::utils::TetrahedronOp;
use crate::ui::geometry::shape::msfx::functions::INJECTED_PRE_CODE;

const GLOBAL_SCOPE: &'static str = "function";

lazy! {
    static INJECTED_PRE_CODE_COMPILED: Vec<MSFXStmt> = MSFXParser::parse_internal(INJECTED_PRE_CODE, false).unwrap().elements;
}

pub struct MSFXParser<'a> {
    lexer: MSFXLexer<'a>,
    functions: HashMap<String, Function>,
    scope: Option<String>,
    locals: Vec<String>,
    inputs: Vec<String>,
}

impl<'a> MSFXParser<'a> {
    pub fn parse(expr: &'a str) -> Result<MSFXAST, String> {
        Self::parse_internal(expr, true)
    }

    fn parse_internal(expr: &'a str, inject: bool) -> Result<MSFXAST, String> {
        let mut this = Self {
            lexer: MSFXLexer::lex(expr),
            functions: HashMap::new(),
            scope: None,
            locals: Vec::new(),
            inputs: Vec::new(),
        };

        let mut stmts = if inject {
            INJECTED_PRE_CODE_COMPILED.clone()
        } else { vec![] };
        let mut next = this.lexer.next();
        while !matches!(next, MSFXToken::EOF) {
            this.lexer.putback(next);
            stmts.push(this.parse_stmt()?);
            next = this.lexer.next();
        }

        Ok(MSFXAST { elements: stmts, functions: this.functions })
    }

    fn ident(&self, ident: String) -> String {
        if !self.locals.contains(&ident) && self.inputs.contains(&ident) {
            format!("{GLOBAL_SCOPE}_{ident}")
        } else {
            if let Some(prefix) = &self.scope {
                format!("{prefix}_{ident}")
            } else {
                format!("{GLOBAL_SCOPE}_{ident}")
            }
        }
    }

    fn parse_stmt(&mut self) -> Result<MSFXStmt, String> {
        match self.lexer.next() {
            MSFXToken::Colon => {
                let mut stmts = Vec::new();
                let mut token = self.lexer.next();
                let mut expect_semi = true;
                loop {
                    if let MSFXToken::Keyword(MSFXKeyword::End) = token {
                        break;
                    } else if let MSFXToken::Keyword(MSFXKeyword::Else) = token {
                        self.lexer.putback(MSFXToken::Keyword(MSFXKeyword::Else));
                        expect_semi = false;
                        break;
                    }
                    self.lexer.putback(token);
                    stmts.push(self.parse_stmt()?);
                    token = self.lexer.next()
                }
                if expect_semi {
                    self.lexer.next_token(MSFXToken::Semicolon)?;
                }
                Ok(MSFXStmt::Block(stmts))
            }
            MSFXToken::Keyword(MSFXKeyword::Function) => {
                if self.scope.is_some() {
                    return Err("Creating nested functions is not allowed".to_string());
                }
                let name = self.lexer.next_ident()?;
                let scope = name.replace("_", "-");
                self.scope = Some(scope.clone());
                self.lexer.next_token(MSFXToken::LBrack)?;
                let params = self.parse_signature(scope.clone())?;

                let body = self.parse_stmt()?;

                let mut locals = Vec::with_capacity(self.locals.len());

                for local in self.locals.drain(..) {
                    locals.push(format!("{scope}_{local}"));
                }

                self.scope = None;
                let function = Function {
                    name: name.clone(),
                    locals,
                    params,
                    body,
                };

                if self.functions.insert(name.clone(), function).is_some() {
                    return Err(format!("Function '{name}' already exists!"));
                }
                Ok(MSFXStmt::Nop)
            }
            MSFXToken::Keyword(MSFXKeyword::Let) => {
                let name = self.lexer.next_ident()?;
                self.lexer
                    .next_token(MSFXToken::Operator(MSFXOperator::Assign))?;
                if self.scope.is_some() {
                    self.locals.push(name.clone());
                }
                let maybe_begin = self.lexer.next();
                if matches!(maybe_begin, MSFXToken::Keyword(MSFXKeyword::Begin)) {
                    self.lexer.next_token(MSFXToken::LBrack)?;
                    let (arguments, _) = self.parse_arguments(String::new())?;
                    if arguments.len() != 1 {
                        return Err("Amount of arguments for `begin` must be 1 in context of shape definition".to_string());
                    }
                    let mode = arguments.into_values().next().unwrap();
                    let maybe_block = self.parse_stmt()?;
                    if let MSFXStmt::Block(block) = maybe_block {
                        Ok(MSFXStmt::Let(DeclStmt {
                            name: self.ident(name),
                            expr: MSFXExpr::Shape(ShapeExpr {
                                mode: Box::new(mode),
                                block,
                            }),
                        }))
                    } else {
                        Err("Shape begin must be followed by a block".to_string())
                    }
                } else {
                    self.lexer.putback(maybe_begin);
                    let expr = self.parse_expression()?;
                    self.lexer.next_token(MSFXToken::Semicolon)?;
                    Ok(MSFXStmt::Let(DeclStmt { name: self.ident(name), expr }))
                }
            }
            MSFXToken::Ident(name) if self.will_assign() => {
                let assign = self.lexer.next();
                let expr = self.parse_expression()?;
                self.lexer.next_token(MSFXToken::Semicolon)?;
                let asignee = match assign {
                    MSFXToken::Operator(MSFXOperator::Assign) => expr,
                    MSFXToken::OperatorAssign(op) => MSFXExpr::Binary(BinaryExpr {
                        op,
                        lhs: Box::new(MSFXExpr::Ident(self.ident(name.clone()))),
                        rhs: Box::new(expr),
                    }),
                    _ => unreachable!(),
                };
                Ok(MSFXStmt::Assign(DeclStmt {
                    name: self.ident(name),
                    expr: asignee,
                }))
            }
            MSFXToken::Keyword(MSFXKeyword::For) => {
                let varname = self.lexer.next_ident()?;
                if self.scope.is_some() {
                    self.locals.push(varname.clone());
                }
                let varname = self.ident(varname);
                self.lexer.next_token(MSFXToken::Keyword(MSFXKeyword::In))?;
                let maybe_begin = self.lexer.next();
                if !matches!(maybe_begin, MSFXToken::Keyword(MSFXKeyword::Begin)) {
                    return Err("Expected a begin[] call with for!".to_string());
                }
                self.lexer.next_token(MSFXToken::LBrack)?;
                let (args, _) = self.parse_arguments(String::new())?;
                let start = args.get("_start").cloned().unwrap_or(MSFXExpr::Literal(0.0));
                let end = args
                    .get("_end")
                    .cloned()
                    .ok_or("The begin[] call requires an 'end' field!")?;
                let step = args.get("_step").cloned().unwrap_or(MSFXExpr::Literal(1.0));
                let block = self.parse_stmt()?;
                Ok(MSFXStmt::For(ForStmt {
                    varname,
                    start,
                    end,
                    step,
                    block: Box::new(block),
                }))
            }
            MSFXToken::Keyword(MSFXKeyword::While) => {
                let expr = self.parse_expression()?;
                let stmt = self.parse_stmt()?;
                Ok(MSFXStmt::While(WhileStmt {
                    cond: expr,
                    block: Box::new(stmt),
                }))
            }
            MSFXToken::Keyword(MSFXKeyword::If) => {
                let expr = self.parse_expression()?;
                let stmt = self.parse_stmt()?;
                let maybe_else = self.lexer.next();
                let false_block = if matches!(maybe_else, MSFXToken::Keyword(MSFXKeyword::Else)) {
                    Box::new(self.parse_stmt()?)
                } else {
                    self.lexer.putback(maybe_else);
                    Box::new(MSFXStmt::Nop)
                };
                Ok(MSFXStmt::If(IfStmt {
                    cond: expr,
                    true_block: Box::new(stmt),
                    false_block,
                }))
            }
            MSFXToken::Keyword(MSFXKeyword::Export) => {
                let next = self.lexer.next_some()?;
                match next {
                    MSFXToken::Keyword(MSFXKeyword::Adaptive) => {
                        self.lexer.next_token(MSFXToken::Colon)?;

                        macro_rules! parse_part {
                            () => {{
                                let exp = self.parse_expression()?;
                                self.lexer.next_token(MSFXToken::Comma)?;
                                exp
                            }};
                            ($dummy:expr) => {{
                                let exp = self.parse_expression()?;
                                exp
                            }};
                        }

                        let parts: [MSFXExpr; 9] = [
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(),
                            parse_part!(false),
                        ];
                        self.lexer.next_token(MSFXToken::Semicolon)?;
                        Ok(MSFXStmt::ExportAdaptive(ExportAdaptiveStmt { parts }))
                    }
                    t => {
                        self.lexer.putback(t);
                        let expr = self.parse_expression()?;
                        self.lexer.next_token(MSFXToken::Semicolon)?;
                        Ok(MSFXStmt::ExportShape(ExportShapeStmt { shape: expr }))
                    }
                }
            }
            MSFXToken::Keyword(MSFXKeyword::Break) => {
                self.lexer.next_token(MSFXToken::Semicolon)?;
                Ok(MSFXStmt::Break)
            }
            MSFXToken::Keyword(MSFXKeyword::Continue) => {
                self.lexer.next_token(MSFXToken::Semicolon)?;
                Ok(MSFXStmt::Continue)
            }
            MSFXToken::Keyword(MSFXKeyword::Input) => {
                let name = self.lexer.next_ident()?;
                self.inputs.push(name.clone());
                let name = self.ident(name);
                self.lexer.next_token(MSFXToken::Colon)?;
                let ty = self.lexer.next();
                let ty = match ty {
                    MSFXToken::Keyword(k) => {
                        MSFXType::from_keyword(k)?
                    }
                    t => {
                        return Err(format!(
                            "Unexpected token, expected type name, found {:?}",
                            t
                        ));
                    }
                };
                let token = self.lexer.next();
                if let MSFXToken::Operator(MSFXOperator::Assign) = token {
                    let expr = self.parse_expression()?;
                    self.lexer.next_token(MSFXToken::Semicolon)?;
                    Ok(MSFXStmt::Input(InputStmt { name, ty, default: Some(expr) }))
                } else {
                    self.lexer.putback(token);
                    self.lexer.next_token(MSFXToken::Semicolon)?;
                    Ok(MSFXStmt::Input(InputStmt { name, ty, default: None }))
                }
            }
            MSFXToken::Keyword(MSFXKeyword::Return) => {
                let maybe_semi = self.lexer.next();
                if let MSFXToken::Semicolon = maybe_semi {
                    Ok(MSFXStmt::Return(MSFXExpr::Empty))
                } else {
                    self.lexer.putback(maybe_semi);
                    let expr = self.parse_expression()?;
                    self.lexer.next_token(MSFXToken::Semicolon)?;
                    Ok(MSFXStmt::Return(expr))
                }
            }
            tkn => {
                self.lexer.putback(tkn);
                let expr = self.parse_expression()?;
                self.lexer.next_token(MSFXToken::Semicolon)?;
                Ok(MSFXStmt::Expr(expr))
            }
        }
    }

    fn will_assign(&mut self) -> bool {
        let token = self.lexer.next();
        let will_assign = matches!(
            token,
            MSFXToken::Operator(MSFXOperator::Assign) | MSFXToken::OperatorAssign(_)
        );
        self.lexer.putback(token);
        will_assign
    }

    fn parse_expression(&mut self) -> Result<MSFXExpr, String> {
        self.parse_expression_with_precedence(0)
    }

    fn parse_expression_with_precedence(&mut self, min_precedence: u8) -> Result<MSFXExpr, String> {
        let mut lhs = self.parse_primary_expression()?;
        let mut token = self.lexer.next();
        while let Some(op) = token.op() {
            let is_assign = token.assign();
            let precedence = is_assign.yn(0, op.precedence());

            if precedence < min_precedence && !is_assign {
                break;
            }

            let mut rhs = self.parse_primary_expression()?;

            if !matches!(op, MSFXOperator::Dot) {
                let mut inner_token = self.lexer.next();
                while let Some(inner_op) = inner_token.op() {
                    let inner_is_assign = inner_token.assign();
                    let inner_precedence = inner_op.precedence();

                    if inner_precedence <= precedence && !inner_is_assign {
                        break;
                    }

                    let extra = self.parse_expression_with_precedence(inner_precedence)?;
                    rhs = MSFXExpr::Binary(BinaryExpr {
                        lhs: Box::new(rhs),
                        op: inner_op,
                        rhs: Box::new(extra),
                    });
                    inner_token = self.lexer.next();
                }
                self.lexer.putback(inner_token);
            }

            if is_assign {
                lhs = MSFXExpr::Binary(BinaryExpr {
                    lhs: Box::new(lhs.clone()),
                    op: MSFXOperator::Assign,
                    rhs: Box::new(MSFXExpr::Binary(BinaryExpr {
                        lhs: Box::new(lhs),
                        op,
                        rhs: Box::new(rhs),
                    })),
                });
                token = self.lexer.next();
                break;
            } else {
                lhs = MSFXExpr::Binary(BinaryExpr {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                });
            }
            token = self.lexer.next();
        }
        self.lexer.putback(token);

        Ok(lhs)
    }

    fn parse_primary_expression(&mut self) -> Result<MSFXExpr, String> {
        let mut token = self.lexer.next();
        if matches!(token, MSFXToken::Keyword(MSFXKeyword::Vec2)) {
            token = MSFXToken::Ident("vec2".to_string());
        }
        match token {
            MSFXToken::Operator(op) if op.is_unary() => {
                let operand = self.parse_expression()?;
                Ok(MSFXExpr::Unary(UnaryExpr {
                    op,
                    inner: Box::new(operand),
                }))
            }
            MSFXToken::Ident(name) => {
                let token = self.lexer.next();
                match token {
                    MSFXToken::LBrack => {
                        let (arguments, order) = self.parse_arguments(name.replace("_", "-"))?;
                        Ok(MSFXExpr::Call(FnExpr {
                            name,
                            params: arguments,
                            order,
                        }))
                    }
                    _ => {
                        self.lexer.putback(token);
                        Ok(MSFXExpr::Ident(self.ident(name)))
                    }
                }
            }
            MSFXToken::LParen => {
                let expr = self.parse_expression()?;
                self.lexer.next_token(MSFXToken::RParen)?;
                Ok(expr)
            }
            MSFXToken::Literal(literal) => Ok(MSFXExpr::Literal(literal)),
            MSFXToken::Keyword(MSFXKeyword::Type) => {
                self.lexer.next_token(MSFXToken::LBrack)?;
                let expr = self.parse_expression()?;
                self.lexer.next_token(MSFXToken::Comma)?;
                let next = self.lexer.next_some()?;
                let ty = match next {
                    MSFXToken::Keyword(k) => MSFXType::from_keyword(k)?,
                    t => return Err(format!("Expected type but found {:?}", t)),
                };
                self.lexer.next_token(MSFXToken::RBrack)?;
                Ok(MSFXExpr::Ty(TyExpr {
                    expr: Box::new(expr),
                    ty,
                }))
            },
            MSFXToken::Keyword(MSFXKeyword::True) => Ok(MSFXExpr::Bool(true)),
            MSFXToken::Keyword(MSFXKeyword::False) => Ok(MSFXExpr::Bool(false)),
            MSFXToken::Hashtag => Ok(MSFXExpr::Empty),
            _ => Err(format!(
                "Expression: Unexpected token, expected Identifier, Literal, UnaryOperator or '(', found {:?}",
                token
            )),
        }
    }

    fn parse_signature(&mut self, scope: String) -> Result<HashMap<String, MSFXType>, String> {
        let mut map = HashMap::new();
        loop {
            let next = self.lexer.next_some()?;
            if let MSFXToken::RBrack = &next {
                return Ok(map);
            } else {
                self.lexer.putback(next);
                let name = self.lexer.next_ident()?;
                self.lexer.next_token(MSFXToken::Colon)?;
                let ty = self.lexer.next();
                if let MSFXToken::Keyword(k) = ty {
                    let ty = MSFXType::from_keyword(k)?;
                    self.locals.push(name.clone());
                    map.insert(format!("{scope}_{name}"), ty);
                } else {
                    return Err(format!("Expected type in function signature but found {:?}", ty));
                }

                let maybe_brack = self.lexer.next_some()?;
                if let MSFXToken::RBrack = maybe_brack {
                    return Ok(map);
                }
                self.lexer.putback(maybe_brack);
                self.lexer.next_token(MSFXToken::Comma)?;
            }
        }
    }

    fn parse_arguments(&mut self, scope: String) -> Result<(HashMap<String, MSFXExpr>, Vec<String>), String> {
        let mut map = HashMap::new();
        let mut order = Vec::new();
        loop {
            let next = self.lexer.next_some()?;
            if let MSFXToken::RBrack = &next {
                return Ok((map, order));
            } else {
                self.lexer.putback(next);
                let maybe_ident = self.lexer.next_some()?;
                let maybe_colon = self.lexer.next_some()?;
                if let MSFXToken::Colon = maybe_colon {
                    let exp = self.parse_expression()?;
                    if let MSFXToken::Keyword(MSFXKeyword::End) = maybe_ident {
                        order.push(format!("{scope}_end"));
                        if map.insert(format!("{scope}_end"), exp).is_some() {
                            return Err("Duplicate function argument: 'end'".to_string());
                        }
                    } else {
                        let name = maybe_ident.to_ident()?;
                        let ident = format!("{scope}_{name}");
                        order.push(ident.clone());
                        if map.insert(ident.clone(), exp).is_some() {
                            return Err(format!("Duplicate function argument: '{}'", name));
                        }
                    }
                } else {
                    self.lexer.putback(maybe_colon);
                    self.lexer.putback(maybe_ident);
                    let exp = self.parse_expression()?;
                    order.push("_".to_string());
                    if map.insert("_".to_string(), exp).is_some() {
                        return Err(
                            "Passing more than one unnamed argument to a function is not allowed"
                                .to_string(),
                        );
                    }
                }

                let maybe_brack = self.lexer.next_some()?;
                if let MSFXToken::RBrack = maybe_brack {
                    return Ok((map, order));
                }
                self.lexer.putback(maybe_brack);
                self.lexer.next_token(MSFXToken::Comma)?;
            }
        }
    }
}
