use std::sync::Arc;

use typst::Library as TypstLibrary;
use typst::comemo::Tracked;
use typst::foundations::{
    Binding, Content, Context, Func, NativeFunc, Repr, SequenceElem, Str, StyleChain, StyledElem,
    SymbolElem, Target, TargetElem, Value, func,
};
use typst::math::EquationElem;
use typst::text::TextElem;
use typst::utils::LazyHash;

pub(super) fn typlite_library(
    library: &Arc<LazyHash<TypstLibrary>>,
) -> Arc<LazyHash<TypstLibrary>> {
    let mut library = library.as_ref().clone();
    let scope = library.global.scope_mut();
    scope.define_func::<__typlite_encode_content>();
    scope.bind(
        "target".into(),
        Binding::detached(Func::from(__typlite_target::data())),
    );
    if let Some(Value::Module(mut std)) = scope.get("std").map(|binding| binding.read().clone()) {
        std.scope_mut().bind(
            "target".into(),
            Binding::detached(Func::from(__typlite_target::data())),
        );
        scope.bind("std".into(), Binding::detached(std));
    }
    if let Value::Module(mut std) = library.std.read().clone() {
        std.scope_mut().bind(
            "target".into(),
            Binding::detached(Func::from(__typlite_target::data())),
        );
        library.std = Binding::detached(std);
    }
    Arc::new(library)
}

#[func(name = "__typlite_encode_content", title = "Typlite content encoder")]
fn __typlite_encode_content(body: Content) -> Str {
    Str::from(serde_json::to_string(&encode_content(&body)).unwrap_or_else(|_| "{}".to_owned()))
}

#[func(contextual, name = "target", title = "Typlite target")]
fn __typlite_target(context: Tracked<Context>) -> typst::diag::HintedStrResult<Str> {
    let target = context.styles()?.get(TargetElem::target);
    Ok(match target {
        Target::Html => Str::from("typlite"),
        Target::Paged => Str::from("paged"),
    })
}

fn encode_content(body: &Content) -> serde_json::Value {
    if let Some(styled) = body.to_packed::<StyledElem>() {
        let styles = StyleChain::new(&styled.styles);
        let mut object = serde_json::Map::new();
        object.insert("func".into(), "styled".into());
        object.insert("child".into(), encode_content(&styled.child));
        object.insert("bold".into(), styles.get(EquationElem::bold).into());
        object.insert("cramped".into(), styles.get(EquationElem::cramped).into());
        object.insert(
            "italic".into(),
            encode_optional_bool(styles.get(EquationElem::italic)),
        );
        object.insert(
            "size".into(),
            encode_math_size(styles.get(EquationElem::size)),
        );
        object.insert(
            "variant".into(),
            encode_math_variant(styles.get(EquationElem::variant)),
        );
        object.insert(
            "text_fill".into(),
            styles.get_cloned(TextElem::fill).repr().as_str().into(),
        );
        object.insert(
            "text_size".into(),
            format!("{:?}", styles.get(TextElem::size)).into(),
        );
        object.insert(
            "text_style".into(),
            format!("{:?}", styles.get(TextElem::style)).into(),
        );
        object.insert(
            "text_weight".into(),
            format!("{:?}", styles.get(TextElem::weight)).into(),
        );
        return serde_json::Value::Object(object);
    }

    if let Some(sequence) = body.to_packed::<SequenceElem>() {
        return serde_json::json!({
            "func": "sequence",
            "children": sequence.children.iter().map(encode_content).collect::<Vec<_>>(),
        });
    }

    if let Some(equation) = body.to_packed::<EquationElem>() {
        return serde_json::json!({
            "func": "equation",
            "block": equation.block.get(StyleChain::default()),
            "body": encode_content(&equation.body),
        });
    }

    if let Some(text) = body.to_packed::<TextElem>() {
        return serde_json::json!({
            "func": "text",
            "text": text.text.as_str(),
        });
    }

    if let Some(symbol) = body.to_packed::<SymbolElem>() {
        return serde_json::json!({
            "func": "symbol",
            "text": symbol.text.as_str(),
        });
    }

    encode_content_fields(body)
}

fn encode_content_fields(body: &Content) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert("func".into(), body.elem().name().into());
    for (name, value) in body.fields().iter() {
        object.insert(name.as_str().into(), encode_value(value));
    }
    serde_json::Value::Object(object)
}

fn encode_value(value: &Value) -> serde_json::Value {
    match value {
        Value::None => serde_json::Value::Null,
        Value::Auto => "auto".into(),
        Value::Bool(value) => (*value).into(),
        Value::Int(value) => (*value).into(),
        Value::Float(value) => serde_json::Number::from_f64(*value)
            .map(serde_json::Value::Number)
            .unwrap_or_else(|| value.repr().as_str().into()),
        Value::Str(value) => value.as_str().into(),
        Value::Content(value) => encode_content(value),
        Value::Array(value) => value.iter().map(encode_value).collect(),
        Value::Dict(value) => value
            .iter()
            .map(|(key, value)| (key.as_str().to_owned(), encode_value(value)))
            .collect(),
        _ => value.repr().as_str().into(),
    }
}

fn encode_optional_bool(value: Option<bool>) -> serde_json::Value {
    match value {
        Some(value) => value.to_string().into(),
        None => serde_json::Value::Null,
    }
}

fn encode_math_size(value: impl std::fmt::Debug) -> serde_json::Value {
    match format!("{value:?}").as_str() {
        "Display" => "display".into(),
        "Text" => "text".into(),
        "Script" => "script".into(),
        "ScriptScript" => "script-script".into(),
        _ => serde_json::Value::Null,
    }
}

fn encode_math_variant(value: impl std::fmt::Debug) -> serde_json::Value {
    match format!("{value:?}").as_str() {
        "Some(Plain)" => "plain".into(),
        "Some(SansSerif)" => "sans-serif".into(),
        "Some(Chancery)" => "chancery".into(),
        "Some(Roundhand)" => "roundhand".into(),
        "Some(Fraktur)" => "fraktur".into(),
        "Some(Monospace)" => "monospace".into(),
        "Some(DoubleStruck)" => "double-struck".into(),
        _ => serde_json::Value::Null,
    }
}
