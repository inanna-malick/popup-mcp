use anyhow::{Context, Result};
use pest::Parser;
use pest_derive::Parser;

use crate::models::{Element, PopupDefinition};

#[derive(Parser)]
#[grammar = "popup.pest"]
pub struct PopupParser;

pub fn parse_popup_dsl(input: &str) -> Result<PopupDefinition> {
    let pairs = PopupParser::parse(Rule::popup, input)
        .context("Failed to parse popup DSL")?;
    
    let pair = pairs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No popup found"))?;
    
    parse_popup(pair)
}

fn parse_popup(pair: pest::iterators::Pair<Rule>) -> Result<PopupDefinition> {
    let mut inner = pair.into_inner();
    
    // Skip "popup" keyword, get title
    let title = parse_string(inner.next().unwrap())?;
    
    // Parse elements
    let mut elements = Vec::new();
    for pair in inner {
        if pair.as_rule() == Rule::EOI {
            continue; // Skip End of Input marker
        }
        if let Ok(element) = parse_element(pair) {
            elements.push(element);
        }
    }
    
    Ok(PopupDefinition { title, elements })
}

fn parse_element(pair: pest::iterators::Pair<Rule>) -> Result<Element> {
    let inner_pair = if pair.as_rule() == Rule::element {
        // If it's an element wrapper, get the actual element inside
        pair.into_inner().next().unwrap()
    } else {
        // Otherwise use the pair directly
        pair
    };
    
    match inner_pair.as_rule() {
        Rule::text => {
            let text = parse_string(inner_pair.into_inner().next().unwrap())?;
            Ok(Element::Text(text))
        }
        
        Rule::slider => {
            let mut inner = inner_pair.into_inner();
            let label = parse_string(inner.next().unwrap())?;
            let min = inner.next().unwrap().as_str().parse::<f32>()?;
            let max = inner.next().unwrap().as_str().parse::<f32>()?;
            
            let default = if let Some(default_pair) = inner.next() {
                Some(default_pair.as_str().parse::<f32>()?)
            } else {
                None
            };
            
            Ok(Element::Slider { label, min, max, default })
        }
        
        Rule::checkbox => {
            let mut inner = inner_pair.into_inner();
            let label = parse_string(inner.next().unwrap())?;
            
            let default = if let Some(default_pair) = inner.next() {
                Some(default_pair.as_str() == "true")
            } else {
                None
            };
            
            Ok(Element::Checkbox { label, default })
        }
        
        Rule::textbox => {
            let mut inner = inner_pair.into_inner();
            let label = parse_string(inner.next().unwrap())?;
            let mut placeholder = None;
            let mut rows = None;
            
            for pair in inner {
                match pair.as_rule() {
                    Rule::string => placeholder = Some(parse_string(pair)?),
                    Rule::number => rows = Some(pair.as_str().parse::<u32>()?),
                    _ => {}
                }
            }
            
            Ok(Element::Textbox { label, placeholder, rows })
        }
        
        Rule::choice => {
            let mut inner = inner_pair.into_inner();
            let label = parse_string(inner.next().unwrap())?;
            
            let mut options = Vec::new();
            for pair in inner {
                if pair.as_rule() == Rule::string {
                    options.push(parse_string(pair)?);
                }
            }
            
            Ok(Element::Choice { label, options })
        }
        
        Rule::group => {
            let mut inner = inner_pair.into_inner();
            let label = parse_string(inner.next().unwrap())?;
            
            let mut elements = Vec::new();
            for pair in inner {
                if let Ok(element) = parse_element(pair) {
                    elements.push(element);
                }
            }
            
            Ok(Element::Group { label, elements })
        }
        
        Rule::buttons => {
            let inner = inner_pair.into_inner();
            let mut buttons = Vec::new();
            
            for pair in inner {
                if pair.as_rule() == Rule::string {
                    buttons.push(parse_string(pair)?);
                }
            }
            
            Ok(Element::Buttons(buttons))
        }
        
        Rule::EOI => {
            // End of input marker, skip it
            Err(anyhow::anyhow!("EOI is not an element"))
        }
        
        _ => Err(anyhow::anyhow!("Unknown element type: {:?}", inner_pair.as_rule()))
    }
}

fn parse_string(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    Ok(pair.into_inner().next().unwrap().as_str().to_string())
}
