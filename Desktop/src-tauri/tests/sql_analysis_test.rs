// Test for SQL analysis without execution plan
#[cfg(test)]
mod sql_analysis_tests {
    use wdrprobe_desktop_lib::commands::execution_plan::analyze_sql_without_plan;

    #[test]
    fn test_analyze_select_star() {
        let sql = "SELECT * FROM users WHERE id = 1";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about SELECT *
        assert!(result.warnings.iter().any(|w| w.contains("SELECT *")));
        assert!(result.suggestions.iter().any(|s| s.contains("Specify only required columns")));
    }

    #[test]
    fn test_analyze_missing_where_clause() {
        let sql = "SELECT name FROM users";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about missing WHERE clause
        assert!(result.warnings.iter().any(|w| w.contains("lacks WHERE clause")));
        assert!(result.suggestions.iter().any(|s| s.contains("Add appropriate WHERE clause")));
    }

    #[test]
    fn test_analyze_like_wildcard() {
        let sql = "SELECT name FROM users WHERE email LIKE '%@gmail.com'";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about leading wildcard
        assert!(result.warnings.iter().any(|w| w.contains("leading wildcard")));
        assert!(result.suggestions.iter().any(|s| s.contains("full-text search")));
    }

    #[test]
    fn test_analyze_update_without_where() {
        let sql = "UPDATE users SET status = 'active'";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about UPDATE without WHERE
        assert!(result.warnings.iter().any(|w| w.contains("UPDATE without WHERE")));
        assert!(result.suggestions.iter().any(|s| s.contains("Add appropriate WHERE clause")));
    }

    #[test]
    fn test_analyze_delete_without_where() {
        let sql = "DELETE FROM users";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about DELETE without WHERE
        assert!(result.warnings.iter().any(|w| w.contains("DELETE without WHERE")));
        assert!(result.suggestions.iter().any(|s| s.contains("Add appropriate WHERE clause")));
    }

    #[test]
    fn test_analyze_insert_without_columns() {
        let sql = "INSERT INTO users VALUES (1, 'John', 'john@example.com')";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about INSERT without column specification
        assert!(result.warnings.iter().any(|w| w.contains("INSERT without column specification")));
        assert!(result.suggestions.iter().any(|s| s.contains("Explicitly specify column names")));
    }

    #[test]
    fn test_analyze_order_by_without_limit() {
        let sql = "SELECT name FROM users ORDER BY created_at";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about ORDER BY without LIMIT
        assert!(result.warnings.iter().any(|w| w.contains("ORDER BY without LIMIT")));
        assert!(result.suggestions.iter().any(|s| s.contains("Add LIMIT clause")));
    }

    #[test]
    fn test_analyze_or_conditions() {
        let sql = "SELECT name FROM users WHERE status = 'active' OR role = 'admin'";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should warn about OR conditions
        assert!(result.warnings.iter().any(|w| w.contains("OR conditions")));
        assert!(result.suggestions.iter().any(|s| s.contains("UNION")));
    }

    #[test]
    fn test_analyze_valid_sql() {
        let sql = "SELECT name FROM users WHERE id = 1";
        let result = analyze_sql_without_plan(sql).unwrap();
        
        // Should not have warnings for this well-formed query
        assert!(!result.warnings.iter().any(|w| w.contains("SELECT *")));
        assert!(!result.warnings.iter().any(|w| w.contains("lacks WHERE clause")));
    }
}