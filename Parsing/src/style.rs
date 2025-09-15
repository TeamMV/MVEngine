pub struct StyleParser;

impl StyleParser {
    pub fn parse_expr(expr: &str) -> StyleExpr {
        let mut e = StyleExpr { entries: vec![] };
        for line in expr.split(';') {
            if let Some((acc, val)) = line.split_once(':') {
                if acc.is_empty() || val.is_empty() { continue }
                let (acc, sub) = if let Some((l, r)) = acc.split_once('(') {
                    let r = r.trim_end_matches(')');
                    (l, Some(r.to_string()))
                } else {
                    (acc, None)
                };
                e.entries.push(StyleExprEntry {
                    accessor: acc.trim().to_string(),
                    sub_accessor: sub,
                    value: val.trim().to_string(),
                });
            }
        }
        
        e
    }
}

pub struct StyleExpr {
    pub entries: Vec<StyleExprEntry>
}

pub struct StyleExprEntry {
    pub accessor: String,
    pub sub_accessor: Option<String>,
    pub value: String
}