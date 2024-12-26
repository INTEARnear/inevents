use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Operator {
    // Numeric operators
    GreaterThan(f64),
    LessThan(f64),
    GreaterOrEqual(f64),
    LessOrEqual(f64),

    // General equality
    Equals(Value),
    NotEqual(Value),

    // String operators
    StartsWith(String),
    EndsWith(String),
    Contains(String),

    // Array operators
    ArrayContains(Value),

    // Object operators
    HasKey(String),

    // Logical operators
    And(Vec<Filter>),
    Or(Vec<Filter>),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Filter {
    pub path: String,
    pub operator: Operator,
}

#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Path not found: {0}")]
    PathNotFound(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    #[error("Invalid array index in path: {0}")]
    InvalidArrayIndex(String),

    #[error("Invalid path format: {0}")]
    InvalidPath(String),
}

impl Filter {
    pub fn new(path: impl Into<String>, operator: Operator) -> Self {
        Self {
            path: path.into(),
            operator,
        }
    }

    pub fn check(&self, value: &Value) -> Result<bool, FilterError> {
        let target = self.resolve_path(value)?;
        self.check_operator(target)
    }

    fn resolve_path<'a>(&self, value: &'a Value) -> Result<&'a Value, FilterError> {
        let mut current = value;

        if self.path == "." {
            return Ok(current);
        }

        for segment in self.path.split('.') {
            if segment.contains('[') && segment.ends_with(']') {
                let (field, index) = self.parse_array_segment(segment)?;

                if !field.is_empty() {
                    current = current
                        .get(&field)
                        .ok_or_else(|| FilterError::PathNotFound(field.to_string()))?;
                }

                current = match current {
                    Value::Array(arr) => arr
                        .get(index)
                        .ok_or_else(|| FilterError::InvalidArrayIndex(index.to_string()))?,
                    _ => {
                        return Err(FilterError::TypeMismatch {
                            expected: "array".to_string(),
                            got: format!("{:?}", current),
                        })
                    }
                };
            } else {
                current = current
                    .get(segment)
                    .ok_or_else(|| FilterError::PathNotFound(segment.to_string()))?;
            }
        }

        Ok(current)
    }

    fn parse_array_segment(&self, segment: &str) -> Result<(String, usize), FilterError> {
        let bracket_idx = segment
            .find('[')
            .ok_or_else(|| FilterError::InvalidPath(segment.to_string()))?;

        let field = segment[..bracket_idx].to_string();
        let index_str = &segment[bracket_idx + 1..segment.len() - 1];

        let index = index_str
            .parse::<usize>()
            .map_err(|_| FilterError::InvalidArrayIndex(index_str.to_string()))?;

        Ok((field, index))
    }

    fn check_operator(&self, value: &Value) -> Result<bool, FilterError> {
        match &self.operator {
            Operator::GreaterThan(n) => {
                if let Value::Number(num) = value {
                    Ok(num.as_f64().unwrap() > *n)
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::LessThan(n) => {
                if let Value::Number(num) = value {
                    Ok(num.as_f64().unwrap() < *n)
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::GreaterOrEqual(n) => {
                if let Value::Number(num) = value {
                    Ok(num.as_f64().unwrap() >= *n)
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::LessOrEqual(n) => {
                if let Value::Number(num) = value {
                    Ok(num.as_f64().unwrap() <= *n)
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "number".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::Equals(target) => Ok(value == target),

            Operator::NotEqual(target) => Ok(value != target),

            Operator::StartsWith(s) => {
                if let Value::String(str) = value {
                    Ok(str.starts_with(s))
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "string".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::EndsWith(s) => {
                if let Value::String(str) = value {
                    Ok(str.ends_with(s))
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "string".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::Contains(s) => {
                if let Value::String(str) = value {
                    Ok(str.contains(s))
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "string".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::ArrayContains(target) => {
                if let Value::Array(arr) = value {
                    Ok(arr.contains(target))
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "array".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::HasKey(key) => {
                if let Value::Object(obj) = value {
                    Ok(obj.contains_key(key))
                } else {
                    Err(FilterError::TypeMismatch {
                        expected: "object".to_string(),
                        got: format!("{:?}", value),
                    })
                }
            }

            Operator::And(filters) => {
                let mut results = Vec::new();
                for filter in filters {
                    results.push(filter.check(value)?);
                }
                Ok(results.iter().all(|&x| x))
            }

            Operator::Or(filters) => {
                let mut results = Vec::new();
                for filter in filters {
                    results.push(filter.check(value)?);
                }
                Ok(results.iter().any(|&x| x))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_numeric_operators() {
        let value = json!({ "age": 25 });

        let filter = Filter::new("age", Operator::GreaterThan(20.0));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("age", Operator::LessThan(30.0));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("age", Operator::GreaterOrEqual(25.0));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("age", Operator::LessOrEqual(25.0));
        assert!(filter.check(&value).unwrap());
    }

    #[test]
    fn test_string_operators() {
        let value = json!({ "name": "John Doe" });

        let filter = Filter::new("name", Operator::StartsWith("John".to_string()));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("name", Operator::EndsWith("Doe".to_string()));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("name", Operator::Contains("hn D".to_string()));
        assert!(filter.check(&value).unwrap());
    }

    #[test]
    fn test_array_operators() {
        let value = json!({ "tags": ["rust", "coding", "json"] });

        let filter = Filter::new("tags", Operator::ArrayContains(json!("rust")));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new("tags[1]", Operator::Equals(json!("coding")));
        assert!(filter.check(&value).unwrap());
    }

    #[test]
    fn test_object_operators() {
        let value = json!({
            "user": {
                "id": 123,
                "details": {
                    "email": "john@example.com"
                }
            }
        });

        let filter = Filter::new("user", Operator::HasKey("id".to_string()));
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new(
            "user.details.email",
            Operator::EndsWith("@example.com".to_string()),
        );
        assert!(filter.check(&value).unwrap());
    }

    #[test]
    fn test_logical_operators() {
        let value = json!({
            "age": 25,
            "name": "John Doe"
        });

        let filter = Filter::new(
            ".",
            Operator::And(vec![
                Filter::new("age", Operator::GreaterThan(20.0)),
                Filter::new("name", Operator::StartsWith("John".to_string())),
            ]),
        );
        assert!(filter.check(&value).unwrap());

        let filter = Filter::new(
            ".",
            Operator::Or(vec![
                Filter::new("age", Operator::GreaterThan(30.0)),
                Filter::new("name", Operator::Contains("John".to_string())),
            ]),
        );
        assert!(filter.check(&value).unwrap());
    }

    #[test]
    fn test_type_mismatch() {
        let value = json!({ "age": "25" }); // age is a string, not a number

        let filter = Filter::new("age", Operator::GreaterThan(20.0));
        assert!(matches!(
            filter.check(&value),
            Err(FilterError::TypeMismatch { .. })
        ));
    }

    #[test]
    fn test_path_not_found() {
        let value = json!({ "name": "John" });

        let filter = Filter::new("age", Operator::GreaterThan(20.0));
        assert!(matches!(
            filter.check(&value),
            Err(FilterError::PathNotFound(..))
        ));
    }
}
