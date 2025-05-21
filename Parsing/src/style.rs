pub struct StyleParser;

impl StyleParser {
    pub fn parse_expr(expr: &str) -> StyleExpr {
        let mut e = StyleExpr { entries: vec![] };
        for line in expr.split(';') {
            if let Some((acc, val)) = line.split_once(':') {
                if acc.is_empty() || val.is_empty() { continue }
                e.entries.push(StyleExprEntry {
                    accessor: acc.trim().to_string(),
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
    pub value: String
}