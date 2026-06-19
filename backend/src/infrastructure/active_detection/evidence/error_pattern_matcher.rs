use regex::Regex;

use super::EvidenceItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPatternCategory {
    Sql,
    NoSql,
    Deserialization,
    Framework,
}

#[derive(Debug, Clone)]
pub struct MatchedErrorPattern {
    pub category: ErrorPatternCategory,
    pub family: &'static str,
    pub matched_text: String,
}

impl MatchedErrorPattern {
    pub fn to_evidence(&self) -> EvidenceItem {
        EvidenceItem::error_pattern(
            self.matched_text.clone(),
            format!(
                "{} error pattern matched: {}",
                self.family, self.matched_text
            ),
        )
    }
}

struct ErrorPattern {
    category: ErrorPatternCategory,
    family: &'static str,
    regex: Regex,
}

pub struct ErrorPatternMatcher {
    patterns: Vec<ErrorPattern>,
}

impl ErrorPatternMatcher {
    pub fn new() -> Self {
        let mut patterns = Vec::new();

        patterns.extend(sql_patterns());
        patterns.extend(nosql_patterns());
        patterns.extend(deserialization_patterns());
        patterns.extend(framework_patterns());

        Self { patterns }
    }

    pub fn match_sql(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.match_category(body, ErrorPatternCategory::Sql)
    }

    pub fn match_nosql(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.match_category(body, ErrorPatternCategory::NoSql)
    }

    pub fn match_deserialization(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.match_category(body, ErrorPatternCategory::Deserialization)
    }

    pub fn match_framework(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.match_category(body, ErrorPatternCategory::Framework)
    }

    pub fn match_any(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.patterns.iter().find_map(|pattern| pattern.find(body))
    }

    fn match_category(
        &self,
        body: &str,
        category: ErrorPatternCategory,
    ) -> Option<MatchedErrorPattern> {
        self.patterns
            .iter()
            .filter(|pattern| pattern.category == category)
            .find_map(|pattern| pattern.find(body))
    }
}

impl Default for ErrorPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorPattern {
    fn new(category: ErrorPatternCategory, family: &'static str, pattern: &str) -> Self {
        Self {
            category,
            family,
            regex: Regex::new(pattern).expect("active detection error regex must compile"),
        }
    }

    fn find(&self, body: &str) -> Option<MatchedErrorPattern> {
        self.regex.find(body).map(|matched| MatchedErrorPattern {
            category: self.category,
            family: self.family,
            matched_text: matched.as_str().to_string(),
        })
    }
}

fn sql_patterns() -> Vec<ErrorPattern> {
    use ErrorPatternCategory::Sql;

    vec![
        ErrorPattern::new(Sql, "MySQL", r"(?i)SQL syntax.*MySQL"),
        ErrorPattern::new(Sql, "MySQL", r"(?i)Warning.*mysql_"),
        ErrorPattern::new(Sql, "MySQL", r"(?i)MySqlException"),
        ErrorPattern::new(Sql, "PostgreSQL", r"(?i)PostgreSQL.*ERROR"),
        ErrorPattern::new(Sql, "PostgreSQL", r"(?i)Warning.*pg_"),
        ErrorPattern::new(Sql, "PostgreSQL", r"(?i)Npgsql"),
        ErrorPattern::new(Sql, "MSSQL", r"(?i)Driver.*SQL[\-_ ]*Server"),
        ErrorPattern::new(Sql, "MSSQL", r"(?i)OLE DB.*SQL Server"),
        ErrorPattern::new(Sql, "MSSQL", r"(?i)SqlException"),
        ErrorPattern::new(Sql, "MSSQL", r"(?i)Unclosed quotation mark"),
        ErrorPattern::new(Sql, "Oracle", r"(?i)ORA-[0-9]{5}"),
        ErrorPattern::new(Sql, "Oracle", r"(?i)Oracle error"),
        ErrorPattern::new(Sql, "Oracle", r"(?i)quoted string not properly terminated"),
        ErrorPattern::new(Sql, "SQLite", r"(?i)SQLite.*Exception"),
        ErrorPattern::new(Sql, "SQLite", r"(?i)Warning.*sqlite_"),
        ErrorPattern::new(Sql, "SQLite", r"(?i)SQLITE_ERROR"),
    ]
}

fn nosql_patterns() -> Vec<ErrorPattern> {
    use ErrorPatternCategory::NoSql;

    vec![
        ErrorPattern::new(NoSql, "MongoDB", r"(?i)Mongo(Error|ServerError|Exception)"),
        ErrorPattern::new(NoSql, "MongoDB", r"(?i)BSONError"),
        ErrorPattern::new(NoSql, "MongoDB", r"(?i)CastError: Cast to"),
        ErrorPattern::new(NoSql, "MongoDB", r"(?i)E11000 duplicate key error"),
        ErrorPattern::new(NoSql, "CouchDB", r"(?i)bad_request.*invalid UTF-8 JSON"),
    ]
}

fn deserialization_patterns() -> Vec<ErrorPattern> {
    use ErrorPatternCategory::Deserialization;

    vec![
        ErrorPattern::new(
            Deserialization,
            "Java",
            r"(?i)java\.io\.(InvalidClass|StreamCorrupted)Exception",
        ),
        ErrorPattern::new(Deserialization, "PHP", r"(?i)unserialize\(\): Error"),
        ErrorPattern::new(Deserialization, "Python", r"(?i)pickle data was truncated"),
    ]
}

fn framework_patterns() -> Vec<ErrorPattern> {
    use ErrorPatternCategory::Framework;

    vec![
        ErrorPattern::new(
            Framework,
            "ASP.NET",
            r"(?i)Server Error in '.+' Application",
        ),
        ErrorPattern::new(Framework, "Rails", r"(?i)ActiveRecord::StatementInvalid"),
        ErrorPattern::new(Framework, "Django", r"(?i)Django.*Traceback"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_sql_errors() {
        let matcher = ErrorPatternMatcher::new();
        let matched = matcher.match_sql("Warning: mysql_fetch_array(): SQL syntax near");

        assert!(matched.is_some());
        assert_eq!(matched.unwrap().family, "MySQL");
    }
}
