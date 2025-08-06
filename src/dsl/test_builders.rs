#[cfg(test)]
pub mod builders {
    use crate::models::{Element, PopupDefinition, Condition, ComparisonOp};
    
    /// Builder for creating PopupDefinition in tests
    pub struct PopupBuilder {
        title: String,
        elements: Vec<Element>,
    }
    
    impl PopupBuilder {
        pub fn new(title: impl Into<String>) -> Self {
            Self {
                title: title.into(),
                elements: vec![],
            }
        }
        
        pub fn text(mut self, content: impl Into<String>) -> Self {
            self.elements.push(Element::Text(content.into()));
            self
        }
        
        pub fn slider(mut self, label: impl Into<String>, min: f32, max: f32) -> Self {
            self.elements.push(Element::Slider {
                label: label.into(),
                min,
                max,
                default: (min + max) / 2.0,  // Default to midpoint like parser does
            });
            self
        }
        
        pub fn slider_with_default(mut self, label: impl Into<String>, min: f32, max: f32, default: f32) -> Self {
            self.elements.push(Element::Slider {
                label: label.into(),
                min,
                max,
                default,
            });
            self
        }
        
        pub fn checkbox(mut self, label: impl Into<String>, default: bool) -> Self {
            self.elements.push(Element::Checkbox {
                label: label.into(),
                default,
            });
            self
        }
        
        pub fn textbox(mut self, label: impl Into<String>, placeholder: Option<impl Into<String>>) -> Self {
            self.elements.push(Element::Textbox {
                label: label.into(),
                placeholder: placeholder.map(Into::into),
                rows: None,
            });
            self
        }
        
        pub fn textbox_multiline(mut self, label: impl Into<String>, placeholder: Option<impl Into<String>>, rows: u32) -> Self {
            self.elements.push(Element::Textbox {
                label: label.into(),
                placeholder: placeholder.map(Into::into),
                rows: Some(rows),
            });
            self
        }
        
        pub fn choice(mut self, label: impl Into<String>, options: Vec<impl Into<String>>) -> Self {
            self.elements.push(Element::Choice {
                label: label.into(),
                options: options.into_iter().map(Into::into).collect(),
            });
            self
        }
        
        pub fn multiselect(mut self, label: impl Into<String>, options: Vec<impl Into<String>>) -> Self {
            self.elements.push(Element::Multiselect {
                label: label.into(),
                options: options.into_iter().map(Into::into).collect(),
            });
            self
        }
        
        pub fn buttons(mut self, labels: Vec<impl Into<String>>) -> Self {
            self.elements.push(Element::Buttons(
                labels.into_iter().map(Into::into).collect()
            ));
            self
        }
        
        pub fn group(mut self, label: impl Into<String>, elements: Vec<Element>) -> Self {
            self.elements.push(Element::Group {
                label: label.into(),
                elements,
            });
            self
        }
        
        pub fn conditional(mut self, condition: Condition, elements: Vec<Element>) -> Self {
            self.elements.push(Element::Conditional {
                condition,
                elements,
            });
            self
        }
        
        pub fn build(self) -> PopupDefinition {
            PopupDefinition {
                title: self.title,
                elements: self.elements,
            }
        }
    }
    
    /// Builder for creating conditions
    pub struct ConditionBuilder;
    
    impl ConditionBuilder {
        pub fn checked(label: impl Into<String>) -> Condition {
            Condition::Checked(label.into())
        }
        
        pub fn selected(label: impl Into<String>, value: impl Into<String>) -> Condition {
            Condition::Selected(label.into(), value.into())
        }
        
        pub fn count_gt(label: impl Into<String>, value: i32) -> Condition {
            Condition::Count(label.into(), ComparisonOp::Greater, value)
        }
        
        pub fn count_lt(label: impl Into<String>, value: i32) -> Condition {
            Condition::Count(label.into(), ComparisonOp::Less, value)
        }
        
        pub fn count_gte(label: impl Into<String>, value: i32) -> Condition {
            Condition::Count(label.into(), ComparisonOp::GreaterEqual, value)
        }
        
        pub fn count_lte(label: impl Into<String>, value: i32) -> Condition {
            Condition::Count(label.into(), ComparisonOp::LessEqual, value)
        }
        
        pub fn count_eq(label: impl Into<String>, value: i32) -> Condition {
            Condition::Count(label.into(), ComparisonOp::Equal, value)
        }
    }
    
    /// Helper for creating elements inline
    pub mod el {
        use crate::models::Element;
        
        pub fn text(content: impl Into<String>) -> Element {
            Element::Text(content.into())
        }
        
        pub fn slider(label: impl Into<String>, min: f32, max: f32) -> Element {
            Element::Slider {
                label: label.into(),
                min,
                max,
                default: (min + max) / 2.0,  // Default to midpoint like parser does
            }
        }
        
        pub fn checkbox(label: impl Into<String>, default: bool) -> Element {
            Element::Checkbox {
                label: label.into(),
                default,
            }
        }
        
        pub fn textbox(label: impl Into<String>, placeholder: Option<impl Into<String>>) -> Element {
            Element::Textbox {
                label: label.into(),
                placeholder: placeholder.map(Into::into),
                rows: None,
            }
        }
        
        pub fn choice(label: impl Into<String>, options: Vec<impl Into<String>>) -> Element {
            Element::Choice {
                label: label.into(),
                options: options.into_iter().map(Into::into).collect(),
            }
        }
        
        pub fn multiselect(label: impl Into<String>, options: Vec<impl Into<String>>) -> Element {
            Element::Multiselect {
                label: label.into(),
                options: options.into_iter().map(Into::into).collect(),
            }
        }
        
        pub fn buttons(labels: Vec<impl Into<String>>) -> Element {
            Element::Buttons(labels.into_iter().map(Into::into).collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builders::{PopupBuilder, ConditionBuilder, el};
    use crate::dsl::simple_parser::parse_popup_dsl;
    
    #[test]
    fn test_builder_pattern() {
        let expected = PopupBuilder::new("Settings")
            .text("Configure your preferences")
            .slider("Volume", 0.0, 100.0)
            .checkbox("Notifications", true)
            .choice("Theme", vec!["Light", "Dark", "Auto"])
            .buttons(vec!["Save", "Cancel"])
            .build();
        
        let dsl = r#"Settings
Configure your preferences
Volume: 0-100
Notifications: yes
Theme: Light | Dark | Auto
[Save | Cancel]"#;
        
        let actual = parse_popup_dsl(dsl).unwrap();
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn test_conditional_builder() {
        let expected = PopupBuilder::new("Advanced")
            .checkbox("Debug", false)
            .conditional(
                ConditionBuilder::checked("Debug"),
                vec![
                    el::slider("Level", 0.0, 10.0),
                    el::textbox("Log file", Some("/tmp/debug.log")),
                ]
            )
            .build();
        
        let dsl = r#"Advanced
Debug: no
[if Debug] {
  Level: 0-10
  Log file: @/tmp/debug.log
}"#;
        
        let actual = parse_popup_dsl(dsl).unwrap();
        assert_eq!(actual, expected);
    }
    
    #[test]
    fn test_inline_element_helpers() {
        let elements = vec![
            el::text("Welcome"),
            el::checkbox("Accept terms", false),
            el::textbox("Email", Some("user@example.com")),
            el::buttons(vec!["Continue", "Cancel"]),
        ];
        
        assert_eq!(elements.len(), 4);
    }
}