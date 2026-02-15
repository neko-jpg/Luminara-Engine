//! Query builder for constructing SurrealQL queries

/// Query builder for constructing SurrealQL queries
///
/// Provides a fluent API for building common queries without writing raw SQL.
///
/// # Example
///
/// ```
/// use luminara_db::QueryBuilder;
///
/// let query = QueryBuilder::new()
///     .select("*")
///     .from("entity")
///     .where_clause("'player' IN tags")
///     .limit(10)
///     .build();
///
/// assert_eq!(query, "SELECT * FROM entity WHERE 'player' IN tags LIMIT 10");
/// ```
#[derive(Debug, Clone, Default)]
pub struct QueryBuilder {
    select: Option<String>,
    from: Option<String>,
    where_clause: Option<String>,
    order_by: Option<String>,
    limit: Option<usize>,
    fetch: Option<String>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the SELECT clause
    pub fn select(mut self, fields: impl Into<String>) -> Self {
        self.select = Some(fields.into());
        self
    }

    /// Set the FROM clause
    pub fn from(mut self, table: impl Into<String>) -> Self {
        self.from = Some(table.into());
        self
    }

    /// Set the WHERE clause
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clause = Some(condition.into());
        self
    }

    /// Add an AND condition to the WHERE clause
    pub fn and(mut self, condition: impl Into<String>) -> Self {
        let condition = condition.into();
        self.where_clause = Some(match self.where_clause {
            Some(existing) => format!("{} AND {}", existing, condition),
            None => condition,
        });
        self
    }

    /// Add an OR condition to the WHERE clause
    pub fn or(mut self, condition: impl Into<String>) -> Self {
        let condition = condition.into();
        self.where_clause = Some(match self.where_clause {
            Some(existing) => format!("{} OR {}", existing, condition),
            None => condition,
        });
        self
    }

    /// Set the ORDER BY clause
    pub fn order_by(mut self, field: impl Into<String>, ascending: bool) -> Self {
        let field = field.into();
        let direction = if ascending { "ASC" } else { "DESC" };
        self.order_by = Some(format!("{} {}", field, direction));
        self
    }

    /// Set the LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set the FETCH clause (for loading related records)
    pub fn fetch(mut self, fields: impl Into<String>) -> Self {
        self.fetch = Some(fields.into());
        self
    }

    /// Build the final query string
    pub fn build(self) -> String {
        let mut query = String::new();

        // SELECT
        if let Some(select) = self.select {
            query.push_str("SELECT ");
            query.push_str(&select);
        } else {
            query.push_str("SELECT *");
        }

        // FROM
        if let Some(from) = self.from {
            query.push_str(" FROM ");
            query.push_str(&from);
        }

        // WHERE
        if let Some(where_clause) = self.where_clause {
            query.push_str(" WHERE ");
            query.push_str(&where_clause);
        }

        // ORDER BY
        if let Some(order_by) = self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(&order_by);
        }

        // LIMIT
        if let Some(limit) = self.limit {
            query.push_str(" LIMIT ");
            query.push_str(&limit.to_string());
        }

        // FETCH
        if let Some(fetch) = self.fetch {
            query.push_str(" FETCH ");
            query.push_str(&fetch);
        }

        query
    }
}

// Convenience functions for common queries

impl QueryBuilder {
    /// Query entities by tag
    pub fn entities_with_tag(tag: impl Into<String>) -> Self {
        Self::new()
            .from("entity")
            .where_clause(format!("'{}' IN tags", tag.into()))
    }

    /// Query entities by name
    pub fn entities_with_name(name: impl Into<String>) -> Self {
        Self::new()
            .from("entity")
            .where_clause(format!("name = '{}'", name.into()))
    }

    /// Query components by type
    pub fn components_of_type(type_name: impl Into<String>) -> Self {
        Self::new()
            .from("component")
            .where_clause(format!("type_name = '{}'", type_name.into()))
    }

    /// Query assets by type
    pub fn assets_of_type(asset_type: impl Into<String>) -> Self {
        Self::new()
            .from("asset")
            .where_clause(format!("asset_type = '{}'", asset_type.into()))
    }

    /// Query recent operations
    pub fn recent_operations(limit: usize) -> Self {
        Self::new()
            .from("operation")
            .order_by("timestamp", false)
            .limit(limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let query = QueryBuilder::new().select("*").from("entity").build();

        assert_eq!(query, "SELECT * FROM entity");
    }

    #[test]
    fn test_query_with_where() {
        let query = QueryBuilder::new()
            .from("entity")
            .where_clause("'player' IN tags")
            .build();

        assert_eq!(query, "SELECT * FROM entity WHERE 'player' IN tags");
    }

    #[test]
    fn test_query_with_and() {
        let query = QueryBuilder::new()
            .from("entity")
            .where_clause("'player' IN tags")
            .and("name = 'Hero'")
            .build();

        assert_eq!(
            query,
            "SELECT * FROM entity WHERE 'player' IN tags AND name = 'Hero'"
        );
    }

    #[test]
    fn test_query_with_order_and_limit() {
        let query = QueryBuilder::new()
            .from("operation")
            .order_by("timestamp", false)
            .limit(10)
            .build();

        assert_eq!(
            query,
            "SELECT * FROM operation ORDER BY timestamp DESC LIMIT 10"
        );
    }

    #[test]
    fn test_entities_with_tag() {
        let query = QueryBuilder::entities_with_tag("player").build();
        assert_eq!(query, "SELECT * FROM entity WHERE 'player' IN tags");
    }

    #[test]
    fn test_recent_operations() {
        let query = QueryBuilder::recent_operations(5).build();
        assert_eq!(
            query,
            "SELECT * FROM operation ORDER BY timestamp DESC LIMIT 5"
        );
    }
}
