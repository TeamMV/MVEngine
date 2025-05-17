pub struct StyleParser;

impl StyleParser {
    pub fn parse_expr(expr: &str) -> StyleExpr {
        let mut e = StyleExpr { entries: vec![] };
        for line in expr.split(';') {
            if let Some((acc, val)) = line.split_once(':') {
                e.entries.push(StyleExprEntry {
                    accessor: acc.to_string(),
                    value: val.to_string(),
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