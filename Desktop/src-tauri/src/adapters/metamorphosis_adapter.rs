// Metamorphosis SQL Rewrite Adapter
// Connects the metamorphosis rewrite engine to WDRProbe's Tauri backend.
// Provides SQL parsing → rewrite → formatting pipeline.

use serde::{Deserialize, Serialize};
use metamorphosis_core::{
    RewriteConfig, RewriteContext, RewriteEngine, RuleRegistry,
};
use metamorphosis_rules::builtin_rules;
use ogsql_parser::Parser;
use ogsql_parser::formatter::SqlFormatter;
use ogsql_parser::analyzer::schema::SchemaMap;
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
pub struct RewriteOutput {
    pub original_sql: String,
    pub rewritten_sql: String,
    pub changed: bool,
    pub suggestions: Vec<RewriteSuggestion>,
    pub match_failures: Vec<MatchFailureInfo>,
    pub rules_applied: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewriteSuggestion {
    pub rule_id: String,
    pub rule_description: String,
    pub confidence: String,
    pub notes: Vec<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MatchFailureInfo {
    pub rule_id: String,
    pub reason: String,
}

pub struct MetamorphosisAdapter {
    engine: RewriteEngine,
}

impl MetamorphosisAdapter {
    pub fn new() -> Self {
        let registry = RuleRegistry::new(builtin_rules());
        Self { engine: RewriteEngine::new(registry) }
    }

    pub fn rewrite(
        &self,
        sql: &str,
        schema: Option<&SchemaMap>,
    ) -> Result<RewriteOutput, String> {
        // 1. Parse SQL
        let (stmt_infos, errors) = Parser::parse_sql(sql);
        if !errors.is_empty() {
            return Err(format!("Parse errors: {:?}", errors));
        }
        if stmt_infos.is_empty() {
            return Err("No statements found in input".to_string());
        }

        let statements: Vec<_> = stmt_infos.into_iter().map(|si| si.statement).collect();

        // 2. Create context
        let config = RewriteConfig::default();
        let ctx = RewriteContext {
            version: None,
            schema,
            config: &config,
            source_file: None,
            known_variables: None,
        };

        // 3. Rewrite
        let result = self.engine.rewrite(&ctx, statements);

        // 4. Format output
        let formatter = SqlFormatter::new();
        let rewritten_sql = if result.changed {
            result.statements.iter()
                .map(|s| formatter.format_statement(s))
                .collect::<Vec<_>>()
                .join(";\n")
        } else {
            sql.to_string()
        };

        // 5. Build output
        let suggestions: Vec<RewriteSuggestion> = result.suggestions.iter().map(|s| RewriteSuggestion {
            rule_id: s.rule_id.clone(),
            rule_description: s.rule_description.clone(),
            confidence: format!("{:?}", s.confidence),
            notes: s.notes.clone(),
            message: match &s.action {
                metamorphosis_core::RewriteAction::Suggest { message, .. } => Some(message.clone()),
                _ => None,
            },
        }).collect();

        let match_failures = result.match_failures.iter().map(|f| MatchFailureInfo {
            rule_id: f.rule_id.clone(),
            reason: f.reason.clone(),
        }).collect();

        // Determine which rules were applied
        let rules_applied: Vec<String> = suggestions.iter()
            .map(|s| s.rule_id.clone())
            .chain(result.match_failures.iter().map(|f| f.rule_id.clone()))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        Ok(RewriteOutput {
            original_sql: sql.to_string(),
            rewritten_sql,
            changed: result.changed,
            suggestions,
            match_failures,
            rules_applied,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RuleInfo {
    pub id: String,
    pub category: String,
    pub safety_level: String,
    pub description: String,
    pub default_enabled: bool,
}

/// Static list of builtin rules metadata
pub fn list_builtin_rules() -> Vec<RuleInfo> {
    vec![
        RuleInfo {
            id: "eliminate-select-star".into(),
            category: "Semantic".into(),
            safety_level: "Safe".into(),
            description: "SELECT * → explicit column list (requires schema)".into(),
            default_enabled: true,
        },
        RuleInfo {
            id: "detect-duplicate-eq-keys".into(),
            category: "DataQuality".into(),
            safety_level: "Manual".into(),
            description: "WHERE equalities → GROUP BY uniqueness probe".into(),
            default_enabled: true,
        },
        RuleInfo {
            id: "subquery-to-join".into(),
            category: "Performance".into(),
            safety_level: "Conditional".into(),
            description: "EXISTS/IN subqueries → JOIN".into(),
            default_enabled: true,
        },
        RuleInfo {
            id: "extract-candidate-values".into(),
            category: "DataQuality".into(),
            safety_level: "Manual".into(),
            description: "Parameterized columns → candidate value probe".into(),
            default_enabled: false,
        },
    ]
}
