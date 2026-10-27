#[derive(Clone)]
pub enum Expr {
    Const(f64),
    X,
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Call(Func, Box<Expr>),
}

#[derive(Clone, Copy)]
pub enum Func {
    Sqrt,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Ln,
    Log10,
    Exp,
    Abs,
}

impl Func {
    fn from_name(name: &str) -> Option<Func> {
        Some(match name {
            "sqrt" => Func::Sqrt,
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" => Func::Asin,
            "acos" => Func::Acos,
            "atan" => Func::Atan,
            "ln" => Func::Ln,
            "log" => Func::Log10,
            "exp" => Func::Exp,
            "abs" => Func::Abs,
            _ => return None,
        })
    }

    fn apply(self, v: f64) -> f64 {
        match self {
            Func::Sqrt => v.sqrt(),
            Func::Sin => v.sin(),
            Func::Cos => v.cos(),
            Func::Tan => v.tan(),
            Func::Asin => v.asin(),
            Func::Acos => v.acos(),
            Func::Atan => v.atan(),
            Func::Ln => v.ln(),
            Func::Log10 => v.log10(),
            Func::Exp => v.exp(),
            Func::Abs => v.abs(),
        }
    }
}

impl Expr {
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Const(v) => *v,
            Expr::X => x,
            Expr::Neg(a) => -a.eval(x),
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Sub(a, b) => a.eval(x) - b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
            Expr::Div(a, b) => a.eval(x) / b.eval(x),
            Expr::Pow(a, b) => a.eval(x).powf(b.eval(x)),
            Expr::Call(f, a) => f.apply(a.eval(x)),
        }
    }
}

struct Parser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.chars.get(self.pos).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_spaces();
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_term()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some(b'+') => {
                    self.pos += 1;
                    node = Expr::Add(Box::new(node), Box::new(self.parse_term()?));
                }
                Some(b'-') => {
                    self.pos += 1;
                    node = Expr::Sub(Box::new(node), Box::new(self.parse_term()?));
                }
                _ => return Ok(node),
            }
        }
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut node = self.parse_power()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some(b'*') => {
                    self.pos += 1;
                    node = Expr::Mul(Box::new(node), Box::new(self.parse_power()?));
                }
                Some(b'/') => {
                    self.pos += 1;
                    node = Expr::Div(Box::new(node), Box::new(self.parse_power()?));
                }
                _ => return Ok(node),
            }
        }
    }

    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_unary()?;
        if self.eat(b'^') {
            let exponent = self.parse_power()?;
            Ok(Expr::Pow(Box::new(base), Box::new(exponent)))
        } else {
            Ok(base)
        }
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.eat(b'-') {
            return Ok(Expr::Neg(Box::new(self.parse_unary()?)));
        }
        if self.eat(b'+') {
            return self.parse_unary();
        }
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr, String> {
        self.skip_spaces();
        match self.peek() {
            Some(b'(') => {
                self.pos += 1;
                let inner = self.parse_expr()?;
                if !self.eat(b')') {
                    return Err("missing closing parenthesis".to_string());
                }
                Ok(inner)
            }
            Some(c) if c.is_ascii_digit() || c == b'.' => self.parse_number(),
            Some(c) if c.is_ascii_alphabetic() => self.parse_identifier(),
            _ => Err(format!("unexpected expression at position {}", self.pos)),
        }
    }

    fn parse_number(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit() || c == b'.') {
            self.pos += 1;
        }
        std::str::from_utf8(&self.chars[start..self.pos])
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Expr::Const)
            .ok_or_else(|| "invalid number".to_string())
    }

    fn parse_identifier(&mut self) -> Result<Expr, String> {
        let start = self.pos;
        while matches!(self.peek(), Some(c) if c.is_ascii_alphabetic()) {
            self.pos += 1;
        }
        let name = std::str::from_utf8(&self.chars[start..self.pos]).unwrap().to_ascii_lowercase();

        self.skip_spaces();
        if self.peek() == Some(b'(') {
            let func = Func::from_name(&name).ok_or_else(|| format!("unknown function: {name}"))?;
            self.pos += 1;
            let arg = self.parse_expr()?;
            if !self.eat(b')') {
                return Err("missing closing parenthesis".to_string());
            }
            return Ok(Expr::Call(func, Box::new(arg)));
        }

        match name.as_str() {
            "x" => Ok(Expr::X),
            "pi" => Ok(Expr::Const(std::f64::consts::PI)),
            "e" => Ok(Expr::Const(std::f64::consts::E)),
            _ => Err(format!("unknown variable: {name}")),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr, String> {
    let mut parser = Parser { chars: input.as_bytes(), pos: 0 };
    let expr = parser.parse_expr()?;
    parser.skip_spaces();
    if parser.pos != parser.chars.len() {
        return Err(format!("extra characters at position {}", parser.pos));
    }
    Ok(expr)
}